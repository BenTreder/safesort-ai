use serde::Serialize;

/// A piece of evidence collected by a detector.
#[derive(Debug, Clone, Serialize)]
pub struct Evidence {
    /// Human-readable evidence kind.
    pub kind: EvidenceKind,
    /// The path this evidence was found at or refers to.
    pub path: String,
    /// Description of what was found.
    pub description: String,
    /// Optional note (e.g., "skipped — permission denied").
    pub note: Option<String>,
}

/// Categories of evidence that affect safety classification.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub enum EvidenceKind {
    /// A system-critical path (/etc, /usr, /var, …).
    SystemCritical,

    /// A sensitive credential / key / config directory.
    SensitivePath,

    /// A sensitive file found (.env, id_rsa, …).
    SensitiveFile,

    /// A project marker found (.git, Cargo.toml, …).
    ProjectMarker,

    /// A symlink was encountered.
    Symlink,
    /// A symlink target was encountered.
    SymlinkTarget,

    /// An absolute path reference in a script / config file.
    ScriptPathRef,

    /// A systemd unit references this path.
    SystemdReference,

    /// A cron entry references this path.
    CronReference,

    /// The item is loose in a safe zone (Download, Desktop).
    SafeZoneLoose,

    /// The item is an archive file.
    ArchiveFile,

    /// The item is a media/image file.
    MediaFile,

    /// The item is a document.
    DocumentFile,

    /// The item is a backup folder or dated backup.
    BackupFolder,

    /// The item is a likely website folder.
    WebsiteFolder,

    /// The item has mixed unknown contents.
    MixedContents,

    /// Permission denied while scanning this path.
    PermissionSkipped,

    /// The item contains shell scripts.
    ContainsScripts,

    /// The item contains PHP files.
    ContainsPhp,

    /// Contains Dockerfile / docker-compose.
    ContainsDockerfile,

    /// Contains Python project files.
    ContainsPython,

    /// Contains Node.js project files.
    ContainsNodeJs,

    /// Contains Rust project files.
    ContainsRust,

    /// Contains wordpress files.
    ContainsWordPress,
}
