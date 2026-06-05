//! Edge types for the dependency graph.

use serde::Serialize;

/// Types of edges representing relationships between nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum EdgeKind {
    /// A script or config references this path.
    References,
    /// A service executes this path.
    Executes,
    /// A service or script uses this directory as working directory.
    UsesWorkingDirectory,
    /// A service or script uses this file for environment variables.
    UsesEnvFile,
    /// A symlink points to this target.
    SymlinksTo,
    /// A directory contains this project marker file.
    ContainsProjectMarker,
    /// A directory contains this secret marker file.
    ContainsSecretMarker,
    /// Moving this node would break the target.
    MayBreakIfMoved,
}

impl EdgeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeKind::References => "references",
            EdgeKind::Executes => "executes",
            EdgeKind::UsesWorkingDirectory => "uses working directory",
            EdgeKind::UsesEnvFile => "uses env file",
            EdgeKind::SymlinksTo => "symlinks to",
            EdgeKind::ContainsProjectMarker => "contains project marker",
            EdgeKind::ContainsSecretMarker => "contains secret marker",
            EdgeKind::MayBreakIfMoved => "may break if moved",
        }
    }
}

/// An edge representing a relationship between two nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Edge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// The kind of relationship.
    pub kind: EdgeKind,
    /// Optional description for display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Edge {
    pub fn new(from: impl Into<String>, to: impl Into<String>, kind: EdgeKind) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            kind,
            description: None,
        }
    }

    pub fn with_description(
        from: impl Into<String>,
        to: impl Into<String>,
        kind: EdgeKind,
        description: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            kind,
            description: Some(description.into()),
        }
    }

    /// Create a REFERENCES edge from a script to a path.
    pub fn references(script: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(script, target, EdgeKind::References)
    }

    /// Create an EXECUTES edge from a service to a binary.
    pub fn executes(service: impl Into<String>, binary: impl Into<String>) -> Self {
        Self::new(service, binary, EdgeKind::Executes)
    }

    /// Create a SYMLINKS_TO edge from a link to its target.
    pub fn symlinks_to(link: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(link, target, EdgeKind::SymlinksTo)
    }

    /// Create a MAY_BREAK_IF_MOVED edge (high impact).
    pub fn may_break_if_moved(
        from: impl Into<String>,
        to: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::with_description(from, to, EdgeKind::MayBreakIfMoved, reason)
    }
}
