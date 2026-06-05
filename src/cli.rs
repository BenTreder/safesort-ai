use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "safesort",
    version,
    about = "SafeSort AI — Safety-First Folder Organizer",
    long_about = "SafeSort AI organizes folders safely without breaking apps, scripts, projects, services, system files, or important paths.\n\nThis is a safety-first read-only scanner with smart placement recommendations. Nothing is moved."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run environment and permission diagnostics
    Doctor,

    /// Generate fake test fixtures for demonstration and testing
    DemoFixture {
        #[arg(short, long, default_value = "safesort_demo")]
        output: String,
    },

    /// Scan a path and classify every item by safety
    Scan {
        /// Path to scan
        #[arg(long, required_unless_present = "home", conflicts_with = "home")]
        path: Option<String>,

        /// Scan the user's home directory
        #[arg(long, action)]
        home: bool,

        /// Organization mode: preview (default), guided, safe-autopilot, locked-down
        #[arg(long, value_enum, default_value = "preview")]
        mode: OrgMode,

        /// Output format
        #[arg(long, value_enum, default_value = "terminal")]
        format: OutputFormat,

        /// Optional output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Maximum traversal depth (default: 2)
        #[arg(long, default_value = "2")]
        depth: usize,

        /// Exclude paths matching this name or substring (repeatable)
        #[arg(long, action = clap::ArgAction::Append, value_name = "PATTERN")]
        exclude: Vec<String>,
    },

    /// Generate a smart placement plan with recommendations
    Plan {
        /// Path to plan
        #[arg(long, required_unless_present = "home", conflicts_with = "home")]
        path: Option<String>,

        /// Plan the user's home directory
        #[arg(long, action)]
        home: bool,

        /// Organization mode: preview (default), guided, safe-autopilot, locked-down
        #[arg(long, value_enum, default_value = "preview")]
        mode: OrgMode,

        /// Optional output file path for the plan
        #[arg(short, long)]
        output: Option<String>,

        /// Maximum traversal depth (default: 2)
        #[arg(long, default_value = "2")]
        depth: usize,

        /// Exclude paths matching this name or substring (repeatable)
        #[arg(long, action = clap::ArgAction::Append, value_name = "PATTERN")]
        exclude: Vec<String>,
    },

    /// Analyze user profile from a path
    Profile {
        /// Path to analyze
        #[arg(long, required_unless_present = "home", conflicts_with = "home")]
        path: Option<String>,

        /// Analyze the user's home directory
        #[arg(long, action)]
        home: bool,
    },

    /// Explain the safety decision for a specific path
    Explain {
        /// Path to explain
        path: String,
    },

    /// Apply a plan (DISABLED in this safety-first build)
    Apply {
        /// Path to the plan file (ignored)
        _plan: Option<String>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Terminal,
    Json,
    Markdown,
}

#[derive(Clone, Copy, ValueEnum, Debug)]
pub enum OrgMode {
    Preview,
    Guided,
    SafeAutopilot,
    LockedDown,
}
