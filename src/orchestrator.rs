use anyhow::Result;
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
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(state) => {
                        let iteration =
                            state.get("iteration").and_then(|i| i.as_u64()).unwrap_or(0);
                        let ideas_count = state
                            .get("ideas")
                            .and_then(|i| i.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0);
                        println!(
                            "State: OK (iteration: {}, ideas: {})",
                            iteration, ideas_count
                        );

                        // Validate idea invariants
                        if let Some(ideas) = state.get("ideas").and_then(|i| i.as_array()) {
                            for idea in ideas {
                                let id = idea.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                                let origin =
                                    idea.get("origin").and_then(|o| o.as_str()).unwrap_or("?");
                                let parents = idea.get("parents").and_then(|p| p.as_array());
                                let has_parents = parents.map(|p| !p.is_empty()).unwrap_or(false);

                                match origin {
                                    "generated" => {
                                        if has_parents {
                                            errors.push(format!(
                                                "Idea {} (generated) has parents",
                                                id
                                            ));
                                        }
                                    }
                                    "refined" | "crossover" | "mutated" => {
                                        if !has_parents {
                                            errors.push(format!(
                                                "Idea {} ({}) has no parents",
                                                id, origin
                                            ));
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
pub fn tournament(run_id: &str, auto: bool) -> Result<()> {
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

    if active_ideas.len() < 2 {
        anyhow::bail!(
            "Need at least 2 active ideas for tournament (found {})",
            active_ideas.len()
        );
    }

    println!("Tournament Mode for run: {}", run_id);
    println!("Active ideas: {}", active_ideas.len());
    println!();

    if auto {
        // Auto mode: just show ranking by score
        println!("=== Auto Mode: Ranking by Score ===\n");

        let mut ranked: Vec<(&serde_json::Value, f64)> = active_ideas
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

        for idea in &active_ideas {
            let id = idea.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
            if !elo_ratings.contains_key(id) {
                elo_ratings.insert(id.to_string(), serde_json::json!(1000.0));
            }
        }
    }

    // Generate pairs for comparison
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..active_ideas.len() {
        for j in (i + 1)..active_ideas.len() {
            pairs.push((i, j));
        }
    }

    println!("=== Interactive Tournament ===");
    println!("Compare ideas and pick your preference.");
    println!("Commands: [A] Choose A | [B] Choose B | [S] Skip | [Q] Quit\n");

    let mut comparison_count = 0;

    for (i, j) in pairs {
        let idea_a = active_ideas[i];
        let idea_b = active_ideas[j];

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
                (ca == Some(&id_a) && cb == Some(&id_b)) || (ca == Some(&id_b) && cb == Some(&id_a))
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

    // Show final rankings
    println!("=== Current Rankings (by Elo) ===\n");

    let elo_ratings = preferences
        .get("elo_ratings")
        .and_then(|e| e.as_object())
        .ok_or_else(|| anyhow::anyhow!("Invalid preferences format"))?;

    let mut ranked: Vec<(&str, f64)> = elo_ratings
        .iter()
        .filter_map(|(id, rating)| rating.as_f64().map(|r| (id.as_str(), r)))
        .collect();

    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (rank, (id, elo)) in ranked.iter().enumerate() {
        // Find the idea title
        let title = active_ideas
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

    if !preferences_path.exists() {
        anyhow::bail!(
            "No preferences found for run {}. Run tournament first.",
            run_id
        );
    }

    let preferences: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&preferences_path)?)?;

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
    let profile = serde_json::json!({
        "version": 1,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "source_run": run_id,
        "stats": {
            "comparisons": comparisons,
            "ideas_rated": elo_ratings
        },
        "preferences": preferences
    });

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
}
