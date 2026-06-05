pub mod json;
pub mod markdown;
pub mod terminal;

use crate::safety::policy::SafetyPolicyDecision;
use crate::scan::classifier::Classification;
use crate::scan::evidence::EvidenceKind;
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
    /// Items skipped due to --exclude patterns.
    pub skipped: usize,
    /// Impact level counts across all scanned items.
    pub impact_critical: usize,
    pub impact_high: usize,
    pub impact_medium: usize,
    pub impact_low: usize,
    pub impact_none: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemResult {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub safety_level: String,
    /// Dependency impact level: CRITICAL, HIGH, MEDIUM, LOW, NONE.
    pub impact_level: String,
    pub score: f64,
    pub reasons: Vec<String>,
}

/// Derive an impact level string from a classification's evidence.
///
/// Priority order: CRITICAL > HIGH > MEDIUM > LOW > NONE.
pub fn impact_from_evidence(classification: &Classification) -> &'static str {
    let mut best: u8 = 0; // 0=NONE 1=LOW 2=MED 3=HIGH 4=CRIT

    for ev in &classification.evidence {
        let rank: u8 = match ev.kind {
            EvidenceKind::SystemCritical
            | EvidenceKind::SensitivePath
            | EvidenceKind::SensitiveFile
            | EvidenceKind::SystemdReference
            | EvidenceKind::CronReference
            | EvidenceKind::WebsiteFolder => 4,

            EvidenceKind::Symlink
            | EvidenceKind::SymlinkTarget
            | EvidenceKind::ScriptPathRef
            | EvidenceKind::InheritedRisk => 3,

            EvidenceKind::ProjectMarker
            | EvidenceKind::ContainsRust
            | EvidenceKind::ContainsNodeJs
            | EvidenceKind::ContainsPython
            | EvidenceKind::ContainsDockerfile
            | EvidenceKind::ContainsPhp
            | EvidenceKind::ContainsWordPress
            | EvidenceKind::ContainsScripts => 2,

            EvidenceKind::SafeZoneLoose
            | EvidenceKind::ArchiveFile
            | EvidenceKind::MediaFile
            | EvidenceKind::DocumentFile
            | EvidenceKind::BackupFolder => 1,

            _ => 0,
        };
        if rank > best {
            best = rank;
        }
    }

    match best {
        4 => "CRITICAL",
        3 => "HIGH",
        2 => "MEDIUM",
        1 => "LOW",
        _ => "NONE",
    }
}

impl ScanReport {
    pub fn build(
        scan_target: String,
        items: Vec<(ScanItem, Classification, SafetyPolicyDecision)>,
        profile: crate::profile::user_profile::UserProfile,
        skipped: usize,
    ) -> Self {
        let mut locked = 0usize;
        let mut review = 0usize;
        let mut safe = 0usize;
        let mut ic = 0usize;
        let mut ih = 0usize;
        let mut im = 0usize;
        let mut il = 0usize;
        let mut in_ = 0usize;

        let mut grouped: IndexMap<String, Vec<ItemResult>> = IndexMap::new();
        grouped.insert("LOCKED".to_string(), Vec::new());
        grouped.insert("REVIEW".to_string(), Vec::new());
        grouped.insert("SAFE".to_string(), Vec::new());

        for (item, classification, decision) in items {
            let level = decision.level;
            let impact = impact_from_evidence(&classification);

            let result = ItemResult {
                path: item.path.to_string_lossy().to_string(),
                name: item.name.clone(),
                is_dir: item.is_dir,
                safety_level: level.as_str().to_string(),
                impact_level: impact.to_string(),
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

            match impact {
                "CRITICAL" => ic += 1,
                "HIGH" => ih += 1,
                "MEDIUM" => im += 1,
                "LOW" => il += 1,
                _ => in_ += 1,
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
                skipped,
                impact_critical: ic,
                impact_high: ih,
                impact_medium: im,
                impact_low: il,
                impact_none: in_,
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
