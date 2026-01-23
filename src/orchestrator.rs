use anyhow::{Context, Result};
use std::path::PathBuf;
use uuid::Uuid;

use crate::config::RunConfig;
use crate::data::{Event, EventType, State};
use crate::llm::{LlmProvider, MockLlmProvider};
use crate::phase::{
    CriticPhase, FinalPhase, GeneratePhase, Phase, PhaseContext, RefinePhase, SelectPhase,
};
use crate::scoring::{check_max_rounds_stop, check_stagnation_stop, check_threshold_stop};
use crate::storage::{FileStorage, Storage};

pub struct Orchestrator {
    config: RunConfig,
    storage: Box<dyn Storage>,
    llm: Box<dyn LlmProvider>,
    state: State,
    phases: Vec<Box<dyn Phase>>,
    schema_dir: PathBuf,
}

impl Orchestrator {
    pub fn new(config: RunConfig) -> Result<Self> {
        let storage = Box::new(FileStorage::new(&config.output_dir));
        let run_id = storage.init_run(&config)?;

        let llm: Box<dyn LlmProvider> = match config.mode.as_str() {
            "mock" => Box::new(MockLlmProvider::new()),
            _ => Box::new(MockLlmProvider::new()), // TODO: Add other providers
        };

        let state = State::new(run_id);

        // MVP pipeline: Generate -> Critic -> Select -> Refine
        let phases: Vec<Box<dyn Phase>> = vec![
            Box::new(GeneratePhase),
            Box::new(CriticPhase),
            Box::new(SelectPhase),
            Box::new(RefinePhase { top_k: 2 }),
        ];

        let schema_dir = PathBuf::from("schemas");

        Ok(Self {
            config,
            storage,
            llm,
            state,
            phases,
            schema_dir,
        })
    }

    pub fn resume(run_id: &str, additional_rounds: Option<u32>) -> Result<Self> {
        let run_uuid: Uuid = run_id.parse().context("Invalid run ID")?;

        // Find the run directory
        let storage = Box::new(FileStorage::new("runs"));
        let mut config = storage.load_config(&run_uuid)?;

        if let Some(extra) = additional_rounds {
            config.max_rounds += extra;
        }

        let state = storage.load_state(&run_uuid)?;

        let llm: Box<dyn LlmProvider> = match config.mode.as_str() {
            "mock" => Box::new(MockLlmProvider::new()),
            _ => Box::new(MockLlmProvider::new()),
        };

        let phases: Vec<Box<dyn Phase>> = vec![
            Box::new(GeneratePhase),
            Box::new(CriticPhase),
            Box::new(SelectPhase),
            Box::new(RefinePhase { top_k: 2 }),
        ];

        let schema_dir = PathBuf::from("schemas");

        Ok(Self {
            config,
            storage,
            llm,
            state,
            phases,
            schema_dir,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        tracing::info!(
            run_id = %self.state.run_id,
            max_rounds = self.config.max_rounds,
            "Starting evolution loop"
        );

        loop {
            self.state.iteration += 1;
            tracing::info!(iteration = self.state.iteration, "Starting iteration");

            // Run phases
            let ctx = PhaseContext {
                config: &self.config,
                storage: self.storage.as_ref(),
                llm: self.llm.as_ref(),
                schema_dir: &self.schema_dir,
            };

            for phase in &self.phases {
                tracing::debug!(phase = phase.name(), "Running phase");
                self.state = phase.run(self.state.clone(), &ctx)?;
                self.storage.save_state(&self.state)?;
            }

            // Check stop conditions
            if check_threshold_stop(self.state.best_score, self.config.score_threshold) {
                tracing::info!(
                    best_score = ?self.state.best_score,
                    threshold = self.config.score_threshold,
                    "Stopping: threshold reached"
                );
                self.record_stop("threshold")?;
                break;
            }

            if check_stagnation_stop(
                self.state.stagnation_counter,
                self.config.stagnation_patience,
            ) {
                tracing::info!(
                    stagnation = self.state.stagnation_counter,
                    patience = self.config.stagnation_patience,
                    "Stopping: stagnation"
                );
                self.record_stop("stagnation")?;
                break;
            }

            if check_max_rounds_stop(self.state.iteration, self.config.max_rounds) {
                tracing::info!(
                    iteration = self.state.iteration,
                    max_rounds = self.config.max_rounds,
                    "Stopping: max rounds reached"
                );
                self.record_stop("max_rounds")?;
                break;
            }
        }

        // Compose final result
        let final_phase = FinalPhase;
        let ctx = PhaseContext {
            config: &self.config,
            storage: self.storage.as_ref(),
            llm: self.llm.as_ref(),
            schema_dir: &self.schema_dir,
        };
        self.state = final_phase.run(self.state.clone(), &ctx)?;
        self.storage.save_state(&self.state)?;

        tracing::info!(
            run_id = %self.state.run_id,
            iterations = self.state.iteration,
            best_score = ?self.state.best_score,
            "Evolution complete"
        );

        // Print result location
        println!("Run complete: runs/{}/final.json", self.state.run_id);

        Ok(())
    }

    fn record_stop(&self, reason: &str) -> Result<()> {
        let event = Event::new(
            self.state.iteration,
            EventType::Stopped,
            serde_json::json!({
                "reason": reason,
                "best_score": self.state.best_score,
                "best_idea_id": self.state.best_idea_id
            }),
        );
        self.storage.append_event(&self.state.run_id, &event)?;
        Ok(())
    }
}

pub fn show_run(run_id: &str, format: &str) -> Result<()> {
    let run_uuid: Uuid = run_id.parse().context("Invalid run ID")?;
    let storage = FileStorage::new("runs");

    let final_path = PathBuf::from("runs").join(run_id).join("final.json");

    if !final_path.exists() {
        println!("Run {} has not completed yet.", run_id);
        let state = storage.load_state(&run_uuid)?;
        println!("Current iteration: {}", state.iteration);
        println!("Active ideas: {}", state.active_ideas().count());
        println!("Best score: {:?}", state.best_score);
        return Ok(());
    }

    let content = std::fs::read_to_string(&final_path)?;

    match format {
        "json" => println!("{}", content),
        "md" => {
            let result: crate::data::FinalResult = serde_json::from_str(&content)?;
            println!("# Best Idea: {}\n", result.best.title);
            println!("{}\n", result.best.summary);
            println!("## Scores");
            println!("- Overall: {:.2}", result.best.overall_score);
            println!("- Feasibility: {:.1}", result.best.scores.feasibility);
            println!("- Speed to Value: {:.1}", result.best.scores.speed_to_value);
            println!(
                "- Differentiation: {:.1}",
                result.best.scores.differentiation
            );
            println!("- Market Size: {:.1}", result.best.scores.market_size);
            println!("- Distribution: {:.1}", result.best.scores.distribution);
            println!("- Moats: {:.1}", result.best.scores.moats);
            println!("- Risk: {:.1}", result.best.scores.risk);
            println!("- Clarity: {:.1}", result.best.scores.clarity);
            println!("\n## Why It Won");
            for reason in &result.best.why_won {
                println!("- {}", reason);
            }
            if !result.runners_up.is_empty() {
                println!("\n## Runners Up");
                for runner in &result.runners_up {
                    println!("- {} (score: {:.2})", runner.title, runner.overall_score);
                }
            }
        }
        _ => println!("{}", content),
    }

    Ok(())
}

pub fn validate_run(run_id: &str) -> Result<()> {
    let run_uuid: Uuid = run_id.parse().context("Invalid run ID")?;
    let storage = FileStorage::new("runs");

    // Validate config exists
    let config = storage.load_config(&run_uuid)?;
    println!(
        "Config: OK (prompt: {}...)",
        &config.prompt[..config.prompt.len().min(30)]
    );

    // Validate state
    let state = storage.load_state(&run_uuid)?;
    println!(
        "State: OK (iteration: {}, ideas: {})",
        state.iteration,
        state.ideas.len()
    );

    // Validate history
    let history_path = PathBuf::from("runs").join(run_id).join("history.ndjson");
    let history_content = std::fs::read_to_string(&history_path)?;
    let event_count = history_content.lines().count();
    println!("History: OK ({} events)", event_count);

    // Validate final if exists
    let final_path = PathBuf::from("runs").join(run_id).join("final.json");
    if final_path.exists() {
        let final_content = std::fs::read_to_string(&final_path)?;
        let result: crate::data::FinalResult = serde_json::from_str(&final_content)?;
        println!("Final: OK (best: {})", result.best.title);
    } else {
        println!("Final: NOT YET (run in progress)");
    }

    // Validate invariants
    let mut errors = Vec::new();

    for idea in &state.ideas {
        // Check origin/parents invariant
        match idea.origin {
            crate::data::Origin::Generated => {
                if !idea.parents.is_empty() {
                    errors.push(format!("Idea {} (generated) has parents", idea.id));
                }
            }
            crate::data::Origin::Crossover
            | crate::data::Origin::Mutated
            | crate::data::Origin::Refined => {
                if idea.parents.is_empty() {
                    errors.push(format!(
                        "Idea {} ({:?}) has no parents",
                        idea.id, idea.origin
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        println!("Invariants: OK");
    } else {
        println!("Invariants: {} errors", errors.len());
        for err in errors {
            println!("  - {}", err);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_orchestrator_mock_run() {
        let temp_dir = TempDir::new().unwrap();
        let config = RunConfig::new(
            "Generate startup ideas".into(),
            "mock".into(),
            2, // Just 2 rounds for quick test
            6,
            2,
            9.5, // High threshold so we don't stop early
            10,  // High patience
            temp_dir.path().to_string_lossy().into(),
        );

        let mut orchestrator = Orchestrator::new(config).unwrap();
        orchestrator.run().unwrap();

        // Verify final.json was created
        let final_path = temp_dir
            .path()
            .join(orchestrator.state.run_id.to_string())
            .join("final.json");
        assert!(final_path.exists());

        // Verify it contains valid JSON with a best idea
        let content = std::fs::read_to_string(final_path).unwrap();
        let result: crate::data::FinalResult = serde_json::from_str(&content).unwrap();
        assert!(!result.best.title.is_empty());
    }

    #[test]
    fn test_orchestrator_stops_on_max_rounds() {
        let temp_dir = TempDir::new().unwrap();
        let config = RunConfig::new(
            "Test".into(),
            "mock".into(),
            3,
            6,
            2,
            10.0, // Impossible threshold
            100,  // Won't stagnate
            temp_dir.path().to_string_lossy().into(),
        );

        let mut orchestrator = Orchestrator::new(config).unwrap();
        orchestrator.run().unwrap();

        assert_eq!(orchestrator.state.iteration, 3);
    }

    #[test]
    fn test_orchestrator_creates_all_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let config = RunConfig::new(
            "Test".into(),
            "mock".into(),
            1,
            4,
            2,
            10.0,
            100,
            temp_dir.path().to_string_lossy().into(),
        );

        let mut orchestrator = Orchestrator::new(config).unwrap();
        orchestrator.run().unwrap();

        let run_dir = temp_dir.path().join(orchestrator.state.run_id.to_string());

        assert!(run_dir.join("config.json").exists());
        assert!(run_dir.join("state.json").exists());
        assert!(run_dir.join("history.ndjson").exists());
        assert!(run_dir.join("final.json").exists());
    }
}
