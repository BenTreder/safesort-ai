use crate::apply::{ApplyOptions, apply_manifest, apply_status, rollback_apply};
use crate::error::{Result, SafeSortError};
use crate::manifest::build_plan_manifest;
use crate::placement::engine::{OrganizationMode, SmartPlacementEngine};
use crate::scan::Scanner;
use crate::scan::risk::SafetyLevel;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

// ─── Paths ─────────────────────────────────────────────────────────

pub fn manifests_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/share/safesort/manifests")
}

pub fn rollbacks_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/share/safesort/rollbacks")
}

// ─── Latest pointer ────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LatestPointer {
    pub manifest_path: String,
    pub scan_target: String,
    pub created_at: String,
}

pub fn load_latest_pointer() -> Result<Option<LatestPointer>> {
    let path = manifests_dir().join("latest.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| SafeSortError::InvalidPath(format!("Cannot read latest.json: {e}")))?;
    let pointer: LatestPointer = serde_json::from_str(&raw)
        .map_err(|e| SafeSortError::InvalidPath(format!("latest.json is malformed: {e}")))?;
    Ok(Some(pointer))
}

// ─── Hash helper ───────────────────────────────────────────────────

pub fn target_hash(target: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    target.hash(&mut hasher);
    format!("{:08x}", hasher.finish())
}

// ─── Newest rollback receipt ───────────────────────────────────────

pub fn find_newest_rollback_receipt() -> Option<PathBuf> {
    let dir = rollbacks_dir();
    if !dir.exists() {
        return None;
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    entries.sort();
    entries.pop()
}

// ─── Core scan logic (pure / testable) ────────────────────────────

/// Scan `target` using safe-autopilot mode, store a manifest under the
/// safesort manifests dir, and update `latest.json`. Returns the manifest path.
pub fn do_scan(target: &Path) -> Result<PathBuf> {
    let target = &target.to_path_buf();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let org = OrganizationMode::SafeAutopilot;
    let depth = 2;

    let excludes: Vec<String> = crate::config::DEFAULT_HEAVY_EXCLUDES
        .iter()
        .map(|s| s.to_string())
        .collect();

    let scanner = Scanner::new();
    let report = scanner.scan(target, &home, depth, &excludes)?;

    let items: Vec<(PathBuf, SafetyLevel)> = report
        .items
        .values()
        .flatten()
        .map(|item| {
            let level = match item.safety_level.as_str() {
                "LOCKED" => SafetyLevel::Locked,
                "REVIEW" => SafetyLevel::Review,
                _ => SafetyLevel::SafeCandidate,
            };
            (PathBuf::from(&item.path), level)
        })
        .collect();

    let engine = SmartPlacementEngine::new(home.clone(), org);
    let placement = engine.run(&items);

    let manifest = build_plan_manifest(
        target,
        org,
        &placement.recommendations,
        None,
        placement.summary.total_files,
    );
    let json = serde_json::to_string_pretty(&manifest)?;

    let mdir = manifests_dir();
    std::fs::create_dir_all(&mdir)?;

    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let hash = target_hash(target);
    let filename = format!("scan-{ts}-{hash}.json");
    let manifest_path = mdir.join(&filename);

    std::fs::write(&manifest_path, &json)?;

    let pointer = LatestPointer {
        manifest_path: manifest_path.to_string_lossy().to_string(),
        scan_target: target.to_string_lossy().to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
    };
    let pointer_json = serde_json::to_string_pretty(&pointer)?;
    std::fs::write(mdir.join("latest.json"), pointer_json)?;

    Ok(manifest_path)
}

// ─── Confirmation helpers ──────────────────────────────────────────

fn read_confirmation(prompt: &str, stdin: &mut impl BufRead, stdout: &mut impl Write) -> String {
    write!(stdout, "{}", prompt).ok();
    stdout.flush().ok();
    let mut line = String::new();
    stdin.read_line(&mut line).ok();
    line.trim().to_string()
}

// ─── safesort -scan ────────────────────────────────────────────────

pub fn cmd_shortcut_scan() -> Result<()> {
    let current_dir = std::env::current_dir().map_err(|e| {
        SafeSortError::InvalidPath(format!("Cannot determine current directory: {e}"))
    })?;

    println!();
    println!("  SafeSort AI — Quick Scan");
    println!("  Target: {}", current_dir.display());
    println!("  Mode:   safe-autopilot (depth 2, default excludes)");
    println!("  This is a DRY RUN — nothing will be moved.");
    println!();

    print!("  Building manifest...");
    io::stdout().flush().ok();
    let manifest_path = do_scan(&current_dir)?;
    println!(" done.");
    println!("  Manifest: {}", manifest_path.display());
    println!();

    // Preflight
    println!("  Running preflight...");
    let preflight_report = crate::preflight::run_preflight(&manifest_path)?;
    print!("{}", preflight_report.render());

    if !preflight_report.all_passed {
        println!("  Preflight did not fully pass — review the report above.");
        println!("  Nothing was moved.");
        return Ok(());
    }

    // Dry-run apply
    println!("  Running dry-run apply (safe-only)...");
    println!();

    let default_backup = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/share/safesort/backups")
        .join(format!(
            "dryrun-{}",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        ));

    let opts = ApplyOptions {
        manifest_path: &manifest_path,
        backup_dir: &default_backup,
        rollback_output: None,
        dry_run: true,
        apply_safe_only: true,
    };

    match apply_manifest(opts) {
        Ok(receipt) => {
            let would_move = receipt
                .entries
                .iter()
                .filter(|e| matches!(e.rollback_status, crate::apply::RollbackStatus::DryRun))
                .count();
            let would_skip = receipt
                .entries
                .iter()
                .filter(|e| matches!(e.rollback_status, crate::apply::RollbackStatus::Skipped))
                .count();
            println!();
            println!("  ─── Dry-Run Results ─────────────────────────────────────────");
            println!("  Would move:  {} file(s)", would_move);
            println!(
                "  Would skip:  {} file(s) (LOCKED/REVIEW/ineligible)",
                would_skip
            );
            println!("  Nothing was moved.");
            println!();
            if would_move > 0 {
                println!("  If the Would-move list looks correct, run from this same folder:");
                println!("    safesort -run");
            } else {
                println!("  No safe files to move in this folder.");
            }
        }
        Err(e) => {
            println!("  Dry-run error: {e}");
            println!("  Nothing was moved.");
        }
    }

    println!();
    Ok(())
}

// ─── safesort -run ─────────────────────────────────────────────────

pub fn cmd_shortcut_run() -> Result<()> {
    let current_dir = std::env::current_dir().map_err(|e| {
        SafeSortError::InvalidPath(format!("Cannot determine current directory: {e}"))
    })?;
    let current_dir_canonical = current_dir
        .canonicalize()
        .unwrap_or_else(|_| current_dir.clone());

    // Load latest pointer.
    let pointer = match load_latest_pointer()? {
        Some(p) => p,
        None => {
            println!();
            println!("  No latest scan found.");
            println!("  Run:  safesort -scan");
            println!("  Nothing was moved.");
            println!();
            return Ok(());
        }
    };

    // Verify current dir matches scan target.
    let pointer_target = PathBuf::from(&pointer.scan_target);
    let pointer_target_canonical = pointer_target
        .canonicalize()
        .unwrap_or_else(|_| pointer_target.clone());

    if pointer_target_canonical != current_dir_canonical {
        println!();
        println!("  Latest plan was for: {}", pointer.scan_target);
        println!("  Current folder is:   {}", current_dir.display());
        println!();
        println!("  These do not match. Run safesort -scan here first:");
        println!("    cd {}", current_dir.display());
        println!("    safesort -scan");
        println!("  Nothing was moved.");
        println!();
        return Ok(());
    }

    let manifest_path = PathBuf::from(&pointer.manifest_path);
    if !manifest_path.exists() {
        println!();
        println!(
            "  Latest manifest no longer exists: {}",
            pointer.manifest_path
        );
        println!("  Run:  safesort -scan");
        println!("  Nothing was moved.");
        println!();
        return Ok(());
    }

    println!();
    println!("  SafeSort AI — Quick Run");
    println!("  Target:   {}", current_dir.display());
    println!("  Manifest: {}", manifest_path.display());
    println!("  Scanned:  {}", pointer.created_at);
    println!();

    // Run preflight again.
    println!("  Running preflight...");
    let preflight_report = crate::preflight::run_preflight(&manifest_path)?;
    print!("{}", preflight_report.render());

    if !preflight_report.all_passed {
        println!("  Preflight did not pass — refusing to apply.");
        println!("  Nothing was moved.");
        return Ok(());
    }

    // Show dry-run summary.
    let default_backup = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/share/safesort/backups")
        .join(format!(
            "run-{}",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        ));

    let default_rollback_dir = rollbacks_dir();
    let rollback_path = default_rollback_dir.join(format!(
        "rollback-{}.json",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    ));

    let dry_opts = ApplyOptions {
        manifest_path: &manifest_path,
        backup_dir: &default_backup,
        rollback_output: None,
        dry_run: true,
        apply_safe_only: true,
    };

    let dry_receipt = match apply_manifest(dry_opts) {
        Ok(r) => r,
        Err(e) => {
            println!("  Dry-run failed: {e}");
            println!("  Nothing was moved.");
            return Ok(());
        }
    };

    let would_move = dry_receipt
        .entries
        .iter()
        .filter(|e| matches!(e.rollback_status, crate::apply::RollbackStatus::DryRun))
        .count();

    if would_move == 0 {
        println!("  No safe files to move. Nothing was moved.");
        return Ok(());
    }

    println!();
    println!("  ─── Files that would be moved ───────────────────────────────");
    println!("  {} file(s) will be moved.", would_move);
    println!();

    // Typed confirmation.
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut stdout = io::stdout();
    let confirmation = read_confirmation(
        "  Type ORGANIZE to continue (anything else cancels): ",
        &mut stdin_lock,
        &mut stdout,
    );

    if confirmation != "ORGANIZE" {
        println!();
        println!("  Cancelled. Nothing was moved.");
        println!();
        return Ok(());
    }

    println!();
    println!("  Applying...");
    println!();

    std::fs::create_dir_all(&default_rollback_dir)?;

    let opts = ApplyOptions {
        manifest_path: &manifest_path,
        backup_dir: &default_backup,
        rollback_output: Some(&rollback_path),
        dry_run: false,
        apply_safe_only: true,
    };

    match apply_manifest(opts) {
        Ok(receipt) => {
            println!();
            println!("  ─── Apply Complete ──────────────────────────────────────────");
            println!("  Files moved:   {}", receipt.total_moved);
            println!("  Files skipped: {}", receipt.total_skipped);
            println!("  Rollback receipt: {}", rollback_path.display());
            println!();
            println!("  To undo:  safesort -rollback");
            println!("  To check: safesort -status");
        }
        Err(e) => {
            println!("  Apply error: {e}");
        }
    }

    println!();
    Ok(())
}

// ─── safesort -status ──────────────────────────────────────────────

pub fn cmd_shortcut_status() -> Result<()> {
    println!();
    match find_newest_rollback_receipt() {
        None => {
            println!("  No rollback receipts found.");
            println!(
                "  (Receipts are stored under: {})",
                rollbacks_dir().display()
            );
        }
        Some(receipt_path) => {
            println!("  Latest rollback receipt: {}", receipt_path.display());
            println!();
            apply_status(&receipt_path)?;
        }
    }
    println!();
    Ok(())
}

// ─── safesort -rollback ────────────────────────────────────────────

pub fn cmd_shortcut_rollback() -> Result<()> {
    println!();
    let receipt_path = match find_newest_rollback_receipt() {
        None => {
            println!("  No rollback receipts found.");
            println!(
                "  (Receipts are stored under: {})",
                rollbacks_dir().display()
            );
            println!();
            return Ok(());
        }
        Some(p) => p,
    };

    println!("  Latest rollback receipt: {}", receipt_path.display());
    println!();
    println!("  WARNING: This will restore files from SafeSort freeze-state backups.");
    println!("  Files at their current locations will NOT be overwritten automatically.");
    println!();

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut stdout = io::stdout();
    let confirmation = read_confirmation(
        "  Type ROLLBACK to continue (anything else cancels): ",
        &mut stdin_lock,
        &mut stdout,
    );

    if confirmation != "ROLLBACK" {
        println!();
        println!("  Cancelled. No files were restored.");
        println!();
        return Ok(());
    }

    println!();
    println!("  Rolling back...");
    println!();

    // confirm_overwrite=false — do not bypass overwrite protections.
    rollback_apply(&receipt_path, false)?;

    println!();
    Ok(())
}

// ─── No-args help ──────────────────────────────────────────────────

pub fn show_shortcut_help() {
    println!();
    println!("  SafeSort AI Quick Commands");
    println!();
    println!("  Simple:");
    println!("    safesort -scan       Preview safe organization for the current folder");
    println!("    safesort -run        Apply the latest safe plan for this same folder");
    println!("    safesort -status     Show latest apply/rollback status");
    println!("    safesort -rollback   Roll back latest apply");
    println!();
    println!("  Advanced:");
    println!("    safesort organize ...");
    println!("    safesort preflight ...");
    println!("    safesort apply ...");
    println!("    safesort rollback ...");
    println!();
    println!("  Safety:");
    println!("    -scan never moves files");
    println!("    -run requires preflight, backup, and typed confirmation");
    println!("    LOCKED/REVIEW/high-impact files never move");
    println!();
}
