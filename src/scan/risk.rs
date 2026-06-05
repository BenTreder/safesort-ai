use serde::Serialize;

/// The safety classification assigned to a scanned item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum SafetyLevel {
    /// Safe to recommend for organizational moves (if the user approves).
    SafeCandidate,
    /// Needs human review before any decision.
    Review,
    /// Never move. Protected by safety engine.
    Locked,
}

impl SafetyLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SafetyLevel::SafeCandidate => "SAFE",
            SafetyLevel::Review => "REVIEW",
            SafetyLevel::Locked => "LOCKED",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            SafetyLevel::SafeCandidate => "✅",
            SafetyLevel::Review => "⚠️ ",
            SafetyLevel::Locked => "🔒",
        }
    }
}

/// Risk score from 0.0 (no risk) to 1.0 (maximum risk).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct RiskScore(pub f64);

impl Default for RiskScore {
    fn default() -> Self {
        Self(0.0)
    }
}

impl RiskScore {
    /// Combine two risk scores (takes the max).
    pub fn combine(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    /// Lift to at least the given floor.
    pub fn at_least(self, floor: f64) -> Self {
        Self(self.0.max(floor))
    }

    /// Map a score to a SafetyLevel.
    pub fn to_level(&self) -> SafetyLevel {
        if self.0 >= 0.8 {
            SafetyLevel::Locked
        } else if self.0 >= 0.4 {
            SafetyLevel::Review
        } else {
            SafetyLevel::SafeCandidate
        }
    }
}
