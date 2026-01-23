use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Read-only file storage for accessing run artifacts
#[allow(dead_code)]
pub struct FileStorage {
    base_dir: PathBuf,
}

#[allow(dead_code)]
impl FileStorage {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    fn run_dir(&self, run_id: &str) -> PathBuf {
        self.base_dir.join(run_id)
    }

    pub fn config_path(&self, run_id: &str) -> PathBuf {
        self.run_dir(run_id).join("config.json")
    }

    pub fn state_path(&self, run_id: &str) -> PathBuf {
        self.run_dir(run_id).join("state.json")
    }

    pub fn history_path(&self, run_id: &str) -> PathBuf {
        self.run_dir(run_id).join("history.ndjson")
    }

    pub fn final_path(&self, run_id: &str) -> PathBuf {
        self.run_dir(run_id).join("final.json")
    }

    pub fn load_config(&self, run_id: &str) -> Result<serde_json::Value> {
        let path = self.config_path(run_id);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config: {:?}", path))?;
        let config: serde_json::Value = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn load_state(&self, run_id: &str) -> Result<serde_json::Value> {
        let path = self.state_path(run_id);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read state: {:?}", path))?;
        let state: serde_json::Value = serde_json::from_str(&content)?;
        Ok(state)
    }

    pub fn load_final(&self, run_id: &str) -> Result<serde_json::Value> {
        let path = self.final_path(run_id);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read final: {:?}", path))?;
        let result: serde_json::Value = serde_json::from_str(&content)?;
        Ok(result)
    }

    pub fn run_exists(&self, run_id: &str) -> bool {
        self.run_dir(run_id).exists()
    }

    pub fn has_final(&self, run_id: &str) -> bool {
        self.final_path(run_id).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths() {
        let storage = FileStorage::new("runs");
        assert_eq!(
            storage.config_path("test-run"),
            PathBuf::from("runs/test-run/config.json")
        );
        assert_eq!(
            storage.state_path("test-run"),
            PathBuf::from("runs/test-run/state.json")
        );
        assert_eq!(
            storage.final_path("test-run"),
            PathBuf::from("runs/test-run/final.json")
        );
    }
}
