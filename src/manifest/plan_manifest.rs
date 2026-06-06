use super::checksum::checksum_file;
use super::rollback::{ManifestEntry, RollbackManifest};
use crate::placement::engine::{OrganizationMode, PlacementRecommendation};
use crate::placement::file_purpose::FilePurpose;
use crate::scan::risk::SafetyLevel;
use std::path::Path;

/// Returns true if the file extension marks it as a shell/system script.
fn is_script_extension(path_str: &str) -> bool {
    let lower = path_str.to_lowercase();
    let ext = std::path::Path::new(&lower)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    matches!(ext, "sh" | "bat" | "cmd" | "ps1" | "bash" | "zsh" | "fish")
}

/// Build a dry-run rollback manifest from a set of placement recommendations.
///
/// Only SAFE_CANDIDATE items with NONE or LOW impact are included as move
/// candidates. LOCKED and REVIEW items, and items with MEDIUM/HIGH/CRITICAL
/// impact, are counted in `excluded_for_safety` and never included as entries.
///
/// Checksums are computed for eligible files when they exist on disk.
/// This function never moves, copies, renames, or deletes any file.
pub fn build_plan_manifest(
    scan_target: &Path,
    plan_mode: OrganizationMode,
    recommendations: &[PlacementRecommendation],
    rule_file_used: Option<&str>,
    total_scanned: usize,
) -> RollbackManifest {
    let run_id = chrono::Local::now().format("%Y%m%d_%H%M%S_%6f").to_string();

    let mut manifest = RollbackManifest::new(
        run_id,
        scan_target.to_string_lossy().to_string(),
        plan_mode.as_str().to_string(),
    );
    manifest.total_scanned = total_scanned;

    for rec in recommendations {
        // Safety gate: never include LOCKED items as move candidates.
        if matches!(rec.safety_level, SafetyLevel::Locked) {
            manifest.excluded_for_safety += 1;
            continue;
        }

        // Safety gate: never include MEDIUM/HIGH/CRITICAL impact as auto candidates.
        let impact_ok = matches!(rec.impact_level.as_str(), "NONE" | "LOW");

        // Only SAFE_CANDIDATE items go into the manifest entries.
        if !matches!(rec.safety_level, SafetyLevel::SafeCandidate) {
            manifest.excluded_for_safety += 1;
            continue;
        }

        // Determine planned destination (first destination if any).
        let planned_destination = rec
            .destinations
            .first()
            .map(|d| d.path.to_string_lossy().to_string())
            .unwrap_or_else(|| "(no destination computed)".to_string());

        // Safety gate: "no destination computed" is never movable.
        if planned_destination.contains("no destination computed") {
            manifest.entries.push(ManifestEntry {
                source_path: rec.file_path.to_string_lossy().to_string(),
                planned_destination,
                checksum_before: None,
                file_size: 0,
                safety_level: rec.safety_level.as_str().to_string(),
                impact_level: rec.impact_level.clone(),
                reason: "No destination computed — manual review required".to_string(),
                confidence: rec.confidence.value(),
                rule_file_used: rule_file_used.map(str::to_string),
                dry_run_only: true,
                auto_plan_eligible: false,
                assisted_plan_eligible: false,
            });
            continue;
        }

        // Compute checksum if file exists on disk.
        let (checksum_before, file_size) = {
            let p = &rec.file_path;
            if p.exists() && p.is_file() {
                match checksum_file(p) {
                    Ok(cs) => {
                        let sz = cs.file_size;
                        (Some(cs), sz)
                    }
                    Err(_) => (None, 0),
                }
            } else {
                (None, 0)
            }
        };

        // Review Needed destinations should never be auto or assisted
        // (matches both legacy "99_Review Needed" and new local "Other/Review Needed")
        let dest_is_review_needed = planned_destination.contains("Review Needed")
            || planned_destination.contains("99_Review");

        let auto_plan_eligible = matches!(rec.safety_level, SafetyLevel::SafeCandidate)
            && impact_ok
            && rec.confidence.value() >= 95
            && !dest_is_review_needed
            // Never auto-plan if owner is Unknown in the destination (legacy path)
            && !planned_destination.contains("/Unknown/")
            // Never auto-plan sensitive documents
            && !matches!(rec.purpose, FilePurpose::SensitiveDocument)
            // Never auto-plan legacy generic catch-all destinations
            && !planned_destination.contains("/Client Reports")
            && !planned_destination.ends_with("/Documents")
            && !planned_destination.contains("07_Media/Product Images");

        // Assisted mode: lower confidence bar, more file types allowed.
        // Sensitive docs allowed in local mode (go to SensitiveDocuments/) but not legacy mode.
        let path_str = rec.file_path.to_string_lossy();
        let dest_is_sensitive_docs = planned_destination.contains("SensitiveDocuments");
        let assisted_plan_eligible = !auto_plan_eligible  // exclusive with auto
            && matches!(rec.safety_level, SafetyLevel::SafeCandidate)
            && impact_ok
            && rec.confidence.value() >= 60
            && !matches!(rec.purpose, FilePurpose::Code | FilePurpose::Unknown)
            // Sensitive docs only allowed in assisted when in local mode (SensitiveDocuments/)
            && (!matches!(rec.purpose, FilePurpose::SensitiveDocument) || dest_is_sensitive_docs)
            && !is_script_extension(&path_str)
            && !path_str.ends_with(".part")
            && !dest_is_review_needed
            && !planned_destination.contains("(no destination computed)");

        manifest.entries.push(ManifestEntry {
            source_path: rec.file_path.to_string_lossy().to_string(),
            planned_destination,
            checksum_before,
            file_size,
            safety_level: rec.safety_level.as_str().to_string(),
            impact_level: rec.impact_level.clone(),
            reason: rec.reason.clone(),
            confidence: rec.confidence.value(),
            rule_file_used: rule_file_used.map(str::to_string),
            dry_run_only: true,
            auto_plan_eligible,
            assisted_plan_eligible,
        });
    }

    manifest
}
