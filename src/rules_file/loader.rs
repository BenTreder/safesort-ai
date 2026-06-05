use super::schema::RulesFile;
use crate::error::{Result, SafeSortError};
use std::path::Path;

/// Load and parse a SafeSort AI rule file from disk.
///
/// Only loads when explicitly requested — never auto-loads from home directory.
/// Returns a clear error if the file is missing or contains invalid TOML.
/// Never creates, writes, or modifies any file.
pub fn load(path: &Path) -> Result<RulesFile> {
    if !path.exists() {
        return Err(SafeSortError::InvalidPath(format!(
            "Rule file not found: {} — check the path and try again",
            path.display()
        )));
    }
    if !path.is_file() {
        return Err(SafeSortError::InvalidPath(format!(
            "Rule file path is not a file: {}",
            path.display()
        )));
    }
    let content = std::fs::read_to_string(path).map_err(|e| {
        SafeSortError::InvalidPath(format!("Cannot read rule file '{}': {e}", path.display()))
    })?;
    toml::from_str::<RulesFile>(&content).map_err(|e| {
        SafeSortError::InvalidPath(format!(
            "Invalid TOML in rule file '{}': {e}",
            path.display()
        ))
    })
}
