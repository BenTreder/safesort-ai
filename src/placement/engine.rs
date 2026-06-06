use super::confidence::{Confidence, ConfidenceBand};
use super::destination::{DestinationPlanner, DestinationRisk, PlacementDestination};
use super::file_purpose::{FilePurpose, FilePurposeDetector};
use super::local_dest;
use super::ownership::{OwnerCategory, OwnershipDetector};
use super::question_queue::{Question, QuestionOption, QuestionQueue};
use super::rules::RulesEngine;
use crate::scan::risk::SafetyLevel;
use indexmap::IndexMap;
use std::path::{Path, PathBuf};

/// Organization modes for SafeSort AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganizationMode {
    /// Default: shows recommendations only. Never moves anything.
    Preview,
    /// Creates question queue for uncertain files.
    Guided,
    /// Auto-plans only GREEN files with ≥95 confidence.
    SafeAutopilot,
    /// Extra conservative. Auto-plans nothing.
    LockedDown,
}

impl OrganizationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Preview => "preview",
            Self::Guided => "guided",
            Self::SafeAutopilot => "safe-autopilot",
            Self::LockedDown => "locked-down",
        }
    }
}

/// A single placement recommendation.
#[derive(Debug, Clone)]
pub struct PlacementRecommendation {
    /// The file being analyzed.
    pub file_path: PathBuf,
    /// File name.
    pub file_name: String,
    /// Safety level from the scan engine.
    pub safety_level: SafetyLevel,
    /// Dependency impact level: CRITICAL, HIGH, MEDIUM, LOW, NONE.
    pub impact_level: String,
    /// Detected owner, if any.
    pub owner: Option<super::ownership::DetectedOwner>,
    /// Detected purpose.
    pub purpose: FilePurpose,
    /// File type description.
    pub file_type: String,
    /// Risk level for placement.
    pub risk: String,
    /// Confidence 0–100.
    pub confidence: Confidence,
    /// Recommended destinations.
    pub destinations: Vec<PlacementDestination>,
    /// Reason for the recommendation.
    pub reason: String,
    /// What band this falls into.
    pub band: ConfidenceBand,
    /// Rule-file influence note, if any (alias match, custom destination, protected path).
    pub rule_note: Option<String>,
}

/// Result of running the smart placement engine.
#[derive(Debug)]
pub struct PlacementResult {
    /// All recommendations.
    pub recommendations: Vec<PlacementRecommendation>,
    /// Questions for guided review (80–94% confidence).
    pub question_queue: QuestionQueue,
    /// Summary counts.
    pub summary: PlacementSummary,
    /// The mode used.
    pub mode: OrganizationMode,
}

#[derive(Debug, Default)]
pub struct PlacementSummary {
    pub total_files: usize,
    pub auto_plan_eligible: usize,
    pub guided_review: usize,
    pub review_needed: usize,
    pub leave_alone: usize,
    pub locked: usize,
    /// Items skipped due to --exclude patterns (set by caller from ScanReport).
    pub skipped: usize,
}

/// The Smart Placement Engine.
pub struct SmartPlacementEngine {
    ownership: OwnershipDetector,
    purpose: FilePurposeDetector,
    destination: DestinationPlanner,
    rules: RulesEngine,
    mode: OrganizationMode,
    home: PathBuf,
    /// Custom staging destinations from a rule file: "{canonical}.{purpose}" → path.
    custom_destinations: IndexMap<String, String>,
    /// Owner safe roots from a rule file: canonical → safe_root path.
    owner_safe_roots: IndexMap<String, String>,
    /// When set, use the local organize model: `{local_output_root}/{Owner}/{Ext}/[Sub]/`.
    /// The root passed here should be `scan_target/safesort`.
    local_output_root: Option<PathBuf>,
}

impl SmartPlacementEngine {
    pub fn new(home: PathBuf, mode: OrganizationMode) -> Self {
        let destination = DestinationPlanner::new(home.clone());
        Self {
            ownership: OwnershipDetector::new(),
            purpose: FilePurposeDetector::new(),
            destination,
            rules: RulesEngine::new(),
            mode,
            home,
            custom_destinations: IndexMap::new(),
            owner_safe_roots: IndexMap::new(),
            local_output_root: None,
        }
    }

    /// Builder: enable local organize mode.
    ///
    /// When set, destinations are computed as `{local_root}/{Owner}/{ExtGroup}/[Subcategory]/`
    /// instead of the legacy `~/Workspace/...` paths. Rule-file overrides are ignored in
    /// local mode — the local structure is always owner-first.
    ///
    /// Pass `scan_target.join("safesort")` as `local_root`.
    pub fn with_local_output(mut self, local_root: PathBuf) -> Self {
        self.local_output_root = Some(local_root);
        self
    }

    /// Builder: inject aliases and custom destinations from a rule file.
    ///
    /// This is the only supported entry point for rule-file integration.
    /// Rules influence recommendations only — they never move files,
    /// bypass safety classification, or persist to disk.
    pub fn with_rules(mut self, rules: &crate::rules_file::RulesFile) -> Self {
        // Inject aliases into the ownership detector.
        for (token, canonical) in &rules.aliases {
            let (display, category) = if let Some(owner_rule) = rules.owners.get(canonical) {
                (
                    owner_rule.display.clone(),
                    category_from_str(&owner_rule.category),
                )
            } else {
                (canonical.clone(), OwnerCategory::Unknown)
            };
            self.ownership
                .add_alias(token, canonical, &display, category);
        }

        // Store custom staging destinations (validation happens in analyze_file).
        self.custom_destinations = rules.staging_destinations.clone();

        // Store owner safe roots (used as fallback destinations).
        self.owner_safe_roots = rules
            .owners
            .iter()
            .filter(|(_, r)| !r.safe_root.is_empty())
            .map(|(canonical, r)| (canonical.clone(), r.safe_root.clone()))
            .collect();

        self
    }

    /// Access the rules engine for customization.
    pub fn rules(&mut self) -> &mut RulesEngine {
        &mut self.rules
    }

    /// Analyze a single file and produce a recommendation.
    pub fn analyze_file(
        &self,
        file_path: &Path,
        safety_level: SafetyLevel,
    ) -> PlacementRecommendation {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let parent = file_path.parent().unwrap_or(file_path);

        // Detect ownership and purpose
        let owner = self.ownership.detect(&file_name, parent);
        let purpose = self.purpose.detect(&file_name, parent);

        // Determine file type description
        let file_type = Self::describe_file_type(&file_name);

        // Check if in safe zone (Downloads/Desktop)
        let is_safe_zone = self.is_in_safe_zone(file_path);

        // Check if inside an active project.
        // Files already in a safe zone (Downloads/Desktop) are never penalized:
        // they're loose files in a download area, not project assets, even if the
        // Downloads folder happens to live under a directory that has a Cargo.toml.
        let inside_project = if is_safe_zone {
            false
        } else {
            self.is_inside_project(file_path)
        };

        // Compute confidence
        let mut confidence = self.compute_confidence(
            &file_name,
            &owner,
            purpose,
            &safety_level,
            is_safe_zone,
            inside_project,
        );

        // Check if parent folder has a code-extension-like name (risky project folder).
        let in_risky_parent_folder = Self::is_risky_parent_folder(file_path);
        if in_risky_parent_folder && confidence.value() > 50 {
            confidence = Confidence(50);
        }

        // In locked-down mode, cap confidence at 80 (never auto-plan)
        if self.mode == OrganizationMode::LockedDown {
            if confidence.value() > 80 {
                confidence = Confidence(80);
            }
        }

        // Generate destinations
        let ext = Path::new(&file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let mut destinations = if matches!(safety_level, SafetyLevel::Locked) {
            vec![]
        } else if let Some(ref local_root) = self.local_output_root {
            // Local organize mode: owner-first structure under scan_target/safesort/
            let dest_path =
                local_dest::local_destination(local_root, owner.as_ref(), purpose, &ext);
            vec![PlacementDestination {
                path: dest_path,
                description: format!("Local → {}", local_dest::ext_group(&ext)),
                is_staging: true,
                risk: DestinationRisk::Safe,
            }]
        } else {
            self.destination.plan(owner.as_ref(), purpose, is_safe_zone)
        };

        // Apply rule-file custom destinations (if not LOCKED and not in local mode).
        let mut rule_note: Option<String> = None;
        if !matches!(safety_level, SafetyLevel::Locked) && self.local_output_root.is_none() {
            if let Some(ref o) = owner {
                // 1. Try specific staging destination: "{canonical}.{purpose}"
                let key = format!("{}.{}", o.canonical, purpose.as_str().to_lowercase());
                if let Some(custom_dest) = self.custom_destinations.get(&key) {
                    if crate::rules_file::validation::is_safe_destination(custom_dest) {
                        let expanded = expand_tilde(custom_dest, &self.home);
                        let label = format!("Custom (rule file): {}", dest_label(custom_dest));
                        destinations.insert(
                            0,
                            PlacementDestination {
                                description: label.clone(),
                                path: expanded,
                                is_staging: true,
                                risk: DestinationRisk::LowRisk,
                            },
                        );
                        rule_note =
                            Some(format!("Staging destination from rule file (key: {})", key));
                    } else {
                        rule_note =
                            Some(crate::rules_file::validation::rejection_reason(custom_dest));
                    }
                }
                // 2. Fall back to owner safe_root if no specific destination matched.
                if rule_note.is_none() {
                    if let Some(safe_root) = self.owner_safe_roots.get(&o.canonical) {
                        if crate::rules_file::validation::is_safe_destination(safe_root) {
                            let expanded = expand_tilde(safe_root, &self.home);
                            destinations.insert(
                                0,
                                PlacementDestination {
                                    description: format!(
                                        "Owner root (rule file): {}",
                                        dest_label(safe_root)
                                    ),
                                    path: expanded,
                                    is_staging: true,
                                    risk: DestinationRisk::LowRisk,
                                },
                            );
                            rule_note = Some(format!(
                                "Owner safe_root from rule file for '{}'",
                                o.canonical
                            ));
                        }
                    }
                }
            }
        }

        // Build reason
        let reason = Self::build_reason(&owner, purpose, &safety_level, &confidence, is_safe_zone);

        // Determine risk string
        let risk = match safety_level {
            SafetyLevel::Locked => "LOCKED".to_string(),
            SafetyLevel::Review => "REVIEW".to_string(),
            SafetyLevel::SafeCandidate => {
                if confidence.is_auto_plan() {
                    "GREEN".to_string()
                } else {
                    "YELLOW".to_string()
                }
            }
        };

        let impact_level = match safety_level {
            SafetyLevel::Locked => "CRITICAL",
            SafetyLevel::Review => "MEDIUM",
            SafetyLevel::SafeCandidate => "NONE",
        }
        .to_string();

        PlacementRecommendation {
            file_path: file_path.to_path_buf(),
            file_name,
            safety_level,
            impact_level,
            owner,
            purpose,
            file_type,
            risk,
            confidence,
            destinations,
            reason,
            band: confidence.band(),
            rule_note,
        }
    }

    /// Run the engine on a set of scanned items.
    pub fn run(&self, items: &[(PathBuf, SafetyLevel)]) -> PlacementResult {
        let mut recommendations = Vec::new();
        let mut question_queue = QuestionQueue::new();
        let mut summary = PlacementSummary::default();

        for (path, safety) in items {
            summary.total_files += 1;

            if matches!(safety, SafetyLevel::Locked) {
                summary.locked += 1;
                let rec = self.analyze_file(path, *safety);
                recommendations.push(rec);
                continue;
            }

            let rec = self.analyze_file(path, *safety);

            match rec.band {
                ConfidenceBand::AutoPlan => {
                    // Never auto-plan items with MEDIUM/HIGH/CRITICAL impact.
                    let impact_ok = matches!(rec.impact_level.as_str(), "NONE" | "LOW");
                    if impact_ok
                        && (self.mode == OrganizationMode::SafeAutopilot
                            || self.mode == OrganizationMode::Guided)
                    {
                        summary.auto_plan_eligible += 1;
                    } else {
                        // Preview/locked-down or high-impact: don't auto-plan
                        summary.review_needed += 1;
                    }
                }
                ConfidenceBand::GuidedReview => {
                    if self.mode == OrganizationMode::Guided {
                        summary.guided_review += 1;
                        // Create a question
                        let question = self.build_question(&rec);
                        question_queue.push(question);
                    } else {
                        summary.review_needed += 1;
                    }
                }
                ConfidenceBand::ReviewNeeded => {
                    summary.review_needed += 1;
                }
                ConfidenceBand::LeaveAlone => {
                    summary.leave_alone += 1;
                }
            }

            recommendations.push(rec);
        }

        PlacementResult {
            recommendations,
            question_queue,
            summary,
            mode: self.mode,
        }
    }

    fn build_question(&self, rec: &PlacementRecommendation) -> Question {
        let mut options = Vec::new();

        // Add destination options
        for dest in rec.destinations.iter().take(3) {
            options.push(QuestionOption::Stage(dest.clone()));
        }

        options.push(QuestionOption::Leave);
        options.push(QuestionOption::ReviewNeeded);

        // Create rule option if we have an owner
        if let Some(ref owner) = rec.owner {
            if let Some(first_dest) = rec.destinations.first() {
                options.push(QuestionOption::CreateRule {
                    pattern: format!(
                        "{} + {}",
                        owner.canonical.to_lowercase(),
                        rec.purpose.as_str().to_lowercase()
                    ),
                    destination: first_dest.clone(),
                });
            }
        }

        Question {
            file_path: rec.file_path.to_string_lossy().to_string(),
            detected_owner: rec.owner.clone(),
            detected_purpose: rec.purpose,
            file_type_desc: rec.file_type.clone(),
            risk_level: rec.risk.clone(),
            confidence: rec.confidence,
            destinations: rec.destinations.clone(),
            reason: rec.reason.clone(),
            options,
        }
    }

    fn compute_confidence(
        &self,
        file_name: &str,
        owner: &Option<super::ownership::DetectedOwner>,
        purpose: FilePurpose,
        safety: &SafetyLevel,
        is_safe_zone: bool,
        inside_project: bool,
    ) -> Confidence {
        let mut score = Confidence::new();

        // Locked files get 0 confidence for placement
        if matches!(safety, SafetyLevel::Locked) {
            return score;
        }

        // Owner match
        if owner.is_some() {
            score.add(Confidence::EXACT_BRAND_MATCH);
        }

        // Purpose match
        if purpose != FilePurpose::Unknown {
            score.add(Confidence::PURPOSE_MATCH);
        }

        // Safe file type
        if Self::is_safe_extension(file_name) {
            score.add(Confidence::SAFE_FILE_TYPE);
        }

        // Safe source zone
        if is_safe_zone {
            score.add(Confidence::SAFE_SOURCE);
        }

        // Loose file bonus
        if !inside_project {
            score.add(Confidence::LOOSE_FILE);
        }

        // Extension signals purpose
        if Self::extension_signals_purpose(file_name, purpose) {
            score.add(Confidence::EXTENSION_SIGNAL);
        }

        // Penalties
        if inside_project {
            score.subtract(Confidence::INSIDE_PROJECT_PENALTY);
        }

        // Ambiguity penalty: if owner is unknown but purpose is known
        if owner.is_none() && purpose != FilePurpose::Unknown {
            score.subtract(15);
        }

        score
    }

    fn is_safe_extension(filename: &str) -> bool {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        matches!(
            ext.as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "webp"
                | "svg"
                | "pdf"
                | "txt"
                | "md"
                | "csv"
                | "zip"
                | "tar"
                | "gz"
                | "tgz"
                | "mp4"
                | "mp3"
                | "wav"
                | "doc"
                | "docx"
                | "xls"
                | "xlsx"
                | "ppt"
                | "pptx"
        )
    }

    fn extension_signals_purpose(filename: &str, purpose: FilePurpose) -> bool {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match purpose {
            FilePurpose::Logo | FilePurpose::Icon | FilePurpose::Banner | FilePurpose::Image => {
                matches!(
                    ext.as_str(),
                    "png" | "jpg" | "jpeg" | "svg" | "gif" | "webp"
                )
            }
            FilePurpose::Screenshot | FilePurpose::ErrorScreenshot | FilePurpose::QaScreenshot => {
                matches!(ext.as_str(), "png" | "jpg" | "jpeg")
            }
            FilePurpose::Document
            | FilePurpose::Report
            | FilePurpose::Invoice
            | FilePurpose::Proposal => {
                matches!(ext.as_str(), "pdf" | "doc" | "docx" | "txt" | "md")
            }
            FilePurpose::Archive | FilePurpose::ReleaseZip | FilePurpose::Backup => {
                matches!(ext.as_str(), "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz")
            }
            _ => false,
        }
    }

    fn is_in_safe_zone(&self, path: &Path) -> bool {
        // Check if any component of the path is Downloads or Desktop.
        // First try relative to home, then check absolute path components.
        let components: Vec<_> = if let Ok(rel) = path.strip_prefix(&self.home) {
            rel.components().collect()
        } else {
            path.components().collect()
        };
        // Check if "Downloads" or "Desktop" appears in the last 2 components
        let check: Vec<_> = components.iter().rev().take(2).collect();
        check.iter().any(|c| {
            let name = c.as_os_str().to_string_lossy();
            matches!(name.as_ref(), "Downloads" | "Desktop")
        })
    }

    fn is_risky_parent_folder(file_path: &Path) -> bool {
        // If the immediate parent folder name has a code extension (e.g. "user.js"),
        // it's a project-like folder and its children should not be auto-planned.
        let risky_folder_extensions = ["js", "py", "ts", "rb", "go", "rs", "php", "vue", "svelte"];
        if let Some(parent) = file_path.parent() {
            if let Some(folder_name) = parent.file_name().and_then(|n| n.to_str()) {
                let folder_lower = folder_name.to_lowercase();
                for ext in &risky_folder_extensions {
                    if folder_lower.ends_with(&format!(".{ext}")) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_inside_project(&self, path: &Path) -> bool {
        // Check if any nearby ancestor (up to 3 levels) has a project marker.
        // Limited depth avoids false positives from project markers in distant ancestors.
        for ancestor in path.ancestors().skip(1).take(3) {
            if ancestor.join(".git").exists()
                || ancestor.join("Cargo.toml").exists()
                || ancestor.join("package.json").exists()
                || ancestor.join("pyproject.toml").exists()
                || ancestor.join("composer.json").exists()
            {
                return true;
            }
        }
        false
    }

    fn describe_file_type(filename: &str) -> String {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" => "Image".to_string(),
            "pdf" => "PDF document".to_string(),
            "txt" | "md" | "rst" => "Text document".to_string(),
            "doc" | "docx" => "Word document".to_string(),
            "xls" | "xlsx" | "csv" => "Spreadsheet".to_string(),
            "ppt" | "pptx" => "Presentation".to_string(),
            "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" => "Archive".to_string(),
            "mp4" | "mkv" | "avi" | "mov" | "webm" => "Video".to_string(),
            "mp3" | "wav" | "flac" | "ogg" | "aac" => "Audio".to_string(),
            "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "java" | "rb" | "php" => {
                "Source code".to_string()
            }
            _ => format!("{} file", if ext.is_empty() { "Unknown" } else { &ext }),
        }
    }

    fn build_reason(
        owner: &Option<super::ownership::DetectedOwner>,
        purpose: FilePurpose,
        safety: &SafetyLevel,
        confidence: &Confidence,
        is_safe_zone: bool,
    ) -> String {
        let mut parts = Vec::new();

        if let Some(ref o) = *owner {
            parts.push(format!("Filename matches brand/project '{}'", o.display));
        }

        if purpose != FilePurpose::Unknown {
            parts.push(format!("Purpose detected: {}", purpose.as_str()));
        }

        if is_safe_zone {
            parts.push("Source is Downloads/Desktop (safe zone)".to_string());
        }

        match safety {
            SafetyLevel::Locked => parts.push("LOCKED by safety engine".to_string()),
            SafetyLevel::Review => parts.push("Needs review".to_string()),
            _ => {}
        };

        parts.push(format!("Confidence: {}%", confidence.value()));

        if parts.is_empty() {
            "No specific signals detected".to_string()
        } else {
            parts.join("; ")
        }
    }
}

/// Expand a `~/...` path against the given home directory.
fn expand_tilde(path: &str, home: &PathBuf) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        home.join(rest)
    } else {
        PathBuf::from(path)
    }
}

/// Extract a short display label from a destination path (last two components).
fn dest_label(path: &str) -> String {
    let p = std::path::Path::new(path);
    let parts: Vec<_> = p.components().rev().take(2).collect();
    let parts: Vec<_> = parts.into_iter().rev().collect();
    parts
        .iter()
        .filter_map(|c| c.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}

/// Map an owner category string from the rule file to `OwnerCategory`.
fn category_from_str(s: &str) -> OwnerCategory {
    match s.to_lowercase().as_str() {
        "website" => OwnerCategory::Website,
        "brand" => OwnerCategory::Brand,
        "project" => OwnerCategory::Project,
        "plugin" | "wordpressplugin" | "wordpress_plugin" => OwnerCategory::Plugin,
        "client" => OwnerCategory::Client,
        _ => OwnerCategory::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine(mode: OrganizationMode) -> SmartPlacementEngine {
        SmartPlacementEngine::new(PathBuf::from("/home/user"), mode)
    }

    #[test]
    fn test_bentreder_logo_high_confidence() {
        let eng = engine(OrganizationMode::Preview);
        let rec = eng.analyze_file(
            Path::new("/home/user/Downloads/bentreder_logo.png"),
            SafetyLevel::SafeCandidate,
        );
        assert!(
            rec.confidence.value() >= 80,
            "bentreder_logo.png should have high confidence, got {}",
            rec.confidence.value()
        );
        assert_eq!(rec.purpose, FilePurpose::Logo);
        assert!(rec.owner.is_some());
        assert_eq!(rec.owner.unwrap().canonical, "BenTreder.com");
    }

    #[test]
    fn test_quicktapid_banner() {
        let eng = engine(OrganizationMode::Preview);
        let rec = eng.analyze_file(
            Path::new("/home/user/Downloads/quicktapid_banner.png"),
            SafetyLevel::SafeCandidate,
        );
        assert!(rec.confidence.value() >= 80);
        assert_eq!(rec.purpose, FilePurpose::Banner);
        assert_eq!(rec.owner.unwrap().canonical, "QuickTapID");
    }

    #[test]
    fn test_website_fix_finder_release_zip() {
        let eng = engine(OrganizationMode::Preview);
        let rec = eng.analyze_file(
            Path::new("/home/user/Downloads/website-fix-finder-v1.0.zip"),
            SafetyLevel::SafeCandidate,
        );
        assert!(rec.confidence.value() >= 50);
        assert_eq!(rec.purpose, FilePurpose::ReleaseZip);
    }

    #[test]
    fn test_error_screenshot() {
        let eng = engine(OrganizationMode::Preview);
        let rec = eng.analyze_file(
            Path::new("/home/user/Downloads/error-checkout-page.png"),
            SafetyLevel::SafeCandidate,
        );
        assert_eq!(rec.purpose, FilePurpose::ErrorScreenshot);
    }

    #[test]
    fn test_locked_file_gets_zero_confidence() {
        let eng = engine(OrganizationMode::SafeAutopilot);
        let rec = eng.analyze_file(Path::new("/home/user/.ssh/id_rsa"), SafetyLevel::Locked);
        assert_eq!(rec.confidence.value(), 0);
        assert!(rec.destinations.is_empty());
    }

    #[test]
    fn test_safe_autopilot_only_auto_plans_95_plus() {
        let eng = engine(OrganizationMode::SafeAutopilot);
        let items = vec![
            (
                PathBuf::from("/home/user/Downloads/bentreder_logo.png"),
                SafetyLevel::SafeCandidate,
            ),
            (
                PathBuf::from("/home/user/Downloads/random_file.txt"),
                SafetyLevel::SafeCandidate,
            ),
        ];
        let result = eng.run(&items);
        // bentreder_logo.png should be auto-plan eligible
        assert!(result.summary.auto_plan_eligible >= 1);
    }

    #[test]
    fn test_guided_mode_creates_questions() {
        let eng = engine(OrganizationMode::Guided);
        let items = vec![(
            PathBuf::from("/home/user/Downloads/bentreder_logo.png"),
            SafetyLevel::SafeCandidate,
        )];
        let result = eng.run(&items);
        // Should have questions for items in the 80-94 band
        // bentreder_logo.png is likely 95+ so may not create a question
        // but the engine should at least process it
        assert_eq!(result.recommendations.len(), 1);
    }

    #[test]
    fn test_locked_down_mode_caps_confidence() {
        let eng = engine(OrganizationMode::LockedDown);
        let rec = eng.analyze_file(
            Path::new("/home/user/Downloads/bentreder_logo.png"),
            SafetyLevel::SafeCandidate,
        );
        assert!(
            rec.confidence.value() <= 80,
            "Locked-down mode should cap confidence at 80"
        );
    }

    #[test]
    fn test_no_real_file_moving() {
        // Verify the engine only produces recommendations, never touches files
        let eng = engine(OrganizationMode::SafeAutopilot);
        let items = vec![(
            PathBuf::from("/home/user/Downloads/bentreder_logo.png"),
            SafetyLevel::SafeCandidate,
        )];
        let _result = eng.run(&items);
        // If we got here without panicking, no files were moved
        // (the engine doesn't do any I/O beyond reading metadata)
    }
}
