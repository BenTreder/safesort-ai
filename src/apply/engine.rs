use super::receipt::{ApplyReceipt, RollbackEntry, RollbackStatus};
use crate::config::DANGEROUS_ROOTS;
use crate::error::{Result, SafeSortError};
use crate::manifest::checksum::checksum_file;
use crate::manifest::rollback::RollbackManifest;
use crate::preflight;
use std::fs;
use std::path::{Path, PathBuf};

// Known live-site path components to reject as destinations.
const LIVE_DEST_REJECT: &[&str] = &[
    "public_html",
    "www",
    "/etc/",
    "/usr/",
    "/var/",
    "/boot/",
    "/run/",
    "/proc/",
    "/sys/",
    "/dev/",
];

/// Top-level options for an apply run.
pub struct ApplyOptions<'a> {
    pub manifest_path: &'a Path,
    pub backup_dir: &'a Path,
    pub rollback_output: Option<&'a Path>,
    pub dry_run: bool,
    /// Only move entries with auto_plan_eligible=true.
    pub apply_safe_only: bool,
}

/// Validate that a destination path is safe to write to.
fn is_safe_destination(dest: &Path) -> bool {
    let dest_str = dest.to_string_lossy();
    for root in DANGEROUS_ROOTS {
        if dest_str == *root || dest_str.starts_with(&format!("{root}/")) {
            return false;
        }
    }
    for live in LIVE_DEST_REJECT {
        if dest_str.contains(live) {
            return false;
        }
    }
    true
}

/// Validate that the backup directory is safe to write to.
fn is_safe_backup_dir(backup_dir: &Path) -> bool {
    let s = backup_dir.to_string_lossy();
    for root in DANGEROUS_ROOTS {
        if s == *root || s.starts_with(&format!("{root}/")) {
            return false;
        }
    }
    true
}

/// Load and validate a SafeSort plan manifest. Returns `Err` if the file is
/// not a valid SafeSort plan manifest (missing required fields, dry_run_only=false, etc.)
pub fn load_plan_manifest(manifest_path: &Path) -> Result<RollbackManifest> {
    let raw = fs::read_to_string(manifest_path).map_err(|e| {
        SafeSortError::InvalidPath(format!(
            "Cannot read manifest {}: {e}",
            manifest_path.display()
        ))
    })?;

    let manifest: RollbackManifest = serde_json::from_str(&raw).map_err(|e| {
        SafeSortError::InvalidPath(format!(
            "Manifest is not a valid SafeSort JSON manifest: {e}"
        ))
    })?;

    // Verify it is a genuine SafeSort plan manifest.
    if manifest.version.is_empty() || manifest.run_id.is_empty() {
        return Err(SafeSortError::InvalidPath(
            "Manifest is missing required SafeSort fields (version, run_id). \
             This does not appear to be a SafeSort-generated manifest."
                .to_string(),
        ));
    }

    // The plan manifest must have dry_run_only=true — it was generated as a plan,
    // not as an apply receipt. We load it and then we are doing the real apply.
    if !manifest.dry_run_only {
        return Err(SafeSortError::InvalidPath(
            "Manifest dry_run_only is false. SafeSort only applies \
             plan manifests that were created with dry_run_only=true."
                .to_string(),
        ));
    }

    Ok(manifest)
}

/// Per-entry safety gate. Returns `Ok(())` if the entry may be moved,
/// or `Err(reason)` explaining why it was refused.
fn entry_safety_gate(
    entry: &crate::manifest::rollback::ManifestEntry,
    apply_safe_only: bool,
) -> std::result::Result<(), String> {
    // Gate 1: auto_plan_eligible required when apply_safe_only.
    if apply_safe_only && !entry.auto_plan_eligible {
        return Err(format!(
            "Entry is not auto_plan_eligible (confidence {}%, impact {}, safety {})",
            entry.confidence, entry.impact_level, entry.safety_level
        ));
    }

    // Gate 2: safety level must be SAFE.
    if entry.safety_level.to_uppercase() != "SAFE" {
        return Err(format!(
            "Safety level is {} — only SAFE entries can be moved",
            entry.safety_level
        ));
    }

    // Gate 3: impact must be NONE or LOW.
    let impact = entry.impact_level.to_uppercase();
    if impact != "NONE" && impact != "LOW" {
        return Err(format!(
            "Impact level is {} — only NONE or LOW impact entries can be moved",
            entry.impact_level
        ));
    }

    // Gate 4: must have a checksum.
    if entry.checksum_before.is_none() {
        return Err("No checksum recorded — cannot verify file integrity".to_string());
    }

    Ok(())
}

/// Verify a source file's current state matches the manifest entry.
/// Returns `Ok(current_sha256)` or `Err(reason)`.
fn verify_source(
    source: &Path,
    entry: &crate::manifest::rollback::ManifestEntry,
) -> std::result::Result<String, String> {
    if !source.exists() {
        return Err(format!(
            "Source file no longer exists: {}",
            source.display()
        ));
    }

    let checksum_data = entry.checksum_before.as_ref().unwrap(); // already checked in gate

    // Verify file size first (cheap).
    let metadata =
        std::fs::metadata(source).map_err(|e| format!("Cannot read source metadata: {e}"))?;
    if metadata.len() != checksum_data.file_size {
        return Err(format!(
            "File size changed since manifest creation (expected {}, got {})",
            checksum_data.file_size,
            metadata.len()
        ));
    }

    // Verify SHA-256.
    let current = checksum_file(source).map_err(|e| format!("Cannot compute checksum: {e}"))?;
    if current.sha256 != checksum_data.sha256 {
        return Err(format!(
            "Checksum mismatch — file changed since manifest creation \
             (expected {}, got {})",
            &checksum_data.sha256[..16],
            &current.sha256[..16]
        ));
    }

    Ok(current.sha256)
}

/// Compute backup path for a source file under backup_dir.
/// Uses the full source path to avoid collisions.
fn backup_path_for(source: &Path, backup_dir: &Path) -> PathBuf {
    // Strip leading '/' and use the full path hierarchy under backup_dir.
    let relative = source.strip_prefix("/").unwrap_or(source);
    backup_dir.join(relative)
}

/// Run a complete apply operation. Moves only eligible files, creates backups
/// before each move, writes a rollback receipt.
pub fn apply_manifest(opts: ApplyOptions<'_>) -> Result<ApplyReceipt> {
    // Validate backup dir safety.
    if !is_safe_backup_dir(opts.backup_dir) {
        return Err(SafeSortError::InvalidPath(format!(
            "Backup directory is unsafe: {}",
            opts.backup_dir.display()
        )));
    }

    // Load and validate the plan manifest.
    let plan = load_plan_manifest(opts.manifest_path)?;

    // Run preflight to catch any issues before we start.
    if !opts.dry_run {
        let preflight_report = preflight::run_preflight(opts.manifest_path)
            .map_err(|e| SafeSortError::InvalidPath(format!("Preflight error: {e}")))?;
        if !preflight_report.all_passed {
            return Err(SafeSortError::InvalidPath(
                "Preflight checks failed — cannot proceed with apply. \
                 Run `safesort preflight <MANIFEST>` for details."
                    .to_string(),
            ));
        }
    }

    let run_id = format!("apply-{}", chrono::Local::now().format("%Y%m%d-%H%M%S"));
    let applied_at = chrono::Local::now().to_rfc3339();

    let mut entries: Vec<RollbackEntry> = Vec::new();
    let mut total_moved = 0usize;
    let mut total_skipped = 0usize;

    for entry in &plan.entries {
        let source = PathBuf::from(&entry.source_path);
        let destination = PathBuf::from(&entry.planned_destination);

        // Per-entry safety gate.
        match entry_safety_gate(entry, opts.apply_safe_only) {
            Err(reason) => {
                println!("  SKIP  {}", source.display());
                println!("        Reason: {reason}");
                total_skipped += 1;
                entries.push(RollbackEntry {
                    original_source_path: entry.source_path.clone(),
                    planned_destination: entry.planned_destination.clone(),
                    backup_path: String::new(),
                    checksum_before: entry
                        .checksum_before
                        .as_ref()
                        .map(|c| c.sha256.clone())
                        .unwrap_or_default(),
                    checksum_after_backup: String::new(),
                    checksum_after_destination: String::new(),
                    file_size: entry.file_size,
                    moved_at: applied_at.clone(),
                    rollback_status: RollbackStatus::Skipped,
                });
                continue;
            }
            Ok(()) => {}
        }

        // Verify destination safety.
        if !is_safe_destination(&destination) {
            println!(
                "  SKIP  {} — destination is unsafe: {}",
                source.display(),
                destination.display()
            );
            total_skipped += 1;
            entries.push(skipped_entry(entry, &applied_at));
            continue;
        }

        // Verify source current state.
        let checksum_before = match verify_source(&source, entry) {
            Ok(cs) => cs,
            Err(reason) => {
                println!("  SKIP  {} — {reason}", source.display());
                total_skipped += 1;
                entries.push(skipped_entry(entry, &applied_at));
                continue;
            }
        };

        // Destination must not already exist.
        if destination.exists() {
            println!(
                "  SKIP  {} — destination already exists: {}",
                source.display(),
                destination.display()
            );
            total_skipped += 1;
            entries.push(skipped_entry(entry, &applied_at));
            continue;
        }

        // Dry-run: record without moving.
        if opts.dry_run {
            let backup_path = backup_path_for(&source, opts.backup_dir);
            println!(
                "  DRY-RUN  {} → {}",
                source.display(),
                destination.display()
            );
            println!("           backup would be: {}", backup_path.display());
            total_skipped += 1;
            entries.push(RollbackEntry {
                original_source_path: entry.source_path.clone(),
                planned_destination: entry.planned_destination.clone(),
                backup_path: backup_path.to_string_lossy().to_string(),
                checksum_before: checksum_before.clone(),
                checksum_after_backup: String::new(),
                checksum_after_destination: String::new(),
                file_size: entry.file_size,
                moved_at: applied_at.clone(),
                rollback_status: RollbackStatus::DryRun,
            });
            continue;
        }

        // Real apply path:

        // Step 1: Create backup directory and copy source to backup.
        let backup_path = backup_path_for(&source, opts.backup_dir);
        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SafeSortError::InvalidPath(format!("Cannot create backup directory: {e}"))
            })?;
        }

        println!("  BACKUP  {} → {}", source.display(), backup_path.display());
        fs::copy(&source, &backup_path).map_err(|e| {
            SafeSortError::InvalidPath(format!(
                "Cannot copy {} to backup {}: {e}",
                source.display(),
                backup_path.display()
            ))
        })?;

        // Step 2: Verify backup checksum.
        let backup_checksum = checksum_file(&backup_path)
            .map_err(|e| SafeSortError::InvalidPath(format!("Cannot checksum backup: {e}")))?;
        if backup_checksum.sha256 != checksum_before {
            return Err(SafeSortError::InvalidPath(format!(
                "Backup checksum mismatch for {} — aborting apply for safety. \
                 Files moved so far have receipts; run apply-status to review.",
                source.display()
            )));
        }

        // Step 3: Create destination parent directory.
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SafeSortError::InvalidPath(format!("Cannot create destination directory: {e}"))
            })?;
        }

        // Step 4: Move source to destination.
        println!("  MOVE    {} → {}", source.display(), destination.display());
        fs::rename(&source, &destination).map_err(|e| {
            SafeSortError::InvalidPath(format!(
                "Cannot move {} to {}: {e}",
                source.display(),
                destination.display()
            ))
        })?;

        // Step 5: Verify destination checksum.
        let dest_checksum = checksum_file(&destination)
            .map_err(|e| SafeSortError::InvalidPath(format!("Cannot checksum destination: {e}")))?;
        if dest_checksum.sha256 != checksum_before {
            return Err(SafeSortError::InvalidPath(format!(
                "Destination checksum mismatch for {} — file may be corrupt. \
                 Backup is at {}",
                destination.display(),
                backup_path.display()
            )));
        }

        println!("  OK      checksum verified at destination");

        entries.push(RollbackEntry {
            original_source_path: entry.source_path.clone(),
            planned_destination: entry.planned_destination.clone(),
            backup_path: backup_path.to_string_lossy().to_string(),
            checksum_before: checksum_before.clone(),
            checksum_after_backup: backup_checksum.sha256,
            checksum_after_destination: dest_checksum.sha256,
            file_size: entry.file_size,
            moved_at: chrono::Local::now().to_rfc3339(),
            rollback_status: RollbackStatus::Moved,
        });
        total_moved += 1;
    }

    let receipt = ApplyReceipt {
        run_id: run_id.clone(),
        applied_at,
        original_manifest_path: opts.manifest_path.to_string_lossy().to_string(),
        backup_dir: opts.backup_dir.to_string_lossy().to_string(),
        entries,
        dry_run: opts.dry_run,
        safesort_version: env!("CARGO_PKG_VERSION").to_string(),
        total_moved,
        total_skipped,
    };

    // Write rollback receipt.
    if let Some(rollback_out) = opts.rollback_output {
        let json = serde_json::to_string_pretty(&receipt).map_err(|e| {
            SafeSortError::InvalidPath(format!("Cannot serialize rollback receipt: {e}"))
        })?;
        if let Some(parent) = rollback_out.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    SafeSortError::InvalidPath(format!(
                        "Cannot create rollback output directory: {e}"
                    ))
                })?;
            }
        }
        fs::write(rollback_out, &json).map_err(|e| {
            SafeSortError::InvalidPath(format!(
                "Cannot write rollback receipt to {}: {e}",
                rollback_out.display()
            ))
        })?;
        println!();
        println!("  Rollback receipt written to: {}", rollback_out.display());
    }

    Ok(receipt)
}

fn skipped_entry(
    entry: &crate::manifest::rollback::ManifestEntry,
    applied_at: &str,
) -> RollbackEntry {
    RollbackEntry {
        original_source_path: entry.source_path.clone(),
        planned_destination: entry.planned_destination.clone(),
        backup_path: String::new(),
        checksum_before: entry
            .checksum_before
            .as_ref()
            .map(|c| c.sha256.clone())
            .unwrap_or_default(),
        checksum_after_backup: String::new(),
        checksum_after_destination: String::new(),
        file_size: entry.file_size,
        moved_at: applied_at.to_string(),
        rollback_status: RollbackStatus::Skipped,
    }
}

/// Show the status of a previous apply run without moving anything.
pub fn apply_status(receipt_path: &Path) -> Result<()> {
    let raw = fs::read_to_string(receipt_path)
        .map_err(|e| SafeSortError::InvalidPath(format!("Cannot read receipt: {e}")))?;
    let receipt: ApplyReceipt = serde_json::from_str(&raw)
        .map_err(|e| SafeSortError::InvalidPath(format!("Invalid receipt JSON: {e}")))?;

    println!();
    println!("  SafeSort AI — Apply Status");
    println!("  Run ID:    {}", receipt.run_id);
    println!("  Applied:   {}", receipt.applied_at);
    println!("  Manifest:  {}", receipt.original_manifest_path);
    println!("  Backup:    {}", receipt.backup_dir);
    println!("  Dry run:   {}", receipt.dry_run);
    println!("  Moved:     {}", receipt.total_moved);
    println!("  Skipped:   {}", receipt.total_skipped);
    println!();

    for e in &receipt.entries {
        let status = match e.rollback_status {
            RollbackStatus::Moved => "MOVED",
            RollbackStatus::RolledBack => "ROLLED BACK",
            RollbackStatus::CannotRollback => "CANNOT ROLLBACK",
            RollbackStatus::DryRun => "DRY-RUN",
            RollbackStatus::Skipped => "SKIPPED",
        };

        let src = PathBuf::from(&e.original_source_path);
        let dest = PathBuf::from(&e.planned_destination);
        let src_exists = src.exists();
        let dest_exists = dest.exists();
        let backup_exists = !e.backup_path.is_empty() && PathBuf::from(&e.backup_path).exists();

        println!("  [{status}]");
        println!("    From:    {}", e.original_source_path);
        println!("    To:      {}", e.planned_destination);
        println!(
            "    Source exists: {}  Dest exists: {}  Backup exists: {}",
            src_exists, dest_exists, backup_exists
        );
    }

    println!();
    println!("  Nothing was moved.");
    Ok(())
}

/// Restore files moved by a previous apply run.
/// Uses backup copies to restore files to their original paths.
pub fn rollback_apply(receipt_path: &Path, confirm_overwrite: bool) -> Result<()> {
    let raw = fs::read_to_string(receipt_path)
        .map_err(|e| SafeSortError::InvalidPath(format!("Cannot read receipt: {e}")))?;
    let mut receipt: ApplyReceipt = serde_json::from_str(&raw)
        .map_err(|e| SafeSortError::InvalidPath(format!("Invalid receipt JSON: {e}")))?;

    println!();
    println!("  SafeSort AI — Rollback");
    println!("  Run ID:  {}", receipt.run_id);
    println!();

    let mut restored = 0usize;
    let mut refused = 0usize;

    for entry in receipt.entries.iter_mut() {
        if entry.rollback_status != RollbackStatus::Moved {
            println!(
                "  SKIP  {} — status is {:?}, not Moved",
                entry.original_source_path, entry.rollback_status
            );
            continue;
        }

        let original = PathBuf::from(&entry.original_source_path);
        let backup = PathBuf::from(&entry.backup_path);
        let dest = PathBuf::from(&entry.planned_destination);

        // Verify backup exists.
        if !backup.exists() {
            println!(
                "  REFUSE  {} — backup no longer exists: {}",
                entry.original_source_path,
                backup.display()
            );
            entry.rollback_status = RollbackStatus::CannotRollback;
            refused += 1;
            continue;
        }

        // Verify backup checksum matches original.
        let backup_cs = checksum_file(&backup)
            .map_err(|e| SafeSortError::InvalidPath(format!("Cannot checksum backup: {e}")))?;
        if backup_cs.sha256 != entry.checksum_before {
            println!(
                "  REFUSE  {} — backup checksum mismatch (backup may be tampered)",
                entry.original_source_path
            );
            entry.rollback_status = RollbackStatus::CannotRollback;
            refused += 1;
            continue;
        }

        // Check if original source path already has a file.
        if original.exists() {
            if !confirm_overwrite {
                println!(
                    "  REFUSE  {} — original path already exists. \
                     Pass --confirm-overwrite-rollback to overwrite.",
                    entry.original_source_path
                );
                refused += 1;
                continue;
            }
            println!(
                "  WARN    Overwriting existing file at {}",
                original.display()
            );
        }

        // Restore: copy backup → original, then remove destination.
        if let Some(parent) = original.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SafeSortError::InvalidPath(format!("Cannot create restore directory: {e}"))
            })?;
        }

        println!("  RESTORE  {} → {}", backup.display(), original.display());
        fs::copy(&backup, &original)
            .map_err(|e| SafeSortError::InvalidPath(format!("Cannot restore from backup: {e}")))?;

        // Verify restored checksum.
        let restored_cs = checksum_file(&original).map_err(|e| {
            SafeSortError::InvalidPath(format!("Cannot checksum restored file: {e}"))
        })?;
        if restored_cs.sha256 != entry.checksum_before {
            return Err(SafeSortError::InvalidPath(format!(
                "Restored file checksum mismatch for {} — aborting rollback",
                original.display()
            )));
        }

        // Remove the file from destination (it was moved there; backup is the source of truth).
        if dest.exists() {
            println!("  REMOVE  destination copy: {}", dest.display());
            fs::remove_file(&dest).map_err(|e| {
                SafeSortError::InvalidPath(format!(
                    "Cannot remove destination file {}: {e}",
                    dest.display()
                ))
            })?;
        }

        entry.rollback_status = RollbackStatus::RolledBack;
        restored += 1;
        println!("  OK      restored {}", original.display());
    }

    // Update the receipt on disk to reflect new rollback statuses.
    let json = serde_json::to_string_pretty(&receipt).map_err(|e| {
        SafeSortError::InvalidPath(format!("Cannot serialize updated receipt: {e}"))
    })?;
    fs::write(receipt_path, &json)
        .map_err(|e| SafeSortError::InvalidPath(format!("Cannot update receipt file: {e}")))?;

    println!();
    println!("  Rollback complete. Restored: {restored}  Refused: {refused}");
    println!("  Nothing was moved (rollback copies from backup).");
    Ok(())
}
