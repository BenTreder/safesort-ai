use crate::cli::{Cli, Commands, OrgMode, OutputFormat};
use crate::detectors;
use crate::error::{Result, SafeSortError};
use crate::placement::engine::{OrganizationMode, SmartPlacementEngine};
use crate::profile::folder_structure;
use crate::reports;
use crate::scan::Scanner;
use std::path::{Path, PathBuf};

const SCAN_DEPTH: usize = 2;

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Doctor => doctor(),
        Commands::DemoFixture { output } => demo_fixture(&output),
        Commands::Scan {
            path,
            home,
            mode,
            format,
            output,
        } => {
            let target = resolve_target(path, home)?;
            cmd_scan(&target, mode, format, output)
        }
        Commands::Plan {
            path,
            home,
            mode,
            output,
        } => {
            let target = resolve_target(path, home)?;
            cmd_plan(&target, mode, output)
        }
        Commands::Profile { path, home } => {
            let target = resolve_target(path, home)?;
            cmd_profile(&target)
        }
        Commands::Explain { path } => cmd_explain(&path),
        Commands::Apply { .. } => cmd_apply(),
    }
}

fn org_mode(mode: OrgMode) -> OrganizationMode {
    match mode {
        OrgMode::Preview => OrganizationMode::Preview,
        OrgMode::Guided => OrganizationMode::Guided,
        OrgMode::SafeAutopilot => OrganizationMode::SafeAutopilot,
        OrgMode::LockedDown => OrganizationMode::LockedDown,
    }
}

fn resolve_target(path: Option<String>, home: bool) -> Result<PathBuf> {
    if home {
        dirs::home_dir().ok_or_else(|| {
            SafeSortError::InvalidPath("Cannot determine home directory".to_string())
        })
    } else if let Some(p) = path {
        let pb = PathBuf::from(p);
        if !pb.exists() {
            return Err(SafeSortError::InvalidPath(format!(
                "Path does not exist: {}",
                pb.display()
            )));
        }
        Ok(pb)
    } else {
        Err(SafeSortError::InvalidPath("No path specified".to_string()))
    }
}

// ─── Doctor ────────────────────────────────────────────────────────

fn doctor() -> Result<()> {
    println!();
    println!("  SafeSort AI — System Diagnostics");
    println!("  ─────────────────────────────────");
    println!();

    match dirs::home_dir() {
        Some(home) => println!("  ✅ Home: {}", home.display()),
        None => println!("  ❌ Home: not found"),
    }

    println!("  ℹ️  OS: {}", std::env::consts::OS);
    println!("  ℹ️  Arch: {}", std::env::consts::ARCH);

    for dir in crate::config::SYSTEMD_PATHS {
        let path = Path::new(dir);
        let status = if path.exists() {
            if std::fs::read_dir(path).is_ok() {
                "✅ readable"
            } else {
                "⚠️  permission denied"
            }
        } else {
            "  (not found)"
        };
        println!("  Systemd {dir}: {status}");
    }

    for dir in crate::config::CRON_PATHS {
        let path = Path::new(dir);
        let status = if path.exists() {
            if std::fs::read_dir(path).is_ok() || std::fs::read_to_string(path).is_ok() {
                "✅ readable"
            } else {
                "⚠️  permission denied"
            }
        } else {
            "  (not found)"
        };
        println!("  Cron {dir}: {status}");
    }

    println!();
    println!("  Note: Permission denied is normal and handled safely.");
    println!("  SafeSort AI will skip inaccessible areas.");
    println!();

    Ok(())
}

// ─── Demo Fixture ──────────────────────────────────────────────────

fn demo_fixture(name: &str) -> Result<()> {
    let base = PathBuf::from(name);
    if base.exists() {
        std::fs::remove_dir_all(&base)?;
    }

    println!("  Creating demo fixture at: {}", base.display());

    // Downloads folder with safe candidates
    let downloads = base.join("Downloads");
    create_file(&downloads.join("Screenshot-2026-06-04.png"));
    create_file(&downloads.join("Screenshot-2026-06-03.jpg"));
    create_file(&downloads.join("report-Q1-2026.pdf"));
    create_file(&downloads.join("notes.txt"));
    create_file(&downloads.join("export.csv"));
    create_file(&downloads.join("project-archive.zip"));
    create_file(&downloads.join("backup-2025.tar.gz"));
    create_file(&downloads.join("installers-backup.tgz"));
    create_file(&downloads.join("presentation.mp4"));
    create_file(&downloads.join("bentreder_logo.png"));
    create_file(&downloads.join("quicktapid_banner.png"));
    create_file(&downloads.join("website-fix-finder-v1.0.zip"));
    create_file(&downloads.join("content-handoff-icon.png"));
    create_file(&downloads.join("linuxpicker_article.docx"));
    create_file(&downloads.join("safesort-roadmap.pdf"));
    create_file(&downloads.join("error-checkout-page.png"));
    create_file(&downloads.join("invoice-client-2026.pdf"));

    // WordPress plugin folder
    let wp_plugin = base.join("Downloads/wp-content/plugins/my-cool-plugin");
    std::fs::create_dir_all(&wp_plugin)?;
    create_file(&wp_plugin.join("my-cool-plugin.php"));
    create_file(&wp_plugin.join("composer.json"));
    create_file(&wp_plugin.join("readme.txt"));

    // Rust project
    let rust_proj = base.join("Projects/safesort-ai");
    std::fs::create_dir_all(rust_proj.join("src"))?;
    create_file(&rust_proj.join("Cargo.toml"));
    create_file(&rust_proj.join("src/main.rs"));
    create_file(&rust_proj.join(".git/config"));

    // Node project
    let node_proj = base.join("Projects/webapp");
    std::fs::create_dir_all(node_proj.join("src"))?;
    create_file(&node_proj.join("package.json"));
    create_file(&node_proj.join("node_modules/.keep"));

    // Python project
    let py_proj = base.join("Projects/data-tool");
    std::fs::create_dir_all(&py_proj)?;
    create_file(&py_proj.join("pyproject.toml"));
    create_file(&py_proj.join("requirements.txt"));

    // Folder with .env (should be LOCKED)
    let secret_proj = base.join("ImportantApp");
    std::fs::create_dir_all(&secret_proj)?;
    create_file(&secret_proj.join(".env"));
    create_file(&secret_proj.join("config.yml"));

    // Fake systemd unit
    let systemd_dir = base.join("fake-systemd/etc/systemd/system");
    std::fs::create_dir_all(&systemd_dir)?;
    create_file_with_content(
        &systemd_dir.join("my-app.service"),
        "[Unit]\nDescription=My App\n\n[Service]\nExecStart=/usr/bin/my-app\nWorkingDirectory=/home/user/ImportantApp\nRestart=always\n\n[Install]\nWantedBy=multi-user.target\n",
    );

    // Shell script with absolute path
    let scripts_dir = base.join("scripts");
    std::fs::create_dir_all(&scripts_dir)?;
    create_file_with_content(
        &scripts_dir.join("deploy.sh"),
        "#!/bin/bash\nDEPLOY_DIR=/home/user/ImportantApp\ncd $DEPLOY_DIR\n./deploy\n",
    );

    // Fake website folder
    let website = base.join("public_html");
    std::fs::create_dir_all(&website)?;
    create_file(&website.join("index.php"));
    create_file(&website.join(".env"));

    // Sensitive directories
    let ssh_dir = base.join(".ssh");
    std::fs::create_dir_all(&ssh_dir)?;
    create_file(&ssh_dir.join("id_rsa"));
    create_file(&ssh_dir.join("id_ed25519"));
    create_file(&ssh_dir.join("config"));

    let aws_dir = base.join(".aws");
    std::fs::create_dir_all(&aws_dir)?;
    create_file_with_content(
        &aws_dir.join("credentials"),
        "[default]\naws_access_key_id = FAKE\naws_secret_access_key = FAKE\n",
    );

    // private_* folder
    let private = base.join("private_keys");
    std::fs::create_dir_all(&private)?;
    create_file(&private.join("backup.pem"));

    // Docker project
    let docker_proj = base.join("Projects/docker-app");
    std::fs::create_dir_all(&docker_proj)?;
    create_file(&docker_proj.join("Dockerfile"));
    create_file(&docker_proj.join("docker-compose.yml"));

    // Backup folder
    let backup = base.join("backups");
    std::fs::create_dir_all(&backup)?;
    create_file(&backup.join("2026-06-01.tar.gz"));

    println!("  ✅ Demo fixture created: {}", base.display());
    println!();
    println!("  Try:");
    println!("    safesort scan --path {}", base.display());
    println!("    safesort plan --path {} --mode guided", base.display());
    println!(
        "    safesort plan --path {} --mode safe-autopilot",
        base.display()
    );
    println!("    safesort explain {}/ImportantApp", base.display());
    println!();

    Ok(())
}

fn create_file(path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap_or(Path::new("/"))).unwrap();
    std::fs::write(path, "").unwrap();
}

fn create_file_with_content(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap_or(Path::new("/"))).unwrap();
    std::fs::write(path, content).unwrap();
}

// ─── Scan (mode-aware) ─────────────────────────────────────────────

fn cmd_scan(
    target: &PathBuf,
    mode: OrgMode,
    format: OutputFormat,
    output: Option<String>,
) -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let org = org_mode(mode);

    let scanner = Scanner::new();
    let report = scanner.scan(target, &home, SCAN_DEPTH)?;

    // Also run systemic detectors
    let _systemd_evidence = detectors::systemd::SystemdDetector::new().scan_all();
    let _cron_evidence = detectors::cron::CronDetector::new().scan_all();
    drop(_systemd_evidence);
    drop(_cron_evidence);

    let rendered = match format {
        OutputFormat::Terminal => reports::terminal::render(&report),
        OutputFormat::Json => {
            reports::json::render(&report).map_err(SafeSortError::Serialization)?
        }
        OutputFormat::Markdown => reports::markdown::render(&report),
    };

    if let Some(out_path) = output {
        std::fs::write(&out_path, &rendered)?;
        println!("  Report written to: {out_path}");
    } else {
        print!("{rendered}");
    }

    // If mode is not preview, show placement summary
    if !matches!(mode, OrgMode::Preview) {
        show_placement_summary(target, &home, org)?;
    }

    Ok(())
}

// ─── Plan (Smart Placement) ────────────────────────────────────────

fn cmd_plan(target: &PathBuf, mode: OrgMode, output: Option<String>) -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let org = org_mode(mode);

    println!();
    println!("  SafeSort AI — Smart Placement Plan");
    println!("  Target: {}", target.display());
    println!("  Mode: {}", org.as_str());
    println!();

    // Run scan first
    let scanner = Scanner::new();
    let report = scanner.scan(target, &home, SCAN_DEPTH)?;

    // Extract items for placement engine
    let items: Vec<(PathBuf, crate::scan::risk::SafetyLevel)> = report
        .items
        .values()
        .flatten()
        .map(|item| {
            let level = match item.safety_level.as_str() {
                "LOCKED" => crate::scan::risk::SafetyLevel::Locked,
                "REVIEW" => crate::scan::risk::SafetyLevel::Review,
                _ => crate::scan::risk::SafetyLevel::SafeCandidate,
            };
            (PathBuf::from(&item.path), level)
        })
        .collect();

    // Run placement engine
    let engine = SmartPlacementEngine::new(home.clone(), org);
    let placement = engine.run(&items);

    // Render results
    render_placement_plan(&placement, &home)?;

    // Write output if requested
    if let Some(out_path) = output {
        let plan_json = serde_json::json!({
            "target": target.to_string_lossy().to_string(),
            "mode": org.as_str(),
            "summary": {
                "total_files": placement.summary.total_files,
                "auto_plan_eligible": placement.summary.auto_plan_eligible,
                "guided_review": placement.summary.guided_review,
                "review_needed": placement.summary.review_needed,
                "leave_alone": placement.summary.leave_alone,
                "locked": placement.summary.locked,
            },
            "questions": placement.question_queue.len(),
        });
        std::fs::write(&out_path, serde_json::to_string_pretty(&plan_json)?)?;
        println!("  Plan written to: {out_path}");
    }

    Ok(())
}

fn render_placement_plan(
    placement: &crate::placement::engine::PlacementResult,
    _home: &PathBuf,
) -> Result<()> {
    let summary = &placement.summary;

    println!("  Placement Summary:");
    println!("  ─────────────────────────────────");
    println!("    Total files scanned:    {}", summary.total_files);
    println!("    🔒 Locked (Critical):   {}", summary.locked);

    if matches!(placement.mode, OrganizationMode::SafeAutopilot) {
        println!("    🟢 Auto-plan eligible:  {}", summary.auto_plan_eligible);
    } else if matches!(placement.mode, OrganizationMode::Guided) {
        println!("    🟡 Guided review:       {}", summary.guided_review);
    }

    println!("    ⚠️  Review needed:       {}", summary.review_needed);
    println!("    ⬜ Leave alone:          {}", summary.leave_alone);
    println!();

    // Show top recommendations
    let mut shown = 0;
    for rec in &placement.recommendations {
        if matches!(rec.safety_level, crate::scan::risk::SafetyLevel::Locked) {
            continue;
        }
        if rec.confidence.value() >= 80 && shown < 5 {
            render_recommendation(rec);
            shown += 1;
        }
    }

    // Show question queue for guided mode
    if matches!(placement.mode, OrganizationMode::Guided) && !placement.question_queue.is_empty() {
        let questions_rendered = placement.question_queue.render();
        print!("{questions_rendered}");
    }

    // Show auto-plan summary for safe-autopilot
    if matches!(placement.mode, OrganizationMode::SafeAutopilot) && summary.auto_plan_eligible > 0 {
        println!(
            "  🟢 Safe Autopilot — {} file(s) eligible for auto-planning:",
            summary.auto_plan_eligible
        );
        for rec in &placement.recommendations {
            if rec.confidence.is_auto_plan()
                && !matches!(rec.safety_level, crate::scan::risk::SafetyLevel::Locked)
            {
                if let Some(ref dest) = rec.destinations.first() {
                    println!("    → {} {}", rec.file_path.display(), dest.path.display());
                }
            }
        }
        println!();
    }

    println!("  Nothing was moved.");
    println!();

    Ok(())
}

fn render_recommendation(rec: &crate::placement::engine::PlacementRecommendation) {
    let impact_icon = match rec.impact_level.as_str() {
        "CRITICAL" => "🔴",
        "HIGH" => "🟠",
        "MEDIUM" => "⚠️ ",
        "LOW" => "🟢",
        _ => "  ",
    };

    println!("  ┌─────────────────────────────────────────────");
    println!("  │ File:       {}", rec.file_path.display());

    if let Some(ref owner) = rec.owner {
        println!("  │ Owner:      {} ({})", owner.display, owner.canonical);
    } else {
        println!("  │ Owner:      (unknown)");
    }

    println!("  │ Purpose:    {}", rec.purpose.as_str());
    println!("  │ Type:       {}", rec.file_type);
    println!("  │ Risk:       {}", rec.risk);
    println!("  │ Impact:     {} {}", impact_icon, rec.impact_level);
    println!("  │ Confidence: {}%", rec.confidence.value());

    if let Some(ref dest) = rec.destinations.first() {
        println!("  │ Dest:       {}", dest.description);
        println!("  │ Path:       {}", dest.path.display());
    }

    println!("  │ Why:        {}", rec.reason);
    println!("  │ Action:     {}", rec.band.as_str());
    println!("  └─────────────────────────────────────────────");
    println!();
}

fn show_placement_summary(target: &PathBuf, home: &PathBuf, org: OrganizationMode) -> Result<()> {
    let scanner = Scanner::new();
    let report = scanner.scan(target, home, SCAN_DEPTH)?;

    let items: Vec<(PathBuf, crate::scan::risk::SafetyLevel)> = report
        .items
        .values()
        .flatten()
        .map(|item| {
            let level = match item.safety_level.as_str() {
                "LOCKED" => crate::scan::risk::SafetyLevel::Locked,
                "REVIEW" => crate::scan::risk::SafetyLevel::Review,
                _ => crate::scan::risk::SafetyLevel::SafeCandidate,
            };
            (PathBuf::from(&item.path), level)
        })
        .collect();

    let engine = SmartPlacementEngine::new(home.clone(), org);
    let placement = engine.run(&items);

    let summary = &placement.summary;
    println!("  Smart Placement Summary ({} mode):", org.as_str());
    println!("    🟢 Auto-plan eligible:  {}", summary.auto_plan_eligible);
    println!("    🟡 Guided review:       {}", summary.guided_review);
    println!("    ⚠️  Review needed:       {}", summary.review_needed);
    println!("    ⬜ Leave alone:          {}", summary.leave_alone);
    println!();

    Ok(())
}

// ─── Profile ───────────────────────────────────────────────────────

fn cmd_profile(target: &PathBuf) -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    let scanner = Scanner::new();
    let report = scanner.scan(target, &home, SCAN_DEPTH)?;

    println!();
    println!("  SafeSort AI — User Profile Analysis");
    println!("  Target: {}", target.display());
    println!();

    if let Some(ref profile) = report.profile {
        let sorted = profile.sorted_scores();
        for (name, score) in sorted.iter().filter(|(_, s)| s.score > 0.0) {
            let bar = "█".repeat((score.score * 2.0).min(20.0) as usize);
            println!(
                "  {:<30} {:>6.1}  {:<10} ({})",
                name, score.score, bar, score.confidence
            );
        }
    }

    println!();
    println!("  Recommended folder structure:");
    println!();

    if let Some(ref profile) = report.profile {
        let structure = folder_structure::recommend(profile);
        for line in structure.lines() {
            println!("{line}");
        }
    }

    println!();
    println!("  Nothing was moved.");
    println!();

    Ok(())
}

// ─── Explain ───────────────────────────────────────────────────────

/// A service file that references a given path.
struct ServiceBinding {
    /// File name of the unit (e.g. "my-app.service").
    service_name: String,
    /// The systemd field that referenced the path (e.g. "WorkingDirectory").
    field: String,
    /// The verbatim path value from the unit file.
    referenced_path: String,
}

/// Walk up to 3 levels from `target.parent()` looking for a `fake-systemd` sibling dir.
fn find_fake_systemd_dir(target: &Path) -> Option<PathBuf> {
    let mut search = target.parent()?;
    for _ in 0..4 {
        let candidate = search.join("fake-systemd");
        if candidate.exists() && candidate.is_dir() {
            return Some(candidate);
        }
        search = search.parent()?;
    }
    None
}

/// Find all service files that reference a path whose basename matches `target`.
fn find_service_bindings(target: &Path) -> Vec<ServiceBinding> {
    let target_name = match target.file_name().and_then(|n| n.to_str()) {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => return vec![],
    };

    let Some(systemd_dir) = find_fake_systemd_dir(target) else {
        return vec![];
    };

    let detector = detectors::systemd::SystemdDetector::new();
    let evidence = detector.scan_dir(&systemd_dir);

    let mut bindings = Vec::new();
    for ev in evidence {
        if ev.kind != crate::scan::evidence::EvidenceKind::SystemdReference {
            continue;
        }
        // Match by basename component in the referenced path
        let ref_path = std::path::Path::new(&ev.path);
        let matched = ref_path.components().any(|c| {
            if let std::path::Component::Normal(n) = c {
                n.to_string_lossy() == target_name
            } else {
                false
            }
        });
        if !matched {
            continue;
        }

        // description: "Referenced by /path/to/my-app.service (WorkingDirectory= …)"
        let service_name = ev
            .description
            .split_whitespace()
            .nth(2) // "Referenced by <path> ..."
            .and_then(|p| std::path::Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("(unknown)")
            .to_string();

        let field = ev
            .description
            .find('(')
            .and_then(|open| ev.description.find('=').map(|eq| (open, eq)))
            .map(|(open, eq)| ev.description[open + 1..eq].trim().to_string())
            .unwrap_or_else(|| "reference".to_string());

        bindings.push(ServiceBinding {
            service_name,
            field,
            referenced_path: ev.path.clone(),
        });
    }

    bindings
}

fn cmd_explain(path: &str) -> Result<()> {
    let target = PathBuf::from(path);
    if !target.exists() {
        return Err(SafeSortError::InvalidPath(format!(
            "Path does not exist: {path}"
        )));
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    let scanner = Scanner::new();
    let parent = target
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let report = scanner.scan(&parent, &home, 2)?;

    let all_items: Vec<_> = report
        .items
        .values()
        .flatten()
        .filter(|i| i.path == path)
        .collect();

    // Check for fake-systemd (and real systemd) service bindings
    let service_bindings = find_service_bindings(&target);
    let is_service_bound = !service_bindings.is_empty();

    println!();
    println!("  SafeSort AI — Safety Explanation");
    println!("  Path: {path}");
    println!();

    if let Some(item) = all_items.first() {
        // If service-bound, upgrade classification display
        let (label, icon) = if is_service_bound {
            ("REVIEW — service-bound ⚠️  (impact: CRITICAL 🔴)", "")
        } else {
            (
                item.safety_level.as_str(),
                match item.safety_level.as_str() {
                    "LOCKED" => "🔒",
                    "REVIEW" => "⚠️ ",
                    _ => "✅",
                },
            )
        };
        println!("  Classification: {} {}", label, icon);
        println!("  Risk score: {:.2}", item.score);
        println!();
        println!("  Reasons:");
        for reason in &item.reasons {
            println!("    • {reason}");
        }
        if is_service_bound {
            println!("    • Referenced by active systemd service(s)");
        }
    } else {
        println!("  Item not found in scan results. Try scanning its parent:");
        println!("    safesort scan --path {}", parent.display());
    }

    if is_service_bound {
        println!();
        println!("  Impact: CRITICAL 🔴");
        println!("  Moving this would likely break:");
        // Deduplicate by service name
        let mut seen = std::collections::HashSet::new();
        for b in &service_bindings {
            if seen.insert(&b.service_name) {
                println!("    - systemd service: {}", b.service_name);
            }
        }
        for b in &service_bindings {
            println!("      • {}: {}", b.field, b.referenced_path);
        }
        println!();
        println!("  Recommendation:");
        println!("    Do not move. Service-bound path.");
        println!("    Use Workspace Overlay instead.");
    }

    println!();
    println!("  Nothing was moved.");
    println!();

    Ok(())
}

// ─── Apply ─────────────────────────────────────────────────────────

fn cmd_apply() -> Result<()> {
    println!();
    println!("  ╔═══════════════════════════════════════════════════╗");
    println!("  ║  Apply is disabled in this safety-first build.   ║");
    println!("  ║  Nothing was moved.                              ║");
    println!("  ╚═══════════════════════════════════════════════════╝");
    println!();
    Ok(())
}
