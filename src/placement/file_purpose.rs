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
    Soq,
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
    // Print / marketing collateral
    PrintInsert,
    NfcInsert,
    Mailer,
    Flyer,
    Postcard,
    StickerSheet,
    Sticker,
    SalesSheet,
    // Career
    JobApplication,
    Resume,
    CoverLetter,
    // Books / content
    BookInterior,
    BookManuscript,
    BookCover,
    BookKindle,
    BookPrint,
    // Sensitive documents
    SensitiveDocument,
    // Client-specific asset types
    Label,
    ComplianceLabel,
    OnboardingDoc,
    ProductList,
    // Media specializations
    CannabisImage,
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
            Self::Soq => "Statement of Qualifications",
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
            Self::PrintInsert => "Print Insert",
            Self::NfcInsert => "NFC Insert",
            Self::Mailer => "Mailer",
            Self::Flyer => "Flyer",
            Self::Postcard => "Postcard",
            Self::StickerSheet => "Sticker Sheet",
            Self::Sticker => "Sticker",
            Self::SalesSheet => "Sales Sheet",
            Self::JobApplication => "Job Application",
            Self::Resume => "Resume",
            Self::CoverLetter => "Cover Letter",
            Self::BookInterior => "Book Interior",
            Self::BookManuscript => "Book Manuscript",
            Self::BookCover => "Book Cover",
            Self::BookKindle => "Book Kindle File",
            Self::BookPrint => "Book Print File",
            Self::SensitiveDocument => "Sensitive Document",
            Self::Label => "Label",
            Self::ComplianceLabel => "Compliance Label",
            Self::OnboardingDoc => "Onboarding Document",
            Self::ProductList => "Product List",
            Self::CannabisImage => "Cannabis / Product Image",
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
            Self::Proposal | Self::Soq => "Proposals",
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
            Self::PrintInsert | Self::NfcInsert => "Inserts",
            Self::Mailer => "Mailers",
            Self::Flyer => "Flyers",
            Self::Postcard => "Postcards",
            Self::StickerSheet | Self::Sticker => "Stickers",
            Self::SalesSheet => "Sales Sheets",
            Self::JobApplication => "Job Applications",
            Self::Resume => "Resumes",
            Self::CoverLetter => "Cover Letters",
            Self::BookInterior | Self::BookManuscript => "Book Drafts",
            Self::BookCover => "Covers",
            Self::BookKindle => "Kindle",
            Self::BookPrint => "Print Files",
            Self::SensitiveDocument => "Sensitive Documents",
            Self::Label => "Labels",
            Self::ComplianceLabel => "Compliance Labels",
            Self::OnboardingDoc => "Onboarding",
            Self::ProductList => "Product Lists",
            Self::CannabisImage => "Cannabis",
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

        // Check purpose tokens in priority order (most specific first)
        let purpose_tokens: &[(FilePurpose, &[&str])] = &[
            // Career documents — checked before generic Document
            (
                FilePurpose::JobApplication,
                &["jobapplication", "jobapp", "application"],
            ),
            (
                FilePurpose::Resume,
                &["resume", "cv", "curriculum", "vitae"],
            ),
            (
                FilePurpose::CoverLetter,
                &["coverletter", "coverltr", "applicationletter"],
            ),
            // SOQ / proposals — before generic Proposal/Report
            (FilePurpose::Soq, &["soq", "qualifications", "ita"]),
            // Book content — before generic Document
            (
                FilePurpose::BookInterior,
                &["interior", "interior6x9", "interior8x11"],
            ),
            (FilePurpose::BookManuscript, &["manuscript", "draft"]),
            // NFC insert — before generic PrintInsert
            (FilePurpose::NfcInsert, &["nfc"]),
            // Print collateral
            (FilePurpose::PrintInsert, &["insert", "inserts"]),
            (FilePurpose::Mailer, &["mailer", "mailers"]),
            (FilePurpose::Flyer, &["flyer", "flyers"]),
            (
                FilePurpose::Postcard,
                &["postcard", "postcards", "4x6", "5x7"],
            ),
            (
                FilePurpose::StickerSheet,
                &["stickersheet", "sticker_sheet"],
            ),
            (FilePurpose::Sticker, &["sticker", "stickers"]),
            (
                FilePurpose::SalesSheet,
                &["salessheet", "sales", "onesheet", "one-sheet"],
            ),
            // Cannabis / product images — before generic Image
            (
                FilePurpose::CannabisImage,
                &["weed", "cannabis", "hemp", "dispensary", "thc", "cbd"],
            ),
            // Standard purposes
            (FilePurpose::Logo, &["logo", "logotype"]),
            (FilePurpose::Icon, &["icon", "icons"]),
            (FilePurpose::Favicon, &["favicon"]),
            (FilePurpose::Banner, &["banner", "hero", "header"]),
            (FilePurpose::Cover, &["thumbnail", "thumb"]),
            (
                FilePurpose::ErrorScreenshot,
                &["error", "bug", "issue", "fail", "failed"],
            ),
            (FilePurpose::QaScreenshot, &["qa", "test", "testing"]),
            (FilePurpose::SocialProof, &["proof", "testimonial"]),
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
            (FilePurpose::Audit, &["audit", "assessment"]),
            (FilePurpose::Backup, &["backup", "bak"]),
            (FilePurpose::ReleaseZip, &["release", "dist"]),
            (FilePurpose::PluginAsset, &["asset"]),
            (FilePurpose::WebsiteAsset, &["web"]),
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

        let filename_lower = filename.to_lowercase();

        // Early detection: Sensitive documents (before all other checks)
        let sensitive_keywords = [
            "creditreport",
            "credit_report",
            "credit-report",
            "boir",
            "cp_575",
            "cp575",
            "irs_",
            "taxreturn",
            "tax_return",
            "tax-return",
            "wageclaim",
            "wage_claim",
            "wage-claim",
            "backupcodes",
            "backup_codes",
            "backup-codes",
            "recoverycodes",
            "recovery_codes",
            "password_backup",
            "passwordbackup",
            "password_export",
            "governmentfiling",
            "government_filing",
            "legalfiling",
            "legal_filing",
            "businessentity",
            "business_entity",
            "mtd_bank",
            "bankstatement",
            "bank_statement",
            "accountstatement",
            "account_statement",
            "claim_doc",
            "claimdoc",
        ];
        for kw in &sensitive_keywords {
            if filename_lower.contains(kw) {
                return FilePurpose::SensitiveDocument;
            }
        }

        // Early detection: Book Kindle / epub / mobi (before cover check)
        if filename_lower.contains("kindle") || ext == "epub" || tokens.iter().any(|t| t == "mobi")
        {
            return FilePurpose::BookKindle;
        }

        // Early detection: Book Print files
        if (filename_lower.contains("print_ready") || filename_lower.contains("printready"))
            || (filename_lower.contains("print")
                && ext == "pdf"
                && (filename_lower.contains("final") || filename_lower.contains("ready")))
        {
            return FilePurpose::BookPrint;
        }

        // Early detection: Book Cover (image/pdf with "cover" but NOT a cover letter)
        if filename_lower.contains("cover")
            && !filename_lower.contains("cover_letter")
            && !filename_lower.contains("coverletter")
            && !filename_lower.contains("applicationletter")
        {
            if is_image_ext(&ext) || ext == "pdf" || ext == "epub" {
                return FilePurpose::BookCover;
            }
        }

        // KDP manuscript: "kdp" token in filename → BookManuscript (before generic Document)
        if tokens.iter().any(|t| t == "kdp") {
            return FilePurpose::BookManuscript;
        }

        // Printer-friendly PDFs → PrintInsert (routes to Brand Assets/{owner}/Print Assets/Inserts)
        if filename_lower.contains("printer_friendly") || filename_lower.contains("printerfriendly")
        {
            return FilePurpose::PrintInsert;
        }

        // Compliance labels — before generic Label check
        if filename_lower.contains("compliance")
            && (filename_lower.contains("label") || filename_lower.contains("labels"))
        {
            return FilePurpose::ComplianceLabel;
        }

        // Label sheets — after compliance check, before generic purposes
        if tokens.iter().any(|t| *t == "label" || *t == "labels") {
            return FilePurpose::Label;
        }

        // Onboarding documents
        if tokens.iter().any(|t| *t == "onboarding") {
            return FilePurpose::OnboardingDoc;
        }

        // Product lists
        if filename_lower.contains("product_list")
            || filename_lower.contains("productlist")
            || (tokens.iter().any(|t| *t == "product") && tokens.iter().any(|t| *t == "list"))
        {
            return FilePurpose::ProductList;
        }

        // Also check filename for sticker_sheet pattern (contains underscore)
        if filename_lower.contains("sticker_sheet")
            || filename_lower.contains("sticker-sheet")
            || (filename_lower.contains("sticker") && filename_lower.contains("sheet"))
        {
            return FilePurpose::StickerSheet;
        }
        // Check for book interior patterns like "interior-6x9" or "82p"
        if (filename_lower.contains("interior")
            && (filename_lower.contains("6x9")
                || filename_lower.contains("8x11")
                || filename_lower.contains("5x8")))
            || (filename_lower.contains("black-and-white") && filename_lower.contains("p-"))
            || filename_lower.contains("82p")
        {
            return FilePurpose::BookInterior;
        }

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
            | "bat"
            | "cmd"
            | "ps1"
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
