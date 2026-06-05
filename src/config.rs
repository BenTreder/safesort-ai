use std::path::PathBuf;

/// Known system paths that are always protected.
pub const LOCKED_SYSTEM_PATHS: &[&str] = &[
    "/etc", "/usr", "/var", "/opt", "/boot", "/srv", "/run", "/proc", "/sys", "/dev",
];

/// Known sensitive home directories.
pub const SENSITIVE_HOME_DIRS: &[&str] = &[
    ".ssh",
    ".gnupg",
    ".aws",
    ".config",
    ".kube",
    ".docker",
    ".local/share",
    ".password-store",
];

/// Known sensitive file markers.
pub const SENSITIVE_FILE_MARKERS: &[&str] = &[".env", "id_rsa", "id_ed25519", ".npmrc", ".pypirc"];

/// Project marker files that indicate REVIEW classification.
pub const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "Cargo.toml",
    "package.json",
    "composer.json",
    "pyproject.toml",
    "requirements.txt",
    "wp-config.php",
    "docker-compose.yml",
    "Dockerfile",
    "Makefile",
    "node_modules",
    "vendor",
    "target",
    "venv",
    ".venv",
];

/// Archive extensions considered safe candidates when loose.
pub const ARCHIVE_EXTENSIONS: &[&str] = &[
    ".zip", ".tar", ".tar.gz", ".tgz", ".tar.bz2", ".tar.xz", ".tar.zst", ".bak", ".old",
];

/// Media extensions.
pub const MEDIA_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp", ".tiff", ".ico", ".heic", ".mp3",
    ".wav", ".flac", ".ogg", ".aac", ".opus", ".mp4", ".mkv", ".avi", ".mov", ".webm", ".flv",
];

/// Document extensions.
pub const DOCUMENT_EXTENSIONS: &[&str] = &[
    ".pdf", ".doc", ".docx", ".odt", ".rtf", ".tex", ".xls", ".xlsx", ".ods", ".csv", ".ppt",
    ".pptx", ".odp", ".txt", ".md", ".rst", ".epub",
];

/// Folders considered "safe loose zones" for classification.
pub const SAFE_LOOSE_ZONES: &[&str] = &["Downloads", "Desktop"];

/// Backup folder patterns.
pub const BACKUP_PATTERNS: &[&str] = &["backup", "backups", "bak", "old", "archive", "archives"];

/// Live-site folder names whose contents should never be SAFE_CANDIDATE.
pub const LIVE_SITE_FOLDER_NAMES: &[&str] = &[
    "public_html",
    "www",
    "htdocs",
    "webroot",
    "live-site",
    "live_site",
];

/// Systemd unit directories.
pub const SYSTEMD_PATHS: &[&str] = &[
    "/etc/systemd/system",
    "/usr/lib/systemd/system",
    "/lib/systemd/system",
];

/// Cron directories.
pub const CRON_PATHS: &[&str] = &[
    "/etc/crontab",
    "/etc/cron.d",
    "/etc/cron.daily",
    "/etc/cron.hourly",
    "/etc/cron.weekly",
    "/etc/cron.monthly",
];

/// Prefix the user's home directory to a relative sensitive-path name.
pub fn sensitive_home_path(home: &std::path::Path, name: &str) -> PathBuf {
    home.join(name)
}
