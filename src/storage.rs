use anyhow::{Context, Result};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::config::RunConfig;
use crate::data::{Event, FinalResult, State};

pub trait Storage: Send + Sync {
    fn init_run(&self, config: &RunConfig) -> Result<Uuid>;
    fn load_config(&self, run_id: &Uuid) -> Result<RunConfig>;
    fn load_state(&self, run_id: &Uuid) -> Result<State>;
    fn save_state(&self, state: &State) -> Result<()>;
    fn append_event(&self, run_id: &Uuid, event: &Event) -> Result<()>;
    fn save_final(&self, result: &FinalResult) -> Result<()>;
}

pub struct FileStorage {
    base_dir: PathBuf,
}

impl FileStorage {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    fn run_dir(&self, run_id: &Uuid) -> PathBuf {
        self.base_dir.join(run_id.to_string())
    }

    fn config_path(&self, run_id: &Uuid) -> PathBuf {
        self.run_dir(run_id).join("config.json")
    }

    fn state_path(&self, run_id: &Uuid) -> PathBuf {
        self.run_dir(run_id).join("state.json")
    }

    fn history_path(&self, run_id: &Uuid) -> PathBuf {
        self.run_dir(run_id).join("history.ndjson")
    }

    fn final_path(&self, run_id: &Uuid) -> PathBuf {
        self.run_dir(run_id).join("final.json")
    }
}

impl Storage for FileStorage {
    fn init_run(&self, config: &RunConfig) -> Result<Uuid> {
        let run_id = config.run_id;
        let run_dir = self.run_dir(&run_id);

        fs::create_dir_all(&run_dir)
            .with_context(|| format!("Failed to create run directory: {:?}", run_dir))?;

        let config_path = self.config_path(&run_id);
        let config_json = serde_json::to_string_pretty(config)?;
        fs::write(&config_path, config_json)
            .with_context(|| format!("Failed to write config: {:?}", config_path))?;

        let state = State::new(run_id);
        self.save_state(&state)?;

        // Create empty history file
        fs::File::create(self.history_path(&run_id))?;

        Ok(run_id)
    }

    fn load_config(&self, run_id: &Uuid) -> Result<RunConfig> {
        let path = self.config_path(run_id);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config: {:?}", path))?;
        let config: RunConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn load_state(&self, run_id: &Uuid) -> Result<State> {
        let path = self.state_path(run_id);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read state: {:?}", path))?;
        let state: State = serde_json::from_str(&content)?;
        Ok(state)
    }

    fn save_state(&self, state: &State) -> Result<()> {
        let path = self.state_path(&state.run_id);
        let json = serde_json::to_string_pretty(state)?;
        fs::write(&path, json).with_context(|| format!("Failed to write state: {:?}", path))?;
        Ok(())
    }

    fn append_event(&self, run_id: &Uuid, event: &Event) -> Result<()> {
        let path = self.history_path(run_id);
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open history: {:?}", path))?;

        let mut writer = BufWriter::new(file);
        let json = serde_json::to_string(event)?;
        writeln!(writer, "{}", json)?;
        writer.flush()?;

        Ok(())
    }

    fn save_final(&self, result: &FinalResult) -> Result<()> {
        let path = self.final_path(&result.run_id);
        let json = serde_json::to_string_pretty(result)?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write final result: {:?}", path))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{EventType, Facets, FinalBest, RunnerUp, Scores};
    use tempfile::TempDir;

    fn make_test_config() -> RunConfig {
        RunConfig::new(
            "Test prompt".into(),
            "mock".into(),
            6,
            12,
            4,
            8.7,
            2,
            "runs".into(),
        )
    }

    #[test]
    fn test_init_run_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        let config = make_test_config();
        let run_id = storage.init_run(&config).unwrap();

        let run_dir = temp_dir.path().join(run_id.to_string());
        assert!(run_dir.exists());
        assert!(run_dir.join("config.json").exists());
        assert!(run_dir.join("state.json").exists());
        assert!(run_dir.join("history.ndjson").exists());
    }

    #[test]
    fn test_load_config_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        let config = make_test_config();
        let run_id = storage.init_run(&config).unwrap();

        let loaded = storage.load_config(&run_id).unwrap();
        assert_eq!(config.prompt, loaded.prompt);
        assert_eq!(config.max_rounds, loaded.max_rounds);
    }

    #[test]
    fn test_save_load_state_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        let config = make_test_config();
        let run_id = storage.init_run(&config).unwrap();

        let mut state = storage.load_state(&run_id).unwrap();
        state.iteration = 3;
        state.stagnation_counter = 1;
        storage.save_state(&state).unwrap();

        let loaded = storage.load_state(&run_id).unwrap();
        assert_eq!(state.iteration, loaded.iteration);
        assert_eq!(state.stagnation_counter, loaded.stagnation_counter);
    }

    #[test]
    fn test_append_event() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        let config = make_test_config();
        let run_id = storage.init_run(&config).unwrap();

        let event1 = Event::new(1, EventType::Generated, serde_json::json!({"count": 5}));
        let event2 = Event::new(1, EventType::Scored, serde_json::json!({"count": 5}));

        storage.append_event(&run_id, &event1).unwrap();
        storage.append_event(&run_id, &event2).unwrap();

        let history_path = temp_dir
            .path()
            .join(run_id.to_string())
            .join("history.ndjson");
        let content = fs::read_to_string(history_path).unwrap();
        let lines: Vec<_> = content.lines().collect();

        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_save_final() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        let config = make_test_config();
        let run_id = storage.init_run(&config).unwrap();

        let facets = Facets {
            audience: "test".into(),
            jtbd: "test".into(),
            differentiator: "test".into(),
            monetization: "test".into(),
            distribution: "test".into(),
            risks: "test".into(),
        };

        let final_result = FinalResult {
            run_id,
            best: FinalBest {
                idea_id: Uuid::new_v4(),
                title: "Best Idea".into(),
                summary: "The best".into(),
                facets,
                scores: Scores::default(),
                overall_score: 9.0,
                why_won: vec!["Great feasibility".into()],
            },
            runners_up: vec![RunnerUp {
                idea_id: Uuid::new_v4(),
                title: "Second".into(),
                overall_score: 8.0,
            }],
        };

        storage.save_final(&final_result).unwrap();

        let final_path = temp_dir.path().join(run_id.to_string()).join("final.json");
        assert!(final_path.exists());

        let content = fs::read_to_string(final_path).unwrap();
        let loaded: FinalResult = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.best.title, "Best Idea");
    }
}
