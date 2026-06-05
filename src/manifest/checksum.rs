use crate::error::{Result, SafeSortError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// SHA-256 checksum and metadata for a single file.
///
/// Computed before any planned operation so a future apply step can verify
/// the file has not changed between planning and execution.
///
/// This struct is data-only. Computing it never moves, modifies, or deletes
/// any file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileChecksum {
    /// SHA-256 hex digest of the file contents at scan time.
    pub sha256: String,
    /// File size in bytes.
    pub file_size: u64,
    /// ISO-8601 last-modified timestamp, if the OS provides it.
    pub modified_at: Option<String>,
}

/// Compute a SHA-256 checksum for a file.
///
/// Returns `Err` if the path is a directory, does not exist, or cannot be read.
/// Never modifies or creates any file.
pub fn checksum_file(path: &Path) -> Result<FileChecksum> {
    if path.is_dir() {
        return Err(SafeSortError::InvalidPath(format!(
            "Cannot checksum a directory: {}",
            path.display()
        )));
    }

    let metadata = std::fs::metadata(path).map_err(|e| {
        SafeSortError::InvalidPath(format!("Cannot read metadata for {}: {e}", path.display()))
    })?;

    let file_size = metadata.len();

    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            // Convert UNIX timestamp to a basic ISO-8601 string via chrono.
            let secs = d.as_secs() as i64;
            chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| format!("unix:{secs}"))
        });

    let content = std::fs::read(path).map_err(|e| {
        SafeSortError::InvalidPath(format!("Cannot read file {}: {e}", path.display()))
    })?;

    let hash = Sha256::digest(&content);
    let sha256 = hash.iter().map(|b| format!("{b:02x}")).collect();

    Ok(FileChecksum {
        sha256,
        file_size,
        modified_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn checksum_produces_64_char_hex() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello safesort").unwrap();
        let cs = checksum_file(f.path()).unwrap();
        assert_eq!(cs.sha256.len(), 64, "SHA-256 hex must be 64 characters");
        assert!(
            cs.sha256.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA-256 must be lowercase hex"
        );
    }

    #[test]
    fn checksum_is_deterministic() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"deterministic test data").unwrap();
        let cs1 = checksum_file(f.path()).unwrap();
        let cs2 = checksum_file(f.path()).unwrap();
        assert_eq!(
            cs1.sha256, cs2.sha256,
            "Same file must produce same checksum"
        );
    }

    #[test]
    fn checksum_differs_for_different_content() {
        let mut a = NamedTempFile::new().unwrap();
        let mut b = NamedTempFile::new().unwrap();
        a.write_all(b"file a content").unwrap();
        b.write_all(b"file b content - different").unwrap();
        let cs_a = checksum_file(a.path()).unwrap();
        let cs_b = checksum_file(b.path()).unwrap();
        assert_ne!(
            cs_a.sha256, cs_b.sha256,
            "Different files must have different checksums"
        );
    }

    #[test]
    fn checksum_known_value() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let f = NamedTempFile::new().unwrap();
        // tempfile creates empty file by default
        let cs = checksum_file(f.path()).unwrap();
        assert_eq!(
            cs.sha256, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "Empty file must match known SHA-256"
        );
        assert_eq!(cs.file_size, 0);
    }

    #[test]
    fn checksum_reports_file_size() {
        let mut f = NamedTempFile::new().unwrap();
        let content = b"exactly twenty bytes";
        f.write_all(content).unwrap();
        let cs = checksum_file(f.path()).unwrap();
        assert_eq!(cs.file_size, content.len() as u64);
    }

    #[test]
    fn checksum_rejects_directory() {
        let dir = tempfile::tempdir().unwrap();
        let result = checksum_file(dir.path());
        assert!(result.is_err(), "Checksumming a directory must fail");
    }

    #[test]
    fn checksum_does_not_modify_file() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"do not touch this content").unwrap();
        let content_before = std::fs::read(f.path()).unwrap();
        let _ = checksum_file(f.path()).unwrap();
        let content_after = std::fs::read(f.path()).unwrap();
        assert_eq!(
            content_before, content_after,
            "checksum_file must not modify the file"
        );
    }
}
