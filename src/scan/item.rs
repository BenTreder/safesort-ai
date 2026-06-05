use serde::Serialize;
use std::path::PathBuf;

/// A single filesystem entry discovered during scanning.
#[derive(Debug, Clone, Serialize)]
pub struct ScanItem {
    /// Absolute path on disk.
    pub path: PathBuf,
    /// File name or directory name.
    pub name: String,
    /// True if the entry is a directory.
    pub is_dir: bool,
    /// True if the entry is a symlink.
    pub is_symlink: bool,
    /// True if the entry is a symlink and its target exists.
    pub symlink_target: Option<PathBuf>,
    /// File extension (lowercase), if any.
    pub extension: Option<String>,
    /// Depth relative to the scan root.
    pub depth: usize,
    /// Whether the entry is hidden (starts with `.`).
    pub is_hidden: bool,
}

impl ScanItem {
    pub fn from_entry(entry: &walkdir::DirEntry, root_depth: usize) -> Self {
        let path = entry.path().to_path_buf();
        let name = entry.file_name().to_str().unwrap_or("").to_string();
        let is_dir = entry.file_type().is_dir();
        let is_symlink = entry.file_type().is_symlink();
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        let depth = entry.depth().saturating_sub(root_depth);
        let is_hidden = name.starts_with('.');

        let symlink_target = if is_symlink {
            std::fs::read_link(&path).ok()
        } else {
            None
        };

        Self {
            path,
            name,
            is_dir,
            is_symlink,
            symlink_target,
            extension,
            depth,
            is_hidden,
        }
    }
}
