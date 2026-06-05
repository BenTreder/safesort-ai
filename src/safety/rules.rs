use crate::scan::evidence::{Evidence, EvidenceKind};
use crate::scan::item::ScanItem;
use crate::scan::risk::SafetyLevel;
use std::path::Path;

/// A safety rule: given an item, produce evidence and a risk score delta.
pub trait SafetyRule: Send + Sync {
    fn evaluate(&self, item: &ScanItem, scan_root: &Path, home: &Path) -> RuleResult;
}

/// Result of evaluating a single rule.
pub struct RuleResult {
    pub evidence: Vec<Evidence>,
    pub risk_delta: f64,
    pub suggested_level: Option<SafetyLevel>,
}

impl RuleResult {
    pub fn none() -> Self {
        Self {
            evidence: vec![],
            risk_delta: 0.0,
            suggested_level: None,
        }
    }

    pub fn with_evidence(kind: EvidenceKind, path: &str, desc: &str) -> Self {
        Self {
            evidence: vec![Evidence {
                kind,
                path: path.to_string(),
                description: desc.to_string(),
                note: None,
            }],
            risk_delta: 0.0,
            suggested_level: None,
        }
    }

    pub fn with_level(level: SafetyLevel, kind: EvidenceKind, path: &str, desc: &str) -> Self {
        Self {
            evidence: vec![Evidence {
                kind,
                path: path.to_string(),
                description: desc.to_string(),
                note: None,
            }],
            risk_delta: 0.0,
            suggested_level: Some(level),
        }
    }
}

/// Rule: system-critical paths are always LOCKED.
pub struct SystemCriticalRule;

impl SafetyRule for SystemCriticalRule {
    fn evaluate(&self, item: &ScanItem, _scan_root: &Path, _home: &Path) -> RuleResult {
        let path_str = item.path.to_string_lossy();
        for prefix in crate::config::LOCKED_SYSTEM_PATHS {
            if path_str.starts_with(prefix) {
                return RuleResult::with_level(
                    SafetyLevel::Locked,
                    EvidenceKind::SystemCritical,
                    &path_str,
                    &format!("System-critical path: starts with {prefix}"),
                );
            }
        }
        RuleResult::none()
    }
}

/// Rule: sensitive home directories are LOCKED.
pub struct SensitiveHomeRule;

impl SafetyRule for SensitiveHomeRule {
    fn evaluate(&self, item: &ScanItem, _scan_root: &Path, home: &Path) -> RuleResult {
        let rel = match item.path.strip_prefix(home) {
            Ok(r) => r,
            Err(_) => return RuleResult::none(),
        };
        if let Some(first) = rel.components().next() {
            let name = first.as_os_str().to_string_lossy();
            for sensitive in crate::config::SENSITIVE_HOME_DIRS {
                if name == *sensitive {
                    return RuleResult::with_level(
                        SafetyLevel::Locked,
                        EvidenceKind::SensitivePath,
                        &item.path.to_string_lossy(),
                        &format!("Sensitive home directory: ~/{sensitive}"),
                    );
                }
            }
        }
        RuleResult::none()
    }
}

/// Rule: private_* folders are LOCKED.
pub struct PrivatePrefixRule;

impl SafetyRule for PrivatePrefixRule {
    fn evaluate(&self, item: &ScanItem, _scan_root: &Path, _home: &Path) -> RuleResult {
        if item.name.to_lowercase().starts_with("private_") {
            return RuleResult::with_level(
                SafetyLevel::Locked,
                EvidenceKind::SensitivePath,
                &item.path.to_string_lossy(),
                "Folder starts with 'private_' — treated as sensitive",
            );
        }
        RuleResult::none()
    }
}

/// Rule: symlink targets are LOCKED.
pub struct SymlinkTargetRule;

impl SafetyRule for SymlinkTargetRule {
    fn evaluate(&self, item: &ScanItem, _scan_root: &Path, _home: &Path) -> RuleResult {
        if item.is_symlink {
            if let Some(ref target) = item.symlink_target {
                return RuleResult::with_level(
                    SafetyLevel::Locked,
                    EvidenceKind::Symlink,
                    &item.path.to_string_lossy(),
                    &format!("Symlink → {}", target.display()),
                );
            }
        }
        RuleResult::none()
    }
}
