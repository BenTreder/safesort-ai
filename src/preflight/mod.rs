use crate::error::{Result, SafeSortError};
use crate::manifest::{RollbackManifest, checksum_file};
use crate::rules_file::validation::is_safe_destination;
use std::path::Path;

/// Result of a single preflight check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

/// Full preflight report for a manifest.
#[derive(Debug)]
pub struct PreflightReport {
    pub manifest_path: String,
    pub checks: Vec<CheckResult>,
    pub all_passed: bool,
}

impl PreflightReport {
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("\n  SafeSort AI — Apply Preflight\n");
        out.push_str("  ─────────────────────────────────\n");
        out.push_str(&format!("  Manifest: {}\n\n", self.manifest_path));

        for c in &self.checks {
            let icon = if c.passed { "✅" } else { "❌" };
            out.push_str(&format!("  {} {}\n", icon, c.label));
            if !c.detail.is_empty() {
                out.push_str(&format!("       {}\n", c.detail));
            }
        }

        out.push('\n');
        if self.all_passed {
            out.push_str(
                "  ✅ Preflight PASSED — all checks passed.\n\
                 \n\
                 ⚠️  Apply is still disabled in this MVP build.\n\
                 \n\
                 This preflight confirms that IF apply were enabled,\n\
                 all safety gates would currently pass for this manifest.\n\
                 Nothing was moved.\n",
            );
        } else {
            out.push_str("  ❌ Preflight FAILED — one or more checks did not pass.\n");
            out.push_str("     Nothing was moved. Fix the issues above before retrying.\n");
        }
        out.push('\n');
        out
    }
}

/// Load a manifest from disk and run all preflight checks.
/// Never moves, copies, renames, or deletes any file.
pub fn run_preflight(manifest_path: &Path) -> Result<PreflightReport> {
    let mut checks: Vec<CheckResult> = Vec::new();

    // ── Check 1: File exists and is valid JSON ───────────────────────
    let raw = match std::fs::read_to_string(manifest_path) {
        Ok(s) => s,
        Err(e) => {
            return Err(SafeSortError::Io(e));
        }
    };

    let manifest: RollbackManifest = match serde_json::from_str::<RollbackManifest>(&raw) {
        Ok(m) => {
            checks.push(CheckResult {
                label: "Manifest loads as valid JSON".to_string(),
                passed: true,
                detail: format!("run_id={}", m.run_id),
            });
            m
        }
        Err(e) => {
            checks.push(CheckResult {
                label: "Manifest loads as valid JSON".to_string(),
                passed: false,
                detail: format!("Parse error: {e}"),
            });
            return Ok(PreflightReport {
                manifest_path: manifest_path.display().to_string(),
                checks,
                all_passed: false,
            });
        }
    };

    // ── Check 2: dry_run_only = true ────────────────────────────────
    checks.push(CheckResult {
        label: "dry_run_only is true".to_string(),
        passed: manifest.dry_run_only,
        detail: if manifest.dry_run_only {
            String::new()
        } else {
            "Manifest has dry_run_only=false — this is not a valid SafeSort manifest".to_string()
        },
    });

    // ── Check 3: No LOCKED entries ───────────────────────────────────
    let locked_entries: Vec<_> = manifest
        .entries
        .iter()
        .filter(|e| e.safety_level.to_uppercase() == "LOCKED")
        .collect();
    checks.push(CheckResult {
        label: format!(
            "No LOCKED entries ({} entries checked)",
            manifest.entries.len()
        ),
        passed: locked_entries.is_empty(),
        detail: if locked_entries.is_empty() {
            String::new()
        } else {
            format!(
                "{} LOCKED entries found — these must not appear in a manifest",
                locked_entries.len()
            )
        },
    });

    // ── Check 4: No MEDIUM/HIGH/CRITICAL impact entries ─────────────
    let high_impact: Vec<_> = manifest
        .entries
        .iter()
        .filter(|e| {
            matches!(
                e.impact_level.to_uppercase().as_str(),
                "MEDIUM" | "HIGH" | "CRITICAL"
            )
        })
        .collect();
    checks.push(CheckResult {
        label: "No MEDIUM/HIGH/CRITICAL impact entries".to_string(),
        passed: high_impact.is_empty(),
        detail: if high_impact.is_empty() {
            String::new()
        } else {
            format!(
                "{} high-impact entries found — these must not be moved",
                high_impact.len()
            )
        },
    });

    // ── Check 5: All source files still exist ───────────────────────
    let mut missing_sources: Vec<String> = Vec::new();
    for entry in &manifest.entries {
        if !Path::new(&entry.source_path).exists() {
            missing_sources.push(entry.source_path.clone());
        }
    }
    checks.push(CheckResult {
        label: format!(
            "All source files exist ({} entries)",
            manifest.entries.len()
        ),
        passed: missing_sources.is_empty(),
        detail: if missing_sources.is_empty() {
            String::new()
        } else {
            format!(
                "{} source(s) no longer exist: {}",
                missing_sources.len(),
                missing_sources.join(", ")
            )
        },
    });

    // ── Check 6: Checksums still match ──────────────────────────────
    let mut checksum_failures: Vec<String> = Vec::new();
    for entry in &manifest.entries {
        let src = Path::new(&entry.source_path);
        if !src.exists() {
            continue; // already caught above
        }
        if let Some(ref expected) = entry.checksum_before {
            match checksum_file(src) {
                Ok(current) => {
                    if current.sha256 != expected.sha256 {
                        checksum_failures.push(format!(
                            "{} (expected={} got={})",
                            entry.source_path,
                            &expected.sha256[..12],
                            &current.sha256[..12],
                        ));
                    }
                }
                Err(_) => {
                    checksum_failures
                        .push(format!("{} (could not re-read file)", entry.source_path));
                }
            }
        }
    }
    checks.push(CheckResult {
        label: "All checksums still match".to_string(),
        passed: checksum_failures.is_empty(),
        detail: if checksum_failures.is_empty() {
            String::new()
        } else {
            format!(
                "{} file(s) changed since planning: {}",
                checksum_failures.len(),
                checksum_failures.join("; ")
            )
        },
    });

    // ── Check 7: File sizes still match ─────────────────────────────
    let mut size_failures: Vec<String> = Vec::new();
    for entry in &manifest.entries {
        let src = Path::new(&entry.source_path);
        if !src.exists() {
            continue;
        }
        if entry.file_size > 0 {
            if let Ok(meta) = std::fs::metadata(src) {
                if meta.len() != entry.file_size {
                    size_failures.push(format!(
                        "{} (expected={} bytes, got={} bytes)",
                        entry.source_path,
                        entry.file_size,
                        meta.len()
                    ));
                }
            }
        }
    }
    checks.push(CheckResult {
        label: "All file sizes still match".to_string(),
        passed: size_failures.is_empty(),
        detail: if size_failures.is_empty() {
            String::new()
        } else {
            format!(
                "{} file(s) changed size: {}",
                size_failures.len(),
                size_failures.join("; ")
            )
        },
    });

    // ── Check 8: All destinations are safe ──────────────────────────
    let mut unsafe_dests: Vec<String> = Vec::new();
    for entry in &manifest.entries {
        if !entry.planned_destination.is_empty() && !is_safe_destination(&entry.planned_destination)
        {
            unsafe_dests.push(entry.planned_destination.clone());
        }
    }
    checks.push(CheckResult {
        label: "All planned destinations are safe".to_string(),
        passed: unsafe_dests.is_empty(),
        detail: if unsafe_dests.is_empty() {
            String::new()
        } else {
            format!(
                "{} unsafe destination(s): {}",
                unsafe_dests.len(),
                unsafe_dests.join("; ")
            )
        },
    });

    // ── Final result ─────────────────────────────────────────────────
    let all_passed = checks.iter().all(|c| c.passed);
    Ok(PreflightReport {
        manifest_path: manifest_path.display().to_string(),
        checks,
        all_passed,
    })
}
