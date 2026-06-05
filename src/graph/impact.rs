//! Impact analysis levels for dependency graph.

use serde::Serialize;

/// Levels of impact for considering a move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
pub enum ImpactLevel {
    /// No impact — safe to move.
    None,
    /// Low impact — minimal risk, easy rollback.
    Low,
    /// Medium impact — some dependencies, review recommended.
    Medium,
    /// High impact — multiple dependencies, careful review needed.
    High,
    /// Critical impact — would break services, scripts, or other critical paths.
    Critical,
}

impl ImpactLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImpactLevel::None => "NONE",
            ImpactLevel::Low => "LOW",
            ImpactLevel::Medium => "MEDIUM",
            ImpactLevel::High => "HIGH",
            ImpactLevel::Critical => "CRITICAL",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            ImpactLevel::None => "✅",
            ImpactLevel::Low => "🟢",
            ImpactLevel::Medium => "⚠️ ",
            ImpactLevel::High => "🟠",
            ImpactLevel::Critical => "🔴",
        }
    }

    /// Convert a risk score to an impact level (for basic heuristic).
    pub fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            ImpactLevel::Critical
        } else if score >= 0.6 {
            ImpactLevel::High
        } else if score >= 0.4 {
            ImpactLevel::Medium
        } else if score >= 0.2 {
            ImpactLevel::Low
        } else {
            ImpactLevel::None
        }
    }
}

/// Impact analysis result for a single path.
#[derive(Debug, Clone, Serialize)]
pub struct ImpactAnalysis {
    /// The path being analyzed.
    pub path: String,
    /// The impact level.
    pub level: ImpactLevel,
    /// List of dependencies that would be affected.
    pub dependencies: Vec<Dependency>,
}

/// A dependency that would be affected by moving a path.
#[derive(Debug, Clone, Serialize)]
pub struct Dependency {
    /// The dependent path or entity.
    pub dependent: String,
    /// The kind of dependency.
    pub kind: crate::graph::EdgeKind,
    /// Why this is a dependency.
    pub reason: String,
}

impl ImpactAnalysis {
    pub fn none(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            level: ImpactLevel::None,
            dependencies: vec![],
        }
    }

    pub fn critical(
        path: impl Into<String>,
        dependent: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            level: ImpactLevel::Critical,
            dependencies: vec![Dependency {
                dependent: dependent.into(),
                kind: crate::graph::EdgeKind::MayBreakIfMoved,
                reason: reason.into(),
            }],
        }
    }

    /// Check if there are any dependencies.
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }
}
