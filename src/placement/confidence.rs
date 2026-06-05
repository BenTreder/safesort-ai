/// Confidence score for a placement recommendation (0–100).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Confidence(pub u8);

impl Confidence {
    /// Exact project/brand token match.
    pub const EXACT_BRAND_MATCH: u8 = 40;
    /// Purpose token match (logo, banner, screenshot, etc.).
    pub const PURPOSE_MATCH: u8 = 25;
    /// Safe file type for the purpose.
    pub const SAFE_FILE_TYPE: u8 = 10;
    /// Source folder is Downloads/Desktop.
    pub const SAFE_SOURCE: u8 = 10;
    /// Matching known project/brand folder exists.
    pub const KNOWN_PROJECT: u8 = 10;
    /// File is loose (not inside a known project).
    pub const LOOSE_FILE: u8 = 5;
    /// Extension strongly signals purpose (e.g. .png for logo).
    pub const EXTENSION_SIGNAL: u8 = 5;

    /// Penalty: ambiguous multiple project matches.
    pub const AMBIGUITY_PENALTY: u8 = 30;
    /// Penalty: file is inside a project directory.
    pub const INSIDE_PROJECT_PENALTY: u8 = 40;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn add(&mut self, points: u8) {
        self.0 = self.0.saturating_add(points).min(100);
    }

    pub fn subtract(&mut self, points: u8) {
        self.0 = self.0.saturating_sub(points);
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    /// Is this high enough for safe-autopilot? (≥95)
    pub fn is_auto_plan(&self) -> bool {
        self.0 >= 95
    }

    /// Is this in the guided review band? (80–94)
    pub fn is_guided_review(&self) -> bool {
        self.0 >= 80 && self.0 < 95
    }

    /// Is this in the manual review band? (50–79)
    pub fn is_review_needed(&self) -> bool {
        self.0 >= 50 && self.0 < 80
    }

    /// Is this too low to recommend? (<50)
    pub fn is_leave_alone(&self) -> bool {
        self.0 < 50
    }

    pub fn band(&self) -> ConfidenceBand {
        if self.is_auto_plan() {
            ConfidenceBand::AutoPlan
        } else if self.is_guided_review() {
            ConfidenceBand::GuidedReview
        } else if self.is_review_needed() {
            ConfidenceBand::ReviewNeeded
        } else {
            ConfidenceBand::LeaveAlone
        }
    }
}

impl Default for Confidence {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceBand {
    /// 95–100: auto-plan allowed in safe-autopilot mode.
    AutoPlan,
    /// 80–94: needs guided review question.
    GuidedReview,
    /// 50–79: needs human review.
    ReviewNeeded,
    /// 0–49: leave alone.
    LeaveAlone,
}

impl ConfidenceBand {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AutoPlan => "AUTO-PLAN",
            Self::GuidedReview => "GUIDED REVIEW",
            Self::ReviewNeeded => "REVIEW NEEDED",
            Self::LeaveAlone => "LEAVE ALONE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_bands() {
        assert!(Confidence(96).is_auto_plan());
        assert!(!Confidence(94).is_auto_plan());
        assert!(Confidence(94).is_guided_review());
        assert!(Confidence(80).is_guided_review());
        assert!(!Confidence(79).is_guided_review());
        assert!(Confidence(79).is_review_needed());
        assert!(Confidence(50).is_review_needed());
        assert!(!Confidence(49).is_review_needed());
        assert!(Confidence(49).is_leave_alone());
        assert!(Confidence(0).is_leave_alone());
    }

    #[test]
    fn test_confidence_arithmetic() {
        let mut c = Confidence::new();
        c.add(Confidence::EXACT_BRAND_MATCH);
        assert_eq!(c.value(), 40);
        c.add(Confidence::PURPOSE_MATCH);
        assert_eq!(c.value(), 65);
        c.add(Confidence::SAFE_FILE_TYPE);
        c.add(Confidence::SAFE_SOURCE);
        c.add(Confidence::KNOWN_PROJECT);
        assert_eq!(c.value(), 95);
        assert!(c.is_auto_plan());

        let mut c2 = Confidence(100);
        c2.add(10); // saturates at 100
        assert_eq!(c2.value(), 100);

        let mut c3 = Confidence(20);
        c3.subtract(50); // saturating to 0
        assert_eq!(c3.value(), 0);
    }
}
