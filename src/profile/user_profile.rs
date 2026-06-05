use super::signals::{self, UserProfileType};
use crate::safety::policy::SafetyPolicyDecision;
use crate::scan::classifier::Classification;
use crate::scan::item::ScanItem;
use serde::Serialize;
use std::collections::HashMap;

/// A scored user profile with signal breakdown.
#[derive(Debug, Clone, Serialize)]
pub struct UserProfile {
    /// Profile type → score.
    pub scores: HashMap<String, ProfileScore>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileScore {
    pub score: f64,
    pub confidence: String,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl UserProfile {
    pub fn new() -> Self {
        let mut scores = HashMap::new();
        for profile in &[
            UserProfileType::Developer,
            UserProfileType::WordPressBuilder,
            UserProfileType::WebsiteOwner,
            UserProfileType::AiPowerUser,
            UserProfileType::SeoContentCreator,
            UserProfileType::ClientServiceFreelancer,
            UserProfileType::DesignerMediaCreator,
            UserProfileType::BusinessOwner,
            UserProfileType::DataReportsUser,
            UserProfileType::GeneralUser,
        ] {
            scores.insert(
                profile.display_name().to_string(),
                ProfileScore {
                    score: 0.0,
                    confidence: "—".to_string(),
                },
            );
        }
        Self { scores }
    }

    /// Infer user profile from scanned items and their classifications.
    pub fn infer(items: &[(ScanItem, Classification, SafetyPolicyDecision)]) -> Self {
        let mut profile = Self::new();
        let signal_weights = signals::all_signals();

        for (_item, classification, _policy) in items {
            for evidence in &classification.evidence {
                for signal in &signal_weights {
                    if std::mem::discriminant(&evidence.kind)
                        == std::mem::discriminant(&signal.evidence)
                    {
                        let entry = profile
                            .scores
                            .entry(signal.profile.display_name().to_string())
                            .or_insert_with(|| ProfileScore {
                                score: 0.0,
                                confidence: "—".to_string(),
                            });
                        entry.score += signal.weight;
                    }
                }
            }
        }

        // Update confidence strings.
        for (_, score) in profile.scores.iter_mut() {
            score.confidence = UserProfileType::confidence(score.score).to_string();
        }

        // Always set GeneralUser as baseline.
        if let Some(gu) = profile.scores.get_mut("General User") {
            gu.score = gu.score.max(1.0);
            gu.confidence = "baseline".to_string();
        }

        profile
    }

    /// Get scored profiles sorted by score descending.
    pub fn sorted_scores(&self) -> Vec<(&String, &ProfileScore)> {
        let mut v: Vec<_> = self.scores.iter().collect();
        v.sort_by(|a, b| {
            b.1.score
                .partial_cmp(&a.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    }
}
