use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod config;
mod data;
mod orchestrator;
mod scoring;
mod storage;

#[derive(Parser)]
#[command(name = "evoidea")]
#[command(about = "Evoidea CLI - utilities for viewing and validating evolution runs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all runs
    List {
        /// Output directory containing runs
        #[arg(long, default_value = "runs")]
        dir: String,
    },

    /// Show run results
    Show {
        /// Run ID to show
        #[arg(long)]
        run_id: String,

        /// Output format (json or md)
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Validate run artifacts
    Validate {
        /// Run ID to validate
        #[arg(long)]
        run_id: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::List { dir } => {
            tracing::info!(dir = %dir, "Listing runs");
            orchestrator::list_runs(&dir)?;
        }
        Commands::Show { run_id, format } => {
            tracing::info!(run_id = %run_id, format = %format, "Showing run");
            orchestrator::show_run(&run_id, &format)?;
        }
        Commands::Validate { run_id } => {
            tracing::info!(run_id = %run_id, "Validating run");
            orchestrator::validate_run(&run_id)?;
        }
    }

    Ok(())
}
