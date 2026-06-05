pub mod classifier;
pub mod evidence;
pub mod item;
pub mod risk;
pub mod walker;

use crate::error::Result;
use crate::profile::user_profile::UserProfile;
use crate::reports::ScanReport;
use crate::safety::policy::SafetyPolicy;
use crate::scan::evidence::{Evidence, EvidenceKind};
use crate::scan::risk::{RiskScore, SafetyLevel};
use classifier::Classifier;
use std::collections::HashSet;
use std::path::Path;

pub struct Scanner {
    classifier: Classifier,
    policy: SafetyPolicy,
    /// Rule-file protected paths: treated as additional LOCKED roots.
    protected_paths: Vec<std::path::PathBuf>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            classifier: Classifier::new(),
            policy: SafetyPolicy::new(),
            protected_paths: Vec::new(),
        }
    }

    /// Builder: treat the given paths as LOCKED roots (from a rule file).
    ///
    /// Children of these paths inherit REVIEW classification via the
    /// existing parent-risk inheritance pass. No filesystem changes are made.
    pub fn with_protected_paths(mut self, paths: Vec<std::path::PathBuf>) -> Self {
        self.protected_paths = paths;
        self
    }

    /// Scan a path up to a given depth and return a scan report.
    ///
    /// Items whose name or path substring matches any entry in `exclude` are
    /// skipped entirely — they are counted in `SafetySummary::skipped` and are
    /// never classified, auto-planned, or presented in output.
    pub fn scan(
        &self,
        root: &std::path::PathBuf,
        home: &std::path::PathBuf,
        max_depth: usize,
        exclude: &[String],
    ) -> Result<ScanReport> {
        let all_items = walker::walk(root, max_depth)?;

        let mut skipped = 0usize;
        let items: Vec<_> = all_items
            .into_iter()
            .filter(|item| {
                if is_excluded(item, exclude) {
                    skipped += 1;
                    false
                } else {
                    true
                }
            })
            .collect();

        // First pass: classify each item independently.
        let mut scanned_items: Vec<_> = items
            .into_iter()
            .map(|item| {
                let classification = self.classifier.classify(&item, root, home);
                let policy_decision = self.policy.evaluate(&item, &classification);
                (item, classification, policy_decision)
            })
            .collect();

        // Second pass: parent-risk inheritance.
        // Collect paths of LOCKED directories so children can inherit REVIEW.
        let mut locked_dirs: HashSet<String> = scanned_items
            .iter()
            .filter(|(item, _, decision)| {
                item.is_dir && matches!(decision.level, SafetyLevel::Locked)
            })
            .map(|(item, _, _)| item.path.to_string_lossy().into_owned())
            .collect();

        // Also add rule-file protected paths as LOCKED roots.
        for p in &self.protected_paths {
            locked_dirs.insert(p.to_string_lossy().into_owned());
        }

        scanned_items = scanned_items
            .into_iter()
            .map(|(item, mut cls, decision)| {
                // Check if this item itself is a rule-file protected path → LOCKED.
                let is_rule_protected = self
                    .protected_paths
                    .iter()
                    .any(|p| item.path == *p || item.path.starts_with(p));

                if is_rule_protected && !matches!(decision.level, SafetyLevel::Locked) {
                    cls.evidence.push(Evidence {
                        kind: EvidenceKind::InheritedRisk,
                        path: item.path.to_string_lossy().into_owned(),
                        description: "Protected by rule file".to_string(),
                        note: None,
                    });
                    cls.level = SafetyLevel::Locked;
                    cls.score = RiskScore(1.0);
                    let new_decision = crate::safety::policy::SafetyPolicyDecision {
                        level: SafetyLevel::Locked,
                        overridden: true,
                        reason: Some("Protected by rule file".to_string()),
                    };
                    return (item, cls, new_decision);
                }

                // Only upgrade SafeCandidate items — never downgrade.
                if !matches!(decision.level, SafetyLevel::SafeCandidate) {
                    return (item, cls, decision);
                }

                let in_locked_parent = item
                    .path
                    .ancestors()
                    .skip(1) // skip the item itself
                    .any(|a| locked_dirs.contains(a.to_string_lossy().as_ref()));

                let in_live_site = is_in_live_site_path(&item.path);

                if in_locked_parent || in_live_site {
                    let reason = if in_locked_parent {
                        "Inside a LOCKED parent directory"
                    } else {
                        "Inside a live-site folder (public_html / www / htdocs / …)"
                    };
                    cls.evidence.push(Evidence {
                        kind: EvidenceKind::InheritedRisk,
                        path: item.path.to_string_lossy().into_owned(),
                        description: reason.to_string(),
                        note: None,
                    });
                    cls.level = SafetyLevel::Review;
                    cls.score = RiskScore(cls.score.0.max(0.5));
                    let new_decision = crate::safety::policy::SafetyPolicyDecision {
                        level: SafetyLevel::Review,
                        overridden: true,
                        reason: Some(reason.to_string()),
                    };
                    (item, cls, new_decision)
                } else {
                    (item, cls, decision)
                }
            })
            .collect();

        let profile = UserProfile::infer(&scanned_items);
        let report = ScanReport::build(
            root.to_string_lossy().to_string(),
            scanned_items,
            profile,
            skipped,
        );
        Ok(report)
    }
}

/// Return true if `item` matches any of the exclude patterns.
/// Matches by exact name or path substring (case-insensitive).
fn is_excluded(item: &crate::scan::item::ScanItem, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let path_str = item.path.to_string_lossy().to_lowercase();
    let name_lower = item.name.to_lowercase();
    patterns.iter().any(|p| {
        let p = p.to_lowercase();
        name_lower == p || path_str.contains(p.as_str())
    })
}

/// Return true if any component of `path` matches a live-site folder name.
fn is_in_live_site_path(path: &Path) -> bool {
    path.components().any(|c| {
        if let std::path::Component::Normal(n) = c {
            let name = n.to_string_lossy().to_lowercase();
            crate::config::LIVE_SITE_FOLDER_NAMES
                .iter()
                .any(|s| name == *s)
        } else {
            false
        }
    })
}
