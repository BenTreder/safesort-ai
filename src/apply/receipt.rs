use serde::{Deserialize, Serialize};

/// An individual file operation recorded in an apply receipt.
/// Written after a successful move so rollback knows what to restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackEntry {
    /// Original source path (where the file lived before apply).
    pub original_source_path: String,
    /// Planned destination directory from the manifest.
    pub planned_destination: String,
    /// Resolved final file path (planned_destination + source filename if needed).
    /// Empty in receipts written before this field was added.
    #[serde(default)]
    pub final_destination_path: String,
    /// Path where a backup copy was stored before moving.
    pub backup_path: String,
    /// SHA-256 of the file before any operation.
    pub checksum_before: String,
    /// SHA-256 of the backup copy (must match checksum_before).
    pub checksum_after_backup: String,
    /// SHA-256 of the file at the destination after move.
    pub checksum_after_destination: String,
    /// File size in bytes.
    pub file_size: u64,
    /// ISO-8601 timestamp when the move completed.
    pub moved_at: String,
    /// Current rollback status for this entry.
    pub rollback_status: RollbackStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RollbackStatus {
    /// File was moved successfully; backup exists; can be rolled back.
    Moved,
    /// File was restored to original location.
    RolledBack,
    /// Rollback not possible (missing backup, checksum mismatch, or other error).
    CannotRollback,
    /// Dry-run: no files were actually moved.
    DryRun,
    /// Skipped during apply (not eligible or safety gate failed).
    Skipped,
}

/// Written to disk after a real apply run. Contains everything needed to
/// verify what was moved and restore files via `safesort rollback`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyReceipt {
    /// Unique run identifier (matches the manifest run_id that triggered this apply).
    pub run_id: String,
    /// ISO-8601 timestamp when apply completed.
    pub applied_at: String,
    /// Path to the plan manifest that was used.
    pub original_manifest_path: String,
    /// Directory where backup copies are stored.
    pub backup_dir: String,
    /// All entries attempted during this apply run.
    pub entries: Vec<RollbackEntry>,
    /// Whether this was a dry run (no real moves made).
    pub dry_run: bool,
    /// SafeSort version that performed this apply.
    pub safesort_version: String,
    /// Number of files successfully moved.
    pub total_moved: usize,
    /// Number of entries skipped (safety gate, not eligible, etc.).
    pub total_skipped: usize,
}
