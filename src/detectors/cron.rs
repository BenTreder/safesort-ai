use crate::scan::evidence::{Evidence, EvidenceKind};
use regex::Regex;

/// Read-only scanner for cron files.
/// Any path referenced by a cron entry is LOCKED.
pub struct CronDetector {
    path_pattern: Regex,
}

impl CronDetector {
    /// Only way to construct: `new()`.
    fn _no_default() {} // intentionally private, prevents Default derive
}

impl CronDetector {
    pub fn new() -> Self {
        Self {
            path_pattern: Regex::new(r#"(/[^\s']+|~/[^\s']+)"#).expect("cron-path regex"),
        }
    }

    /// Scan all cron directories/files for path references.
    pub fn scan_all(&self) -> Vec<Evidence> {
        let mut evidence = Vec::new();

        // /etc/crontab (file)
        let crontab = std::path::Path::new("/etc/crontab");
        if crontab.exists() {
            if let Ok(content) = std::fs::read_to_string(crontab) {
                evidence.extend(self.parse_crontab(&content, "/etc/crontab"));
            } else {
                evidence.push(Evidence {
                    kind: EvidenceKind::PermissionSkipped,
                    path: "/etc/crontab".to_string(),
                    description: "Cannot read /etc/crontab".to_string(),
                    note: Some("skipped — permission denied".to_string()),
                });
            }
        }

        // /etc/cron.d (directory)
        self.scan_cron_dir("/etc/cron.d", &mut evidence);

        // cron.{daily,hourly,weekly,monthly}
        for sub in &["daily", "hourly", "weekly", "monthly"] {
            self.scan_cron_dir(&format!("/etc/cron.{sub}"), &mut evidence);
        }

        evidence
    }

    fn scan_cron_dir(&self, dir_str: &str, evidence: &mut Vec<Evidence>) {
        let dir = std::path::Path::new(dir_str);
        if !dir.exists() || !dir.is_dir() {
            evidence.push(Evidence {
                kind: EvidenceKind::PermissionSkipped,
                path: dir_str.to_string(),
                description: format!("Cron directory not accessible: {dir_str}"),
                note: Some("skipped — not found".to_string()),
            });
            return;
        }

        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if path.extension().and_then(|e| e.to_str()) == Some("sh")
                            || path
                                .file_name()
                                .and_then(|f| f.to_str())
                                .is_some_and(|f| !f.contains('.'))
                        {
                            evidence.extend(self.parse_crontab(&content, &path.to_string_lossy()));
                        }
                    }
                }
            }
            Err(_) => {
                evidence.push(Evidence {
                    kind: EvidenceKind::PermissionSkipped,
                    path: dir_str.to_string(),
                    description: format!("Permission denied reading cron dir: {dir_str}"),
                    note: Some("skipped — permission denied".to_string()),
                });
            }
        }
    }

    /// Parse a crontab-style file for path references in commands.
    fn parse_crontab(&self, content: &str, source: &str) -> Vec<Evidence> {
        let mut evidence = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            // Skip comments, variable assignments, and empty lines
            if line.is_empty()
                || line.starts_with('#')
                || line.contains('=') && !line.contains("PATH=")
            {
                // But we still want to look for PATH= entries
                if !line.contains("PATH=") {
                    continue;
                }
            }

            // Check for PATH= assignment
            if line.starts_with("PATH=") {
                let val = &line[5..].trim();
                for segment in val.split(':') {
                    let segment = segment.trim_matches('"').trim_matches('\'');
                    if segment.starts_with('/') {
                        evidence.push(Evidence {
                            kind: EvidenceKind::CronReference,
                            path: segment.to_string(),
                            description: format!("PATH reference in {source}"),
                            note: None,
                        });
                    }
                }
                continue;
            }

            // Skip cron schedule lines that look like @reboot etc.
            if line.starts_with('@') {
                // Still extract paths from the rest of the line
                let after_at = line
                    .split_whitespace()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join(" ");
                for cap in self.path_pattern.captures_iter(&after_at) {
                    if let Some(m) = cap.get(1) {
                        let val = m.as_str();
                        if val.starts_with('/') && val.len() > 1 {
                            evidence.push(Evidence {
                                kind: EvidenceKind::CronReference,
                                path: val.to_string(),
                                description: format!("Cron entry in {source}"),
                                note: None,
                            });
                        }
                    }
                }
                continue;
            }

            // Standard cron line: minute hour dom month dow  command...
            // Skip the 5 schedule fields then look for paths in the command
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 5 {
                let command = parts[5..].join(" ");
                for cap in self.path_pattern.captures_iter(&command) {
                    if let Some(m) = cap.get(1) {
                        let val = m.as_str();
                        if val.starts_with('/') && val.len() > 1 {
                            evidence.push(Evidence {
                                kind: EvidenceKind::CronReference,
                                path: val.to_string(),
                                description: format!("Cron entry in {source}"),
                                note: None,
                            });
                        }
                    }
                }
            }
        }

        evidence
    }
}
