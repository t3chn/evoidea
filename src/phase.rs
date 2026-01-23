use anyhow::Result;
use std::path::Path;

use crate::config::RunConfig;
use crate::data::{Event, EventType, IdeaStatus, Origin, State};
use crate::llm::{apply_critic_patches, parse_generated_ideas, LlmProvider, LlmTask};
use crate::scoring::{calculate_overall_score, select_ideas};
use crate::storage::Storage;

/// Context passed to phases during execution
pub struct PhaseContext<'a> {
    pub config: &'a RunConfig,
    pub storage: &'a dyn Storage,
    pub llm: &'a dyn LlmProvider,
    pub schema_dir: &'a Path,
}

/// Trait for pipeline phases
pub trait Phase: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, state: State, ctx: &PhaseContext) -> Result<State>;
}

/// Generate new ideas
pub struct GeneratePhase;

impl Phase for GeneratePhase {
    fn name(&self) -> &str {
        "generate"
    }

    fn run(&self, mut state: State, ctx: &PhaseContext) -> Result<State> {
        let active_count = state.active_ideas().count();
        let to_generate = ctx
            .config
            .population_size
            .saturating_sub(active_count as u32) as usize;

        if to_generate == 0 {
            return Ok(state);
        }

        let task = LlmTask::Generate {
            prompt: ctx.config.prompt.clone(),
            count: to_generate,
        };

        let schema_path = ctx.schema_dir.join("generator.output.schema.json");
        let output = ctx.llm.generate_json(task, &schema_path)?;

        let new_ideas = parse_generated_ideas(&output, state.iteration)?;
        let generated_count = new_ideas.len();

        state.ideas.extend(new_ideas);

        let event = Event::new(
            state.iteration,
            EventType::Generated,
            serde_json::json!({ "count": generated_count }),
        );
        ctx.storage.append_event(&state.run_id, &event)?;

        tracing::info!(count = generated_count, "Generated new ideas");
        Ok(state)
    }
}

/// Score ideas with critic
pub struct CriticPhase;

impl Phase for CriticPhase {
    fn name(&self) -> &str {
        "critic"
    }

    fn run(&self, mut state: State, ctx: &PhaseContext) -> Result<State> {
        let unscored: Vec<_> = state
            .ideas
            .iter()
            .filter(|i| i.status == IdeaStatus::Active && i.overall_score.is_none())
            .map(|i| (i.id, i.title.clone(), i.summary.clone()))
            .collect();

        if unscored.is_empty() {
            return Ok(state);
        }

        let task = LlmTask::Critic {
            ideas: unscored.clone(),
        };
        let schema_path = ctx.schema_dir.join("critic.output.schema.json");
        let output = ctx.llm.generate_json(task, &schema_path)?;

        apply_critic_patches(&mut state.ideas, &output)?;

        // Recalculate overall scores using our weighting (including risk inversion)
        for idea in state.ideas.iter_mut() {
            if idea.status == IdeaStatus::Active {
                let calculated = calculate_overall_score(&idea.scores, &ctx.config.scoring_weights);
                idea.overall_score = Some(calculated);
            }
        }

        let event = Event::new(
            state.iteration,
            EventType::Scored,
            serde_json::json!({ "count": unscored.len() }),
        );
        ctx.storage.append_event(&state.run_id, &event)?;

        tracing::info!(count = unscored.len(), "Scored ideas");
        Ok(state)
    }
}

/// Select elite + diversity
pub struct SelectPhase;

impl Phase for SelectPhase {
    fn name(&self) -> &str {
        "select"
    }

    fn run(&self, mut state: State, ctx: &PhaseContext) -> Result<State> {
        let selected_ids = select_ideas(
            &mut state.ideas,
            ctx.config.elite_count as usize,
            ctx.config.population_size as usize,
        );

        // Archive non-selected ideas
        let mut archived_count = 0;
        for idea in state.ideas.iter_mut() {
            if idea.status == IdeaStatus::Active && !selected_ids.contains(&idea.id) {
                idea.status = IdeaStatus::Archived;
                archived_count += 1;
            }
        }

        // Update best idea
        let previous_best = state.best_score;
        if let Some(best) = state
            .ideas
            .iter()
            .filter(|i| i.status == IdeaStatus::Active)
            .max_by(|a, b| {
                a.overall_score
                    .partial_cmp(&b.overall_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            state.best_idea_id = Some(best.id);
            state.best_score = best.overall_score;
        }

        // Update stagnation
        state.stagnation_counter = crate::scoring::update_stagnation(
            state.best_score,
            previous_best,
            state.stagnation_counter,
        );

        let event = Event::new(
            state.iteration,
            EventType::Selected,
            serde_json::json!({
                "selected": selected_ids.len(),
                "archived": archived_count,
                "best_score": state.best_score
            }),
        );
        ctx.storage.append_event(&state.run_id, &event)?;

        tracing::info!(
            selected = selected_ids.len(),
            archived = archived_count,
            best_score = ?state.best_score,
            "Selection complete"
        );

        Ok(state)
    }
}

/// Refine top-K ideas
pub struct RefinePhase {
    pub top_k: usize,
}

impl Phase for RefinePhase {
    fn name(&self) -> &str {
        "refine"
    }

    fn run(&self, mut state: State, ctx: &PhaseContext) -> Result<State> {
        // Collect candidate data (clone to avoid borrow issues)
        let mut candidates: Vec<_> = state
            .ideas
            .iter()
            .filter(|i| i.status == IdeaStatus::Active && i.judge_notes.is_some())
            .map(|i| {
                (
                    i.id,
                    i.title.clone(),
                    i.summary.clone(),
                    i.facets.clone(),
                    i.judge_notes.clone(),
                    i.overall_score,
                )
            })
            .collect();

        candidates.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal));

        let to_refine: Vec<_> = candidates.into_iter().take(self.top_k).collect();

        if to_refine.is_empty() {
            return Ok(state);
        }

        let mut refined_count = 0;
        let schema_path = ctx.schema_dir.join("refiner.output.schema.json");
        let mut new_ideas = Vec::new();

        for (idea_id, title, summary, facets, judge_notes, _) in to_refine {
            let task = LlmTask::Refine {
                idea_id,
                title: title.clone(),
                summary: summary.clone(),
                facets: facets.clone(),
                judge_notes: judge_notes.unwrap_or_default(),
            };

            let output = ctx.llm.generate_json(task, &schema_path)?;

            if let Some(patch) = output.get("patch") {
                // Create refined version as new idea
                let new_title = patch
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&title);
                let new_summary = patch
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&summary);

                let new_facets = if let Some(f) = patch.get("facets") {
                    crate::data::Facets {
                        audience: f
                            .get("audience")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&facets.audience)
                            .into(),
                        jtbd: f
                            .get("jtbd")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&facets.jtbd)
                            .into(),
                        differentiator: f
                            .get("differentiator")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&facets.differentiator)
                            .into(),
                        monetization: f
                            .get("monetization")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&facets.monetization)
                            .into(),
                        distribution: f
                            .get("distribution")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&facets.distribution)
                            .into(),
                        risks: f
                            .get("risks")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&facets.risks)
                            .into(),
                    }
                } else {
                    facets.clone()
                };

                let refined = crate::data::Idea::new(
                    new_title.into(),
                    new_summary.into(),
                    new_facets,
                    state.iteration,
                    Origin::Refined,
                )
                .with_parents(vec![idea_id]);

                new_ideas.push(refined);
                refined_count += 1;
            }
        }

        state.ideas.extend(new_ideas);

        let event = Event::new(
            state.iteration,
            EventType::Refined,
            serde_json::json!({ "count": refined_count }),
        );
        ctx.storage.append_event(&state.run_id, &event)?;

        tracing::info!(count = refined_count, "Refined ideas");
        Ok(state)
    }
}

/// Compose final result
pub struct FinalPhase;

impl Phase for FinalPhase {
    fn name(&self) -> &str {
        "final"
    }

    fn run(&self, state: State, ctx: &PhaseContext) -> Result<State> {
        let mut active: Vec<_> = state
            .ideas
            .iter()
            .filter(|i| i.status == IdeaStatus::Active && i.overall_score.is_some())
            .collect();

        active.sort_by(|a, b| {
            b.overall_score
                .partial_cmp(&a.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if active.is_empty() {
            return Err(anyhow::anyhow!("No active ideas to compose final result"));
        }

        let best = active[0];
        let runners_up: Vec<_> = active
            .iter()
            .skip(1)
            .take(4)
            .map(|i| crate::data::RunnerUp {
                idea_id: i.id,
                title: i.title.clone(),
                overall_score: i.overall_score.unwrap_or(0.0),
            })
            .collect();

        let final_result = crate::data::FinalResult {
            run_id: state.run_id,
            best: crate::data::FinalBest {
                idea_id: best.id,
                title: best.title.clone(),
                summary: best.summary.clone(),
                facets: best.facets.clone(),
                scores: best.scores.clone(),
                overall_score: best.overall_score.unwrap_or(0.0),
                why_won: vec![
                    format!(
                        "Highest overall score: {:.2}",
                        best.overall_score.unwrap_or(0.0)
                    ),
                    format!("Feasibility: {:.1}", best.scores.feasibility),
                    format!("Low risk: {:.1}", best.scores.risk),
                ],
            },
            runners_up,
        };

        ctx.storage.save_final(&final_result)?;

        tracing::info!(
            best_id = %best.id,
            best_title = %best.title,
            best_score = ?best.overall_score,
            "Final result composed"
        );

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RunConfig;
    use crate::llm::MockLlmProvider;
    use crate::storage::FileStorage;
    use tempfile::TempDir;

    fn setup_test_context(temp_dir: &TempDir) -> (RunConfig, FileStorage, MockLlmProvider) {
        let config = RunConfig::new(
            "Test prompt".into(),
            "mock".into(),
            6,
            12,
            4,
            8.7,
            2,
            temp_dir.path().to_string_lossy().into(),
        );
        let storage = FileStorage::new(temp_dir.path());
        let llm = MockLlmProvider::new();
        (config, storage, llm)
    }

    #[test]
    fn test_generate_phase() {
        let temp_dir = TempDir::new().unwrap();
        let (config, storage, llm) = setup_test_context(&temp_dir);

        let run_id = storage.init_run(&config).unwrap();
        let state = State::new(run_id);

        let ctx = PhaseContext {
            config: &config,
            storage: &storage,
            llm: &llm,
            schema_dir: Path::new("schemas"),
        };

        let phase = GeneratePhase;
        let new_state = phase.run(state, &ctx).unwrap();

        assert!(!new_state.ideas.is_empty());
        assert!(new_state.ideas.len() <= config.population_size as usize);
    }

    #[test]
    fn test_critic_phase() {
        let temp_dir = TempDir::new().unwrap();
        let (config, storage, llm) = setup_test_context(&temp_dir);

        let run_id = storage.init_run(&config).unwrap();
        let mut state = State::new(run_id);

        // Add unscored ideas
        let facets = crate::data::Facets {
            audience: "test".into(),
            jtbd: "test".into(),
            differentiator: "test".into(),
            monetization: "test".into(),
            distribution: "test".into(),
            risks: "test".into(),
        };
        state.ideas.push(crate::data::Idea::new(
            "Test".into(),
            "Summary".into(),
            facets,
            1,
            Origin::Generated,
        ));

        let ctx = PhaseContext {
            config: &config,
            storage: &storage,
            llm: &llm,
            schema_dir: Path::new("schemas"),
        };

        let phase = CriticPhase;
        let new_state = phase.run(state, &ctx).unwrap();

        assert!(new_state.ideas[0].overall_score.is_some());
    }

    #[test]
    fn test_select_phase_updates_best() {
        let temp_dir = TempDir::new().unwrap();
        let (config, storage, llm) = setup_test_context(&temp_dir);

        let run_id = storage.init_run(&config).unwrap();
        let mut state = State::new(run_id);

        // Add scored ideas
        let facets = crate::data::Facets {
            audience: "test".into(),
            jtbd: "test".into(),
            differentiator: "test".into(),
            monetization: "test".into(),
            distribution: "test".into(),
            risks: "test".into(),
        };

        let mut idea1 = crate::data::Idea::new(
            "Best".into(),
            "".into(),
            facets.clone(),
            1,
            Origin::Generated,
        );
        idea1.overall_score = Some(9.0);

        let mut idea2 =
            crate::data::Idea::new("Second".into(), "".into(), facets, 1, Origin::Generated);
        idea2.overall_score = Some(7.0);

        state.ideas.push(idea1);
        state.ideas.push(idea2);

        let ctx = PhaseContext {
            config: &config,
            storage: &storage,
            llm: &llm,
            schema_dir: Path::new("schemas"),
        };

        let phase = SelectPhase;
        let new_state = phase.run(state, &ctx).unwrap();

        assert!(new_state.best_idea_id.is_some());
        assert_eq!(new_state.best_score, Some(9.0));
    }
}
