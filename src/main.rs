use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod config;
mod data;
mod llm;
mod orchestrator;
mod phase;
mod scoring;
mod storage;

#[derive(Parser)]
#[command(name = "evoidea")]
#[command(about = "Memetic idea evolution CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a new idea evolution session
    Run {
        /// The prompt describing what kind of idea to generate
        #[arg(long)]
        prompt: String,

        /// LLM provider mode
        #[arg(long, default_value = "mock")]
        mode: String,

        /// Maximum number of evolution rounds
        #[arg(long, default_value = "6")]
        max_rounds: u32,

        /// Population size
        #[arg(long, default_value = "12")]
        population: u32,

        /// Number of elite ideas to keep
        #[arg(long, default_value = "4")]
        elite: u32,

        /// Score threshold to stop early
        #[arg(long, default_value = "8.7")]
        threshold: f32,

        /// Stagnation patience (rounds without improvement)
        #[arg(long, default_value = "2")]
        stagnation: u32,

        /// Output directory for run artifacts
        #[arg(long, default_value = "runs")]
        out: String,
    },

    /// Resume an existing run
    Resume {
        /// Run ID to resume
        #[arg(long)]
        run_id: String,

        /// Additional rounds to run
        #[arg(long)]
        max_rounds: Option<u32>,
    },

    /// Show run results
    Show {
        /// Run ID to show
        #[arg(long)]
        run_id: String,

        /// Output format
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
        Commands::Run {
            prompt,
            mode,
            max_rounds,
            population,
            elite,
            threshold,
            stagnation,
            out,
        } => {
            tracing::info!(
                prompt = %prompt,
                mode = %mode,
                max_rounds = max_rounds,
                "Starting evolution run"
            );
            let run_config = config::RunConfig::new(
                prompt, mode, max_rounds, population, elite, threshold, stagnation, out,
            );
            let mut orchestrator = orchestrator::Orchestrator::new(run_config)?;
            orchestrator.run()?;
        }
        Commands::Resume { run_id, max_rounds } => {
            tracing::info!(run_id = %run_id, "Resuming run");
            let mut orchestrator = orchestrator::Orchestrator::resume(&run_id, max_rounds)?;
            orchestrator.run()?;
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
