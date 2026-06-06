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

        /// Path to a TOML rule file for custom aliases, protected paths, and staging destinations
        #[arg(long, value_name = "FILE")]
        rule_file: Option<String>,
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

        /// Path to a TOML rule file for custom aliases, protected paths, and staging destinations
        #[arg(long, value_name = "FILE")]
        rule_file: Option<String>,

        /// Write a dry-run rollback manifest to this file (JSON). Manifest only — nothing is moved.
        #[arg(long, value_name = "FILE")]
        manifest_output: Option<String>,
    },

    /// Generate a dry-run rollback manifest with checksums (nothing is moved)
    Manifest {
        /// Path to scan
        #[arg(long)]
        path: String,

        /// Maximum traversal depth (default: 2)
        #[arg(long, default_value = "2")]
        depth: usize,

        /// Exclude paths matching this name or substring (repeatable)
        #[arg(long, action = clap::ArgAction::Append, value_name = "PATTERN")]
        exclude: Vec<String>,

        /// Path to a TOML rule file
        #[arg(long, value_name = "FILE")]
        rule_file: Option<String>,

        /// Write manifest JSON to this file instead of stdout
        #[arg(long, value_name = "FILE")]
        output: Option<String>,
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

        /// Path to a TOML rule file for custom aliases, protected paths, and staging destinations
        #[arg(long, value_name = "FILE")]
        rule_file: Option<String>,
    },

    /// Validate a manifest before any future apply (moves nothing)
    Preflight {
        /// Path to the manifest JSON file produced by `safesort manifest` or `safesort plan --manifest-output`
        manifest: String,
    },

    /// Premium guided organization workflow (safe — nothing is moved)
    Organize {
        #[arg(long)]
        path: Option<String>,
        #[arg(long, value_enum, default_value = "preview")]
        mode: OrgMode,
        #[arg(long, default_value = "2")]
        depth: usize,
        #[arg(long, action = clap::ArgAction::Append, value_name = "PATTERN")]
        exclude: Vec<String>,
        #[arg(long, value_name = "FILE")]
        rule_file: Option<String>,
        #[arg(long, value_name = "FILE")]
        manifest_output: Option<String>,
        /// Skip default heavy-folder auto-excludes (node_modules, target, .venv, etc.)
        #[arg(long)]
        no_default_excludes: bool,
    },

    /// Apply a plan manifest — moves only SAFE/NONE-LOW-impact auto-eligible files
    ///
    /// Requires --confirm, --i-understand-this-moves-files, --backup, and --apply-safe-only.
    /// Creates a freeze-state backup before each move. Writes a rollback receipt.
    /// Use --dry-run to preview without moving anything.
    Apply {
        /// Path to the SafeSort plan manifest JSON file
        manifest: Option<String>,

        /// First required acknowledgement flag
        #[arg(long)]
        confirm: bool,

        /// Second required acknowledgement flag
        #[arg(long = "i-understand-this-moves-files")]
        i_understand: bool,

        /// Required: create a freeze-state backup before moving each file
        #[arg(long)]
        backup: bool,

        /// Only move entries with auto_plan_eligible=true (≥95% confidence, NONE/LOW impact, SAFE)
        #[arg(long = "apply-safe-only")]
        apply_safe_only: bool,

        /// Preview what would be moved without actually moving anything
        #[arg(long = "dry-run")]
        dry_run: bool,

        /// Override the backup/freeze directory (default: ~/.local/share/safesort/backups/<run_id>/)
        #[arg(long = "backup-dir")]
        backup_dir: Option<String>,

        /// Write the rollback receipt to this file
        #[arg(long = "rollback-output")]
        rollback_output: Option<String>,
    },

    /// Show the status of a previous apply run (moves nothing)
    ApplyStatus {
        /// Path to the rollback receipt JSON
        receipt: String,
    },

    /// Restore files moved by a previous apply run using the rollback receipt
    Rollback {
        /// Path to the rollback receipt JSON written by apply
        receipt: String,

        /// Allow overwriting existing files at the original source paths
        #[arg(long = "confirm-overwrite-rollback")]
        confirm_overwrite: bool,
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
