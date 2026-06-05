use crate::placement::ownership::tokenize;
use std::path::Path;

/// The detected purpose of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilePurpose {
    Logo,
    Icon,
    Favicon,
    Banner,
    Cover,
    Screenshot,
    ErrorScreenshot,
    QaScreenshot,
    SocialProof,
    Report,
    Audit,
    Invoice,
    Receipt,
    Proposal,
    Contract,
    Backup,
    ReleaseZip,
    PluginAsset,
    WebsiteAsset,
    SocialPost,
    Document,
    Installer,
    Archive,
    Image,
    Video,
    Audio,
    Spreadsheet,
    Presentation,
    Code,
    Unknown,
}

impl FilePurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Logo => "Logo",
            Self::Icon => "Icon",
            Self::Favicon => "Favicon",
            Self::Banner => "Banner",
            Self::Cover => "Cover",
            Self::Screenshot => "Screenshot",
            Self::ErrorScreenshot => "Error Screenshot",
            Self::QaScreenshot => "QA Screenshot",
            Self::SocialProof => "Social Proof",
            Self::Report => "Report",
            Self::Audit => "Audit",
            Self::Invoice => "Invoice",
            Self::Receipt => "Receipt",
            Self::Proposal => "Proposal",
            Self::Contract => "Contract",
            Self::Backup => "Backup",
            Self::ReleaseZip => "Release Zip",
            Self::PluginAsset => "Plugin Asset",
            Self::WebsiteAsset => "Website Asset",
            Self::SocialPost => "Social Post",
            Self::Document => "Document",
            Self::Installer => "Installer",
            Self::Archive => "Archive",
            Self::Image => "Image",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Spreadsheet => "Spreadsheet",
            Self::Presentation => "Presentation",
            Self::Code => "Code",
            Self::Unknown => "Unknown",
        }
    }

    /// The default folder name for this purpose within a destination subtree.
    pub fn default_folder(&self) -> &'static str {
        match self {
            Self::Logo | Self::Icon | Self::Favicon => "Logos",
            Self::Banner | Self::Cover => "Banners",
            Self::Screenshot => "Screenshots",
            Self::ErrorScreenshot => "Errors",
            Self::QaScreenshot => "QA",
            Self::SocialProof => "Social Proof",
            Self::Report | Self::Audit => "Reports",
            Self::Invoice | Self::Receipt => "Receipts",
            Self::Proposal => "Proposals",
            Self::Contract => "Contracts",
            Self::Backup => "Backups",
            Self::ReleaseZip => "Release Zips",
            Self::PluginAsset | Self::WebsiteAsset => "Assets",
            Self::SocialPost => "Social Posts",
            Self::Document => "Documents",
            Self::Installer => "Installers",
            Self::Archive => "Archives",
            Self::Image => "Images",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Spreadsheet => "Spreadsheets",
            Self::Presentation => "Presentations",
            Self::Code => "Code",
            Self::Unknown => "Unsorted",
        }
    }
}

/// Detects the purpose of a file from its name, extension, and path.
pub struct FilePurposeDetector;

impl FilePurposeDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(&self, filename: &str, _parent_path: &Path) -> FilePurpose {
        let tokens = tokenize(filename);
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check purpose tokens in priority order
        let purpose_tokens: &[(FilePurpose, &[&str])] = &[
            (FilePurpose::Logo, &["logo", "logotype"]),
            (FilePurpose::Icon, &["icon", "icons"]),
            (FilePurpose::Favicon, &["favicon"]),
            (FilePurpose::Banner, &["banner", "hero", "header"]),
            (FilePurpose::Cover, &["cover", "thumbnail", "thumb"]),
            (
                FilePurpose::ErrorScreenshot,
                &["error", "bug", "issue", "fail", "failed"],
            ),
            (FilePurpose::QaScreenshot, &["qa", "test", "testing"]),
            (
                FilePurpose::SocialProof,
                &["proof", "testimonial", "review"],
            ),
            (
                FilePurpose::Screenshot,
                &["screenshot", "screen", "capture", "snap"],
            ),
            (FilePurpose::Invoice, &["invoice", "bill", "billing"]),
            (FilePurpose::Receipt, &["receipt", "receipts"]),
            (FilePurpose::Proposal, &["proposal", "pitch", "bid"]),
            (
                FilePurpose::Contract,
                &["contract", "agreement", "terms", "sla"],
            ),
            (
                FilePurpose::Report,
                &["report", "summary", "findings", "analysis"],
            ),
            (FilePurpose::Audit, &["audit", "review", "assessment"]),
            (FilePurpose::Backup, &["backup", "bak"]),
            (FilePurpose::ReleaseZip, &["release", "dist"]),
            (FilePurpose::PluginAsset, &["asset"]),
            (FilePurpose::WebsiteAsset, &["asset", "web"]),
            (
                FilePurpose::SocialPost,
                &[
                    "social",
                    "post",
                    "tweet",
                    "linkedin",
                    "facebook",
                    "instagram",
                ],
            ),
            (FilePurpose::Installer, &["setup", "install", "installer"]),
        ];

        for (purpose, keywords) in purpose_tokens {
            for token in &tokens {
                if keywords.contains(&token.as_str()) {
                    return *purpose;
                }
            }
        }

        // File type based classification
        if is_archive_ext(&ext) {
            if tokens
                .iter()
                .any(|t| t == "release" || t == "dist" || t.starts_with("v"))
            {
                return FilePurpose::ReleaseZip;
            }
            if tokens
                .iter()
                .any(|t| t.contains("backup") || t.contains("bak"))
            {
                return FilePurpose::Backup;
            }
            return FilePurpose::Archive;
        }

        if is_document_ext(&ext) {
            return FilePurpose::Document;
        }

        if is_image_ext(&ext) {
            return FilePurpose::Image;
        }

        if is_video_ext(&ext) {
            return FilePurpose::Video;
        }

        if is_audio_ext(&ext) {
            return FilePurpose::Audio;
        }

        if is_spreadsheet_ext(&ext) {
            return FilePurpose::Spreadsheet;
        }

        if is_presentation_ext(&ext) {
            return FilePurpose::Presentation;
        }

        if is_code_ext(&ext) {
            return FilePurpose::Code;
        }

        FilePurpose::Unknown
    }
}

impl Default for FilePurposeDetector {
    fn default() -> Self {
        Self::new()
    }
}

fn is_image_ext(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "tiff" | "ico" | "heic" | "avif"
    )
}

fn is_video_ext(ext: &str) -> bool {
    matches!(ext, "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv")
}

fn is_audio_ext(ext: &str) -> bool {
    matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "aac" | "opus" | "m4a")
}

fn is_document_ext(ext: &str) -> bool {
    matches!(
        ext,
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "tex" | "txt" | "md" | "rst" | "epub"
    )
}

fn is_spreadsheet_ext(ext: &str) -> bool {
    matches!(ext, "xls" | "xlsx" | "ods" | "csv")
}

fn is_presentation_ext(ext: &str) -> bool {
    matches!(ext, "ppt" | "pptx" | "odp" | "key")
}

fn is_archive_ext(ext: &str) -> bool {
    matches!(
        ext,
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "zst" | "7z" | "rar"
    )
}

fn is_code_ext(ext: &str) -> bool {
    matches!(
        ext,
        "rs" | "py"
            | "js"
            | "ts"
            | "go"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "java"
            | "rb"
            | "php"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "pl"
            | "lua"
            | "swift"
            | "kt"
            | "scala"
            | "html"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "xml"
            | "sql"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logo_detection() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("bentreder_logo.png", Path::new("/tmp")),
            FilePurpose::Logo
        );
        assert_eq!(
            d.detect("mybrand-logo.svg", Path::new("/tmp")),
            FilePurpose::Logo
        );
        assert_eq!(
            d.detect("logo_main.png", Path::new("/tmp")),
            FilePurpose::Logo
        );
    }

    #[test]
    fn test_banner_detection() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("quicktapid_banner.png", Path::new("/tmp")),
            FilePurpose::Banner
        );
        assert_eq!(
            d.detect("homepage-hero.jpg", Path::new("/tmp")),
            FilePurpose::Banner
        );
    }

    #[test]
    fn test_screenshot_detection() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("screenshot-2026-06-04.png", Path::new("/tmp")),
            FilePurpose::Screenshot
        );
    }

    #[test]
    fn test_error_screenshot_detection() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("error-checkout-page.png", Path::new("/tmp")),
            FilePurpose::ErrorScreenshot
        );
        assert_eq!(
            d.detect("bug-mobile-view.jpg", Path::new("/tmp")),
            FilePurpose::ErrorScreenshot
        );
    }

    #[test]
    fn test_invoice_detection() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("invoice-client-2026.pdf", Path::new("/tmp")),
            FilePurpose::Invoice
        );
    }

    #[test]
    fn test_report_detection() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("seo-report-may.pdf", Path::new("/tmp")),
            FilePurpose::Report
        );
    }

    #[test]
    fn test_archive_detection() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("project-archive.zip", Path::new("/tmp")),
            FilePurpose::Archive
        );
        assert_eq!(
            d.detect("backup-2025.tar.gz", Path::new("/tmp")),
            FilePurpose::Backup
        );
        assert_eq!(
            d.detect("release-v1.0.zip", Path::new("/tmp")),
            FilePurpose::ReleaseZip
        );
    }

    #[test]
    fn test_document_detection() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("roadmap.pdf", Path::new("/tmp")),
            FilePurpose::Document
        );
        assert_eq!(
            d.detect("notes.txt", Path::new("/tmp")),
            FilePurpose::Document
        );
    }

    #[test]
    fn test_spreadsheet_detection() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("export.csv", Path::new("/tmp")),
            FilePurpose::Spreadsheet
        );
    }

    #[test]
    fn test_unknown_purpose() {
        let d = FilePurposeDetector::new();
        assert_eq!(
            d.detect("xyz123.foo", Path::new("/tmp")),
            FilePurpose::Unknown
        );
    }

    #[test]
    fn test_default_folders() {
        assert_eq!(FilePurpose::Logo.default_folder(), "Logos");
        assert_eq!(FilePurpose::Banner.default_folder(), "Banners");
        assert_eq!(FilePurpose::ErrorScreenshot.default_folder(), "Errors");
        assert_eq!(FilePurpose::Report.default_folder(), "Reports");
        assert_eq!(FilePurpose::Invoice.default_folder(), "Receipts");
        assert_eq!(FilePurpose::Backup.default_folder(), "Backups");
        assert_eq!(FilePurpose::ReleaseZip.default_folder(), "Release Zips");
    }
}
