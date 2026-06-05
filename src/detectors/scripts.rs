use crate::scan::evidence::{Evidence, EvidenceKind};
use crate::scan::item::ScanItem;
use regex::Regex;
use std::path::Path;

/// Reads text/script/config files and detects absolute path references.
/// Does NOT edit files — only reports references.
pub struct ScriptPathDetector {
    path_pattern: Regex,
    home_pattern: Regex,
    tilde_pattern: Regex,
}

impl ScriptPathDetector {
    pub fn new() -> Self {
        Self {
            path_pattern: Regex::new(
                r#"(/home/[^\s'":<>;|]+|/var/www/[^\s'":<>;|]+|/srv/[^\s'":<>;|]+|/opt/[^\s'":<>;|]+)"#,
            )
            .expect("absolute-path regex"),
            home_pattern: Regex::new(r#"HOME\s*=\s*["']?(/[^"'\s]+)"#)
                .expect("home-var regex"),
            tilde_pattern: Regex::new(r#"~(/[^\s'":<>;|]*)"#)
                .expect("tilde-path regex"),
        }
    }

    /// Check if a file is a script or config file worth scanning.
    pub fn is_interesting(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext.to_lowercase().as_str(),
                "sh" | "bash"
                    | "zsh"
                    | "fish"
                    | "py"
                    | "pl"
                    | "rb"
                    | "js"
                    | "ts"
                    | "php"
                    | "conf"
                    | "cfg"
                    | "ini"
                    | "yaml"
                    | "yml"
                    | "toml"
                    | "json"
                    | "env"
                    | "service"
                    | "timer"
                    | "target"
                    | "socket"
                    | "mount"
            )
        } else {
            // Files without extension that look like scripts
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                name.contains("rc") || name.starts_with('.')
            } else {
                false
            }
        }
    }

    /// Read a text file and detect absolute path references.
    pub fn scan_file(&self, item: &ScanItem) -> Vec<Evidence> {
        let mut evidence = Vec::new();

        if item.is_dir || item.is_symlink {
            return evidence;
        }

        if !Self::is_interesting(&item.path) {
            return evidence;
        }

        // Read up to 8 KiB for safety.
        let content = match std::fs::read_to_string(&item.path) {
            Ok(c) => c,
            Err(_) => return evidence,
        };

        // Limit scan to first 8 KiB
        let scan_content = if content.len() > 8192 {
            &content[..8192]
        } else {
            &content
        };

        for cap in self.path_pattern.captures_iter(scan_content) {
            if let Some(m) = cap.get(1) {
                evidence.push(Evidence {
                    kind: EvidenceKind::ScriptPathRef,
                    path: m.as_str().to_string(),
                    description: format!("Absolute path reference in {}", item.path.display()),
                    note: None,
                });
            }
        }

        for cap in self.home_pattern.captures_iter(scan_content) {
            if let Some(m) = cap.get(1) {
                let val = m.as_str().to_string();
                // Avoid duplicates already caught by absolute pattern
                if !evidence.iter().any(|e: &Evidence| e.path == val) {
                    evidence.push(Evidence {
                        kind: EvidenceKind::ScriptPathRef,
                        path: val.clone(),
                        description: format!("HOME=/path reference in {}", item.path.display()),
                        note: None,
                    });
                }
            }
        }

        for cap in self.tilde_pattern.captures_iter(scan_content) {
            if let Some(m) = cap.get(1) {
                let val = format!("~{}", m.as_str());
                if !evidence.iter().any(|e: &Evidence| e.path.starts_with('~')) {
                    evidence.push(Evidence {
                        kind: EvidenceKind::ScriptPathRef,
                        path: val,
                        description: format!("Tilde path reference in {}", item.path.display()),
                        note: None,
                    });
                }
            }
        }

        // Check for shebang → this is a script
        if scan_content.starts_with("#!") {
            let shebang = scan_content.lines().next().unwrap_or("").trim();
            evidence.push(Evidence {
                kind: EvidenceKind::ContainsScripts,
                path: item.path.to_string_lossy().to_string(),
                description: format!("Shell/interpreted script: {shebang}"),
                note: None,
            });
        }

        evidence
    }
}
