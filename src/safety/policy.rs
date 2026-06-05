use crate::scan::classifier::Classification;
use crate::scan::evidence::EvidenceKind;
use crate::scan::item::ScanItem;
use crate::scan::risk::SafetyLevel;

/// The final decision from the safety policy for a scanned item.
#[derive(Debug, Clone)]
pub struct SafetyPolicyDecision {
    pub level: SafetyLevel,
    pub overridden: bool,
    pub reason: Option<String>,
}

/// The top-level safety policy that aggregates all detectors.
pub struct SafetyPolicy {
    system_critical: crate::detectors::sensitive::SensitivePathDetector,
}

impl Default for SafetyPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl SafetyPolicy {
    pub fn new() -> Self {
        Self {
            system_critical: crate::detectors::sensitive::SensitivePathDetector::new(),
        }
    }

    /// Evaluate the safety policy for a fully classified item.
    /// This can override a classification if a higher-priority rule fires.
    pub fn evaluate(
        &self,
        item: &ScanItem,
        classification: &Classification,
    ) -> SafetyPolicyDecision {
        // System-critical paths are always LOCKED.
        let path_str = item.path.to_string_lossy();
        for prefix in crate::config::LOCKED_SYSTEM_PATHS {
            if path_str.starts_with(prefix) {
                return SafetyPolicyDecision {
                    level: SafetyLevel::Locked,
                    overridden: true,
                    reason: Some(format!("System-critical path: starts with {prefix}")),
                };
            }
        }

        // Sensitive home directories are always LOCKED.
        if self.system_critical.is_sensitive_home_prefix(&item.path) {
            return SafetyPolicyDecision {
                level: SafetyLevel::Locked,
                overridden: true,
                reason: Some("Sensitive home directory".to_string()),
            };
        }

        // items containing .env / secrets are LOCKED
        if classification
            .evidence
            .iter()
            .any(|e| matches!(e.kind, EvidenceKind::SensitiveFile))
        {
            return SafetyPolicyDecision {
                level: SafetyLevel::Locked,
                overridden: true,
                reason: Some("Sensitive file detected (.env, key, token, …)".to_string()),
            };
        }

        // symlink targets are LOCKED
        if classification
            .evidence
            .iter()
            .any(|e| matches!(e.kind, EvidenceKind::SymlinkTarget))
        {
            return SafetyPolicyDecision {
                level: SafetyLevel::Locked,
                overridden: true,
                reason: Some("Symlink target".to_string()),
            };
        }

        // systemd / cron referenced paths are LOCKED
        if classification.evidence.iter().any(|e| {
            matches!(
                e.kind,
                EvidenceKind::SystemdReference | EvidenceKind::CronReference
            )
        }) {
            return SafetyPolicyDecision {
                level: SafetyLevel::Locked,
                overridden: true,
                reason: Some("Referenced by systemd or cron".to_string()),
            };
        }

        // `private_*` folders are LOCKED
        if item.name.to_lowercase().starts_with("private_") {
            return SafetyPolicyDecision {
                level: SafetyLevel::Locked,
                overridden: true,
                reason: Some("Folder starts with 'private_'".to_string()),
            };
        }

        SafetyPolicyDecision {
            level: classification.level,
            overridden: false,
            reason: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::evidence::Evidence;
    use crate::scan::risk::RiskScore;

    fn fake_item(name: &str) -> ScanItem {
        ScanItem {
            path: std::path::PathBuf::from("/tmp/test").join(name),
            name: name.to_string(),
            is_dir: true,
            is_symlink: false,
            symlink_target: None,
            extension: None,
            depth: 1,
            is_hidden: false,
        }
    }

    #[test]
    fn test_env_is_locked() {
        let item = fake_item("my-project");
        let classification = Classification {
            level: SafetyLevel::Review,
            score: RiskScore(0.5),
            evidence: vec![Evidence {
                kind: EvidenceKind::SensitiveFile,
                path: item.path.to_string_lossy().to_string(),
                description: ".env file found".into(),
                note: None,
            }],
        };
        let policy = SafetyPolicy::new();
        let decision = policy.evaluate(&item, &classification);
        assert_eq!(decision.level, SafetyLevel::Locked);
        assert!(decision.overridden);
    }

    #[test]
    fn test_systemd_ref_is_locked() {
        let item = fake_item("my-app");
        let classification = Classification {
            level: SafetyLevel::Review,
            score: RiskScore(0.5),
            evidence: vec![Evidence {
                kind: EvidenceKind::SystemdReference,
                path: item.path.to_string_lossy().to_string(),
                description: "referenced by systemd".into(),
                note: None,
            }],
        };
        let policy = SafetyPolicy::new();
        let decision = policy.evaluate(&item, &classification);
        assert_eq!(decision.level, SafetyLevel::Locked);
    }

    #[test]
    fn test_safe_candidate_passes_through() {
        let item = fake_item("some-folder");
        let classification = Classification {
            level: SafetyLevel::SafeCandidate,
            score: RiskScore(0.1),
            evidence: vec![],
        };
        let policy = SafetyPolicy::new();
        let decision = policy.evaluate(&item, &classification);
        assert_eq!(decision.level, SafetyLevel::SafeCandidate);
        assert!(!decision.overridden);
    }
}
