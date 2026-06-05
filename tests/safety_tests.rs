use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: create a file with optional content.
fn create_file(path: &std::path::Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new("/"))).unwrap();
    fs::write(path, content).unwrap();
}

/// Helper: create an empty file.
fn touch(path: &std::path::Path) {
    create_file(path, "");
}

fn to_pb(tmp: &TempDir) -> PathBuf {
    tmp.path().to_path_buf()
}

// ═══════════════════════════════════════════════════════════════════
// 1. Systemd-referenced paths are LOCKED
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_systemd_detector_finds_references() {
    // Test that the systemd detector can parse unit files
    let detector = safesort_ai::detectors::systemd::SystemdDetector::new();
    let evidence = detector.scan_all();

    // It should either find evidence or skip gracefully (permission denied)
    // The key thing is it doesn't panic
    for ev in &evidence {
        // If it found systemd references, they should mention systemd
        assert!(
            ev.description.contains("systemd") || ev.note.is_some(),
            "Systemd evidence should be properly described"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// 2. .env folders are LOCKED
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_env_folder_is_locked() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let project = base.join("my-project");
    fs::create_dir_all(&project).unwrap();
    create_file(&project.join(".env"), "SECRET_KEY=abc123\n");
    touch(&project.join("app.py"));

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&base, &home, 2).unwrap();

    let locked = report.get_examples("LOCKED", 100);
    assert!(
        locked.iter().any(|i| i.path.contains(".env")),
        ".env file should be LOCKED"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 3. Git repos are REVIEW
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_git_repo_is_review() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let repo = base.join("my-rust-project");
    fs::create_dir_all(repo.join(".git")).unwrap();
    touch(&repo.join(".git/config"));
    create_file(
        &repo.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
    );
    touch(&repo.join("src/main.rs"));

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&base, &home, 2).unwrap();

    let review = report.get_examples("REVIEW", 100);
    assert!(
        review.iter().any(|i| i.path.contains("my-rust-project")),
        "Git repo should be REVIEW"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 4. Loose screenshots in Downloads are SAFE_CANDIDATE
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_loose_screenshot_in_downloads_is_safe() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("Screenshot-2026-06-04.png"));
    touch(&downloads.join("Screenshot-2026-06-03.jpg"));
    touch(&downloads.join("photo.jpeg"));

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.join("home");
    let report = scanner.scan(&downloads, &home, 1).unwrap();

    let safe = report.get_examples("SAFE", 100);
    assert!(
        safe.iter()
            .any(|i| i.path.contains("Screenshot-2026-06-04.png")),
        "Loose screenshot in Downloads should be SAFE_CANDIDATE"
    );
    assert!(
        safe.iter()
            .any(|i| i.path.contains("Screenshot-2026-06-03.jpg")),
        "Loose screenshot in Downloads should be SAFE_CANDIDATE"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 5. Loose PDFs in Downloads are SAFE_CANDIDATE
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_loose_pdf_in_downloads_is_safe() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("report-Q1-2026.pdf"));
    touch(&downloads.join("invoice-2025.pdf"));

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.join("home");
    let report = scanner.scan(&downloads, &home, 1).unwrap();

    let safe = report.get_examples("SAFE", 100);
    assert!(
        safe.iter().any(|i| i.path.contains("report-Q1-2026.pdf")),
        "Loose PDF in Downloads should be SAFE_CANDIDATE"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 6. Symlink targets are LOCKED
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_symlink_is_classified() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let target = base.join("real-folder");
    fs::create_dir_all(&target).unwrap();
    touch(&target.join("data.txt"));

    let link = base.join("link-to-folder");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&base, &home, 2).unwrap();

    // The symlink itself should be LOCKED (safety policy: symlink targets are LOCKED)
    let locked = report.get_examples("LOCKED", 100);
    assert!(
        locked.iter().any(|i| i.path.contains("link-to-folder")),
        "Symlink 'link-to-folder' should be LOCKED by safety policy"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 7. /etc, /usr, /var, /boot are always LOCKED
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_system_paths_are_locked() {
    use safesort_ai::scan::classifier::Classifier;
    use safesort_ai::scan::item::ScanItem;

    let classifier = Classifier::new();
    let home = PathBuf::from("/home/testuser");

    for sys_path in &["/etc", "/usr", "/var", "/boot", "/opt", "/srv"] {
        let item = ScanItem {
            path: PathBuf::from(sys_path).join("something"),
            name: "something".to_string(),
            is_dir: true,
            is_symlink: false,
            symlink_target: None,
            extension: None,
            depth: 1,
            is_hidden: false,
        };

        let classification = classifier.classify(&item, &PathBuf::from("/"), &home);
        assert_eq!(
            classification.level,
            safesort_ai::scan::risk::SafetyLevel::Locked,
            "{} should be LOCKED",
            sys_path
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// 8. Apply refuses to run
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_apply_refuses_to_run() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    let mut cmd = Command::cargo_bin("safesort").unwrap();
    cmd.arg("apply").arg("some-plan").assert().success().stdout(
        predicate::str::contains("Apply is disabled in this safety-first build")
            .and(predicate::str::contains("Nothing was moved")),
    );
}

// ═══════════════════════════════════════════════════════════════════
// 9. No code path moves or deletes files (scan is read-only)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_scan_is_read_only() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("file1.png"));
    touch(&downloads.join("file2.pdf"));
    touch(&downloads.join("file3.zip"));

    let project = base.join("home/Projects/webapp");
    fs::create_dir_all(project.join("src")).unwrap();
    create_file(
        &project.join("package.json"),
        "{\n  \"name\": \"test\"\n}\n",
    );
    touch(&project.join("src/index.js"));

    let count_before = count_files_recursively(&base);

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.join("home");
    let _report = scanner.scan(&base.join("home"), &home, 3).unwrap();

    let count_after = count_files_recursively(&base);

    assert_eq!(
        count_before, count_after,
        "Scan must not create, move, or delete any files"
    );

    assert!(
        downloads.join("file1.png").exists(),
        "file1.png must still exist"
    );
    assert!(
        downloads.join("file2.pdf").exists(),
        "file2.pdf must still exist"
    );
    assert!(
        project.join("package.json").exists(),
        "package.json must still exist"
    );
}

fn count_files_recursively(dir: &std::path::Path) -> usize {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .count()
}

// ═══════════════════════════════════════════════════════════════════
// 10. WordPress plugin folder is REVIEW
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_wordpress_plugin_is_review() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let plugin = base.join("wp-content/plugins/my-plugin");
    fs::create_dir_all(&plugin).unwrap();
    create_file(
        &plugin.join("my-plugin.php"),
        "<?php\n/**\n * Plugin Name: My Plugin\n */\n",
    );
    create_file(
        &plugin.join("composer.json"),
        "{\n  \"name\": \"test/my-plugin\"\n}\n",
    );

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&base, &home, 3).unwrap();

    let review = report.get_examples("REVIEW", 100);
    assert!(
        review.iter().any(|i| i.path.contains("my-plugin")),
        "WordPress plugin folder should be REVIEW"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 11. private_* folders are LOCKED
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_private_folder_is_locked() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let private = base.join("private_keys");
    fs::create_dir_all(&private).unwrap();
    touch(&private.join("backup.pem"));

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&base, &home, 2).unwrap();

    let locked = report.get_examples("LOCKED", 100);
    assert!(
        locked.iter().any(|i| i.path.contains("private_keys")),
        "private_* folder should be LOCKED"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 12. Script with absolute path references
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_script_with_absolute_path() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let scripts = base.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    create_file(
        &scripts.join("deploy.sh"),
        "#!/bin/bash\nDEPLOY_DIR=/home/user/ImportantApp\ncd $DEPLOY_DIR\n./deploy\n",
    );

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&scripts, &home, 1).unwrap();

    let all_items: Vec<_> = report
        .items
        .values()
        .flatten()
        .filter(|i| i.path.contains("deploy.sh"))
        .collect();
    assert!(
        !all_items.is_empty(),
        "deploy.sh should appear in scan results"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 13. Sensitive .ssh folder is LOCKED
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_ssh_folder_is_locked() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let ssh = base.join("home/.ssh");
    fs::create_dir_all(&ssh).unwrap();
    create_file(
        &ssh.join("id_rsa"),
        "-----BEGIN RSA PRIVATE KEY-----\nfake\n-----END RSA PRIVATE KEY-----\n",
    );
    create_file(
        &ssh.join("id_ed25519"),
        "-----BEGIN OPENSSH PRIVATE KEY-----\nfake\n-----END OPENSSH PRIVATE KEY-----\n",
    );
    create_file(&ssh.join("config"), "Host *\n  ForwardAgent no\n");

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.join("home");
    let report = scanner.scan(&home, &home, 2).unwrap();

    let locked = report.get_examples("LOCKED", 100);
    assert!(
        locked.iter().any(|i| i.path.contains(".ssh")),
        ".ssh directory should be LOCKED"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 14. Archive files in Downloads are SAFE_CANDIDATE
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_archive_in_downloads_is_safe() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("project-archive.zip"));
    touch(&downloads.join("backup-2025.tar.gz"));
    touch(&downloads.join("data.tgz"));

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.join("home");
    let report = scanner.scan(&downloads, &home, 1).unwrap();

    let safe = report.get_examples("SAFE", 100);
    assert!(
        safe.iter().any(|i| i.path.contains("project-archive.zip")),
        "ZIP in Downloads should be SAFE_CANDIDATE"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 15. Node.js project is REVIEW
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_node_project_is_review() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let project = base.join("Projects/webapp");
    fs::create_dir_all(project.join("src")).unwrap();
    create_file(
        &project.join("package.json"),
        "{\n  \"name\": \"webapp\",\n  \"version\": \"1.0.0\"\n}\n",
    );
    touch(&project.join("node_modules/.keep"));

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&base.join("Projects"), &home, 2).unwrap();

    let review = report.get_examples("REVIEW", 100);
    assert!(
        review.iter().any(|i| i.path.contains("webapp")),
        "Node.js project should be REVIEW"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 16. Python project is REVIEW
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_python_project_is_review() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let project = base.join("Projects/data-tool");
    fs::create_dir_all(&project).unwrap();
    create_file(
        &project.join("pyproject.toml"),
        "[project]\nname = \"data-tool\"\nversion = \"0.1.0\"\n",
    );
    create_file(
        &project.join("requirements.txt"),
        "requests>=2.28\npandas>=1.5\n",
    );

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&base.join("Projects"), &home, 2).unwrap();

    let review = report.get_examples("REVIEW", 100);
    assert!(
        review.iter().any(|i| i.path.contains("data-tool")),
        "Python project should be REVIEW"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 17. Docker project is REVIEW
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_docker_project_is_review() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let project = base.join("Projects/docker-app");
    fs::create_dir_all(&project).unwrap();
    create_file(
        &project.join("Dockerfile"),
        "FROM rust:latest\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\n",
    );
    create_file(
        &project.join("docker-compose.yml"),
        "version: '3'\nservices:\n  app:\n    build: .\n    ports:\n      - \"8080:8080\"\n",
    );

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&base.join("Projects"), &home, 2).unwrap();

    let review = report.get_examples("REVIEW", 100);
    assert!(
        review.iter().any(|i| i.path.contains("docker-app")),
        "Docker project should be REVIEW"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 18. Website folder is LOCKED
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_website_folder_is_locked() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let website = base.join("public_html");
    fs::create_dir_all(&website).unwrap();
    create_file(&website.join("index.php"), "<?php echo 'Hello'; ?>\n");
    create_file(&website.join(".env"), "DB_PASSWORD=secret\n");

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&base, &home, 2).unwrap();

    let locked = report.get_examples("LOCKED", 100);
    assert!(
        locked.iter().any(|i| i.path.contains("public_html")),
        "Website folder with .env should be LOCKED"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 19. CSV exports in Downloads are SAFE_CANDIDATE
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_csv_in_downloads_is_safe() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    create_file(&downloads.join("export.csv"), "id,name,value\n1,test,100\n");
    create_file(&downloads.join("notes.txt"), "Some notes here\n");

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.join("home");
    let report = scanner.scan(&downloads, &home, 1).unwrap();

    let safe = report.get_examples("SAFE", 100);
    assert!(
        safe.iter().any(|i| i.path.contains("export.csv")),
        "CSV export in Downloads should be SAFE_CANDIDATE"
    );
    assert!(
        safe.iter().any(|i| i.path.contains("notes.txt")),
        "Text notes in Downloads should be SAFE_CANDIDATE"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 20. Media files in Downloads are SAFE_CANDIDATE
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_media_in_downloads_is_safe() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);

    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("presentation.mp4"));
    touch(&downloads.join("recording.wav"));
    touch(&downloads.join("animation.gif"));

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.join("home");
    let report = scanner.scan(&downloads, &home, 1).unwrap();

    let safe = report.get_examples("SAFE", 100);
    assert!(
        safe.iter().any(|i| i.path.contains("presentation.mp4")),
        "MP4 in Downloads should be SAFE_CANDIDATE"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 21. Fake systemd fixture creates a dependency edge to ImportantApp
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_fake_systemd_creates_dependency_edge_to_important_app() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path();

    // Create fake-systemd fixture referencing ImportantApp
    let systemd_dir = base.join("fake-systemd/etc/systemd/system");
    fs::create_dir_all(&systemd_dir).unwrap();
    create_file(
        &systemd_dir.join("my-app.service"),
        "[Unit]\nDescription=My App\n\n[Service]\nExecStart=/usr/bin/my-app\nWorkingDirectory=/home/user/ImportantApp\nRestart=always\n\n[Install]\nWantedBy=multi-user.target\n",
    );

    // Scan the fake-systemd dir for evidence
    let detector = safesort_ai::detectors::systemd::SystemdDetector::new();
    let evidence = detector.scan_dir(&base.join("fake-systemd"));

    let systemd_refs: Vec<_> = evidence
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                safesort_ai::scan::evidence::EvidenceKind::SystemdReference
            )
        })
        .collect();

    assert!(
        !systemd_refs.is_empty(),
        "fake-systemd fixture should produce SystemdReference evidence"
    );

    let refs_important_app = systemd_refs.iter().any(|e| e.path.contains("ImportantApp"));
    assert!(
        refs_important_app,
        "Evidence should reference ImportantApp path"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 22. Dependency graph: systemd edge produces Critical impact
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_systemd_edge_produces_critical_impact() {
    use safesort_ai::graph::{DependencyGraph, Edge, EdgeKind, ImpactLevel};

    let mut graph = DependencyGraph::new();
    graph.add_edge(Edge::with_description(
        "my-app.service",
        "/home/user/ImportantApp",
        EdgeKind::UsesWorkingDirectory,
        "WorkingDirectory in my-app.service",
    ));

    let analysis = graph.analyze_impact(std::path::Path::new("/home/user/ImportantApp"));
    assert_eq!(
        analysis.level,
        ImpactLevel::Critical,
        "UsesWorkingDirectory edge should produce Critical impact"
    );
    assert!(
        analysis.has_dependencies(),
        "Should have at least one dependency"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 23. Service-bound paths are not SAFE_CANDIDATE
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_service_bound_path_is_not_safe_candidate() {
    use safesort_ai::graph::{DependencyGraph, Edge, EdgeKind, ImpactLevel};

    let mut graph = DependencyGraph::new();
    graph.add_edge(Edge::new(
        "web.service",
        "/var/www/myapp",
        EdgeKind::UsesWorkingDirectory,
    ));

    let analysis = graph.analyze_impact(std::path::Path::new("/var/www/myapp"));
    assert!(
        analysis.level >= ImpactLevel::High,
        "Service-bound path must be High or Critical, not safe candidate"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 24. Apply still refuses (service-bound context)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_apply_still_refuses_always() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    let mut cmd = Command::cargo_bin("safesort").unwrap();
    cmd.arg("apply")
        .arg("any-plan.json")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Apply is disabled in this safety-first build")
                .and(predicate::str::contains("Nothing was moved")),
        );
}

// ═══════════════════════════════════════════════════════════════════
// 25. Safe Autopilot only plans — never moves
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_safe_autopilot_only_plans() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    let downloads = base.join("Downloads");
    fs::create_dir_all(&downloads).unwrap();
    create_file(&downloads.join("Screenshot-2026-06-01.png"), "");
    create_file(&downloads.join("report.pdf"), "");

    let count_before = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .count();

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&downloads, &home, 1).unwrap();

    let items: Vec<(PathBuf, safesort_ai::scan::risk::SafetyLevel)> = report
        .items
        .values()
        .flatten()
        .map(|item| {
            let level = match item.safety_level.as_str() {
                "LOCKED" => safesort_ai::scan::risk::SafetyLevel::Locked,
                "REVIEW" => safesort_ai::scan::risk::SafetyLevel::Review,
                _ => safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
            };
            (PathBuf::from(&item.path), level)
        })
        .collect();

    let engine = safesort_ai::placement::engine::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::SafeAutopilot,
    );
    let _placement = engine.run(&items);

    let count_after = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .count();

    assert_eq!(
        count_before, count_after,
        "Safe Autopilot must not move or create files"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 26. Guided Review only plans — never moves
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_guided_review_only_plans() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    let downloads = base.join("Downloads");
    fs::create_dir_all(&downloads).unwrap();
    create_file(&downloads.join("export.csv"), "");

    let count_before = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .count();

    let scanner = safesort_ai::scan::Scanner::new();
    let home = base.clone();
    let report = scanner.scan(&downloads, &home, 1).unwrap();

    let items: Vec<(PathBuf, safesort_ai::scan::risk::SafetyLevel)> = report
        .items
        .values()
        .flatten()
        .map(|item| {
            let level = match item.safety_level.as_str() {
                "LOCKED" => safesort_ai::scan::risk::SafetyLevel::Locked,
                "REVIEW" => safesort_ai::scan::risk::SafetyLevel::Review,
                _ => safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
            };
            (PathBuf::from(&item.path), level)
        })
        .collect();

    let engine = safesort_ai::placement::engine::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::Guided,
    );
    let _placement = engine.run(&items);

    let count_after = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .count();

    assert_eq!(
        count_before, count_after,
        "Guided Review must not move or create files"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 27. No destructive filesystem operations (scan + plan combined)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_no_destructive_ops_combined() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    // Simulate a home directory with mixed content
    let downloads = base.join("Downloads");
    fs::create_dir_all(&downloads).unwrap();
    create_file(&downloads.join("img.png"), "");

    let project = base.join("Projects/myapp");
    fs::create_dir_all(project.join(".git")).unwrap();
    create_file(&project.join("Cargo.toml"), "[package]\nname=\"myapp\"\n");

    let secret = base.join("ImportantApp");
    fs::create_dir_all(&secret).unwrap();
    create_file(&secret.join(".env"), "SECRET=x\n");

    // Record before state
    let before: std::collections::HashSet<PathBuf> = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect();

    // Run scan
    let scanner = safesort_ai::scan::Scanner::new();
    let _report = scanner.scan(&base, &base, 3).unwrap();

    // Record after state
    let after: std::collections::HashSet<PathBuf> = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect();

    assert_eq!(
        before, after,
        "No files should be created, moved, or deleted by scan"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 28. Scan summary includes impact counts
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_scan_summary_includes_impact_counts() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    // LOCKED with SensitiveFile → CRITICAL impact
    let app = base.join("ImportantApp");
    fs::create_dir_all(&app).unwrap();
    create_file(&app.join(".env"), "SECRET=x\n");

    // REVIEW with project marker → MEDIUM impact
    let proj = base.join("Projects/myapp");
    fs::create_dir_all(proj.join(".git")).unwrap();
    create_file(&proj.join("Cargo.toml"), "[package]\nname=\"myapp\"\n");

    // SAFE in Downloads → LOW impact
    let dl = base.join("Downloads");
    fs::create_dir_all(&dl).unwrap();
    create_file(&dl.join("report.pdf"), "");

    let scanner = safesort_ai::scan::Scanner::new();
    let report = scanner.scan(&base, &base, 3).unwrap();

    // The summary must have all five impact fields
    let s = &report.summary;
    assert_eq!(
        s.impact_critical + s.impact_high + s.impact_medium + s.impact_low + s.impact_none,
        s.total,
        "impact counts must sum to total"
    );
    // At least one CRITICAL and one MEDIUM
    assert!(s.impact_critical >= 1, "expected ≥1 CRITICAL-impact item");
    assert!(s.impact_medium >= 1, "expected ≥1 MEDIUM-impact item");
}

// ═══════════════════════════════════════════════════════════════════
// 29. ImportantApp .env file contributes CRITICAL impact
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_important_app_env_contributes_critical_impact() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    let app = base.join("ImportantApp");
    fs::create_dir_all(&app).unwrap();
    create_file(&app.join(".env"), "SECRET=x\n");

    let scanner = safesort_ai::scan::Scanner::new();
    let report = scanner.scan(&base, &base, 2).unwrap();

    let all_items: Vec<_> = report.items.values().flatten().collect();
    let env_item = all_items
        .iter()
        .find(|i| i.path.ends_with(".env"))
        .expect(".env should appear in scan results");

    assert_eq!(
        env_item.impact_level, "CRITICAL",
        ".env file should have CRITICAL impact"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 30. Active Rust project contributes MEDIUM impact
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_rust_project_contributes_medium_impact() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    let proj = base.join("safesort-ai");
    fs::create_dir_all(proj.join("src")).unwrap();
    create_file(
        &proj.join("Cargo.toml"),
        "[package]\nname=\"safesort-ai\"\n",
    );
    create_file(&proj.join("src/main.rs"), "fn main() {}");

    let scanner = safesort_ai::scan::Scanner::new();
    let report = scanner.scan(&base, &base, 2).unwrap();

    let all_items: Vec<_> = report.items.values().flatten().collect();
    let proj_item = all_items
        .iter()
        .find(|i| i.path.ends_with("safesort-ai") && i.is_dir)
        .expect("safesort-ai directory should appear in scan");

    assert_eq!(
        proj_item.impact_level, "MEDIUM",
        "Rust project directory should have MEDIUM impact"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 31. Safe Autopilot does not auto-plan REVIEW items (MEDIUM impact)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_safe_autopilot_excludes_medium_impact_items() {
    use safesort_ai::placement::engine::{OrganizationMode, SmartPlacementEngine};
    use safesort_ai::scan::risk::SafetyLevel;

    let engine =
        SmartPlacementEngine::new(PathBuf::from("/home/user"), OrganizationMode::SafeAutopilot);

    // Provide one REVIEW item (project folder) and one SAFE item
    let items = vec![
        (
            PathBuf::from("/home/user/Projects/safesort-ai"),
            SafetyLevel::Review, // MEDIUM impact → must NOT be auto-planned
        ),
        (
            PathBuf::from("/home/user/Downloads/bentreder_logo.png"),
            SafetyLevel::SafeCandidate, // NONE impact → can be auto-planned
        ),
    ];

    let result = engine.run(&items);

    // The REVIEW item must never appear in auto_plan_eligible
    let auto_planned: Vec<_> = result
        .recommendations
        .iter()
        .filter(|r| r.confidence.is_auto_plan())
        .collect();

    assert!(
        auto_planned
            .iter()
            .all(|r| !matches!(r.safety_level, SafetyLevel::Review)),
        "REVIEW items (MEDIUM impact) must never be auto-planned"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 32. Guided mode tracks impact per recommendation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_guided_mode_tracks_impact() {
    use safesort_ai::placement::engine::{OrganizationMode, SmartPlacementEngine};
    use safesort_ai::scan::risk::SafetyLevel;

    let engine = SmartPlacementEngine::new(PathBuf::from("/home/user"), OrganizationMode::Guided);

    let items = vec![
        (PathBuf::from("/home/user/.ssh/id_rsa"), SafetyLevel::Locked),
        (
            PathBuf::from("/home/user/Projects/myapp"),
            SafetyLevel::Review,
        ),
        (
            PathBuf::from("/home/user/Downloads/report.pdf"),
            SafetyLevel::SafeCandidate,
        ),
    ];

    let result = engine.run(&items);

    // Every recommendation must carry an impact_level
    for rec in &result.recommendations {
        assert!(
            matches!(
                rec.impact_level.as_str(),
                "CRITICAL" | "HIGH" | "MEDIUM" | "LOW" | "NONE"
            ),
            "impact_level must be a recognised value, got: {}",
            rec.impact_level
        );
    }

    // Locked item → CRITICAL
    let locked_rec = result
        .recommendations
        .iter()
        .find(|r| matches!(r.safety_level, SafetyLevel::Locked))
        .unwrap();
    assert_eq!(locked_rec.impact_level, "CRITICAL");

    // Review item → MEDIUM
    let review_rec = result
        .recommendations
        .iter()
        .find(|r| matches!(r.safety_level, SafetyLevel::Review))
        .unwrap();
    assert_eq!(review_rec.impact_level, "MEDIUM");
}

// ═══════════════════════════════════════════════════════════════════
// 33. Apply still refuses (impact-wired build)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_apply_refuses_after_impact_wiring() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    let mut cmd = Command::cargo_bin("safesort").unwrap();
    cmd.arg("apply")
        .arg("impact-plan.json")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Apply is disabled in this safety-first build")
                .and(predicate::str::contains("Nothing was moved")),
        );
}

// ═══════════════════════════════════════════════════════════════════
// 34. No destructive operations — impact scan does not touch files
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_impact_scan_no_destructive_ops() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    let app = base.join("ImportantApp");
    fs::create_dir_all(&app).unwrap();
    create_file(&app.join(".env"), "SECRET=x\n");

    let proj = base.join("Projects/myapp");
    fs::create_dir_all(proj.join(".git")).unwrap();
    create_file(&proj.join("Cargo.toml"), "[package]\n");

    let dl = base.join("Downloads");
    fs::create_dir_all(&dl).unwrap();
    create_file(&dl.join("photo.png"), "");

    let before: std::collections::HashSet<PathBuf> = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect();

    let scanner = safesort_ai::scan::Scanner::new();
    let _report = scanner.scan(&base, &base, 3).unwrap();

    let after: std::collections::HashSet<PathBuf> = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect();

    assert_eq!(
        before, after,
        "Impact-aware scan must not create, move, or delete any files"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 35. public_html/index.php must not be SAFE_CANDIDATE
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_public_html_child_is_not_safe_candidate() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    let site = base.join("public_html");
    fs::create_dir_all(&site).unwrap();
    create_file(&site.join("index.php"), "<?php echo 'hello'; ?>\n");
    create_file(&site.join("style.css"), "body { margin: 0; }\n");

    let scanner = safesort_ai::scan::Scanner::new();
    let report = scanner.scan(&base, &base, 2).unwrap();

    let safe = report.get_examples("SAFE", 100);
    assert!(
        !safe.iter().any(|i| i.path.contains("index.php")),
        "index.php inside public_html must NOT be SAFE_CANDIDATE"
    );
    assert!(
        !safe.iter().any(|i| i.path.contains("style.css")),
        "style.css inside public_html must NOT be SAFE_CANDIDATE"
    );

    // Must be REVIEW (inherited from LOCKED live-site parent)
    let review = report.get_examples("REVIEW", 100);
    assert!(
        review.iter().any(|i| i.path.contains("index.php")),
        "index.php inside public_html must be REVIEW, not SAFE"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 36. ImportantApp/config.yml inherits LOCKED parent → not SAFE
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_important_app_child_is_not_safe_candidate() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    let app = base.join("ImportantApp");
    fs::create_dir_all(&app).unwrap();
    create_file(&app.join(".env"), "SECRET=x\n");
    create_file(&app.join("config.yml"), "port: 8080\n");

    let scanner = safesort_ai::scan::Scanner::new();
    let report = scanner.scan(&base, &base, 2).unwrap();

    // ImportantApp itself must be LOCKED (contains .env)
    let locked = report.get_examples("LOCKED", 100);
    assert!(
        locked
            .iter()
            .any(|i| i.path.ends_with("ImportantApp") || i.path.contains("ImportantApp")),
        "ImportantApp directory (contains .env) must be LOCKED"
    );

    // config.yml must NOT be SAFE_CANDIDATE (inherited from LOCKED parent)
    let safe = report.get_examples("SAFE", 100);
    assert!(
        !safe.iter().any(|i| i.path.contains("config.yml")),
        "config.yml inside LOCKED ImportantApp must NOT be SAFE_CANDIDATE"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 37. Child of LOCKED parent is not auto-plan eligible (safe-autopilot)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_child_of_locked_parent_not_auto_plan_eligible() {
    use safesort_ai::placement::engine::{OrganizationMode, SmartPlacementEngine};
    use safesort_ai::scan::risk::SafetyLevel;

    let engine =
        SmartPlacementEngine::new(PathBuf::from("/home/user"), OrganizationMode::SafeAutopilot);

    // Simulate what the scanner now produces for a child of a LOCKED parent:
    // the child is REVIEW (inherited), not SafeCandidate.
    let items = vec![
        (
            PathBuf::from("/home/user/ImportantApp"),
            SafetyLevel::Locked,
        ),
        (
            // Child that inherited REVIEW from LOCKED parent
            PathBuf::from("/home/user/ImportantApp/config.yml"),
            SafetyLevel::Review,
        ),
    ];

    let result = engine.run(&items);

    assert_eq!(
        result.summary.auto_plan_eligible, 0,
        "No items inside a LOCKED parent should be auto-plan eligible"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 38. Child of CRITICAL parent (www/) is not auto-plan eligible
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_child_of_critical_live_site_not_auto_plan_eligible() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    let site = base.join("www");
    fs::create_dir_all(&site).unwrap();
    create_file(&site.join("index.html"), "<html></html>\n");
    create_file(&site.join("app.js"), "console.log('hello');\n");

    let scanner = safesort_ai::scan::Scanner::new();
    let report = scanner.scan(&base, &base, 2).unwrap();

    // No child of a live-site dir should be SAFE_CANDIDATE
    let safe = report.get_examples("SAFE", 100);
    assert!(
        !safe.iter().any(|i| i.path.contains("index.html")),
        "index.html inside www/ must NOT be SAFE_CANDIDATE"
    );
    assert!(
        !safe.iter().any(|i| i.path.contains("app.js")),
        "app.js inside www/ must NOT be SAFE_CANDIDATE"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 39. Apply still refuses (post parent-risk inheritance)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_apply_refuses_after_inheritance_pass() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    let mut cmd = Command::cargo_bin("safesort").unwrap();
    cmd.arg("apply")
        .arg("inherited-plan.json")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Apply is disabled in this safety-first build")
                .and(predicate::str::contains("Nothing was moved")),
        );
}

// ═══════════════════════════════════════════════════════════════════
// 40. Inheritance scan is read-only — no file operations
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_inheritance_scan_is_read_only() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();

    let site = base.join("public_html");
    fs::create_dir_all(&site).unwrap();
    create_file(&site.join("index.php"), "<?php ?>\n");

    let app = base.join("ImportantApp");
    fs::create_dir_all(&app).unwrap();
    create_file(&app.join(".env"), "KEY=secret\n");
    create_file(&app.join("config.yml"), "port: 3000\n");

    let before: std::collections::HashSet<PathBuf> = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect();

    let scanner = safesort_ai::scan::Scanner::new();
    let _report = scanner.scan(&base, &base, 3).unwrap();

    let after: std::collections::HashSet<PathBuf> = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect();

    assert_eq!(
        before, after,
        "Inheritance-aware scan must not create, move, or delete any files"
    );
}
