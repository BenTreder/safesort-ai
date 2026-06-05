use crate::scan::evidence::{Evidence, EvidenceKind};
use std::path::Path;

/// Read-only scanner for systemd unit files.
/// Any path referenced by a systemd unit is LOCKED.
pub struct SystemdDetector;

impl SystemdDetector {
    pub fn new() -> Self {
        Self
    }

    /// Scan all systemd unit directories for path references.
    pub fn scan_all(&self) -> Vec<Evidence> {
        let mut evidence = Vec::new();

        for dir_str in crate::config::SYSTEMD_PATHS {
            let dir = Path::new(dir_str);
            if !dir.exists() || !dir.is_dir() {
                evidence.push(Evidence {
                    kind: EvidenceKind::PermissionSkipped,
                    path: dir_str.to_string(),
                    description: format!("Systemd directory not accessible: {dir_str}"),
                    note: Some("skipped — not found or not a directory".to_string()),
                });
                continue;
            }

            match std::fs::read_dir(dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("service")
                            || path.extension().and_then(|e| e.to_str()) == Some("timer")
                            || path.extension().and_then(|e| e.to_str()) == Some("target")
                            || path.extension().and_then(|e| e.to_str()) == Some("socket")
                            || path.extension().and_then(|e| e.to_str()) == Some("mount")
                        {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                evidence
                                    .extend(self.extract_paths(&content, &path.to_string_lossy()));
                            }
                        }
                    }
                }
                Err(_) => {
                    evidence.push(Evidence {
                        kind: EvidenceKind::PermissionSkipped,
                        path: dir_str.to_string(),
                        description: format!("Permission denied reading systemd dir: {dir_str}"),
                        note: Some("skipped — permission denied".to_string()),
                    });
                }
            }
        }

        // Also check user systemd
        if let Some(home) = dirs::home_dir() {
            let user_systemd = home.join(".config/systemd/user");
            if user_systemd.exists() {
                match std::fs::read_dir(&user_systemd) {
                    Ok(entries) => {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                evidence
                                    .extend(self.extract_paths(&content, &path.to_string_lossy()));
                            }
                        }
                    }
                    Err(_) => {
                        evidence.push(Evidence {
                            kind: EvidenceKind::PermissionSkipped,
                            path: user_systemd.to_string_lossy().to_string(),
                            description: "Permission denied reading user systemd dir".to_string(),
                            note: Some("skipped — permission denied".to_string()),
                        });
                    }
                }
            }
        }

        evidence
    }

    /// Scan an arbitrary directory tree for systemd unit files.
    /// Used for fake-systemd fixtures in demo/test contexts.
    pub fn scan_dir(&self, dir: &Path) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        self.scan_dir_recursive(dir, &mut evidence);
        evidence
    }

    fn scan_dir_recursive(&self, dir: &Path, evidence: &mut Vec<Evidence>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_dir_recursive(&path, evidence);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(ext, "service" | "timer" | "target" | "socket" | "mount") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        evidence.extend(self.extract_paths(&content, &path.to_string_lossy()));
                    }
                }
            }
        }
    }

    /// Extract path references from a systemd unit file's content.
    fn extract_paths(&self, content: &str, unit_path: &str) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        let path_keys = [
            "ExecStart",
            "ExecStartPre",
            "ExecStartPost",
            "ExecReload",
            "ExecStop",
            "ExecStopPost",
            "WorkingDirectory",
            "EnvironmentFile",
            "ReadWritePaths",
            "ReadOnlyPaths",
            "CacheDirectory",
            "LogsDirectory",
            "ConfigurationDirectory",
            "RuntimeDirectory",
            "StateDirectory",
        ];

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            for key in &path_keys {
                // Match Key=Value or Key= Value
                if let Some(pos) = line.find('=') {
                    let (k, v) = line.split_at(pos);
                    let k = k.trim();
                    let v = v[1..].trim(); // skip the '='

                    if k == *key && !v.is_empty() {
                        // Handle multiple space-separated paths
                        for path_val in v.split_whitespace() {
                            let path_val = path_val.trim_matches('"').trim_matches('\'');
                            if path_val.starts_with('/')
                                || path_val.starts_with('~')
                                || path_val.starts_with('%')
                            {
                                evidence.push(Evidence {
                                    kind: EvidenceKind::SystemdReference,
                                    path: path_val.to_string(),
                                    description: format!("Referenced by {unit_path} ({key}= …)"),
                                    note: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        evidence
    }
}
