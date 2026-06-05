use crate::scan::evidence::{Evidence, EvidenceKind};
use crate::scan::item::ScanItem;
use std::path::Path;

/// Detects project markers: .git, Cargo.toml, package.json, etc.
pub struct ProjectDetector;

impl ProjectDetector {
    pub fn new() -> Self {
        Self
    }

    /// Check if a directory contains any project markers.
    pub fn detect_in_dir(&self, dir: &Path) -> Vec<Evidence> {
        let mut evidence = Vec::new();

        for marker in crate::config::PROJECT_MARKERS {
            let candidate = dir.join(marker);
            if candidate.exists() {
                let kind = match marker.as_ref() {
                    ".git" => EvidenceKind::ProjectMarker,
                    "Cargo.toml" => EvidenceKind::ContainsRust,
                    "package.json" | "node_modules" => EvidenceKind::ContainsNodeJs,
                    "composer.json" | "wp-config.php" | "vendor" => EvidenceKind::ContainsWordPress,
                    "pyproject.toml" | "requirements.txt" | "venv" | ".venv" => {
                        EvidenceKind::ContainsPython
                    }
                    "docker-compose.yml" | "Dockerfile" => EvidenceKind::ContainsDockerfile,
                    "Makefile" => EvidenceKind::ProjectMarker,
                    _ => EvidenceKind::ProjectMarker,
                };
                evidence.push(Evidence {
                    kind,
                    path: candidate.to_string_lossy().to_string(),
                    description: format!("Project marker found: {marker}"),
                    note: None,
                });
            }
        }

        evidence
    }

    /// Check if a single file is a project marker.
    pub fn detect_file(&self, item: &ScanItem) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        let name_lower = item.name.to_lowercase();

        for marker in crate::config::PROJECT_MARKERS {
            if name_lower == marker.to_lowercase() {
                evidence.push(Evidence {
                    kind: EvidenceKind::ProjectMarker,
                    path: item.path.to_string_lossy().to_string(),
                    description: format!("Project marker file: {marker}"),
                    note: None,
                });
            }
        }

        evidence
    }
}
