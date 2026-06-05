pub mod classifier;
pub mod evidence;
pub mod item;
pub mod risk;
pub mod walker;

use crate::error::Result;
use crate::profile::user_profile::UserProfile;
use crate::reports::ScanReport;
use crate::safety::policy::SafetyPolicy;
use classifier::Classifier;

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

        let scanned_items: Vec<_> = items
            .into_iter()
            .map(|item| {
                let classification = self.classifier.classify(&item, root, home);
                let policy_decision = self.policy.evaluate(&item, &classification);
                (item, classification, policy_decision)
            })
            .collect();

        let profile = UserProfile::infer(&scanned_items);

        let report = ScanReport::build(root.to_string_lossy().to_string(), scanned_items, profile);
        Ok(report)
    }
}
