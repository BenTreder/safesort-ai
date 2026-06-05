use crate::scan::evidence::{Evidence, EvidenceKind};
use crate::scan::item::ScanItem;
use std::path::Path;

/// Detects sensitive paths: .ssh, .gnupg, .aws, etc.
pub struct SensitivePathDetector {
    sensitive_home: Vec<String>,
}

impl SensitivePathDetector {
    pub fn new() -> Self {
        Self {
            sensitive_home: crate::config::SENSITIVE_HOME_DIRS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    /// Check if the path is or is inside a sensitive home directory.
    pub fn is_sensitive_home_prefix(&self, path: &Path) -> bool {
        // look at path components
        let components: Vec<_> = path.components().collect();
        for c in &components {
            if let Some(s) = c.as_os_str().to_str() {
                for sensitive in &self.sensitive_home {
                    if s == sensitive {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Detect sensitive file by name (id_rsa, .env, .npmrc, etc.)
    pub fn detect_file(&self, item: &ScanItem) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        let name = &item.name;

        for marker in crate::config::SENSITIVE_FILE_MARKERS {
            if name == *marker {
                evidence.push(Evidence {
                    kind: EvidenceKind::SensitiveFile,
                    path: item.path.to_string_lossy().to_string(),
                    description: format!("Sensitive file: {name}"),
                    note: None,
                });
            }
        }

        // Check for key-like patterns
        let name_lower = name.to_lowercase();
        if name_lower.contains("private_key")
            || name_lower.contains("secret")
            || name_lower.contains("credential")
            || name_lower.contains("token")
            || name_lower.ends_with(".pem")
            || name_lower.ends_with(".key")
        {
            evidence.push(Evidence {
                kind: EvidenceKind::SensitiveFile,
                path: item.path.to_string_lossy().to_string(),
                description: format!("Potential secret/key file: {name}"),
                note: None,
            });
        }

        evidence
    }

    /// Detect sensitive directories.
    pub fn detect_dir(&self, item: &ScanItem) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        let name = &item.name;

        for sensitive in &self.sensitive_home {
            if name == sensitive {
                evidence.push(Evidence {
                    kind: EvidenceKind::SensitivePath,
                    path: item.path.to_string_lossy().to_string(),
                    description: format!("Sensitive directory: ~/{name}"),
                    note: None,
                });
            }
        }

        evidence
    }
}
