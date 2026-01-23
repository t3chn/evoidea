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

    /// Export run results in landing page format
    Export {
        /// Run ID to export
        #[arg(long)]
        run_id: String,

        /// Export format (landing)
        #[arg(long, default_value = "landing")]
        format: String,
    },

    /// Interactive tournament mode for preference learning
    Tournament {
        /// Run ID to run tournament on
        #[arg(long)]
        run_id: String,

        /// Use automatic mode (no interaction, scores only)
        #[arg(long)]
        auto: bool,
    },

    /// Manage preference profiles for scoring calibration
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
}

#[derive(Subcommand)]
enum ProfileAction {
    /// Export preferences from a run to a portable profile
    Export {
        /// Run ID to export from
        #[arg(long)]
        run_id: String,

        /// Output file (default: stdout)
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Import a profile into a run
    Import {
        /// Profile file to import
        #[arg(long, short)]
        file: String,

        /// Run ID to import into
        #[arg(long)]
        run_id: String,
    },

    /// Show current profile information
    Show {
        /// Run ID to show profile for
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
        Commands::Export { run_id, format } => {
            tracing::info!(run_id = %run_id, format = %format, "Exporting run");
            orchestrator::export_run(&run_id, &format)?;
        }
        Commands::Tournament { run_id, auto } => {
            tracing::info!(run_id = %run_id, auto = %auto, "Running tournament");
            orchestrator::tournament(&run_id, auto)?;
        }
        Commands::Profile { action } => match action {
            ProfileAction::Export { run_id, output } => {
                tracing::info!(run_id = %run_id, "Exporting profile");
                orchestrator::profile_export(&run_id, output.as_deref())?;
            }
            ProfileAction::Import { file, run_id } => {
                tracing::info!(run_id = %run_id, file = %file, "Importing profile");
                orchestrator::profile_import(&file, &run_id)?;
            }
            ProfileAction::Show { run_id } => {
                tracing::info!(run_id = %run_id, "Showing profile");
                orchestrator::profile_show(&run_id)?;
            }
        },
    }

    Ok(())
}
