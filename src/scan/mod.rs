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
        }
    }

    /// Scan a path up to a given depth and return a scan report.
    pub fn scan(
        &self,
        root: &std::path::PathBuf,
        home: &std::path::PathBuf,
        max_depth: usize,
    ) -> Result<ScanReport> {
        let items = walker::walk(root, max_depth)?;

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
        let locked_dirs: HashSet<String> = scanned_items
            .iter()
            .filter(|(item, _, decision)| {
                item.is_dir && matches!(decision.level, SafetyLevel::Locked)
            })
            .map(|(item, _, _)| item.path.to_string_lossy().into_owned())
            .collect();

        scanned_items = scanned_items
            .into_iter()
            .map(|(item, mut cls, decision)| {
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
        let report = ScanReport::build(root.to_string_lossy().to_string(), scanned_items, profile);
        Ok(report)
    }
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
