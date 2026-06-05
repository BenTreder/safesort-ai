pub mod json;
pub mod markdown;
pub mod terminal;

use crate::safety::policy::SafetyPolicyDecision;
use crate::scan::classifier::Classification;
use crate::scan::item::ScanItem;
use crate::scan::risk::SafetyLevel;
use indexmap::IndexMap;
use serde::Serialize;

/// A complete scan report.
#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub scan_target: String,
    pub generated_at: String,
    /// Count by safety level.
    pub summary: SafetySummary,
    /// Per-item results, grouped by safety level.
    pub items: IndexMap<String, Vec<ItemResult>>,
    /// Detected profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<crate::profile::user_profile::UserProfile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SafetySummary {
    pub locked: usize,
    pub review: usize,
    pub safe_candidate: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemResult {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub safety_level: String,
    pub score: f64,
    pub reasons: Vec<String>,
}

impl ScanReport {
    pub fn build(
        scan_target: String,
        items: Vec<(ScanItem, Classification, SafetyPolicyDecision)>,
        profile: crate::profile::user_profile::UserProfile,
    ) -> Self {
        let mut locked = 0usize;
        let mut review = 0usize;
        let mut safe = 0usize;

        let mut grouped: IndexMap<String, Vec<ItemResult>> = IndexMap::new();
        grouped.insert("LOCKED".to_string(), Vec::new());
        grouped.insert("REVIEW".to_string(), Vec::new());
        grouped.insert("SAFE".to_string(), Vec::new());

        for (item, classification, decision) in items {
            let level = decision.level;
            let result = ItemResult {
                path: item.path.to_string_lossy().to_string(),
                name: item.name.clone(),
                is_dir: item.is_dir,
                safety_level: level.as_str().to_string(),
                score: classification.score.0,
                reasons: classification
                    .evidence
                    .iter()
                    .map(|e| e.description.clone())
                    .collect(),
            };

            let key = level.as_str();
            grouped.entry(key.to_string()).or_default().push(result);

            match level {
                SafetyLevel::Locked => locked += 1,
                SafetyLevel::Review => review += 1,
                SafetyLevel::SafeCandidate => safe += 1,
            }
        }

        Self {
            scan_target,
            generated_at: chrono::Local::now().to_rfc3339(),
            summary: SafetySummary {
                locked,
                review,
                safe_candidate: safe,
                total: locked + review + safe,
            },
            items: grouped,
            profile: Some(profile),
        }
    }

    pub fn get_examples(&self, level: &str, max: usize) -> Vec<&ItemResult> {
        self.items
            .get(level)
            .map(|v| v.iter().take(max).collect())
            .unwrap_or_default()
    }
}
