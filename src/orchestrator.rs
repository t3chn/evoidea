use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// List all runs in the given directory
pub fn list_runs(dir: &str) -> Result<()> {
    let runs_path = PathBuf::from(dir);

    if !runs_path.exists() {
        println!("No runs directory found at: {}", dir);
        return Ok(());
    }

    let mut runs: Vec<(String, String, Option<f32>)> = Vec::new();

    for entry in fs::read_dir(&runs_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let run_id = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let final_path = path.join("final.json");
            let state_path = path.join("state.json");

            let (status, best_score) = if final_path.exists() {
                // Try to parse final.json to get best score
                let score = fs::read_to_string(&final_path)
                    .ok()
                    .and_then(|content| {
                        serde_json::from_str::<serde_json::Value>(&content).ok()
                    })
                    .and_then(|v| {
                        v.get("best_idea")
                            .and_then(|b| b.get("overall_score"))
                            .and_then(|s| s.as_f64())
                            .map(|s| s as f32)
                    });
                ("complete".to_string(), score)
            } else if state_path.exists() {
                ("in_progress".to_string(), None)
            } else {
                ("unknown".to_string(), None)
            };

            runs.push((run_id, status, best_score));
        }
    }

    if runs.is_empty() {
        println!("No runs found in: {}", dir);
        return Ok(());
    }

    // Sort by run_id (newest first, assuming timestamp-based IDs)
    runs.sort_by(|a, b| b.0.cmp(&a.0));

    println!("Runs in {}:", dir);
    println!("{:<30} {:<12} BEST SCORE", "RUN ID", "STATUS");
    println!("{}", "-".repeat(55));

    for (run_id, status, best_score) in runs {
        let score_str = best_score
            .map(|s| format!("{:.2}", s))
            .unwrap_or_else(|| "-".to_string());
        println!("{:<30} {:<12} {}", run_id, status, score_str);
    }

    Ok(())
}

/// Show run results
pub fn show_run(run_id: &str, format: &str) -> Result<()> {
    let final_path = PathBuf::from("runs").join(run_id).join("final.json");

    if !final_path.exists() {
        // Check if run exists at all
        let state_path = PathBuf::from("runs").join(run_id).join("state.json");
        if state_path.exists() {
            let content = fs::read_to_string(&state_path)?;
            let state: serde_json::Value = serde_json::from_str(&content)?;

            println!("Run {} has not completed yet.", run_id);
            if let Some(iteration) = state.get("iteration") {
                println!("Current iteration: {}", iteration);
            }
            if let Some(ideas) = state.get("ideas").and_then(|i| i.as_array()) {
                let active = ideas.iter()
                    .filter(|i| i.get("status").and_then(|s| s.as_str()) == Some("active"))
                    .count();
                println!("Active ideas: {}", active);
            }
            if let Some(best_score) = state.get("best_score") {
                println!("Best score: {}", best_score);
            }
            return Ok(());
        }

        anyhow::bail!("Run {} not found", run_id);
    }

    let content = fs::read_to_string(&final_path)?;

    match format {
        "json" => println!("{}", content),
        "md" => {
            let result: serde_json::Value = serde_json::from_str(&content)?;

            if let Some(best) = result.get("best_idea") {
                let title = best.get("title").and_then(|t| t.as_str()).unwrap_or("Unknown");
                let summary = best.get("summary").and_then(|s| s.as_str()).unwrap_or("");
                let score = best.get("overall_score")
                    .and_then(|s| s.as_f64())
                    .map(|s| format!("{:.2}", s))
                    .unwrap_or_else(|| "-".to_string());

                println!("# Best Idea: {}\n", title);
                println!("**Score:** {}/10\n", score);
                println!("{}\n", summary);

                if let Some(facets) = best.get("facets") {
                    println!("## Details\n");
                    if let Some(audience) = facets.get("audience").and_then(|a| a.as_str()) {
                        println!("**Audience:** {}", audience);
                    }
                    if let Some(jtbd) = facets.get("jtbd").and_then(|j| j.as_str()) {
                        println!("**Problem:** {}", jtbd);
                    }
                    if let Some(diff) = facets.get("differentiator").and_then(|d| d.as_str()) {
                        println!("**Unique:** {}", diff);
                    }
                    if let Some(mon) = facets.get("monetization").and_then(|m| m.as_str()) {
                        println!("**Monetization:** {}", mon);
                    }
                    if let Some(dist) = facets.get("distribution").and_then(|d| d.as_str()) {
                        println!("**Distribution:** {}", dist);
                    }
                    if let Some(risks) = facets.get("risks").and_then(|r| r.as_str()) {
                        println!("**Risks:** {}", risks);
                    }
                }
            }

            if let Some(runner_up) = result.get("runner_up") {
                if !runner_up.is_null() {
                    let title = runner_up.get("title").and_then(|t| t.as_str()).unwrap_or("Unknown");
                    println!("\n## Runner Up: {}", title);
                }
            }
        }
        _ => println!("{}", content),
    }

    Ok(())
}

/// Validate run artifacts
pub fn validate_run(run_id: &str) -> Result<()> {
    let run_dir = PathBuf::from("runs").join(run_id);

    if !run_dir.exists() {
        anyhow::bail!("Run directory not found: {}", run_id);
    }

    let mut errors = Vec::new();

    // Validate config exists
    let config_path = run_dir.join("config.json");
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(config) => {
                        let prompt = config.get("prompt")
                            .and_then(|p| p.as_str())
                            .unwrap_or("?");
                        let truncated = if prompt.len() > 30 { &prompt[..30] } else { prompt };
                        println!("Config: OK (prompt: {}...)", truncated);
                    }
                    Err(e) => errors.push(format!("Config JSON invalid: {}", e)),
                }
            }
            Err(e) => errors.push(format!("Config read error: {}", e)),
        }
    } else {
        errors.push("Config: MISSING".to_string());
    }

    // Validate state
    let state_path = run_dir.join("state.json");
    if state_path.exists() {
        match fs::read_to_string(&state_path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(state) => {
                        let iteration = state.get("iteration").and_then(|i| i.as_u64()).unwrap_or(0);
                        let ideas_count = state.get("ideas")
                            .and_then(|i| i.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0);
                        println!("State: OK (iteration: {}, ideas: {})", iteration, ideas_count);

                        // Validate idea invariants
                        if let Some(ideas) = state.get("ideas").and_then(|i| i.as_array()) {
                            for idea in ideas {
                                let id = idea.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                                let origin = idea.get("origin").and_then(|o| o.as_str()).unwrap_or("?");
                                let parents = idea.get("parents").and_then(|p| p.as_array());
                                let has_parents = parents.map(|p| !p.is_empty()).unwrap_or(false);

                                match origin {
                                    "generated" => {
                                        if has_parents {
                                            errors.push(format!("Idea {} (generated) has parents", id));
                                        }
                                    }
                                    "refined" | "crossover" | "mutated" => {
                                        if !has_parents {
                                            errors.push(format!("Idea {} ({}) has no parents", id, origin));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Err(e) => errors.push(format!("State JSON invalid: {}", e)),
                }
            }
            Err(e) => errors.push(format!("State read error: {}", e)),
        }
    } else {
        errors.push("State: MISSING".to_string());
    }

    // Validate history
    let history_path = run_dir.join("history.ndjson");
    if history_path.exists() {
        match fs::read_to_string(&history_path) {
            Ok(content) => {
                let event_count = content.lines().count();
                println!("History: OK ({} events)", event_count);
            }
            Err(e) => errors.push(format!("History read error: {}", e)),
        }
    } else {
        errors.push("History: MISSING".to_string());
    }

    // Validate final if exists
    let final_path = run_dir.join("final.json");
    if final_path.exists() {
        match fs::read_to_string(&final_path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(result) => {
                        let title = result.get("best_idea")
                            .and_then(|b| b.get("title"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("?");
                        println!("Final: OK (best: {})", title);
                    }
                    Err(e) => errors.push(format!("Final JSON invalid: {}", e)),
                }
            }
            Err(e) => errors.push(format!("Final read error: {}", e)),
        }
    } else {
        println!("Final: NOT YET (run in progress)");
    }

    // Report invariant errors
    if errors.is_empty() {
        println!("Invariants: OK");
    } else {
        println!("Errors: {} found", errors.len());
        for err in &errors {
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
    fn test_list_runs_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = list_runs(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_runs_nonexistent_dir() {
        let result = list_runs("/nonexistent/path");
        assert!(result.is_ok()); // Should handle gracefully
    }
}
