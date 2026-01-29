use anyhow::Result;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::fs;
use std::io::{self, Write};
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
            let run_id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let final_path = path.join("final.json");
            let state_path = path.join("state.json");

            let (status, best_score) = if final_path.exists() {
                // Try to parse final.json to get best score
                let score = fs::read_to_string(&final_path)
                    .ok()
                    .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
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
                let active = ideas
                    .iter()
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
                let title = best
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Unknown");
                let summary = best.get("summary").and_then(|s| s.as_str()).unwrap_or("");
                let score = best
                    .get("overall_score")
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
                    let title = runner_up
                        .get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("Unknown");
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
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(config) => {
                    let prompt = config.get("prompt").and_then(|p| p.as_str()).unwrap_or("?");
                    let truncated = if prompt.len() > 30 {
                        &prompt[..30]
                    } else {
                        prompt
                    };
                    println!("Config: OK (prompt: {}...)", truncated);
                }
                Err(e) => errors.push(format!("Config JSON invalid: {}", e)),
            },
            Err(e) => errors.push(format!("Config read error: {}", e)),
        }
    } else {
        errors.push("Config: MISSING".to_string());
    }

    // Validate state
    let state_path = run_dir.join("state.json");
    if state_path.exists() {
        match fs::read_to_string(&state_path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(state) => {
                    let iteration = state.get("iteration").and_then(|i| i.as_u64()).unwrap_or(0);
                    let ideas_count = state
                        .get("ideas")
                        .and_then(|i| i.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    println!(
                        "State: OK (iteration: {}, ideas: {})",
                        iteration, ideas_count
                    );

                    errors.extend(validate_state_idea_invariants(&state));
                }
                Err(e) => errors.push(format!("State JSON invalid: {}", e)),
            },
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
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(result) => {
                    let title = result
                        .get("best_idea")
                        .and_then(|b| b.get("title"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("?");
                    println!("Final: OK (best: {})", title);
                }
                Err(e) => errors.push(format!("Final JSON invalid: {}", e)),
            },
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

fn validate_state_idea_invariants(state: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();

    let Some(ideas) = state.get("ideas").and_then(|i| i.as_array()) else {
        return errors;
    };

    for idea in ideas {
        let id = idea.get("id").and_then(|i| i.as_str()).unwrap_or("?");
        let origin = idea.get("origin").and_then(|o| o.as_str()).unwrap_or("?");
        let parents = idea.get("parents").and_then(|p| p.as_array());
        let has_parents = parents.map(|p| !p.is_empty()).unwrap_or(false);
        let status = idea.get("status").and_then(|s| s.as_str()).unwrap_or("?");

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

        // Active ideas should always be scored (tournament/profile export depends on it).
        if status == "active" {
            if extract_scores(idea).is_none() {
                errors.push(format!("Idea {} (active) has missing/invalid scores", id));
            }
            if idea.get("overall_score").and_then(|s| s.as_f64()).is_none() {
                errors.push(format!(
                    "Idea {} (active) has missing/invalid overall_score",
                    id
                ));
            }
        }
    }

    errors
}

/// Export run results in various preset formats
pub fn export_run(run_id: &str, preset: &str) -> Result<()> {
    let run_dir = PathBuf::from("runs").join(run_id);
    let final_path = run_dir.join("final.json");
    let config_path = run_dir.join("config.json");
    let state_path = run_dir.join("state.json");

    if !final_path.exists() {
        anyhow::bail!("Run {} has no final.json (not completed yet)", run_id);
    }

    let final_content = fs::read_to_string(&final_path)?;
    let result: serde_json::Value = serde_json::from_str(&final_content)?;

    let config: Option<serde_json::Value> = if config_path.exists() {
        Some(serde_json::from_str(&fs::read_to_string(&config_path)?)?)
    } else {
        None
    };

    let state: Option<serde_json::Value> = if state_path.exists() {
        Some(serde_json::from_str(&fs::read_to_string(&state_path)?)?)
    } else {
        None
    };

    let (output, filename) = match preset {
        "landing" => (generate_landing_page(&result, config.as_ref())?, "landing.md"),
        "decision-log" => (generate_decision_log(&result, config.as_ref(), state.as_ref())?, "decision-log.md"),
        "stakeholder-brief" => (generate_stakeholder_brief(&result, config.as_ref())?, "stakeholder-brief.md"),
        "changelog-entry" => (generate_changelog_entry(&result, config.as_ref())?, "changelog-entry.md"),
        _ => anyhow::bail!("Unknown preset: {} (supported: landing, decision-log, stakeholder-brief, changelog-entry)", preset),
    };

    // Create exports directory
    let exports_dir = run_dir.join("exports");
    fs::create_dir_all(&exports_dir)?;

    let output_path = exports_dir.join(filename);
    fs::write(&output_path, &output)?;

    println!("Exported to: {}", output_path.display());
    println!();
    println!("{}", output);

    Ok(())
}

fn generate_landing_page(
    result: &serde_json::Value,
    config: Option<&serde_json::Value>,
) -> Result<String> {
    // Handle both "best_idea" and "best" formats
    let best = result
        .get("best_idea")
        .or_else(|| result.get("best"))
        .ok_or_else(|| anyhow::anyhow!("No best_idea or best in final.json"))?;

    let title = best
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown Product");
    let summary = best.get("summary").and_then(|s| s.as_str()).unwrap_or("");
    // Try multiple paths for score
    let score = best
        .get("overall_score")
        .or_else(|| best.get("scores").and_then(|s| s.get("overall")))
        .and_then(|s| s.as_f64())
        .map(|s| format!("{:.1}", s))
        .unwrap_or_else(|| "N/A".to_string());

    let facets = best.get("facets");
    let audience = facets
        .and_then(|f| f.get("audience"))
        .and_then(|a| a.as_str())
        .unwrap_or("");
    let jtbd = facets
        .and_then(|f| f.get("jtbd"))
        .and_then(|j| j.as_str())
        .unwrap_or("");
    let differentiator = facets
        .and_then(|f| f.get("differentiator"))
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let monetization = facets
        .and_then(|f| f.get("monetization"))
        .and_then(|m| m.as_str())
        .unwrap_or("");
    let distribution = facets
        .and_then(|f| f.get("distribution"))
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let risks = facets
        .and_then(|f| f.get("risks"))
        .and_then(|r| r.as_str())
        .unwrap_or("");

    let prompt = config
        .and_then(|c| c.get("prompt"))
        .and_then(|p| p.as_str())
        .unwrap_or("");

    // Extract product name (first part before colon if present)
    let product_name = title.split(':').next().unwrap_or(title).trim();

    // Generate hero headline
    let hero = format!("# {}", product_name);

    // Generate tagline from summary (first sentence or truncated)
    let tagline = summary.split('.').next().unwrap_or(summary).trim();

    let mut output = String::new();

    // Header with metadata
    output.push_str(&format!(
        "<!-- Source: {} | Score: {}/10 -->\n",
        run_id_from_result(result),
        score
    ));
    if !prompt.is_empty() {
        output.push_str(&format!("<!-- Prompt: {} -->\n", prompt));
    }
    output.push('\n');

    // Hero section
    output.push_str(&hero);
    output.push_str("\n\n");
    output.push_str(&format!("**{}**\n\n", tagline));

    // Value proposition
    output.push_str("## The Problem\n\n");
    output.push_str(&format!("{}\n\n", jtbd));

    // Benefits (3 key points)
    output.push_str("## Why Choose Us\n\n");
    output.push_str(&format!("**1. Unique Approach:** {}\n\n", differentiator));
    output.push_str(&format!("**2. Built For:** {}\n\n", audience));
    output.push_str(&format!("**3. Clear Path to Value:** {}\n\n", distribution));

    // CTA section
    output.push_str("## Get Started\n\n");
    output.push_str(&format!("**Pricing:** {}\n\n", monetization));
    output.push_str("[Start Free Trial] [Book a Demo]\n\n");

    // Risk acknowledgment (shows transparency)
    output.push_str("## Our Commitment\n\n");
    output.push_str(&format!("We know the challenges: {}\n\n", risks));
    output.push_str("That's why we're committed to helping you succeed.\n\n");

    // Footer
    output.push_str("---\n");
    output.push_str(&format!("*Evolution Score: {}/10*\n", score));

    Ok(output)
}

fn run_id_from_result(result: &serde_json::Value) -> &str {
    result
        .get("run_id")
        .and_then(|r| r.as_str())
        .unwrap_or("unknown")
}

/// Generate decision log format for engineering documentation
fn generate_decision_log(
    result: &serde_json::Value,
    config: Option<&serde_json::Value>,
    state: Option<&serde_json::Value>,
) -> Result<String> {
    let best = result
        .get("best_idea")
        .or_else(|| result.get("best"))
        .ok_or_else(|| anyhow::anyhow!("No best_idea in final.json"))?;

    let run_id = run_id_from_result(result);
    let title = best
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown");
    let summary = best.get("summary").and_then(|s| s.as_str()).unwrap_or("");
    let score = best
        .get("overall_score")
        .and_then(|s| s.as_f64())
        .unwrap_or(0.0);

    let facets = best.get("facets");
    let audience = facets
        .and_then(|f| f.get("audience"))
        .and_then(|a| a.as_str())
        .unwrap_or("");
    let jtbd = facets
        .and_then(|f| f.get("jtbd"))
        .and_then(|j| j.as_str())
        .unwrap_or("");
    let differentiator = facets
        .and_then(|f| f.get("differentiator"))
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let risks = facets
        .and_then(|f| f.get("risks"))
        .and_then(|r| r.as_str())
        .unwrap_or("");

    let prompt = config
        .and_then(|c| c.get("prompt"))
        .and_then(|p| p.as_str())
        .unwrap_or("");
    let iterations = result
        .get("iterations_completed")
        .and_then(|i| i.as_i64())
        .unwrap_or(0);
    let stop_reason = result
        .get("stop_reason")
        .and_then(|s| s.as_str())
        .unwrap_or("");

    // Count alternatives considered
    let alternatives_count = state
        .and_then(|s| s.get("ideas"))
        .and_then(|i| i.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let runner_up = result.get("runner_up");

    let mut output = String::new();

    output.push_str(&format!("# Decision Log: {}\n\n", title));
    output.push_str(&format!(
        "**Date:** {}\n",
        chrono::Utc::now().format("%Y-%m-%d")
    ));
    output.push_str(&format!("**Run ID:** `{}`\n", run_id));
    output.push_str("**Status:** Decided\n\n");

    output.push_str("## Context\n\n");
    output.push_str(&format!("**Problem Statement:** {}\n\n", prompt));
    output.push_str(&format!("**Target Audience:** {}\n\n", audience));

    output.push_str("## Decision\n\n");
    output.push_str(&format!("**Selected:** {}\n\n", title));
    output.push_str(&format!("{}\n\n", summary));

    output.push_str("## Rationale\n\n");
    output.push_str(&format!("- **Confidence Score:** {:.1}/10\n", score));
    output.push_str(&format!("- **Key Differentiator:** {}\n", differentiator));
    output.push_str(&format!("- **Problem Solved:** {}\n\n", jtbd));

    output.push_str("## Alternatives Considered\n\n");
    output.push_str(&format!(
        "- **Total evaluated:** {} ideas over {} iterations\n",
        alternatives_count, iterations
    ));

    if let Some(runner) = runner_up {
        let runner_title = runner
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown");
        let runner_score = runner
            .get("overall_score")
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);
        output.push_str(&format!(
            "- **Runner-up:** {} ({:.1}/10)\n",
            runner_title, runner_score
        ));
    }

    output.push_str("- **Selection method:** Evolutionary algorithm with scoring\n");
    output.push_str(&format!("- **Stop reason:** {}\n\n", stop_reason));

    output.push_str("## Risks & Mitigations\n\n");
    output.push_str(&format!("{}\n\n", risks));

    output.push_str("---\n");
    output.push_str(&format!(
        "*Generated by evoidea | Run: {} | Score: {:.1}/10*\n",
        run_id, score
    ));

    Ok(output)
}

/// Generate stakeholder brief for non-technical audiences
fn generate_stakeholder_brief(
    result: &serde_json::Value,
    config: Option<&serde_json::Value>,
) -> Result<String> {
    let best = result
        .get("best_idea")
        .or_else(|| result.get("best"))
        .ok_or_else(|| anyhow::anyhow!("No best_idea in final.json"))?;

    let run_id = run_id_from_result(result);
    let title = best
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown");
    let summary = best.get("summary").and_then(|s| s.as_str()).unwrap_or("");
    let score = best
        .get("overall_score")
        .and_then(|s| s.as_f64())
        .unwrap_or(0.0);

    let facets = best.get("facets");
    let audience = facets
        .and_then(|f| f.get("audience"))
        .and_then(|a| a.as_str())
        .unwrap_or("");
    let jtbd = facets
        .and_then(|f| f.get("jtbd"))
        .and_then(|j| j.as_str())
        .unwrap_or("");
    let differentiator = facets
        .and_then(|f| f.get("differentiator"))
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let monetization = facets
        .and_then(|f| f.get("monetization"))
        .and_then(|m| m.as_str())
        .unwrap_or("");
    let distribution = facets
        .and_then(|f| f.get("distribution"))
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let risks = facets
        .and_then(|f| f.get("risks"))
        .and_then(|r| r.as_str())
        .unwrap_or("");

    let prompt = config
        .and_then(|c| c.get("prompt"))
        .and_then(|p| p.as_str())
        .unwrap_or("");

    // Extract product name
    let product_name = title.split(':').next().unwrap_or(title).trim();

    let mut output = String::new();

    output.push_str(&format!("# {} - Executive Summary\n\n", product_name));

    output.push_str("## The Opportunity\n\n");
    output.push_str(&format!("**Direction explored:** {}\n\n", prompt));
    output.push_str(&format!("**Recommended approach:** {}\n\n", title));
    output.push_str(&format!("{}\n\n", summary));

    output.push_str("## Key Points\n\n");
    output.push_str("| Aspect | Details |\n");
    output.push_str("|--------|----------|\n");
    output.push_str(&format!("| Target Market | {} |\n", audience));
    output.push_str(&format!("| Problem Solved | {} |\n", jtbd));
    output.push_str(&format!("| Competitive Edge | {} |\n", differentiator));
    output.push_str(&format!("| Revenue Model | {} |\n", monetization));
    output.push_str(&format!("| Go-to-Market | {} |\n\n", distribution));

    output.push_str("## Confidence Assessment\n\n");
    let confidence_label = if score >= 7.0 {
        "High"
    } else if score >= 5.0 {
        "Medium"
    } else {
        "Low"
    };
    output.push_str(&format!(
        "**Overall Confidence:** {} ({:.1}/10)\n\n",
        confidence_label, score
    ));
    output.push_str("This assessment is based on automated evaluation of feasibility, market potential, differentiation, and risk factors.\n\n");

    output.push_str("## Known Risks\n\n");
    output.push_str(&format!("{}\n\n", risks));

    output.push_str("## Next Steps\n\n");
    output.push_str("1. Review and validate assumptions with domain experts\n");
    output.push_str("2. Conduct customer discovery interviews\n");
    output.push_str("3. Build minimal prototype for early feedback\n\n");

    output.push_str("---\n");
    output.push_str(&format!(
        "*Generated by evoidea | {} | Confidence: {:.1}/10*\n",
        run_id, score
    ));

    Ok(output)
}

/// Generate changelog entry format
fn generate_changelog_entry(
    result: &serde_json::Value,
    config: Option<&serde_json::Value>,
) -> Result<String> {
    let best = result
        .get("best_idea")
        .or_else(|| result.get("best"))
        .ok_or_else(|| anyhow::anyhow!("No best_idea in final.json"))?;

    let run_id = run_id_from_result(result);
    let title = best
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown");
    let summary = best.get("summary").and_then(|s| s.as_str()).unwrap_or("");
    let score = best
        .get("overall_score")
        .and_then(|s| s.as_f64())
        .unwrap_or(0.0);

    let facets = best.get("facets");
    let audience = facets
        .and_then(|f| f.get("audience"))
        .and_then(|a| a.as_str())
        .unwrap_or("");
    let jtbd = facets
        .and_then(|f| f.get("jtbd"))
        .and_then(|j| j.as_str())
        .unwrap_or("");

    let prompt = config
        .and_then(|c| c.get("prompt"))
        .and_then(|p| p.as_str())
        .unwrap_or("");
    let iterations = result
        .get("iterations_completed")
        .and_then(|i| i.as_i64())
        .unwrap_or(0);

    // Extract product name
    let product_name = title.split(':').next().unwrap_or(title).trim();

    let date = chrono::Utc::now().format("%Y-%m-%d");

    let mut output = String::new();

    output.push_str(&format!("## [Ideation] {} - {}\n\n", product_name, date));

    output.push_str("### Added\n\n");
    output.push_str(&format!("- **New concept explored:** {}\n", title));
    output.push_str(&format!("- **Problem space:** {}\n", prompt));
    output.push_str(&format!("- **Target users:** {}\n\n", audience));

    output.push_str("### Details\n\n");
    output.push_str(&format!("{}\n\n", summary));
    output.push_str(&format!("**Core value:** {}\n\n", jtbd));

    output.push_str("### Metrics\n\n");
    output.push_str(&format!("- Confidence score: {:.1}/10\n", score));
    output.push_str(&format!("- Evolution iterations: {}\n", iterations));
    output.push_str(&format!("- Run ID: `{}`\n\n", run_id));

    output.push_str("---\n");
    output.push_str("*Entry generated by evoidea evolutionary ideation*\n");

    Ok(output)
}

/// Interactive tournament mode for preference learning
pub fn tournament(run_id: &str, auto: bool, pairwise: bool) -> Result<()> {
    let run_dir = PathBuf::from("runs").join(run_id);
    let state_path = run_dir.join("state.json");

    if !state_path.exists() {
        anyhow::bail!("Run {} has no state.json", run_id);
    }

    let state_content = fs::read_to_string(&state_path)?;
    let state: serde_json::Value = serde_json::from_str(&state_content)?;

    // Get all active ideas
    let ideas = state
        .get("ideas")
        .and_then(|i| i.as_array())
        .ok_or_else(|| anyhow::anyhow!("No ideas in state.json"))?;

    let active_ideas: Vec<&serde_json::Value> = ideas
        .iter()
        .filter(|idea| {
            idea.get("status")
                .and_then(|s| s.as_str())
                .map(|s| s == "active")
                .unwrap_or(false)
        })
        .collect();

    let eligible_ideas: Vec<&serde_json::Value> = active_ideas
        .iter()
        .copied()
        .filter(|idea| idea_has_complete_scores(idea))
        .collect();

    if eligible_ideas.len() < 2 {
        anyhow::bail!(
            "Need at least 2 scored active ideas for tournament (found {} scored of {} active).",
            eligible_ideas.len(),
            active_ideas.len()
        );
    }

    println!("Tournament Mode for run: {}", run_id);
    println!("Active ideas: {}", active_ideas.len());
    if eligible_ideas.len() != active_ideas.len() {
        println!(
            "Warning: {} active ideas have missing scores and were excluded.",
            active_ideas.len() - eligible_ideas.len()
        );
    }
    println!();

    if auto {
        // Auto mode: just show ranking by score
        println!("=== Auto Mode: Ranking by Score ===\n");

        let mut ranked: Vec<(&serde_json::Value, f64)> = eligible_ideas
            .iter()
            .map(|idea| {
                let score = idea
                    .get("overall_score")
                    .and_then(|s| s.as_f64())
                    .unwrap_or(0.0);
                (*idea, score)
            })
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (rank, (idea, score)) in ranked.iter().enumerate() {
            let title = idea
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown");
            let short_title: String = title.chars().take(60).collect();
            println!("{}. [{:.2}] {}", rank + 1, score, short_title);
        }

        return Ok(());
    }

    // Interactive tournament mode
    let preferences_path = run_dir.join("preferences.json");
    let mut preferences: serde_json::Value = if preferences_path.exists() {
        serde_json::from_str(&fs::read_to_string(&preferences_path)?)?
    } else {
        serde_json::json!({
            "comparisons": [],
            "elo_ratings": {}
        })
    };

    // Initialize Elo ratings if needed
    {
        let elo_ratings = preferences
            .get_mut("elo_ratings")
            .and_then(|e| e.as_object_mut())
            .ok_or_else(|| anyhow::anyhow!("Invalid preferences format"))?;

        for idea in &eligible_ideas {
            let id = idea.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
            if !elo_ratings.contains_key(id) {
                elo_ratings.insert(id.to_string(), serde_json::json!(1000.0));
            }
        }
    }

    let mut comparison_count = 0;

    if pairwise {
        // Pairwise mode: smart sampling with ~2n comparisons
        let max_comparisons = calculate_pairwise_limit(eligible_ideas.len());

        println!("=== Pairwise Comparison Mode ===");
        println!(
            "Smart sampling: up to {} comparisons (vs {} for exhaustive)",
            max_comparisons,
            eligible_ideas.len() * (eligible_ideas.len() - 1) / 2
        );
        println!("Pick your preference: [A] or [B]. [S] Skip | [Q] Quit\n");

        // Build data structures for pair selection
        let ids: Vec<String> = eligible_ideas
            .iter()
            .map(|idea| {
                idea.get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            })
            .collect();

        // Build compared set from existing comparisons
        let mut compared: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        if let Some(comps) = preferences.get("comparisons").and_then(|c| c.as_array()) {
            for c in comps {
                let id_a = c.get("idea_a").and_then(|a| a.as_str()).unwrap_or("");
                let id_b = c.get("idea_b").and_then(|b| b.as_str()).unwrap_or("");
                let pair_key = if id_a < id_b {
                    (id_a.to_string(), id_b.to_string())
                } else {
                    (id_b.to_string(), id_a.to_string())
                };
                compared.insert(pair_key);
            }
        }

        while comparison_count < max_comparisons {
            // Get current Elo ratings
            let elo_ratings: std::collections::HashMap<String, f64> = preferences
                .get("elo_ratings")
                .and_then(|e| e.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_f64().map(|r| (k.clone(), r)))
                        .collect()
                })
                .unwrap_or_default();

            // Select next pair
            let pair = select_next_pair(&ids, &elo_ratings, &compared);
            if pair.is_none() {
                println!("All pairs compared!");
                break;
            }

            let (id_a, id_b) = pair.unwrap();

            // Find idea details
            let idea_a = eligible_ideas
                .iter()
                .find(|idea| idea.get("id").and_then(|i| i.as_str()) == Some(&id_a))
                .ok_or_else(|| anyhow::anyhow!("Idea {} not found", id_a))?;
            let idea_b = eligible_ideas
                .iter()
                .find(|idea| idea.get("id").and_then(|i| i.as_str()) == Some(&id_b))
                .ok_or_else(|| anyhow::anyhow!("Idea {} not found", id_b))?;

            let title_a = idea_a
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown");
            let title_b = idea_b
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown");
            let elo_a = elo_ratings.get(&id_a).unwrap_or(&1000.0);
            let elo_b = elo_ratings.get(&id_b).unwrap_or(&1000.0);

            println!(
                "--- Comparison {}/{} ---",
                comparison_count + 1,
                max_comparisons
            );
            println!();
            println!("[A] {} (Elo: {:.0})", title_a, elo_a);
            println!();
            println!("[B] {} (Elo: {:.0})", title_b, elo_b);
            println!();
            print!("Which is better? [A/B/S/Q]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim().to_uppercase();

            // Mark as compared regardless of choice
            let pair_key = if id_a < id_b {
                (id_a.clone(), id_b.clone())
            } else {
                (id_b.clone(), id_a.clone())
            };

            match choice.as_str() {
                "A" => {
                    compared.insert(pair_key);
                    {
                        let comparisons = preferences
                            .get_mut("comparisons")
                            .and_then(|c| c.as_array_mut())
                            .ok_or_else(|| anyhow::anyhow!("Invalid preferences format"))?;
                        comparisons.push(serde_json::json!({
                            "idea_a": id_a,
                            "idea_b": id_b,
                            "winner": id_a
                        }));
                    }
                    update_elo(&mut preferences, &id_a, &id_b)?;
                    comparison_count += 1;
                    println!("-> {} wins\n", title_a.chars().take(40).collect::<String>());
                }
                "B" => {
                    compared.insert(pair_key);
                    {
                        let comparisons = preferences
                            .get_mut("comparisons")
                            .and_then(|c| c.as_array_mut())
                            .ok_or_else(|| anyhow::anyhow!("Invalid preferences format"))?;
                        comparisons.push(serde_json::json!({
                            "idea_a": id_a,
                            "idea_b": id_b,
                            "winner": id_b
                        }));
                    }
                    update_elo(&mut preferences, &id_b, &id_a)?;
                    comparison_count += 1;
                    println!("-> {} wins\n", title_b.chars().take(40).collect::<String>());
                }
                "S" => {
                    compared.insert(pair_key);
                    println!("Skipped\n");
                }
                "Q" => {
                    println!("Quitting tournament...\n");
                    break;
                }
                _ => {
                    println!("Invalid choice, try again\n");
                    continue;
                }
            }

            // Save after each comparison
            fs::write(
                &preferences_path,
                serde_json::to_string_pretty(&preferences)?,
            )?;
        }
    } else {
        // Original exhaustive mode: compare all pairs
        let mut pairs: Vec<(usize, usize)> = Vec::new();
        for i in 0..eligible_ideas.len() {
            for j in (i + 1)..eligible_ideas.len() {
                pairs.push((i, j));
            }
        }

        println!("=== Interactive Tournament ===");
        println!("Compare ideas and pick your preference.");
        println!("Commands: [A] Choose A | [B] Choose B | [S] Skip | [Q] Quit\n");

        for (i, j) in pairs {
            let idea_a = eligible_ideas[i];
            let idea_b = eligible_ideas[j];

            let id_a = idea_a
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("unknown")
                .to_string();
            let id_b = idea_b
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("unknown")
                .to_string();

            // Check if we've already compared these
            let already_compared = {
                let comparisons = preferences
                    .get("comparisons")
                    .and_then(|c| c.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Invalid preferences format"))?;
                comparisons.iter().any(|c| {
                    let ca = c.get("idea_a").and_then(|a| a.as_str());
                    let cb = c.get("idea_b").and_then(|b| b.as_str());
                    (ca == Some(&id_a) && cb == Some(&id_b))
                        || (ca == Some(&id_b) && cb == Some(&id_a))
                })
            };

            if already_compared {
                continue;
            }

            let title_a = idea_a
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown");
            let title_b = idea_b
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown");
            let score_a = idea_a
                .get("overall_score")
                .and_then(|s| s.as_f64())
                .unwrap_or(0.0);
            let score_b = idea_b
                .get("overall_score")
                .and_then(|s| s.as_f64())
                .unwrap_or(0.0);

            println!("--- Comparison {} ---", comparison_count + 1);
            println!();
            println!("[A] {} (score: {:.2})", title_a, score_a);
            println!();
            println!("[B] {} (score: {:.2})", title_b, score_b);
            println!();
            print!("Your choice [A/B/S/Q]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim().to_uppercase();

            match choice.as_str() {
                "A" => {
                    {
                        let comparisons = preferences
                            .get_mut("comparisons")
                            .and_then(|c| c.as_array_mut())
                            .ok_or_else(|| anyhow::anyhow!("Invalid preferences format"))?;
                        comparisons.push(serde_json::json!({
                            "idea_a": id_a,
                            "idea_b": id_b,
                            "winner": id_a
                        }));
                    }
                    update_elo(&mut preferences, &id_a, &id_b)?;
                    comparison_count += 1;
                    println!(
                        "Recorded: {} wins\n",
                        title_a.chars().take(40).collect::<String>()
                    );
                }
                "B" => {
                    {
                        let comparisons = preferences
                            .get_mut("comparisons")
                            .and_then(|c| c.as_array_mut())
                            .ok_or_else(|| anyhow::anyhow!("Invalid preferences format"))?;
                        comparisons.push(serde_json::json!({
                            "idea_a": id_a,
                            "idea_b": id_b,
                            "winner": id_b
                        }));
                    }
                    update_elo(&mut preferences, &id_b, &id_a)?;
                    comparison_count += 1;
                    println!(
                        "Recorded: {} wins\n",
                        title_b.chars().take(40).collect::<String>()
                    );
                }
                "S" => {
                    println!("Skipped\n");
                }
                "Q" => {
                    println!("Quitting tournament...\n");
                    break;
                }
                _ => {
                    println!("Invalid choice, skipping\n");
                }
            }

            // Save after each comparison
            fs::write(
                &preferences_path,
                serde_json::to_string_pretty(&preferences)?,
            )?;
        }
    }

    // Show final rankings
    println!("=== Current Rankings (by Elo) ===\n");

    let elo_ratings = preferences
        .get("elo_ratings")
        .and_then(|e| e.as_object())
        .ok_or_else(|| anyhow::anyhow!("Invalid preferences format"))?;

    let eligible_ids: std::collections::HashSet<&str> = eligible_ideas
        .iter()
        .filter_map(|idea| idea.get("id").and_then(|i| i.as_str()))
        .collect();

    let mut ranked: Vec<(&str, f64)> = elo_ratings
        .iter()
        .filter(|(id, _)| eligible_ids.contains(id.as_str()))
        .filter_map(|(id, rating)| rating.as_f64().map(|r| (id.as_str(), r)))
        .collect();

    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (rank, (id, elo)) in ranked.iter().enumerate() {
        // Find the idea title
        let title = eligible_ideas
            .iter()
            .find(|idea| idea.get("id").and_then(|i| i.as_str()) == Some(*id))
            .and_then(|idea| idea.get("title").and_then(|t| t.as_str()))
            .unwrap_or("Unknown");
        let short_title: String = title.chars().take(50).collect();
        println!("{}. [Elo: {:.0}] {}", rank + 1, elo, short_title);
    }

    println!("\nPreferences saved to: {}", preferences_path.display());
    println!("Comparisons made: {}", comparison_count);

    Ok(())
}

fn idea_has_complete_scores(idea: &serde_json::Value) -> bool {
    if idea.get("overall_score").and_then(|s| s.as_f64()).is_none() {
        return false;
    }
    extract_scores(idea).is_some()
}

/// Calculate the maximum number of comparisons for pairwise mode.
/// Returns approximately 2n comparisons, which is enough to establish a ranking
/// with the adaptive pair selection algorithm.
fn calculate_pairwise_limit(n: usize) -> usize {
    2 * n
}

/// Select the next best pair to compare for pairwise tournament.
/// Returns the pair with closest Elo ratings that hasn't been compared yet.
/// This minimizes comparisons needed to establish ranking (~2n instead of n²).
fn select_next_pair(
    ids: &[String],
    elo_ratings: &std::collections::HashMap<String, f64>,
    compared: &std::collections::HashSet<(String, String)>,
) -> Option<(String, String)> {
    let mut best_pair: Option<(String, String)> = None;
    let mut smallest_diff = f64::MAX;

    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            let id_a = &ids[i];
            let id_b = &ids[j];

            // Check if already compared (order-independent)
            let pair_key = if id_a < id_b {
                (id_a.clone(), id_b.clone())
            } else {
                (id_b.clone(), id_a.clone())
            };

            if compared.contains(&pair_key) {
                continue;
            }

            let elo_a = *elo_ratings.get(id_a).unwrap_or(&1000.0);
            let elo_b = *elo_ratings.get(id_b).unwrap_or(&1000.0);
            let diff = (elo_a - elo_b).abs();

            if diff < smallest_diff {
                smallest_diff = diff;
                best_pair = Some((id_a.clone(), id_b.clone()));
            }
        }
    }

    best_pair
}

fn update_elo(preferences: &mut serde_json::Value, winner_id: &str, loser_id: &str) -> Result<()> {
    let k_factor = 32.0;

    let elo_ratings = preferences
        .get_mut("elo_ratings")
        .and_then(|e| e.as_object_mut())
        .ok_or_else(|| anyhow::anyhow!("Invalid preferences format"))?;

    let winner_elo = elo_ratings
        .get(winner_id)
        .and_then(|e| e.as_f64())
        .unwrap_or(1000.0);
    let loser_elo = elo_ratings
        .get(loser_id)
        .and_then(|e| e.as_f64())
        .unwrap_or(1000.0);

    // Calculate expected scores
    let expected_winner = 1.0 / (1.0 + 10.0_f64.powf((loser_elo - winner_elo) / 400.0));
    let expected_loser = 1.0 - expected_winner;

    // Update ratings (winner gets 1.0, loser gets 0.0)
    let new_winner_elo = winner_elo + k_factor * (1.0 - expected_winner);
    let new_loser_elo = loser_elo + k_factor * (0.0 - expected_loser);

    elo_ratings.insert(winner_id.to_string(), serde_json::json!(new_winner_elo));
    elo_ratings.insert(loser_id.to_string(), serde_json::json!(new_loser_elo));

    Ok(())
}

/// Export preferences from a run to a portable profile
pub fn profile_export(run_id: &str, output: Option<&str>) -> Result<()> {
    let run_dir = PathBuf::from("runs").join(run_id);
    let preferences_path = run_dir.join("preferences.json");
    let state_path = run_dir.join("state.json");

    if !preferences_path.exists() {
        anyhow::bail!(
            "No preferences found for run {}. Run tournament first.",
            run_id
        );
    }

    let preferences: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&preferences_path)?)?;

    let state: Option<serde_json::Value> = if state_path.exists() {
        Some(serde_json::from_str(&fs::read_to_string(&state_path)?)?)
    } else {
        None
    };

    let profile = build_portable_profile(run_id, &preferences, state.as_ref());

    let json_output = serde_json::to_string_pretty(&profile)?;

    match output {
        Some(path) => {
            fs::write(path, &json_output)?;
            println!("Profile exported to: {}", path);
        }
        None => {
            println!("{}", json_output);
        }
    }

    Ok(())
}

fn build_portable_profile(
    run_id: &str,
    preferences: &serde_json::Value,
    state: Option<&serde_json::Value>,
) -> serde_json::Value {
    // Extract comparison count and compute derived stats
    let comparisons = preferences
        .get("comparisons")
        .and_then(|c| c.as_array())
        .map(|c| c.len())
        .unwrap_or(0);

    let elo_ratings = preferences
        .get("elo_ratings")
        .and_then(|e| e.as_object())
        .map(|e| e.len())
        .unwrap_or(0);

    // Build portable profile with metadata
    let mut profile = serde_json::json!({
        "version": 1,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "source_run": run_id,
        "stats": {
            "comparisons": comparisons,
            "ideas_rated": elo_ratings
        },
        "preferences": preferences
    });

    if let Some(state) = state {
        if let Some(derived) = derive_preference_profile(preferences, state) {
            if let Some(obj) = profile.as_object_mut() {
                obj.insert("derived".to_string(), derived);
            }
        }
    }

    profile
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RiskMode {
    AsBenefit,
    Invert,
}

fn infer_risk_mode(state: &serde_json::Value) -> RiskMode {
    let ideas = state.get("ideas").and_then(|i| i.as_array());
    let Some(ideas) = ideas else {
        return RiskMode::AsBenefit;
    };

    let mut abs_err_benefit = 0.0f64;
    let mut abs_err_invert = 0.0f64;
    let mut n = 0u64;

    for idea in ideas {
        let Some(scores) = extract_scores(idea) else {
            continue;
        };

        let Some(overall) = idea.get("overall_score").and_then(|s| s.as_f64()) else {
            continue;
        };

        let predicted_benefit = average_score(&scores, RiskMode::AsBenefit);
        let predicted_invert = average_score(&scores, RiskMode::Invert);
        abs_err_benefit += (predicted_benefit - overall).abs();
        abs_err_invert += (predicted_invert - overall).abs();
        n += 1;
    }

    // Default to AsBenefit unless we have strong evidence otherwise.
    if n >= 3 && abs_err_invert + 1e-6 < abs_err_benefit {
        RiskMode::Invert
    } else {
        RiskMode::AsBenefit
    }
}

fn average_score(scores: &crate::data::Scores, risk_mode: RiskMode) -> f64 {
    let mut vals = [
        scores.feasibility as f64,
        scores.speed_to_value as f64,
        scores.differentiation as f64,
        scores.market_size as f64,
        scores.distribution as f64,
        scores.moats as f64,
        scores.risk as f64,
        scores.clarity as f64,
    ];

    if risk_mode == RiskMode::Invert {
        vals[6] = 10.0 - vals[6];
    }

    vals.iter().sum::<f64>() / vals.len() as f64
}

fn derive_preference_profile(
    preferences: &serde_json::Value,
    state: &serde_json::Value,
) -> Option<serde_json::Value> {
    let comparisons = preferences.get("comparisons")?.as_array()?;
    if comparisons.is_empty() {
        return None;
    }

    let risk_mode = infer_risk_mode(state);
    let scores_by_id = build_scores_by_id(state);

    let mut pairs: Vec<(String, String)> = Vec::new();
    for comp in comparisons {
        let idea_a = comp.get("idea_a").and_then(|v| v.as_str());
        let idea_b = comp.get("idea_b").and_then(|v| v.as_str());
        let winner = comp.get("winner").and_then(|v| v.as_str());
        let (Some(idea_a), Some(idea_b), Some(winner)) = (idea_a, idea_b, winner) else {
            continue;
        };

        let loser = if winner == idea_a {
            idea_b
        } else if winner == idea_b {
            idea_a
        } else {
            continue;
        };

        if scores_by_id.contains_key(winner) && scores_by_id.contains_key(loser) {
            pairs.push((winner.to_string(), loser.to_string()));
        }
    }

    if pairs.is_empty() {
        return None;
    }

    let (weights, holdout_accuracy) =
        fit_criterion_weights_pairwise_mw(&pairs, &scores_by_id, risk_mode, 0.2, 1);

    let summary = summarize_weights(&weights);

    Some(serde_json::json!({
        "criterion_weights": weights,
        "fit": {
            "method": "pairwise-multiplicative-weights",
            "comparisons_used": pairs.len(),
            "holdout_accuracy": holdout_accuracy,
        },
        "summary": summary,
    }))
}

fn build_scores_by_id(
    state: &serde_json::Value,
) -> std::collections::HashMap<String, crate::data::Scores> {
    let mut out = std::collections::HashMap::new();
    let ideas = state.get("ideas").and_then(|i| i.as_array());
    let Some(ideas) = ideas else {
        return out;
    };

    for idea in ideas {
        let Some(id) = idea.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(scores) = extract_scores(idea) else {
            continue;
        };
        out.insert(id.to_string(), scores);
    }

    out
}

fn extract_scores(idea: &serde_json::Value) -> Option<crate::data::Scores> {
    let scores = idea.get("scores")?.as_object()?;
    Some(crate::data::Scores {
        feasibility: scores.get("feasibility")?.as_f64()? as f32,
        speed_to_value: scores.get("speed_to_value")?.as_f64()? as f32,
        differentiation: scores.get("differentiation")?.as_f64()? as f32,
        market_size: scores.get("market_size")?.as_f64()? as f32,
        distribution: scores.get("distribution")?.as_f64()? as f32,
        moats: scores.get("moats")?.as_f64()? as f32,
        risk: scores.get("risk")?.as_f64()? as f32,
        clarity: scores.get("clarity")?.as_f64()? as f32,
    })
}

fn summarize_weights(weights: &crate::config::ScoringWeights) -> Vec<String> {
    let mut items: Vec<(&str, f32)> = vec![
        ("feasibility", weights.feasibility),
        ("speed_to_value", weights.speed_to_value),
        ("differentiation", weights.differentiation),
        ("market_size", weights.market_size),
        ("distribution", weights.distribution),
        ("moats", weights.moats),
        ("risk", weights.risk),
        ("clarity", weights.clarity),
    ];
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let top: Vec<&str> = items.iter().take(2).map(|(k, _)| *k).collect();
    let bottom: Vec<&str> = items.iter().rev().take(2).map(|(k, _)| *k).collect();

    let top1 = top.first().copied().unwrap_or("unknown");
    let top2 = top.get(1).copied().unwrap_or("unknown");
    let bottom1 = bottom.first().copied().unwrap_or("unknown");
    let bottom2 = bottom.get(1).copied().unwrap_or("unknown");

    vec![
        format!("Prioritizes {} and {} over other criteria.", top1, top2),
        format!(
            "De-emphasizes {} and {} relative to other criteria.",
            bottom1, bottom2
        ),
    ]
}

fn fit_criterion_weights_pairwise_mw(
    pairs: &[(String, String)],
    scores_by_id: &std::collections::HashMap<String, crate::data::Scores>,
    risk_mode: RiskMode,
    holdout_fraction: f64,
    seed: u64,
) -> (crate::config::ScoringWeights, Option<f64>) {
    let mut indices: Vec<usize> = (0..pairs.len()).collect();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    indices.shuffle(&mut rng);

    let test_count = ((pairs.len() as f64) * holdout_fraction).round() as usize;
    let test_count = test_count.min(pairs.len());

    let (test_idx, train_idx) = indices.split_at(test_count);

    let weights_train =
        fit_criterion_weights_pairwise_mw_on_indices(pairs, scores_by_id, risk_mode, train_idx);

    let holdout_accuracy = if test_idx.is_empty() {
        None
    } else {
        Some(evaluate_pairwise_accuracy(
            pairs,
            scores_by_id,
            risk_mode,
            &weights_train,
            test_idx,
        ))
    };

    let weights_all =
        fit_criterion_weights_pairwise_mw_on_indices(pairs, scores_by_id, risk_mode, &indices);

    (weights_all, holdout_accuracy)
}

fn fit_criterion_weights_pairwise_mw_on_indices(
    pairs: &[(String, String)],
    scores_by_id: &std::collections::HashMap<String, crate::data::Scores>,
    risk_mode: RiskMode,
    indices: &[usize],
) -> crate::config::ScoringWeights {
    // Start from a uniform, positive prior.
    let mut w = [1.0f64; 8];
    let lr = 0.05f64;
    let clamp_min = 0.1f64;
    let clamp_max = 10.0f64;

    for &idx in indices {
        let (winner_id, loser_id) = &pairs[idx];
        let (Some(winner), Some(loser)) = (scores_by_id.get(winner_id), scores_by_id.get(loser_id))
        else {
            continue;
        };

        let f_w = scores_to_features(winner, risk_mode);
        let f_l = scores_to_features(loser, risk_mode);

        for i in 0..w.len() {
            let delta = f_w[i] - f_l[i];
            w[i] *= (lr * delta).exp();
            w[i] = w[i].clamp(clamp_min, clamp_max);
        }

        normalize_in_place(&mut w);
    }

    crate::config::ScoringWeights {
        feasibility: w[0] as f32,
        speed_to_value: w[1] as f32,
        differentiation: w[2] as f32,
        market_size: w[3] as f32,
        distribution: w[4] as f32,
        moats: w[5] as f32,
        risk: w[6] as f32,
        clarity: w[7] as f32,
    }
}

fn evaluate_pairwise_accuracy(
    pairs: &[(String, String)],
    scores_by_id: &std::collections::HashMap<String, crate::data::Scores>,
    risk_mode: RiskMode,
    weights: &crate::config::ScoringWeights,
    indices: &[usize],
) -> f64 {
    let w = [
        weights.feasibility as f64,
        weights.speed_to_value as f64,
        weights.differentiation as f64,
        weights.market_size as f64,
        weights.distribution as f64,
        weights.moats as f64,
        weights.risk as f64,
        weights.clarity as f64,
    ];

    let mut correct = 0u64;
    let mut total = 0u64;

    for &idx in indices {
        let (winner_id, loser_id) = &pairs[idx];
        let (Some(winner), Some(loser)) = (scores_by_id.get(winner_id), scores_by_id.get(loser_id))
        else {
            continue;
        };

        let f_w = scores_to_features(winner, risk_mode);
        let f_l = scores_to_features(loser, risk_mode);
        let delta = dot(&w, &f_w) - dot(&w, &f_l);

        total += 1;
        if delta >= 0.0 {
            correct += 1;
        }
    }

    if total == 0 {
        0.0
    } else {
        (correct as f64) / (total as f64)
    }
}

fn normalize_in_place(w: &mut [f64; 8]) {
    let sum = w.iter().sum::<f64>();
    if sum <= 0.0 {
        *w = [1.0 / 8.0; 8];
        return;
    }
    for wi in w.iter_mut() {
        *wi /= sum;
    }
}

fn scores_to_features(scores: &crate::data::Scores, risk_mode: RiskMode) -> [f64; 8] {
    let risk = match risk_mode {
        RiskMode::AsBenefit => scores.risk as f64,
        RiskMode::Invert => 10.0 - (scores.risk as f64),
    };

    [
        scores.feasibility as f64,
        scores.speed_to_value as f64,
        scores.differentiation as f64,
        scores.market_size as f64,
        scores.distribution as f64,
        scores.moats as f64,
        risk,
        scores.clarity as f64,
    ]
}

fn dot(w: &[f64; 8], f: &[f64; 8]) -> f64 {
    w.iter().zip(f.iter()).map(|(a, b)| a * b).sum()
}

/// Import a profile into a run
pub fn profile_import(file: &str, run_id: &str) -> Result<()> {
    let run_dir = PathBuf::from("runs").join(run_id);

    if !run_dir.exists() {
        anyhow::bail!("Run {} not found", run_id);
    }

    let profile: serde_json::Value = serde_json::from_str(&fs::read_to_string(file)?)?;

    // Validate profile format
    let version = profile
        .get("version")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow::anyhow!("Invalid profile: missing version"))?;

    if version != 1 {
        anyhow::bail!("Unsupported profile version: {}", version);
    }

    let preferences = profile
        .get("preferences")
        .ok_or_else(|| anyhow::anyhow!("Invalid profile: missing preferences"))?;

    // Write preferences to run
    let preferences_path = run_dir.join("preferences.json");
    fs::write(
        &preferences_path,
        serde_json::to_string_pretty(preferences)?,
    )?;

    let source_run = profile
        .get("source_run")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    println!("Imported profile from {} into {}", source_run, run_id);
    println!("Preferences written to: {}", preferences_path.display());

    Ok(())
}

/// Render evolution tree visualization
pub fn render_tree(run_id: &str, format: &str) -> Result<()> {
    let run_dir = PathBuf::from("runs").join(run_id);
    let state_path = run_dir.join("state.json");

    if !state_path.exists() {
        anyhow::bail!("Run {} not found", run_id);
    }

    let state: serde_json::Value = serde_json::from_str(&fs::read_to_string(&state_path)?)?;
    let ideas = state
        .get("ideas")
        .and_then(|i| i.as_array())
        .ok_or_else(|| anyhow::anyhow!("Invalid state: missing ideas"))?;

    if ideas.is_empty() {
        println!("No ideas in run {}", run_id);
        return Ok(());
    }

    // Build parent -> children map
    let mut children_map: std::collections::HashMap<String, Vec<&serde_json::Value>> =
        std::collections::HashMap::new();
    let mut roots: Vec<&serde_json::Value> = Vec::new();

    for idea in ideas {
        let parents = idea
            .get("parents")
            .and_then(|p| p.as_array())
            .map(|p| p.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        if parents.is_empty() {
            roots.push(idea);
        } else {
            for parent_id in parents {
                children_map
                    .entry(parent_id.to_string())
                    .or_default()
                    .push(idea);
            }
        }
    }

    match format {
        "mermaid" => render_mermaid_tree(&roots, &children_map, run_id),
        _ => render_ascii_tree(&roots, &children_map, run_id),
    }
}

fn render_ascii_tree(
    roots: &[&serde_json::Value],
    children_map: &std::collections::HashMap<String, Vec<&serde_json::Value>>,
    run_id: &str,
) -> Result<()> {
    println!("=== Evolution Tree: {} ===\n", run_id);

    for root in roots {
        print_idea_node(root, children_map, "", true);
    }

    // Legend
    println!("\nLegend: [score] status title");
    println!("  * = active, ~ = archived, x = eliminated");

    Ok(())
}

fn print_idea_node(
    idea: &serde_json::Value,
    children_map: &std::collections::HashMap<String, Vec<&serde_json::Value>>,
    prefix: &str,
    is_last: bool,
) {
    let id = idea.get("id").and_then(|i| i.as_str()).unwrap_or("?");
    let title = idea
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown");
    let score = idea
        .get("overall_score")
        .and_then(|s| s.as_f64())
        .unwrap_or(0.0);
    let status = idea.get("status").and_then(|s| s.as_str()).unwrap_or("?");

    let status_char = match status {
        "active" => "*",
        "archived" => "~",
        "eliminated" => "x",
        _ => "?",
    };

    let connector = if is_last { "└── " } else { "├── " };
    let short_title: String = title.chars().take(40).collect();
    let title_display = if title.len() > 40 {
        format!("{}...", short_title)
    } else {
        short_title
    };

    println!(
        "{}{}{} [{:.1}] {} {}",
        prefix, connector, status_char, score, id, title_display
    );

    // Print children
    if let Some(children) = children_map.get(id) {
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        for (i, child) in children.iter().enumerate() {
            let child_is_last = i == children.len() - 1;
            print_idea_node(child, children_map, &new_prefix, child_is_last);
        }
    }
}

fn render_mermaid_tree(
    roots: &[&serde_json::Value],
    children_map: &std::collections::HashMap<String, Vec<&serde_json::Value>>,
    run_id: &str,
) -> Result<()> {
    println!("```mermaid");
    println!("flowchart TD");
    println!(
        "    subgraph {}[\"Evolution: {}\"]",
        run_id.replace('-', "_"),
        run_id
    );

    // Collect all nodes
    let mut all_ideas: Vec<&serde_json::Value> = roots.to_vec();
    for children in children_map.values() {
        all_ideas.extend(children.iter());
    }

    // Print nodes with styling
    for idea in &all_ideas {
        let id = idea.get("id").and_then(|i| i.as_str()).unwrap_or("?");
        let title = idea
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown");
        let score = idea
            .get("overall_score")
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);
        let status = idea.get("status").and_then(|s| s.as_str()).unwrap_or("?");

        let short_title: String = title.chars().take(25).collect();
        let safe_id = id.replace('-', "_");

        let shape = match status {
            "active" => format!("{}([\"{}\\n{:.1}\"])", safe_id, short_title, score),
            "eliminated" => format!("{}{{\"{}\\n{:.1}\"}}", safe_id, short_title, score),
            _ => format!("{}[\"{}\\n{:.1}\"]", safe_id, short_title, score),
        };

        println!("    {}", shape);
    }

    // Print edges
    for (parent_id, children) in children_map {
        let safe_parent = parent_id.replace('-', "_");
        for child in children {
            let child_id = child.get("id").and_then(|i| i.as_str()).unwrap_or("?");
            let safe_child = child_id.replace('-', "_");
            println!("    {} --> {}", safe_parent, safe_child);
        }
    }

    // Styling
    println!("    end");
    println!("    classDef active fill:#90EE90,stroke:#228B22");
    println!("    classDef archived fill:#D3D3D3,stroke:#808080");
    println!("    classDef eliminated fill:#FFB6C1,stroke:#DC143C");

    // Apply classes
    for idea in &all_ideas {
        let id = idea.get("id").and_then(|i| i.as_str()).unwrap_or("?");
        let status = idea.get("status").and_then(|s| s.as_str()).unwrap_or("?");
        let safe_id = id.replace('-', "_");

        println!("    class {} {}", safe_id, status);
    }

    println!("```");

    Ok(())
}

/// Show profile information for a run
pub fn profile_show(run_id: &str) -> Result<()> {
    let run_dir = PathBuf::from("runs").join(run_id);
    let preferences_path = run_dir.join("preferences.json");

    if !preferences_path.exists() {
        println!("No preferences found for run {}", run_id);
        println!(
            "Run 'evoidea tournament --run-id {}' to generate preferences",
            run_id
        );
        return Ok(());
    }

    let preferences: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&preferences_path)?)?;

    let comparisons = preferences
        .get("comparisons")
        .and_then(|c| c.as_array())
        .map(|c| c.len())
        .unwrap_or(0);

    let elo_ratings = preferences.get("elo_ratings").and_then(|e| e.as_object());

    println!("=== Profile for {} ===\n", run_id);
    println!("Comparisons: {}", comparisons);

    if let Some(ratings) = elo_ratings {
        println!("Ideas rated: {}", ratings.len());
        println!("\nElo Rankings:");

        let mut ranked: Vec<(&str, f64)> = ratings
            .iter()
            .filter_map(|(id, elo)| elo.as_f64().map(|e| (id.as_str(), e)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (rank, (id, elo)) in ranked.iter().enumerate() {
            let short_id = if id.len() > 30 { &id[..30] } else { id };
            println!("  {}. [{:.0}] {}", rank + 1, elo, short_id);
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

    #[test]
    fn test_select_next_pair_picks_closest_elo() {
        // Given 4 items with Elo ratings, should pick the pair with closest ratings
        let mut elo_ratings = std::collections::HashMap::new();
        elo_ratings.insert("a".to_string(), 1000.0);
        elo_ratings.insert("b".to_string(), 1050.0); // closest to a
        elo_ratings.insert("c".to_string(), 1200.0);
        elo_ratings.insert("d".to_string(), 1500.0);

        let compared: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        let ids: Vec<String> = vec!["a", "b", "c", "d"]
            .into_iter()
            .map(String::from)
            .collect();

        let pair = select_next_pair(&ids, &elo_ratings, &compared);

        assert!(pair.is_some());
        let (id1, id2) = pair.unwrap();
        // Should pick a and b (closest ratings: 50 diff)
        assert!((id1 == "a" && id2 == "b") || (id1 == "b" && id2 == "a"));
    }

    #[test]
    fn test_select_next_pair_skips_already_compared() {
        let mut elo_ratings = std::collections::HashMap::new();
        elo_ratings.insert("a".to_string(), 1000.0);
        elo_ratings.insert("b".to_string(), 1050.0);
        elo_ratings.insert("c".to_string(), 1100.0);

        // a-b already compared
        let mut compared: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        compared.insert(("a".to_string(), "b".to_string()));

        let ids: Vec<String> = vec!["a", "b", "c"].into_iter().map(String::from).collect();

        let pair = select_next_pair(&ids, &elo_ratings, &compared);

        assert!(pair.is_some());
        let (id1, id2) = pair.unwrap();
        // Should pick b-c (next closest, 50 diff) since a-b is done
        assert!((id1 == "b" && id2 == "c") || (id1 == "c" && id2 == "b"));
    }

    #[test]
    fn test_select_next_pair_returns_none_when_done() {
        let mut elo_ratings = std::collections::HashMap::new();
        elo_ratings.insert("a".to_string(), 1000.0);
        elo_ratings.insert("b".to_string(), 1050.0);

        // Only pair already compared
        let mut compared: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        compared.insert(("a".to_string(), "b".to_string()));

        let ids: Vec<String> = vec!["a", "b"].into_iter().map(String::from).collect();

        let pair = select_next_pair(&ids, &elo_ratings, &compared);

        assert!(pair.is_none());
    }

    #[test]
    fn test_pairwise_elo_updates_after_comparison() {
        // After a pairwise comparison, Elo ratings should update correctly
        let mut preferences = serde_json::json!({
            "comparisons": [],
            "elo_ratings": {
                "idea-001": 1000.0,
                "idea-002": 1000.0
            }
        });

        // idea-001 wins
        update_elo(&mut preferences, "idea-001", "idea-002").unwrap();

        let ratings = preferences.get("elo_ratings").unwrap();
        let winner_elo = ratings.get("idea-001").unwrap().as_f64().unwrap();
        let loser_elo = ratings.get("idea-002").unwrap().as_f64().unwrap();

        // Winner should gain, loser should lose
        assert!(winner_elo > 1000.0);
        assert!(loser_elo < 1000.0);
        // Zero-sum: changes should be equal and opposite
        assert!((winner_elo - 1000.0 + loser_elo - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_pairwise_comparison_limit_is_reasonable() {
        // For n items, pairwise mode should need ~2n comparisons to converge
        // (not the O(n²) of full pairwise)
        let n = 10;
        let ids: Vec<String> = (0..n).map(|i| format!("idea-{:03}", i)).collect();
        let mut elo_ratings = std::collections::HashMap::new();
        for id in &ids {
            elo_ratings.insert(id.clone(), 1000.0);
        }

        let max_comparisons = calculate_pairwise_limit(n);

        // Should be around 2n, definitely less than n*(n-1)/2
        assert!(max_comparisons <= 3 * n);
        assert!(max_comparisons < n * (n - 1) / 2);
    }

    #[test]
    fn test_derive_preference_profile_returns_none_without_comparisons() {
        let preferences = serde_json::json!({
            "comparisons": [],
            "elo_ratings": {}
        });

        let state = serde_json::json!({
            "ideas": []
        });

        let derived = derive_preference_profile(&preferences, &state);
        assert!(derived.is_none());
    }

    #[test]
    fn test_derive_preference_profile_learns_risk_weight_when_risk_is_benefit() {
        // In current run artifacts, higher "risk" score means safer (benefit) and contributes
        // positively to overall_score (i.e., no inversion). This test ensures we infer that mode
        // and learn a higher weight for risk when the user consistently prefers the safer idea.
        let state = serde_json::json!({
            "ideas": [
                {
                    "id": "safe",
                    "scores": {"feasibility": 5, "speed_to_value": 5, "differentiation": 5, "market_size": 5, "distribution": 5, "moats": 5, "risk": 9, "clarity": 5},
                    "overall_score": 5.5
                },
                {
                    "id": "risky",
                    "scores": {"feasibility": 5, "speed_to_value": 5, "differentiation": 5, "market_size": 5, "distribution": 5, "moats": 5, "risk": 1, "clarity": 5},
                    "overall_score": 4.5
                }
            ]
        });

        let preferences = serde_json::json!({
            "comparisons": [
                { "idea_a": "safe", "idea_b": "risky", "winner": "safe" }
            ],
            "elo_ratings": {}
        });

        let derived = derive_preference_profile(&preferences, &state).expect("derived");
        let weights = derived.get("criterion_weights").expect("criterion_weights");
        let risk = weights.get("risk").and_then(|v| v.as_f64()).unwrap();
        let feasibility = weights.get("feasibility").and_then(|v| v.as_f64()).unwrap();

        assert!(risk > feasibility);

        // Weights should be normalized to sum ~= 1.
        let sum: f64 = [
            "feasibility",
            "speed_to_value",
            "differentiation",
            "market_size",
            "distribution",
            "moats",
            "risk",
            "clarity",
        ]
        .iter()
        .map(|k| weights.get(*k).and_then(|v| v.as_f64()).unwrap())
        .sum();
        assert!((sum - 1.0).abs() < 1e-6);

        let fit = derived.get("fit").expect("fit");
        assert_eq!(
            fit.get("method").and_then(|v| v.as_str()).unwrap(),
            "pairwise-multiplicative-weights"
        );
        assert_eq!(
            fit.get("comparisons_used")
                .and_then(|v| v.as_u64())
                .unwrap(),
            1
        );

        let summary = derived.get("summary").and_then(|v| v.as_array()).unwrap();
        assert_eq!(summary.len(), 2);
    }

    #[test]
    fn test_validate_state_invariants_flags_unscored_active_ideas() {
        let state = serde_json::json!({
            "ideas": [
                {
                    "id": "idea-1",
                    "origin": "refined",
                    "parents": ["idea-0"],
                    "status": "active"
                }
            ]
        });

        let errors = validate_state_idea_invariants(&state);
        assert!(errors.iter().any(|e| e.contains("missing/invalid scores")));
        assert!(errors
            .iter()
            .any(|e| e.contains("missing/invalid overall_score")));
    }

    #[test]
    fn test_validate_state_invariants_accepts_scored_active_ideas() {
        let state = serde_json::json!({
            "ideas": [
                {
                    "id": "idea-1",
                    "origin": "generated",
                    "parents": [],
                    "status": "active",
                    "scores": {"feasibility": 5, "speed_to_value": 5, "differentiation": 5, "market_size": 5, "distribution": 5, "moats": 5, "risk": 5, "clarity": 5},
                    "overall_score": 5.0
                }
            ]
        });

        let errors = validate_state_idea_invariants(&state);
        assert!(errors.is_empty());
    }
}
