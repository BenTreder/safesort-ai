/// Local organize model: owner/category-first destination planner.
///
/// Produces paths of the form:
///   `{safesort_root}/{Owner}/{ExtGroup}/[Subcategory]/`
///
/// where `safesort_root` is `current_dir/safesort`.
use crate::placement::file_purpose::FilePurpose;
use crate::placement::ownership::DetectedOwner;
use std::path::{Path, PathBuf};

// ─── Extension group labels ─────────────────────────────────────────

/// Map a file extension (without dot, lowercase) to a display group name.
pub fn ext_group(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "pdf" => "PDFs",
        "png" => "PNGs",
        "jpg" | "jpeg" => "JPGs",
        "webp" => "WEBPs",
        "gif" => "GIFs",
        "svg" => "SVGs",
        "bmp" | "ico" => "Images",
        "zip" => "ZIPs",
        "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" => "Archives",
        "mp3" => "MP3s",
        "wav" => "WAV",
        "flac" | "ogg" | "aac" => "Audio",
        "mp4" | "mov" | "mkv" | "avi" | "webm" => "MP4s",
        "docx" | "doc" => "DOCX",
        "epub" => "EPUB",
        "txt" => "TXT",
        "csv" => "CSV",
        "xlsx" | "xls" => "Spreadsheets",
        "pptx" | "ppt" => "Presentations",
        "html" | "htm" => "HTML",
        _ => "Other",
    }
}

// ─── Owner folder name sanitization ────────────────────────────────

/// Strip common domain suffixes from a canonical owner name.
fn strip_domain_suffix(s: &str) -> &str {
    let s_lower = s.to_lowercase();
    for suffix in &[".com", ".net", ".org", ".io", ".co", ".ai", ".dev", ".us"] {
        if s_lower.ends_with(suffix) {
            return &s[..s.len() - suffix.len()];
        }
    }
    s
}

/// Convert an owner canonical name to a safe, readable folder name.
///
/// Rules:
/// - Strip domain suffixes (.com, .net, …)
/// - Split on spaces, hyphens, underscores, dots
/// - Capitalize the first letter of each word
/// - Preserve digits
/// - No spaces or special characters in output
///
/// Examples:
/// - "BenTreder.com"  → "BenTreder"
/// - "Ladybug Honey"  → "LadybugHoney"
/// - "Big Win Jerky"  → "BigWinJerky"
/// - "916 Hookup"     → "916Hookup"
/// - "QuickTapID"     → "QuickTapID"
pub fn clean_owner_folder_name(canonical: &str) -> String {
    let without_suffix = strip_domain_suffix(canonical);
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in without_suffix.chars() {
        if ch.is_alphanumeric() {
            if capitalize_next && ch.is_alphabetic() {
                result.extend(ch.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
                capitalize_next = false;
            }
        } else {
            // Delimiter: space, hyphen, underscore, dot → next letter is capitalized
            capitalize_next = true;
        }
    }

    if result.is_empty() {
        "Other".to_string()
    } else {
        result
    }
}

// ─── Subcategory labels ─────────────────────────────────────────────

/// Optional subcategory folder for a given purpose.
/// Returns None if no subcategory is needed.
/// May contain a "/" to indicate nested subcategories (e.g., "Labels/Compliance").
pub fn subcategory_for(purpose: FilePurpose) -> Option<&'static str> {
    match purpose {
        FilePurpose::Logo | FilePurpose::Icon | FilePurpose::Favicon => Some("Logos"),
        FilePurpose::Banner | FilePurpose::Cover => Some("Banners"),
        FilePurpose::Screenshot | FilePurpose::ErrorScreenshot | FilePurpose::QaScreenshot => {
            Some("Screenshots")
        }
        FilePurpose::SocialProof | FilePurpose::SocialPost => Some("Social"),
        FilePurpose::NfcInsert => Some("NFC Inserts"),
        FilePurpose::PrintInsert => Some("Inserts"),
        FilePurpose::Mailer => Some("Mailers"),
        FilePurpose::Sticker | FilePurpose::StickerSheet => Some("Stickers"),
        FilePurpose::Flyer => Some("Flyers"),
        FilePurpose::SalesSheet => Some("Sales Sheets"),
        FilePurpose::Postcard => Some("Postcards"),
        FilePurpose::BookCover => Some("Covers"),
        FilePurpose::BookInterior => Some("Interior Drafts"),
        FilePurpose::BookKindle => Some("Kindle"),
        FilePurpose::BookPrint => Some("Print Files"),
        FilePurpose::BookManuscript => Some("Manuscripts"),
        FilePurpose::Label => Some("Labels"),
        FilePurpose::ComplianceLabel => Some("Labels/Compliance"),
        FilePurpose::Resume => Some("Resumes"),
        FilePurpose::JobApplication => Some("Job Applications"),
        FilePurpose::CoverLetter => Some("Cover Letters"),
        FilePurpose::Soq => Some("SOQ"),
        FilePurpose::Invoice => Some("Invoices"),
        FilePurpose::Proposal | FilePurpose::Contract => Some("Proposals"),
        FilePurpose::Image | FilePurpose::CannabisImage => Some("Product Images"),
        FilePurpose::OnboardingDoc => Some("Onboarding"),
        FilePurpose::ProductList => Some("Product Lists"),
        FilePurpose::Audit => Some("Audits"),
        _ => None,
    }
}

// ─── Main destination function ──────────────────────────────────────

/// Determine the local destination path for a file.
///
/// Returns `safesort_root/{TopLevel}/{ExtGroup}/[Subcategory]/`.
///
/// The `safesort_root` is `scan_target/safesort`.
pub fn local_destination(
    safesort_root: &Path,
    owner: Option<&DetectedOwner>,
    purpose: FilePurpose,
    extension: &str,
) -> PathBuf {
    let ext = ext_group(extension);

    // Purposes that always use a fixed top-level category regardless of owner
    match purpose {
        FilePurpose::SensitiveDocument => {
            return safesort_root.join("SensitiveDocuments").join(ext);
        }
        FilePurpose::Audio => {
            return safesort_root.join("Audio").join(ext);
        }
        FilePurpose::Video => {
            return safesort_root.join("Video").join(ext);
        }
        FilePurpose::Receipt => {
            return safesort_root.join("Receipts").join(ext);
        }
        FilePurpose::Installer => {
            return safesort_root.join("Apps").join(ext);
        }
        FilePurpose::Code | FilePurpose::Unknown => {
            // Route to Other/Review Needed — not moved in any auto/assisted mode
            return safesort_root.join("Other").join("Review Needed");
        }
        _ => {}
    }

    // Determine top-level folder
    let top_level: String = match owner {
        Some(o) => clean_owner_folder_name(&o.canonical),
        None => match purpose {
            FilePurpose::Report | FilePurpose::Audit | FilePurpose::Document => {
                "Reports".to_string()
            }
            FilePurpose::Invoice => "Receipts".to_string(),
            FilePurpose::ReleaseZip | FilePurpose::PluginAsset => "Plugins".to_string(),
            FilePurpose::Backup | FilePurpose::Archive => "Other".to_string(),
            _ => "Other".to_string(),
        },
    };

    // Build path: safesort_root / top_level / ext / [subcategory parts]
    let mut path = safesort_root.join(&top_level).join(ext);
    if let Some(sub) = subcategory_for(purpose) {
        for part in sub.split('/') {
            path = path.join(part);
        }
    }
    path
}

// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::placement::ownership::{DetectedOwner, OwnerCategory};

    fn root() -> PathBuf {
        PathBuf::from("/tmp/test_run/safesort")
    }

    fn owner(canonical: &str, cat: OwnerCategory) -> DetectedOwner {
        DetectedOwner {
            canonical: canonical.to_string(),
            display: canonical.to_string(),
            category: cat,
        }
    }

    #[test]
    fn test_clean_owner_folder_name_simple() {
        assert_eq!(clean_owner_folder_name("BenTreder.com"), "BenTreder");
        assert_eq!(clean_owner_folder_name("Ladybug Honey"), "LadybugHoney");
        assert_eq!(clean_owner_folder_name("Big Win Jerky"), "BigWinJerky");
        assert_eq!(
            clean_owner_folder_name("Big Win Seasonings"),
            "BigWinSeasonings"
        );
        assert_eq!(clean_owner_folder_name("916 Hookup"), "916Hookup");
        assert_eq!(clean_owner_folder_name("QuickTapID"), "QuickTapID");
        assert_eq!(
            clean_owner_folder_name("The Ghost Circuit"),
            "TheGhostCircuit"
        );
        assert_eq!(
            clean_owner_folder_name("Break Build Blaze"),
            "BreakBuildBlaze"
        );
        assert_eq!(
            clean_owner_folder_name("The Website That Saved Main Street"),
            "TheWebsiteThatSavedMainStreet"
        );
    }

    #[test]
    fn test_ext_group() {
        assert_eq!(ext_group("pdf"), "PDFs");
        assert_eq!(ext_group("PNG"), "PNGs");
        assert_eq!(ext_group("jpg"), "JPGs");
        assert_eq!(ext_group("jpeg"), "JPGs");
        assert_eq!(ext_group("webp"), "WEBPs");
        assert_eq!(ext_group("mp3"), "MP3s");
        assert_eq!(ext_group("mp4"), "MP4s");
        assert_eq!(ext_group("docx"), "DOCX");
        assert_eq!(ext_group("zip"), "ZIPs");
        assert_eq!(ext_group("epub"), "EPUB");
        assert_eq!(ext_group("unknown_ext"), "Other");
    }

    #[test]
    fn test_ladybug_honey_nfc_insert_pdf() {
        let o = owner("Ladybug Honey", OwnerCategory::Client);
        let dest = local_destination(&root(), Some(&o), FilePurpose::NfcInsert, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.contains("LadybugHoney"), "got: {s}");
        assert!(s.contains("PDFs"), "got: {s}");
        assert!(s.contains("NFC Inserts"), "got: {s}");
    }

    #[test]
    fn test_quicktapid_insert_pdf() {
        let o = owner("QuickTapID", OwnerCategory::Client);
        let dest = local_destination(&root(), Some(&o), FilePurpose::PrintInsert, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.contains("QuickTapID"), "got: {s}");
        assert!(s.contains("PDFs"), "got: {s}");
        assert!(s.contains("Inserts"), "got: {s}");
    }

    #[test]
    fn test_916hookup_sticker_pdf() {
        let o = owner("916 Hookup", OwnerCategory::Client);
        let dest = local_destination(&root(), Some(&o), FilePurpose::Sticker, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.contains("916Hookup"), "got: {s}");
        assert!(s.contains("PDFs"), "got: {s}");
        assert!(s.contains("Stickers"), "got: {s}");
    }

    #[test]
    fn test_bigwinjerky_webp_product_images() {
        let o = owner("Big Win Jerky", OwnerCategory::Client);
        let dest = local_destination(&root(), Some(&o), FilePurpose::Image, "webp");
        let s = dest.to_string_lossy();
        assert!(s.contains("BigWinJerky"), "got: {s}");
        assert!(s.contains("WEBPs"), "got: {s}");
        assert!(s.contains("Product Images"), "got: {s}");
    }

    #[test]
    fn test_ghost_circuit_cover_pdf() {
        let o = owner("The Ghost Circuit", OwnerCategory::Brand);
        let dest = local_destination(&root(), Some(&o), FilePurpose::BookCover, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.contains("TheGhostCircuit"), "got: {s}");
        assert!(s.contains("PDFs"), "got: {s}");
        assert!(s.contains("Covers"), "got: {s}");
    }

    #[test]
    fn test_break_build_blaze_docx_manuscripts() {
        let o = owner("Break Build Blaze", OwnerCategory::Brand);
        let dest = local_destination(&root(), Some(&o), FilePurpose::BookManuscript, "docx");
        let s = dest.to_string_lossy();
        assert!(s.contains("BreakBuildBlaze"), "got: {s}");
        assert!(s.contains("DOCX"), "got: {s}");
        assert!(s.contains("Manuscripts"), "got: {s}");
    }

    #[test]
    fn test_unknown_png_goes_to_other() {
        let dest = local_destination(&root(), None, FilePurpose::Image, "png");
        let s = dest.to_string_lossy();
        assert!(s.contains("Other"), "got: {s}");
        assert!(s.contains("PNGs"), "got: {s}");
    }

    #[test]
    fn test_sensitive_doc_goes_to_sensitive_documents() {
        let dest = local_destination(&root(), None, FilePurpose::SensitiveDocument, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.contains("SensitiveDocuments"), "got: {s}");
        assert!(s.contains("PDFs"), "got: {s}");
    }

    #[test]
    fn test_audio_goes_to_audio() {
        let dest = local_destination(&root(), None, FilePurpose::Audio, "mp3");
        let s = dest.to_string_lossy();
        assert!(s.contains("Audio"), "got: {s}");
        assert!(s.contains("MP3s"), "got: {s}");
    }

    #[test]
    fn test_video_goes_to_video() {
        let dest = local_destination(&root(), None, FilePurpose::Video, "mp4");
        let s = dest.to_string_lossy();
        assert!(s.contains("Video"), "got: {s}");
        assert!(s.contains("MP4s"), "got: {s}");
    }

    #[test]
    fn test_code_goes_to_review_needed() {
        let dest = local_destination(&root(), None, FilePurpose::Code, "js");
        let s = dest.to_string_lossy();
        assert!(s.contains("Review Needed"), "got: {s}");
    }

    #[test]
    fn test_unknown_purpose_goes_to_review_needed() {
        let dest = local_destination(&root(), None, FilePurpose::Unknown, "bin");
        let s = dest.to_string_lossy();
        assert!(s.contains("Review Needed"), "got: {s}");
    }

    #[test]
    fn test_compliance_label_nested() {
        let o = owner("Big Win Jerky", OwnerCategory::Client);
        let dest = local_destination(&root(), Some(&o), FilePurpose::ComplianceLabel, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.contains("BigWinJerky"), "got: {s}");
        assert!(s.contains("Labels"), "got: {s}");
        assert!(s.contains("Compliance"), "got: {s}");
    }

    #[test]
    fn test_receipts_fixed_category() {
        let dest = local_destination(&root(), None, FilePurpose::Receipt, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.contains("Receipts"), "got: {s}");
        // Should NOT contain an owner-derived folder
        assert!(!s.contains("Other"), "got: {s}");
    }

    #[test]
    fn test_destination_is_under_safesort_root() {
        let safesort_root = PathBuf::from("/home/user/Downloads/safesort");
        let o = owner("QuickTapID", OwnerCategory::Client);
        let dest = local_destination(&safesort_root, Some(&o), FilePurpose::NfcInsert, "pdf");
        assert!(
            dest.starts_with(&safesort_root),
            "Destination must be inside safesort_root, got: {}",
            dest.display()
        );
    }

    #[test]
    fn test_no_path_traversal() {
        // An adversarial owner name should not escape the safesort root
        let safe_root = PathBuf::from("/tmp/test/safesort");
        let evil = owner("../../etc/passwd", OwnerCategory::Unknown);
        let dest = local_destination(&safe_root, Some(&evil), FilePurpose::Image, "png");
        let s = dest.to_string_lossy();
        // clean_owner_folder_name strips dots, so ".." becomes empty → "Other"
        assert!(
            !s.contains("etc") && !s.contains("passwd"),
            "Path traversal must be prevented, got: {s}"
        );
    }
}
