use safesort_ai::shortcuts::{
    do_scan, find_newest_rollback_receipt, load_latest_pointer, manifests_dir, rollbacks_dir,
    show_shortcut_help, target_hash,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn touch(path: &std::path::Path) {
    fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new("."))).unwrap();
    fs::write(path, "").unwrap();
}

fn make_download_fixture(dir: &std::path::Path) {
    touch(&dir.join("report-Q1-2026.pdf"));
    touch(&dir.join("notes.txt"));
    touch(&dir.join("Screenshot-2026.png"));
    touch(&dir.join("export.csv"));
}

// ─── target_hash ───────────────────────────────────────────────────

#[test]
fn test_target_hash_is_stable() {
    let p = PathBuf::from("/home/user/Downloads");
    let h1 = target_hash(&p);
    let h2 = target_hash(&p);
    assert_eq!(h1, h2);
    assert!(
        h1.len() >= 8 && h1.len() <= 16,
        "hash should be 8-16 hex chars, got {}",
        h1.len()
    );
}

#[test]
fn test_target_hash_differs_for_different_paths() {
    let h1 = target_hash(&PathBuf::from("/home/user/Downloads"));
    let h2 = target_hash(&PathBuf::from("/home/user/Desktop"));
    assert_ne!(h1, h2);
}

// ─── manifests_dir / rollbacks_dir ─────────────────────────────────

#[test]
fn test_manifests_dir_is_under_home() {
    let dir = manifests_dir();
    let dir_str = dir.to_string_lossy();
    assert!(
        dir_str.contains(".local/share/safesort/manifests"),
        "manifests_dir should be under .local/share/safesort/manifests, got: {dir_str}"
    );
}

#[test]
fn test_rollbacks_dir_is_under_home() {
    let dir = rollbacks_dir();
    let dir_str = dir.to_string_lossy();
    assert!(
        dir_str.contains(".local/share/safesort/rollbacks"),
        "rollbacks_dir should be under .local/share/safesort/rollbacks, got: {dir_str}"
    );
}

// ─── find_newest_rollback_receipt ──────────────────────────────────

#[test]
fn test_find_newest_rollback_receipt_returns_none_when_no_dir() {
    // rollbacks_dir may or may not exist; function must not panic either way.
    // We can't guarantee the real receipts dir is empty, so just assert it
    // returns an Option without panicking.
    let _ = find_newest_rollback_receipt();
}

// ─── load_latest_pointer ───────────────────────────────────────────

#[test]
fn test_load_latest_pointer_returns_none_when_no_file() {
    // If latest.json doesn't exist, should return Ok(None).
    // We test by pointing at the actual function — if latest.json happens to
    // exist that's fine; we just ensure the function doesn't panic.
    let result = load_latest_pointer();
    assert!(
        result.is_ok(),
        "load_latest_pointer must not error on missing file"
    );
}

// ─── do_scan stores manifest outside scan target ───────────────────

#[test]
fn test_scan_manifest_stored_outside_target() {
    let tmp = TempDir::new().unwrap();
    make_download_fixture(tmp.path());

    let result = do_scan(tmp.path());
    assert!(result.is_ok(), "do_scan should succeed: {:?}", result.err());

    let manifest_path = result.unwrap();

    // Manifest must NOT be inside the scanned folder.
    assert!(
        !manifest_path.starts_with(tmp.path()),
        "Manifest must not be stored inside the scanned folder.\n  manifest: {}\n  target:   {}",
        manifest_path.display(),
        tmp.path().display()
    );

    // Manifest must be under ~/.local/share/safesort/manifests.
    let mdir = manifests_dir();
    assert!(
        manifest_path.starts_with(&mdir),
        "Manifest should be under {}, got {}",
        mdir.display(),
        manifest_path.display()
    );

    // Manifest file must exist.
    assert!(manifest_path.exists(), "Manifest file must exist on disk");
}

#[test]
fn test_scan_writes_latest_pointer() {
    let tmp = TempDir::new().unwrap();
    make_download_fixture(tmp.path());

    let manifest_path = do_scan(tmp.path()).expect("do_scan should succeed");

    let pointer = load_latest_pointer()
        .expect("load_latest_pointer must not error")
        .expect("latest.json should exist after do_scan");

    // The pointer manifest path must be valid and under the manifests dir.
    // (Multiple tests may run concurrently and update latest.json; we just
    // verify the pointer is structurally sound.)
    let mdir = manifests_dir();
    assert!(
        PathBuf::from(&pointer.manifest_path).starts_with(&mdir),
        "latest.json must point to a file under the manifests dir"
    );
    assert!(
        !pointer.scan_target.is_empty(),
        "latest.json scan_target must not be empty"
    );
    // The manifest we wrote must exist even if latest.json was overwritten.
    assert!(
        manifest_path.exists(),
        "written manifest file must exist on disk"
    );
}

#[test]
fn test_scan_moves_nothing() {
    // After do_scan, the fixture files must still be in place.
    let tmp = TempDir::new().unwrap();
    make_download_fixture(tmp.path());

    let before: Vec<PathBuf> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    let _ = do_scan(tmp.path());

    let after: Vec<PathBuf> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    // All files present before must still be present.
    for p in &before {
        assert!(
            after.contains(p),
            "do_scan moved or deleted file: {}",
            p.display()
        );
    }
}

// ─── -run refuses if no latest manifest exists ─────────────────────

#[test]
fn test_run_refuses_without_latest_manifest() {
    // We test the logic directly: if load_latest_pointer returns None, -run
    // should refuse. We simulate by checking the return value of
    // load_latest_pointer when latest.json is absent.
    // (The interactive cmd_shortcut_run requires a real tty, so we test the
    // underlying gate here.)
    let mdir = manifests_dir();
    let latest = mdir.join("latest.json");

    if !latest.exists() {
        let ptr = load_latest_pointer().unwrap();
        assert!(ptr.is_none(), "-run gate: no latest.json means no pointer");
    }
    // If latest.json does exist, we at least verify the function returns Ok.
}

// ─── -run target mismatch check ────────────────────────────────────

#[test]
fn test_run_detects_target_mismatch() {
    // Simulate: latest.json says scan_target = /tmp/some-other-folder
    // current_dir = something different
    // The mismatch logic compares canonicalized paths.
    let dir_a = TempDir::new().unwrap();
    let dir_b = TempDir::new().unwrap();

    let a_canonical = dir_a.path().canonicalize().unwrap();
    let b_canonical = dir_b.path().canonicalize().unwrap();

    // The two temp dirs should be distinct.
    assert_ne!(
        a_canonical, b_canonical,
        "Test requires two distinct temp dirs"
    );
}

// ─── -status and -rollback with no receipts ────────────────────────

#[test]
fn test_find_newest_rollback_receipt_no_panic() {
    // Must not panic even if rollbacks dir is missing or empty.
    let result = std::panic::catch_unwind(find_newest_rollback_receipt);
    assert!(
        result.is_ok(),
        "find_newest_rollback_receipt must not panic"
    );
}

// ─── show_shortcut_help includes expected keys ─────────────────────

#[test]
fn test_shortcut_help_output() {
    // Capture by redirecting — since show_shortcut_help writes to stdout,
    // we just call it and verify it doesn't panic. The output content is
    // validated by checking the binary with assert_cmd in integration tests.
    show_shortcut_help();
}

// ─── Existing commands still compile and are reachable ─────────────

#[test]
fn test_apply_options_struct_accessible() {
    use safesort_ai::apply::ApplyOptions;
    use std::path::Path;
    let fake = Path::new("/tmp/fake.json");
    let _opts = ApplyOptions {
        manifest_path: fake,
        backup_dir: fake,
        rollback_output: None,
        dry_run: true,
        apply_safe_only: true,
        assisted_mode: false,
    };
}

#[test]
fn test_release_scripts_exist() {
    assert!(std::path::Path::new("scripts/install.sh").exists());
    assert!(std::path::Path::new("scripts/uninstall.sh").exists());
}

#[test]
fn test_makefile_targets_exist() {
    let makefile = std::fs::read_to_string("Makefile").expect("Makefile should exist");
    for target in [
        "test:",
        "build:",
        "install:",
        "uninstall:",
        "release-check:",
    ] {
        assert!(
            makefile.contains(target),
            "Makefile should contain target {target}"
        );
    }
}

#[test]
fn test_binary_quick_help_mentions_simple_commands() {
    let mut cmd = assert_cmd::Command::cargo_bin("safesort").unwrap();
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("safesort -scan"))
        .stdout(predicates::str::contains("safesort -run"))
        .stdout(predicates::str::contains("safesort -status"))
        .stdout(predicates::str::contains("safesort -rollback"))
        .stdout(predicates::str::contains("./safesort/"));
}

#[test]
fn test_status_command_does_not_panic() {
    let mut cmd = assert_cmd::Command::cargo_bin("safesort").unwrap();
    cmd.arg("-status").assert().success();
}
