use super::checksum::FileChecksum;
use serde::{Deserialize, Serialize};

/// A single entry in a rollback manifest — one planned (but not executed) file operation.
///
/// `dry_run_only` is always `true`. This struct describes what *would* happen,
/// not what has happened. No file is moved, copied, renamed, or deleted by
/// creating this struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Absolute or project-relative path of the file being considered.
    pub source_path: String,
    /// Recommended destination path (for display / future apply reference).
    pub planned_destination: String,
    /// SHA-256 checksum of the file at plan time, if the file was readable.
    pub checksum_before: Option<FileChecksum>,
    /// File size in bytes (0 if unreadable or not computed).
    pub file_size: u64,
    /// Safety classification: LOCKED, REVIEW, or SAFE.
    pub safety_level: String,
    /// Dependency impact: CRITICAL, HIGH, MEDIUM, LOW, or NONE.
    pub impact_level: String,
    /// Human-readable reason for this recommendation.
    pub reason: String,
    /// Placement confidence score (0–100).
    pub confidence: u8,
    /// Rule file path that was active during planning, if any.
    pub rule_file_used: Option<String>,
    /// Always `true` — this manifest describes a dry run, never a real apply.
    pub dry_run_only: bool,
    /// Whether this entry is eligible for safe autopilot (≥95% confidence, NONE/LOW impact, SAFE).
    pub auto_plan_eligible: bool,
}

/// A full rollback manifest produced by a planning run.
///
/// Contains everything needed to verify files before a future apply step
/// and to undo any moves if apply is ever implemented.
///
/// `dry_run_only` is always `true` at this phase. The manifest is a plan
/// artifact only — it does not represent a completed operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackManifest {
    /// Unique run identifier (timestamp-based, no external dependency).
    pub run_id: String,
    /// ISO-8601 timestamp when this manifest was generated.
    pub created_at: String,
    /// SafeSort AI version that generated this manifest.
    pub version: String,
    /// Path that was scanned.
    pub scan_target: String,
    /// Organization mode used during planning.
    pub plan_mode: String,
    /// All plan entries (SAFE_CANDIDATE + NONE/LOW impact only as move candidates).
    pub entries: Vec<ManifestEntry>,
    /// Total number of items scanned (including LOCKED and REVIEW).
    pub total_scanned: usize,
    /// Number of items excluded from the manifest due to LOCKED/REVIEW/HIGH-IMPACT status.
    pub excluded_for_safety: usize,
    /// Always `true` — apply is not implemented. This manifest is a dry run only.
    pub dry_run_only: bool,
    /// Human-readable safety note printed alongside the manifest.
    pub safety_note: String,
}

impl RollbackManifest {
    /// Construct an empty manifest with safe defaults.
    pub fn new(run_id: String, scan_target: String, plan_mode: String) -> Self {
        Self {
            run_id,
            created_at: chrono::Local::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            scan_target,
            plan_mode,
            entries: Vec::new(),
            total_scanned: 0,
            excluded_for_safety: 0,
            dry_run_only: true,
            safety_note: "This manifest is a DRY RUN ONLY. No files have been moved. \
                          apply is disabled in this build. \
                          Checksums are recorded so a future apply step can verify \
                          files have not changed between planning and execution."
                .to_string(),
        }
    }

    /// Return only entries that are safe autopilot eligible.
    pub fn auto_plan_entries(&self) -> Vec<&ManifestEntry> {
        self.entries
            .iter()
            .filter(|e| e.auto_plan_eligible)
            .collect()
    }

    /// Return entries that are plan-eligible but not auto-plan (guided review zone).
    pub fn guided_entries(&self) -> Vec<&ManifestEntry> {
        self.entries
            .iter()
            .filter(|e| !e.auto_plan_eligible)
            .collect()
    }
}
