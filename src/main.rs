#![allow(dead_code)]

use clap::Parser;

mod app;
mod apply;
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
mod shortcuts;

use cli::Cli;
use error::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    // Intercept shortcut flags before clap sees them.
    // clap does not accept single-dash long flags, so we handle them here.
    let raw_args: Vec<String> = std::env::args().collect();
    let args = &raw_args[1..];

    if args.is_empty() {
        shortcuts::show_shortcut_help();
        return Ok(());
    }

    if args.len() == 1 {
        match args[0].as_str() {
            "-learn" => return shortcuts::cmd_shortcut_learn(),
            "-scan" => return shortcuts::cmd_shortcut_scan(),
            "-run" => return shortcuts::cmd_shortcut_run(),
            "-status" => return shortcuts::cmd_shortcut_status(),
            "-rollback" => return shortcuts::cmd_shortcut_rollback(),
            _ => {}
        }
    }

    if args.len() == 2 && args[0] == "-run" {
        match args[1].as_str() {
            "--auto-safe-only" => return shortcuts::cmd_shortcut_run_auto_safe_only(),
            "--assisted" => return shortcuts::cmd_shortcut_run(),
            _ => {}
        }
    }

    let cli = Cli::parse();
    app::run(cli)
}
