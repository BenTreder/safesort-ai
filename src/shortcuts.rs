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

// ─── Scan summary ──────────────────────────────────────────────────

/// Category counts from a scan run, used for display in -scan.
#[derive(Debug, Default, Clone)]
pub struct ScanCounts {
    pub auto_safe: usize,
    pub assisted: usize,
    pub review_only: usize,
    pub never_touch: usize,
    pub total: usize,
}

/// Rich result returned by do_scan.
#[derive(Debug)]
pub struct DoScanResult {
    pub manifest_path: PathBuf,
    pub counts: ScanCounts,
    /// (source_name, destination_label) pairs for preview (limited to first 10 auto + 10 assisted)
    pub auto_preview: Vec<(String, String)>,
    pub assisted_preview: Vec<(String, String)>,
}

// ─── Core scan logic (pure / testable) ────────────────────────────

/// Scan `target`, store a manifest, and return counts and preview lists.
pub fn do_scan(target: &Path) -> Result<PathBuf> {
    do_scan_full(target).map(|r| r.manifest_path)
}

/// Like do_scan but returns the full DoScanResult (counts + previews).
pub fn do_scan_full(target: &Path) -> Result<DoScanResult> {
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

    // Count LOCKED items from the scan report (never_touch)
    let never_touch_count = report
        .items
        .values()
        .flatten()
        .filter(|item| item.safety_level == "LOCKED")
        .count();

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

    // Compute counts from manifest entries
    let auto_safe = manifest.entries.iter().filter(|e| e.auto_plan_eligible).count();
    let assisted = manifest.entries.iter().filter(|e| e.assisted_plan_eligible).count();
    let review_only_entries = manifest
        .entries
        .iter()
        .filter(|e| !e.auto_plan_eligible && !e.assisted_plan_eligible)
        .count();
    // REVIEW-level files go into excluded_for_safety but we captured never_touch separately;
    // remaining excluded items are REVIEW-level
    let review_level = manifest.excluded_for_safety.saturating_sub(never_touch_count);
    let review_only = review_only_entries + review_level;
    let total = manifest.total_scanned;

    // Build preview lists (up to 10 each)
    let auto_preview: Vec<(String, String)> = manifest
        .entries
        .iter()
        .filter(|e| e.auto_plan_eligible)
        .take(10)
        .map(|e| {
            let name = PathBuf::from(&e.source_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| e.source_path.clone());
            (name, e.planned_destination.clone())
        })
        .collect();

    let assisted_preview: Vec<(String, String)> = manifest
        .entries
        .iter()
        .filter(|e| e.assisted_plan_eligible)
        .take(10)
        .map(|e| {
            let name = PathBuf::from(&e.source_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| e.source_path.clone());
            (name, e.planned_destination.clone())
        })
        .collect();

    Ok(DoScanResult {
        manifest_path,
        counts: ScanCounts {
            auto_safe,
            assisted,
            review_only,
            never_touch: never_touch_count,
            total,
        },
        auto_preview,
        assisted_preview,
    })
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
    println!("  Mode:   safe-autopilot + assisted-eligible (depth 2, default excludes)");
    println!("  This is a DRY RUN — nothing will be moved.");
    println!();

    print!("  Building manifest...");
    io::stdout().flush().ok();
    let scan_result = do_scan_full(&current_dir)?;
    println!(" done.");
    println!("  Manifest: {}", scan_result.manifest_path.display());
    println!();

    let c = &scan_result.counts;
    println!("  ─── Scan Results ────────────────────────────────────────────");
    println!(
        "  AUTO-SAFE:    {:>4}  (can move immediately in safe mode)",
        c.auto_safe
    );
    println!(
        "  ASSISTED:     {:>4}  (can organize with backup + rollback)",
        c.assisted
    );
    println!(
        "  REVIEW ONLY:  {:>4}  (need manual review)",
        c.review_only
    );
    println!(
        "  NEVER TOUCH:  {:>4}  (system/project/risky files)",
        c.never_touch
    );
    println!();
    println!("  Total scanned: {}", c.total);
    println!();

    if !scan_result.auto_preview.is_empty() {
        println!("  ─── AUTO-SAFE Preview ───────────────────────────────────────");
        for (name, dest) in &scan_result.auto_preview {
            println!("    {}  →  {}", name, dest);
        }
        if c.auto_safe > scan_result.auto_preview.len() {
            println!("    ... and {} more", c.auto_safe - scan_result.auto_preview.len());
        }
        println!();
    }

    if !scan_result.assisted_preview.is_empty() {
        println!("  ─── ASSISTED Preview ────────────────────────────────────────");
        for (name, dest) in &scan_result.assisted_preview {
            println!("    {}  →  {}", name, dest);
        }
        if c.assisted > scan_result.assisted_preview.len() {
            println!("    ... and {} more", c.assisted - scan_result.assisted_preview.len());
        }
        println!();
    }

    println!("  Nothing moved.");
    println!();

    let movable = c.auto_safe + c.assisted;
    if movable > 0 {
        println!(
            "  Run `safesort -run` to organize {} AUTO-SAFE + ASSISTED files",
            movable
        );
        println!("  with freeze-state backup. After organizing, SafeSort will ask");
        println!("  whether to KEEP or ROLLBACK.");
        if c.auto_safe > 0 {
            println!();
            println!("  Run `safesort -run --auto-safe-only` to move only AUTO-SAFE files.");
        }
    } else {
        println!("  No files are ready to organize in this folder.");
    }

    println!();
    Ok(())
}

// ─── safesort -run (shared implementation) ─────────────────────────

/// Whether -run operates in assisted mode (default) or auto-safe-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Move AUTO-SAFE + ASSISTED files. Default.
    Assisted,
    /// Move only AUTO-SAFE files (strict conservative mode).
    AutoSafeOnly,
}

pub fn cmd_shortcut_run() -> Result<()> {
    cmd_shortcut_run_mode(RunMode::Assisted)
}

pub fn cmd_shortcut_run_auto_safe_only() -> Result<()> {
    cmd_shortcut_run_mode(RunMode::AutoSafeOnly)
}

pub fn cmd_shortcut_run_mode(mode: RunMode) -> Result<()> {
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

    let mode_label = match mode {
        RunMode::Assisted => "ASSISTED (AUTO-SAFE + ASSISTED files)",
        RunMode::AutoSafeOnly => "AUTO-SAFE ONLY",
    };

    println!();
    println!("  SafeSort AI — Quick Run");
    println!("  Target:   {}", current_dir.display());
    println!("  Manifest: {}", manifest_path.display());
    println!("  Scanned:  {}", pointer.created_at);
    println!("  Mode:     {}", mode_label);
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

    let apply_safe_only = mode == RunMode::AutoSafeOnly;
    let assisted_mode = mode == RunMode::Assisted;

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

    // Dry-run to count what would move.
    let dry_opts = ApplyOptions {
        manifest_path: &manifest_path,
        backup_dir: &default_backup,
        rollback_output: None,
        dry_run: true,
        apply_safe_only,
        assisted_mode,
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
        println!("  No eligible files to move in this mode. Nothing was moved.");
        if mode == RunMode::AutoSafeOnly {
            println!("  Try `safesort -run --assisted` or re-scan with `safesort -scan`.");
        }
        return Ok(());
    }

    println!();
    println!("  ─── Files that will be moved ────────────────────────────────");
    println!("  {} file(s) will be organized.", would_move);
    println!();
    println!("  Safety: freeze-state backup will be created before each move.");
    println!("  You will be asked to KEEP or ROLLBACK after organizing.");
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
        apply_safe_only,
        assisted_mode,
    };

    match apply_manifest(opts) {
        Ok(receipt) => {
            println!();
            println!("  ─── Organize Complete ───────────────────────────────────────");
            println!("  Files moved:      {}", receipt.total_moved);
            println!("  Files skipped:    {}", receipt.total_skipped);
            println!("  Rollback receipt: {}", rollback_path.display());
            println!();

            // Post-apply KEEP or ROLLBACK prompt.
            println!("  SafeSort moved {} file(s).", receipt.total_moved);
            println!("  Type KEEP to keep these changes, or ROLLBACK to undo now:");
            println!();

            let answer = read_confirmation(
                "  Your choice (KEEP / ROLLBACK): ",
                &mut stdin_lock,
                &mut stdout,
            );

            println!();
            if answer == "ROLLBACK" {
                println!("  Rolling back now...");
                println!();
                rollback_apply(&rollback_path, false)?;
                println!();
                println!("  Rollback complete. Files restored.");
            } else if answer == "KEEP" {
                println!("  Changes kept.");
                println!("  Rollback receipt saved at: {}", rollback_path.display());
                println!("  To undo later:  safesort -rollback");
            } else {
                println!(
                    "  Unrecognized input '{}'. Changes have been kept.",
                    answer
                );
                println!("  Rollback receipt saved at: {}", rollback_path.display());
                println!("  To undo later:  safesort -rollback");
            }
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
    println!("    safesort -scan                Preview organization for the current folder");
    println!("    safesort -run                 Organize AUTO-SAFE + ASSISTED files (with backup + rollback)");
    println!("    safesort -run --auto-safe-only  Organize only AUTO-SAFE files (strictest mode)");
    println!("    safesort -status              Show latest apply/rollback status");
    println!("    safesort -rollback            Roll back latest apply");
    println!();
    println!("  Advanced:");
    println!("    safesort organize ...");
    println!("    safesort preflight ...");
    println!("    safesort apply ...");
    println!("    safesort rollback ...");
    println!();
    println!("  Safety:");
    println!("    -scan never moves files");
    println!("    -run requires preflight, backup, typed confirmation, and KEEP/ROLLBACK prompt");
    println!("    LOCKED / HIGH-impact / sensitive / code files never move");
    println!("    REVIEW ONLY files are shown but never moved automatically");
    println!();
}
