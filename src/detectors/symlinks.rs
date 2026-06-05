use crate::scan::evidence::{Evidence, EvidenceKind};
use crate::scan::item::ScanItem;

/// Detects symlinks and marks them (and their targets) for special handling.
pub struct SymlinkDetector;

impl SymlinkDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(&self, item: &ScanItem) -> Vec<Evidence> {
        let mut evidence = Vec::new();

        if item.is_symlink {
            let target_desc = item
                .symlink_target
                .as_ref()
                .map(|t| format!(" → {}", t.display()))
                .unwrap_or_else(|| " → [broken]".to_string());

            evidence.push(Evidence {
                kind: EvidenceKind::Symlink,
                path: item.path.to_string_lossy().to_string(),
                description: format!("Symlink{target_desc}"),
                note: None,
            });

            // If the target exists, mark it as a symlink target (LOCKED).
            if let Some(ref target) = item.symlink_target {
                let abs_target = if target.is_absolute() {
                    target.clone()
                } else {
                    item.path.parent().unwrap_or(target).join(target)
                };
                if abs_target.exists() {
                    evidence.push(Evidence {
                        kind: EvidenceKind::SymlinkTarget,
                        path: abs_target.to_string_lossy().to_string(),
                        description: format!("Symlink target of {}", item.path.to_string_lossy()),
                        note: None,
                    });
                }
            }
        }

        evidence
    }
}
