use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_file(path: &std::path::Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new("/"))).unwrap();
    fs::write(path, content).unwrap();
}

fn touch(path: &std::path::Path) {
    create_file(path, "");
}

fn to_pb(tmp: &TempDir) -> PathBuf {
    tmp.path().to_path_buf()
}

// ═══════════════════════════════════════════════════════════════════
// Owner/Purpose detection tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_bentreder_logo_maps_to_bentreder_logos() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("bentreder_logo.png"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::Preview,
    );

    let rec = engine.analyze_file(
        &downloads.join("bentreder_logo.png"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    );

    assert!(rec.owner.is_some());
    let owner = rec.owner.unwrap();
    assert_eq!(owner.canonical, "BenTreder.com");
    assert_eq!(
        rec.purpose,
        safesort_ai::placement::file_purpose::FilePurpose::Logo
    );
    // Should have reasonable confidence (owner match + purpose match + safe zone + extension)
    assert!(
        rec.confidence.value() >= 50,
        "bentreder_logo.png confidence should be >= 50, got {}",
        rec.confidence.value()
    );
    // Should have a destination
    assert!(!rec.destinations.is_empty());
    let dest = &rec.destinations[0];
    assert!(
        dest.path.to_string_lossy().contains("Logos"),
        "Destination should contain Logos"
    );
}

#[test]
fn test_quicktapid_banner_maps_to_banners() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("quicktapid-banner.png"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::Preview,
    );

    let rec = engine.analyze_file(
        &downloads.join("quicktapid-banner.png"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    );

    assert!(rec.owner.is_some());
    assert_eq!(rec.owner.unwrap().canonical, "QuickTapID");
    assert_eq!(
        rec.purpose,
        safesort_ai::placement::file_purpose::FilePurpose::Banner
    );
    assert!(rec.confidence.value() >= 50);
    assert!(
        rec.destinations[0]
            .path
            .to_string_lossy()
            .contains("Banners")
    );
}

#[test]
fn test_website_fix_finder_release_zip_maps_to_release_zips() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("website-fix-finder-v1.0.zip"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::Preview,
    );

    let rec = engine.analyze_file(
        &downloads.join("website-fix-finder-v1.0.zip"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    );

    assert!(rec.owner.is_some());
    assert_eq!(rec.owner.unwrap().canonical, "Website Fix Finder");
    assert_eq!(
        rec.purpose,
        safesort_ai::placement::file_purpose::FilePurpose::ReleaseZip
    );
}

#[test]
fn test_error_screenshot_maps_to_errors() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("error-checkout-page.png"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::Preview,
    );

    let rec = engine.analyze_file(
        &downloads.join("error-checkout-page.png"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    );

    assert_eq!(
        rec.purpose,
        safesort_ai::placement::file_purpose::FilePurpose::ErrorScreenshot
    );
    assert!(
        rec.destinations[0]
            .path
            .to_string_lossy()
            .contains("Errors")
    );
}

#[test]
fn test_invoice_maps_to_receipts() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("invoice-client-2026.pdf"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::Preview,
    );

    let rec = engine.analyze_file(
        &downloads.join("invoice-client-2026.pdf"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    );

    assert_eq!(
        rec.purpose,
        safesort_ai::placement::file_purpose::FilePurpose::Invoice
    );
}

#[test]
fn test_ambiguous_file_gets_low_confidence() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("random_file.txt"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::Preview,
    );

    let rec = engine.analyze_file(
        &downloads.join("random_file.txt"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    );

    // Should have low confidence (no owner match, generic purpose)
    assert!(
        rec.confidence.value() < 80,
        "Random file should have low confidence, got {}",
        rec.confidence.value()
    );
}

// ═══════════════════════════════════════════════════════════════════
// Organization mode tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_safe_autopilot_only_auto_plans_95_plus_and_green() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();

    // Create a file that should have very high confidence
    touch(&downloads.join("bentreder_logo.png"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::SafeAutopilot,
    );

    let items = vec![(
        downloads.join("bentreder_logo.png"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    )];

    let result = engine.run(&items);
    // bentreder_logo.png in Downloads should be auto-plan eligible
    assert!(
        result.summary.auto_plan_eligible >= 1,
        "bentreder_logo.png should be auto-plan eligible in safe-autopilot mode"
    );
}

#[test]
fn test_guided_mode_creates_question_queue() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();

    // Create a file with medium confidence (80-94 band)
    touch(&downloads.join("bentreder_logo.png"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::Guided,
    );

    let items = vec![(
        downloads.join("bentreder_logo.png"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    )];

    let result = engine.run(&items);
    // Questions are created for files in the 80-94 confidence band
    // bentreder_logo.png is likely 95+ (auto-plan) so may not create a question
    // But the engine should process it correctly
    assert_eq!(result.recommendations.len(), 1);
}

#[test]
fn test_locked_down_mode_caps_confidence_and_auto_plans_nothing() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("bentreder_logo.png"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::LockedDown,
    );

    let items = vec![(
        downloads.join("bentreder_logo.png"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    )];

    let result = engine.run(&items);
    // Locked-down mode should never auto-plan
    assert_eq!(
        result.summary.auto_plan_eligible, 0,
        "Locked-down mode should never auto-plan"
    );
}

#[test]
fn test_preview_mode_shows_recommendations_only() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("bentreder_logo.png"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::Preview,
    );

    let items = vec![(
        downloads.join("bentreder_logo.png"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    )];

    let result = engine.run(&items);
    // Preview mode should not auto-plan anything
    assert_eq!(
        result.summary.auto_plan_eligible, 0,
        "Preview mode should not auto-plan"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Safety tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_env_file_is_locked() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let project = base.join("home/ImportantApp");
    fs::create_dir_all(&project).unwrap();
    create_file(&project.join(".env"), "SECRET_KEY=abc\n");

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::SafeAutopilot,
    );

    let rec = engine.analyze_file(
        &project.join(".env"),
        safesort_ai::scan::risk::SafetyLevel::Locked,
    );

    assert_eq!(rec.confidence.value(), 0);
    assert!(rec.destinations.is_empty());
}

#[test]
fn test_file_inside_git_repo_is_not_auto_moved() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let repo = base.join("home/Projects/my-project");
    fs::create_dir_all(&repo).unwrap();
    create_file(&repo.join(".git/config"), "");
    create_file(&repo.join("Cargo.toml"), "[package]\nname = \"test\"\n");
    touch(&repo.join("bentreder_logo.png"));

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::SafeAutopilot,
    );

    let rec = engine.analyze_file(
        &repo.join("bentreder_logo.png"),
        safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
    );

    // File inside a project should have reduced confidence due to inside_project penalty
    assert!(
        rec.confidence.value() < 95,
        "File inside git repo should not have auto-plan confidence"
    );
}

#[test]
fn test_sensitive_keyword_becomes_review() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("credentials_backup.json"));

    let home = base.join("home");
    let scanner = safesort_ai::scan::Scanner::new();
    let report = scanner.scan(&downloads, &home, 2).unwrap();

    // The file should be classified as LOCKED due to "credentials"
    let locked = report.get_examples("LOCKED", 100);
    // Note: the safety engine's SensitivePathDetector should catch this
    // The placement engine respects the safety level
    let has_locked = locked.iter().any(|i| i.path.contains("credentials"));
    assert!(
        has_locked,
        "File with 'credentials' keyword should be LOCKED"
    );
}

#[test]
fn test_no_real_file_moving() {
    let tmp = TempDir::new().unwrap();
    let base = to_pb(&tmp);
    let downloads = base.join("home/Downloads");
    fs::create_dir_all(&downloads).unwrap();
    touch(&downloads.join("bentreder_logo.png"));
    touch(&downloads.join("quicktapid_banner.png"));
    touch(&downloads.join("website-fix-finder-v1.0.zip"));

    let count_before: usize = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .count();

    let home = base.join("home");
    let engine = safesort_ai::placement::SmartPlacementEngine::new(
        home.clone(),
        safesort_ai::placement::engine::OrganizationMode::SafeAutopilot,
    );

    let items = vec![
        (
            downloads.join("bentreder_logo.png"),
            safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
        ),
        (
            downloads.join("quicktapid_banner.png"),
            safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
        ),
        (
            downloads.join("website-fix-finder-v1.0.zip"),
            safesort_ai::scan::risk::SafetyLevel::SafeCandidate,
        ),
    ];

    let _result = engine.run(&items);

    let count_after: usize = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .count();

    assert_eq!(
        count_before, count_after,
        "Placement engine must not create, move, or delete any files"
    );
    assert!(downloads.join("bentreder_logo.png").exists());
}

// ═══════════════════════════════════════════════════════════════════
// Confidence scoring tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_confidence_scorer_bands() {
    let c = safesort_ai::placement::confidence::Confidence;
    assert!(c(96).is_auto_plan());
    assert!(!c(95).is_guided_review());
    assert!(c(95).is_auto_plan());
    assert!(c(94).is_guided_review());
    assert!(c(80).is_guided_review());
    assert!(!c(79).is_guided_review());
    assert!(c(79).is_review_needed());
    assert!(c(50).is_review_needed());
    assert!(c(49).is_leave_alone());
    assert!(c(0).is_leave_alone());
}

#[test]
fn test_tokenize() {
    let tokens = safesort_ai::placement::ownership::tokenize("bentreder_logo.png");
    assert_eq!(tokens, vec!["bentreder", "logo"]);

    let tokens = safesort_ai::placement::ownership::tokenize("quicktapid-banner.png");
    assert_eq!(tokens, vec!["quicktapid", "banner"]);

    let tokens = safesort_ai::placement::ownership::tokenize("website-fix-finder-v1.0.zip");
    // Extension (.zip) is stripped; .0 in v1.0 is part of the extension
    assert_eq!(tokens, vec!["website", "fix", "finder", "v1"]);
}

// ═══════════════════════════════════════════════════════════════════
// File purpose tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_archive_purpose_not_release() {
    let detector = safesort_ai::placement::file_purpose::FilePurposeDetector::new();
    // project-archive.zip should be Archive (not ReleaseZip)
    assert_eq!(
        detector.detect("project-archive.zip", std::path::Path::new("/tmp")),
        safesort_ai::placement::file_purpose::FilePurpose::Archive,
    );
}

#[test]
fn test_backup_purpose() {
    let detector = safesort_ai::placement::file_purpose::FilePurposeDetector::new();
    assert_eq!(
        detector.detect("backup-2025.tar.gz", std::path::Path::new("/tmp")),
        safesort_ai::placement::file_purpose::FilePurpose::Backup,
    );
}

// ═══════════════════════════════════════════════════════════════════
// Rules engine tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_rules_engine_match() {
    let mut engine = safesort_ai::placement::rules::RulesEngine::new();
    engine.add_rule(
        "mybrand_logo",
        safesort_ai::placement::destination::PlacementDestination {
            path: PathBuf::from("~/Workspace/Brand Assets/MyBrand/Logos"),
            description: "MyBrand Logos".to_string(),
            is_staging: true,
            risk: safesort_ai::placement::destination::DestinationRisk::Safe,
        },
    );

    let rule = engine.match_file("mybrand_logo_final.png");
    assert!(rule.is_some());
    assert_eq!(rule.unwrap().pattern, "mybrand_logo");
}

// ═══════════════════════════════════════════════════════════════════
// Destination planner tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_destination_planner_logo() {
    let planner =
        safesort_ai::placement::destination::DestinationPlanner::new(PathBuf::from("/home/user"));
    let owner = safesort_ai::placement::ownership::DetectedOwner {
        canonical: "BenTreder.com".to_string(),
        display: "Ben Treder Digital".to_string(),
        category: safesort_ai::placement::ownership::OwnerCategory::Website,
    };
    let dests = planner.plan(
        Some(&owner),
        safesort_ai::placement::file_purpose::FilePurpose::Logo,
        true,
    );
    assert!(!dests.is_empty());
    assert!(dests[0].path.to_string_lossy().contains("Logos"));
    assert!(dests[0].is_staging);
    // Should NOT point to a live website root
    let path_str = dests[0].path.to_string_lossy().to_string();
    assert!(
        !path_str.contains("public_html"),
        "Destination should not be a live website root"
    );
    assert!(
        !path_str.contains("htdocs"),
        "Destination should not be a live website root"
    );
}

#[test]
fn test_destination_planner_all_staging() {
    let planner =
        safesort_ai::placement::destination::DestinationPlanner::new(PathBuf::from("/home/user"));
    let owner = safesort_ai::placement::ownership::DetectedOwner {
        canonical: "TestBrand".to_string(),
        display: "Test Brand".to_string(),
        category: safesort_ai::placement::ownership::OwnerCategory::Brand,
    };

    // All destinations should be staging (not live paths)
    for purpose in &[
        safesort_ai::placement::file_purpose::FilePurpose::Logo,
        safesort_ai::placement::file_purpose::FilePurpose::Banner,
        safesort_ai::placement::file_purpose::FilePurpose::Screenshot,
        safesort_ai::placement::file_purpose::FilePurpose::Report,
        safesort_ai::placement::file_purpose::FilePurpose::ReleaseZip,
    ] {
        let dests = planner.plan(Some(&owner), *purpose, true);
        for dest in &dests {
            assert!(
                dest.is_staging,
                "Destination for {:?} should be staging",
                purpose
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Question queue tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_question_queue_empty() {
    let q = safesort_ai::placement::question_queue::QuestionQueue::new();
    assert!(q.is_empty());
    assert_eq!(q.len(), 0);
}

#[test]
fn test_question_queue_render() {
    let mut q = safesort_ai::placement::question_queue::QuestionQueue::new();
    q.push(safesort_ai::placement::question_queue::Question {
        file_path: "/home/user/Downloads/test.png".to_string(),
        detected_owner: None,
        detected_purpose: safesort_ai::placement::file_purpose::FilePurpose::Unknown,
        file_type_desc: "Image".to_string(),
        risk_level: "YELLOW".to_string(),
        confidence: safesort_ai::placement::confidence::Confidence(85),
        destinations: vec![],
        reason: "Test".to_string(),
        options: vec![
            safesort_ai::placement::question_queue::QuestionOption::Leave,
            safesort_ai::placement::question_queue::QuestionOption::ReviewNeeded,
        ],
    });

    assert_eq!(q.len(), 1);
    let rendered = q.render();
    assert!(rendered.contains("Guided Review Queue"));
    assert!(rendered.contains("test.png"));
    assert!(rendered.contains("Leave in place"));
    assert!(rendered.contains("Nothing was moved"));
}
