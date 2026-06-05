use crate::scan::evidence::{Evidence, EvidenceKind};
use crate::scan::item::ScanItem;

pub struct ArchiveDetector;

impl ArchiveDetector {
    pub fn new() -> Self {
        Self
    }

    /// Check if a file has an archive-like name/extension.
    pub fn detect(&self, item: &ScanItem) -> Vec<Evidence> {
        let mut evidence = Vec::new();

        if item.is_dir {
            let name = item.name.to_lowercase();
            for pat in crate::config::BACKUP_PATTERNS {
                if name.contains(pat) {
                    evidence.push(Evidence {
                        kind: EvidenceKind::BackupFolder,
                        path: item.path.to_string_lossy().to_string(),
                        description: format!("Backup/old folder detected: {}", item.name),
                        note: None,
                    });
                    break;
                }
            }
            return evidence;
        }

        let name_lower = item.name.to_lowercase();

        // Check compound extensions like .tar.gz
        for ext in &[".tar.gz", ".tar.bz2", ".tar.xz", ".tar.zst"] {
            if name_lower.ends_with(ext) {
                evidence.push(Evidence {
                    kind: EvidenceKind::ArchiveFile,
                    path: item.path.to_string_lossy().to_string(),
                    description: format!("Archive file: {ext}"),
                    note: None,
                });
                return evidence;
            }
        }

        if let Some(ref ext) = item.extension {
            let ext_lower = ext.to_lowercase();
            for cfg_ext in crate::config::ARCHIVE_EXTENSIONS {
                if ext_lower == cfg_ext.trim_start_matches('.') {
                    evidence.push(Evidence {
                        kind: EvidenceKind::ArchiveFile,
                        path: item.path.to_string_lossy().to_string(),
                        description: format!("Archive file: .{ext_lower}"),
                        note: None,
                    });
                    return evidence;
                }
            }
        }

        evidence
    }
}
