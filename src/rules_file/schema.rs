use indexmap::IndexMap;
use serde::Deserialize;

/// Top-level structure of a SafeSort AI rule file (TOML).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RulesFile {
    /// Token → canonical owner name. Used to improve ownership detection.
    #[serde(default)]
    pub aliases: IndexMap<String, String>,

    /// Canonical owner name → owner metadata.
    #[serde(default)]
    pub owners: IndexMap<String, OwnerRule>,

    /// Paths that should be treated as protected (LOCKED / never SAFE).
    #[serde(default)]
    pub protected_paths: ProtectedPaths,

    /// Staging destination overrides: "{canonical}.{purpose}" → destination path.
    #[serde(default)]
    pub staging_destinations: IndexMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProtectedPaths {
    #[serde(default)]
    pub paths: Vec<String>,
}

/// Metadata for a known owner/brand/project.
#[derive(Debug, Clone, Deserialize)]
pub struct OwnerRule {
    /// Human-readable display name.
    pub display: String,
    /// Category: Website, Brand, Project, Plugin, WordPressPlugin, etc.
    pub category: String,
    /// Safe staging root for this owner (tilde-expanded at runtime).
    #[serde(default)]
    pub safe_root: String,
}
