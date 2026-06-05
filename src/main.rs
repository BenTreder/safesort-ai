use clap::Parser;

mod app;
mod cli;
mod config;
mod detectors;
mod error;
mod manifest;
mod placement;
mod preflight;
mod profile;
mod reports;
mod rules_file;
mod safety;
mod scan;

use cli::Cli;
use error::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    app::run(cli)
}
