use super::evidence::{Evidence, EvidenceKind};
use super::item::ScanItem;
use super::risk::{RiskScore, SafetyLevel};
use crate::config;
use crate::detectors::archives::ArchiveDetector;
use crate::detectors::projects::ProjectDetector;
use crate::detectors::scripts::ScriptPathDetector;
use crate::detectors::sensitive::SensitivePathDetector;
use crate::detectors::symlinks::SymlinkDetector;
use std::path::Path;

/// Full classification result for a scanned item.
#[derive(Debug, Clone)]
pub struct Classification {
    pub level: SafetyLevel,
    pub score: RiskScore,
    pub evidence: Vec<Evidence>,
}

/// Classifies items based on all available detectors.
pub struct Classifier {
    project_detector: ProjectDetector,
    sensitive_detector: SensitivePathDetector,
    symlink_detector: SymlinkDetector,
    script_detector: ScriptPathDetector,
    archive_detector: ArchiveDetector,
}

impl Default for Classifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Classifier {
    pub fn new() -> Self {
        Self {
            project_detector: ProjectDetector::new(),
            sensitive_detector: SensitivePathDetector::new(),
            symlink_detector: SymlinkDetector::new(),
            script_detector: ScriptPathDetector::new(),
            archive_detector: ArchiveDetector::new(),
        }
    }

    /// Classify a single scan item.
    pub fn classify(&self, item: &ScanItem, scan_root: &Path, home: &Path) -> Classification {
        let mut evidence = Vec::new();
        let mut risk = RiskScore(0.0);
        let mut max_level = SafetyLevel::SafeCandidate;

        // 1. System-critical paths
        let path_str = item.path.to_string_lossy();
        for prefix in config::LOCKED_SYSTEM_PATHS {
            if path_str.starts_with(prefix) {
                evidence.push(Evidence {
                    kind: EvidenceKind::SystemCritical,
                    path: path_str.to_string(),
                    description: format!("System-critical: starts with {prefix}"),
                    note: None,
                });
                return Classification {
                    level: SafetyLevel::Locked,
                    score: RiskScore(1.0),
                    evidence,
                };
            }
        }

        // 2. Sensitive home directories
        let mut sensitive_ev = self.sensitive_detector.detect_dir(item);
        if !sensitive_ev.is_empty() {
            evidence.append(&mut sensitive_ev);
            risk = risk.at_least(0.95);
            max_level = SafetyLevel::Locked;
        }

        // 3. Sensitive files (file itself is a sensitive marker)
        let mut sen_file_ev = self.sensitive_detector.detect_file(item);
        if !sen_file_ev.is_empty() {
            evidence.append(&mut sen_file_ev);
            risk = risk.at_least(0.95);
            max_level = SafetyLevel::Locked;
        }

        // 3b. For directories: check whether they contain sensitive files (.env etc.)
        //     A folder that hosts a credential file should be LOCKED, not REVIEW.
        if item.is_dir {
            let mut dir_sensitive_ev = self.sensitive_detector.detect_sensitive_in_dir(&item.path);
            if !dir_sensitive_ev.is_empty() {
                evidence.append(&mut dir_sensitive_ev);
                risk = risk.at_least(0.95);
                max_level = SafetyLevel::Locked;
            }
        }

        // 4. Symlinks
        let mut sym_ev = self.symlink_detector.detect(item);
        if !sym_ev.is_empty() {
            evidence.append(&mut sym_ev);
            risk = risk.at_least(0.7);
            if matches!(max_level, SafetyLevel::SafeCandidate) {
                max_level = SafetyLevel::Review;
            }
            // Symlink targets are LOCKED — but that's the target, not the link
            if item.is_symlink {
                // The symlink itself is REVIEW, target is LOCKED (handled in policy)
            }
        }

        // 5. Project markers
        if item.is_dir {
            let mut proj_ev = self.project_detector.detect_in_dir(&item.path);
            if !proj_ev.is_empty() {
                evidence.append(&mut proj_ev);
                risk = risk.at_least(0.5);
                if matches!(max_level, SafetyLevel::SafeCandidate) {
                    max_level = SafetyLevel::Review;
                }
            }
        }
        let mut proj_file_ev = self.project_detector.detect_file(item);
        if !proj_file_ev.is_empty() {
            evidence.append(&mut proj_file_ev);
            risk = risk.at_least(0.6);
            if matches!(max_level, SafetyLevel::SafeCandidate) {
                max_level = SafetyLevel::Review;
            }
        }

        // 6. Scripts / config path references
        let mut script_ev = self.script_detector.scan_file(item);
        if !script_ev.is_empty() {
            let has_path_refs = script_ev
                .iter()
                .any(|e| matches!(e.kind, EvidenceKind::ScriptPathRef));
            evidence.append(&mut script_ev);
            if has_path_refs {
                risk = risk.at_least(0.7);
                max_level = SafetyLevel::Locked;
            } else {
                risk = risk.at_least(0.4);
                if matches!(max_level, SafetyLevel::SafeCandidate) {
                    max_level = SafetyLevel::Review;
                }
            }
        }

        // 7. Archives
        let mut arch_ev = self.archive_detector.detect(item);
        if !arch_ev.is_empty() {
            evidence.append(&mut arch_ev);
            // Archives in safe loose zones are SAFE_CANDIDATE
            if Self::is_in_safe_loose_zone(&item.path, scan_root, home) {
                evidence.push(Evidence {
                    kind: EvidenceKind::SafeZoneLoose,
                    path: item.path.to_string_lossy().to_string(),
                    description: "Loose file in safe zone (Downloads/Desktop)".to_string(),
                    note: None,
                });
                risk = risk.at_least(0.1);
                // Keep max_level as is (already SafeCandidate)
            } else {
                risk = risk.at_least(0.3);
                if matches!(max_level, SafetyLevel::SafeCandidate) {
                    max_level = SafetyLevel::Review;
                }
            }
        }

        // 8. Media / document files
        let mut media_doc_ev = Self::detect_media_document(item);
        if !media_doc_ev.is_empty() {
            evidence.append(&mut media_doc_ev);
            if Self::is_in_safe_loose_zone(&item.path, scan_root, home) {
                evidence.push(Evidence {
                    kind: EvidenceKind::SafeZoneLoose,
                    path: item.path.to_string_lossy().to_string(),
                    description: "Loose file in safe zone (Downloads/Desktop)".to_string(),
                    note: None,
                });
            } else {
                if matches!(max_level, SafetyLevel::SafeCandidate) {
                    max_level = SafetyLevel::Review;
                }
            }
        }

        // 9. private_* folders
        if item.name.to_lowercase().starts_with("private_") {
            evidence.push(Evidence {
                kind: EvidenceKind::SensitivePath,
                path: item.path.to_string_lossy().to_string(),
                description: "Folder starts with 'private_'".to_string(),
                note: None,
            });
            risk = risk.at_least(0.9);
            max_level = SafetyLevel::Locked;
        }

        // 10. Hidden directories that aren't already classified are REVIEW
        if item.is_dir && item.is_hidden && evidence.is_empty() {
            evidence.push(Evidence {
                kind: EvidenceKind::MixedContents,
                path: item.path.to_string_lossy().to_string(),
                description: "Hidden directory".to_string(),
                note: None,
            });
            risk = risk.at_least(0.4);
            max_level = SafetyLevel::Review;
        }

        // 11. Unknown mixed contents
        if item.is_dir && evidence.is_empty() {
            evidence.push(Evidence {
                kind: EvidenceKind::MixedContents,
                path: item.path.to_string_lossy().to_string(),
                description: "Directory with no recognizable markers".to_string(),
                note: None,
            });
            risk = risk.at_least(0.3);
            max_level = SafetyLevel::Review;
        }

        // 12. Website folder detection
        if item.is_dir {
            let name_lower = item.name.to_lowercase();
            if name_lower.contains("website")
                || name_lower.contains("site")
                || name_lower.contains("www")
                || name_lower.contains("public_html")
                || name_lower.contains("htdocs")
            {
                evidence.push(Evidence {
                    kind: EvidenceKind::WebsiteFolder,
                    path: item.path.to_string_lossy().to_string(),
                    description: "Possible live website folder".to_string(),
                    note: None,
                });
                risk = risk.at_least(0.7);
                max_level = SafetyLevel::Locked;
            }
        }

        Classification {
            level: max_level,
            score: risk,
            evidence,
        }
    }

    fn is_in_safe_loose_zone(path: &Path, scan_root: &Path, home: &Path) -> bool {
        // Primary check: is the file under ~/Downloads or ~/Desktop?
        if let Ok(rel) = path.strip_prefix(home) {
            if let Some(first) = rel.components().next() {
                let name = first.as_os_str().to_string_lossy();
                if config::SAFE_LOOSE_ZONES.iter().any(|z| name == *z) {
                    return true;
                }
            }
        }
        // Fallback: scan root itself is a safe loose zone (e.g. ./safesort_demo/Downloads).
        // This handles demo fixtures and explicit --path Downloads invocations.
        if let Some(root_name) = scan_root.file_name() {
            let name = root_name.to_string_lossy();
            if config::SAFE_LOOSE_ZONES.iter().any(|z| name == *z) {
                return path.starts_with(scan_root);
            }
        }
        false
    }

    fn detect_media_document(item: &ScanItem) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        if item.is_dir {
            return evidence;
        }

        if let Some(ref ext) = item.extension {
            let ext_lower = ext.to_lowercase();
            if config::MEDIA_EXTENSIONS
                .iter()
                .any(|e| e.trim_start_matches('.') == ext_lower)
            {
                evidence.push(Evidence {
                    kind: EvidenceKind::MediaFile,
                    path: item.path.to_string_lossy().to_string(),
                    description: format!("Media file: .{ext_lower}"),
                    note: None,
                });
            } else if config::DOCUMENT_EXTENSIONS
                .iter()
                .any(|e| e.trim_start_matches('.') == ext_lower)
            {
                evidence.push(Evidence {
                    kind: EvidenceKind::DocumentFile,
                    path: item.path.to_string_lossy().to_string(),
                    description: format!("Document file: .{ext_lower}"),
                    note: None,
                });
            }
        }

        evidence
    }
}
