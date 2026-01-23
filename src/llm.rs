use anyhow::Result;
use serde_json::Value;
use std::path::Path;

use crate::data::{Facets, Idea, Origin, Scores};

/// LLM task types for structured outputs
#[derive(Debug, Clone)]
#[allow(dead_code)] // Merge/Mutate will be used in P3 crossover/mutation phases
pub enum LlmTask {
    Generate {
        #[allow(dead_code)] // Used by real LLM providers
        prompt: String,
        count: usize,
    },
    Critic {
        ideas: Vec<(uuid::Uuid, String, String)>, // id, title, summary
    },
    Merge {
        idea_a: (String, String, Facets), // title, summary, facets
        idea_b: (String, String, Facets),
    },
    Mutate {
        idea: (String, String, Facets),
        mutation_type: String,
    },
    Refine {
        idea_id: uuid::Uuid,
        title: String,
        summary: String,
        facets: Facets,
        judge_notes: String,
    },
}

/// Trait for LLM providers
pub trait LlmProvider: Send + Sync {
    fn generate_json(&self, task: LlmTask, schema_path: &Path) -> Result<Value>;
}

/// Mock provider for deterministic testing
pub struct MockLlmProvider {
    gen_counter: std::sync::atomic::AtomicU32,
}

impl MockLlmProvider {
    pub fn new() -> Self {
        Self {
            gen_counter: std::sync::atomic::AtomicU32::new(0),
        }
    }

    fn make_mock_idea(&self, idx: usize, gen: u32) -> Value {
        serde_json::json!({
            "title": format!("Mock Idea {} (gen {})", idx, gen),
            "summary": format!("This is mock idea {} generated in generation {}", idx, gen),
            "facets": {
                "audience": "Developers",
                "jtbd": "Automate repetitive tasks",
                "differentiator": "AI-powered automation",
                "monetization": "SaaS subscription",
                "distribution": "Developer communities",
                "risks": "Competition from incumbents"
            }
        })
    }

    fn make_mock_scores(&self, id: &uuid::Uuid, idx: usize) -> Value {
        // Deterministic scores based on index
        let base_score = 7.0 + (idx as f32 * 0.3);
        serde_json::json!({
            "id": id.to_string(),
            "scores": {
                "feasibility": (base_score).min(10.0),
                "speed_to_value": (base_score - 0.5).clamp(0.0, 10.0),
                "differentiation": (base_score + 0.2).min(10.0),
                "market_size": (base_score - 0.3).clamp(0.0, 10.0),
                "distribution": (base_score).min(10.0),
                "moats": (base_score - 1.0).clamp(0.0, 10.0),
                "risk": (5.0 - idx as f32 * 0.2).clamp(1.0, 10.0),
                "clarity": (base_score + 0.5).min(10.0)
            },
            "overall_score": base_score.min(10.0),
            "judge_notes": format!("Mock evaluation for idea {}", id)
        })
    }
}

impl Default for MockLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmProvider for MockLlmProvider {
    fn generate_json(&self, task: LlmTask, _schema_path: &Path) -> Result<Value> {
        match task {
            LlmTask::Generate { count, .. } => {
                let gen = self
                    .gen_counter
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let ideas: Vec<Value> = (0..count).map(|i| self.make_mock_idea(i, gen)).collect();
                Ok(serde_json::json!({ "ideas": ideas }))
            }
            LlmTask::Critic { ideas } => {
                let patches: Vec<Value> = ideas
                    .iter()
                    .enumerate()
                    .map(|(idx, (id, _, _))| self.make_mock_scores(id, idx))
                    .collect();
                Ok(serde_json::json!({ "patches": patches }))
            }
            LlmTask::Merge { idea_a, idea_b } => Ok(serde_json::json!({
                "idea": {
                    "title": format!("{} + {}", idea_a.0, idea_b.0),
                    "summary": format!("Merged: {} and {}", idea_a.1, idea_b.1),
                    "facets": {
                        "audience": idea_a.2.audience.clone(),
                        "jtbd": idea_b.2.jtbd.clone(),
                        "differentiator": format!("{} with {}", idea_a.2.differentiator, idea_b.2.differentiator),
                        "monetization": idea_a.2.monetization.clone(),
                        "distribution": idea_b.2.distribution.clone(),
                        "risks": format!("{} and {}", idea_a.2.risks, idea_b.2.risks)
                    }
                }
            })),
            LlmTask::Mutate {
                idea,
                mutation_type,
            } => {
                let mut facets = idea.2.clone();
                match mutation_type.as_str() {
                    "audience" => facets.audience = format!("{} (mutated)", facets.audience),
                    "monetization" => {
                        facets.monetization = format!("{} (mutated)", facets.monetization)
                    }
                    "distribution" => {
                        facets.distribution = format!("{} (mutated)", facets.distribution)
                    }
                    "differentiator" => {
                        facets.differentiator = format!("{} (mutated)", facets.differentiator)
                    }
                    "jtbd" => facets.jtbd = format!("{} (mutated)", facets.jtbd),
                    _ => {}
                }
                Ok(serde_json::json!({
                    "mutation_type": mutation_type,
                    "idea": {
                        "title": format!("{} (mutated)", idea.0),
                        "summary": idea.1,
                        "facets": facets
                    }
                }))
            }
            LlmTask::Refine {
                idea_id,
                title,
                summary,
                facets,
                judge_notes,
            } => Ok(serde_json::json!({
                "patch": {
                    "id": idea_id.to_string(),
                    "title": format!("{} (refined)", title),
                    "summary": format!("{} Improvements based on: {}", summary, judge_notes),
                    "facets": facets,
                    "changes": ["Improved based on feedback"]
                }
            })),
        }
    }
}

/// Parse generator output into ideas
pub fn parse_generated_ideas(output: &Value, gen: u32) -> Result<Vec<Idea>> {
    let ideas_array = output
        .get("ideas")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Expected 'ideas' array in output"))?;

    let mut ideas = Vec::new();
    for idea_val in ideas_array {
        let title = idea_val
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let summary = idea_val
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let facets_val = idea_val.get("facets");
        let facets = if let Some(f) = facets_val {
            Facets {
                audience: f
                    .get("audience")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                jtbd: f.get("jtbd").and_then(|v| v.as_str()).unwrap_or("").into(),
                differentiator: f
                    .get("differentiator")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                monetization: f
                    .get("monetization")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                distribution: f
                    .get("distribution")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                risks: f.get("risks").and_then(|v| v.as_str()).unwrap_or("").into(),
            }
        } else {
            Facets {
                audience: "".into(),
                jtbd: "".into(),
                differentiator: "".into(),
                monetization: "".into(),
                distribution: "".into(),
                risks: "".into(),
            }
        };

        ideas.push(Idea::new(title, summary, facets, gen, Origin::Generated));
    }

    Ok(ideas)
}

/// Parse critic output and apply scores to ideas
pub fn apply_critic_patches(ideas: &mut [Idea], output: &Value) -> Result<()> {
    let patches = output
        .get("patches")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Expected 'patches' array in output"))?;

    for patch in patches {
        let id_str = patch
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Patch missing 'id'"))?;

        let id: uuid::Uuid = id_str.parse()?;

        if let Some(idea) = ideas.iter_mut().find(|i| i.id == id) {
            if let Some(scores_val) = patch.get("scores") {
                idea.scores = Scores {
                    feasibility: scores_val
                        .get("feasibility")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32,
                    speed_to_value: scores_val
                        .get("speed_to_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32,
                    differentiation: scores_val
                        .get("differentiation")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32,
                    market_size: scores_val
                        .get("market_size")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32,
                    distribution: scores_val
                        .get("distribution")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32,
                    moats: scores_val
                        .get("moats")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32,
                    risk: scores_val
                        .get("risk")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32,
                    clarity: scores_val
                        .get("clarity")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32,
                };
            }

            idea.overall_score = patch
                .get("overall_score")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32);

            idea.judge_notes = patch
                .get("judge_notes")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mock_provider_generate() {
        let provider = MockLlmProvider::new();
        let task = LlmTask::Generate {
            prompt: "Test".into(),
            count: 3,
        };

        let result = provider.generate_json(task, &PathBuf::new()).unwrap();
        let ideas = result.get("ideas").unwrap().as_array().unwrap();

        assert_eq!(ideas.len(), 3);
        assert!(ideas[0].get("title").is_some());
        assert!(ideas[0].get("facets").is_some());
    }

    #[test]
    fn test_mock_provider_critic() {
        let provider = MockLlmProvider::new();
        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();

        let task = LlmTask::Critic {
            ideas: vec![
                (id1, "Idea 1".into(), "Summary 1".into()),
                (id2, "Idea 2".into(), "Summary 2".into()),
            ],
        };

        let result = provider.generate_json(task, &PathBuf::new()).unwrap();
        let patches = result.get("patches").unwrap().as_array().unwrap();

        assert_eq!(patches.len(), 2);
        assert!(patches[0].get("scores").is_some());
        assert!(patches[0].get("overall_score").is_some());
    }

    #[test]
    fn test_parse_generated_ideas() {
        let output = serde_json::json!({
            "ideas": [
                {
                    "title": "Test Idea",
                    "summary": "A test idea",
                    "facets": {
                        "audience": "devs",
                        "jtbd": "testing",
                        "differentiator": "unique",
                        "monetization": "free",
                        "distribution": "github",
                        "risks": "none"
                    }
                }
            ]
        });

        let ideas = parse_generated_ideas(&output, 1).unwrap();

        assert_eq!(ideas.len(), 1);
        assert_eq!(ideas[0].title, "Test Idea");
        assert_eq!(ideas[0].facets.audience, "devs");
        assert_eq!(ideas[0].gen, 1);
        assert_eq!(ideas[0].origin, Origin::Generated);
    }

    #[test]
    fn test_apply_critic_patches() {
        let facets = Facets {
            audience: "test".into(),
            jtbd: "test".into(),
            differentiator: "test".into(),
            monetization: "test".into(),
            distribution: "test".into(),
            risks: "test".into(),
        };

        let mut ideas = vec![Idea::new(
            "Test".into(),
            "Summary".into(),
            facets,
            1,
            Origin::Generated,
        )];

        let id = ideas[0].id;
        let patches = serde_json::json!({
            "patches": [
                {
                    "id": id.to_string(),
                    "scores": {
                        "feasibility": 8.0,
                        "speed_to_value": 7.0,
                        "differentiation": 6.0,
                        "market_size": 9.0,
                        "distribution": 7.0,
                        "moats": 5.0,
                        "risk": 3.0,
                        "clarity": 8.0
                    },
                    "overall_score": 7.5,
                    "judge_notes": "Good idea"
                }
            ]
        });

        apply_critic_patches(&mut ideas, &patches).unwrap();

        assert_eq!(ideas[0].scores.feasibility, 8.0);
        assert_eq!(ideas[0].overall_score, Some(7.5));
        assert_eq!(ideas[0].judge_notes, Some("Good idea".into()));
    }
}
