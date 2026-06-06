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
/// Used for owner-subfolder labels, for example QuickTapID/PDFs.
pub fn ext_group(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "pdf" => "PDFs",
        "png" => "PNGs",
        "jpg" | "jpeg" => "JPGs",
        "webp" => "WEBPs",
        "gif" => "GIFs",
        "svg" => "SVGs",
        "bmp" | "ico" | "tiff" | "heic" | "avif" => "Images",

        "zip" => "ZIPs",
        "tar" => "TARs",
        "gz" | "tgz" => "GZs",
        "bz2" => "BZ2s",
        "xz" => "XZs",
        "zst" => "ZSTs",
        "7z" => "7Zs",
        "rar" => "RARs",

        "mp3" => "MP3s",
        "wav" => "WAVs",
        "flac" => "FLACs",
        "ogg" => "OGGs",
        "aac" => "AACs",
        "opus" => "OPUSs",
        "m4a" => "M4As",

        "mp4" => "MP4s",
        "mov" => "MOVs",
        "mkv" => "MKVs",
        "avi" => "AVIs",
        "webm" => "WEBMs",

        "docx" => "DOCX",
        "doc" => "DOCs",
        "epub" => "EPUB",
        "txt" => "TXTs",
        "md" => "MDs",
        "rtf" => "RTFs",
        "json" => "JSONs",
        "xml" => "XMLs",

        "csv" => "CSVs",
        "xlsx" => "XLSXs",
        "xls" => "XLSs",

        "pptx" => "PPTXs",
        "ppt" => "PPTs",
        "html" | "htm" => "HTML",

        _ => "Other",
    }
}

/// Map a file extension to a top-level extension-fallback folder name.
/// Used when no owner is known. This intentionally creates simple flat folders
/// like `safesort/PDFs/`, `safesort/MP3s/`, `safesort/JSONs/`.
pub fn fallback_folder(extension: &str) -> &'static str {
    ext_group(extension)
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
/// May contain a "/" to indicate nested subcategories, for example Labels/Compliance.
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
/// Known owners:
///   safesort/{Owner}/{ExtGroup}/[Subcategory]/
///
/// Unknown safe files:
///   safesort/{ExtGroup}/
///
/// Sensitive/private info:
///   safesort/SensitiveInfo/{ExtGroup}/
///
/// Risky code/unknown:
///   safesort/Other/Review Needed/
pub fn local_destination(
    safesort_root: &Path,
    owner: Option<&DetectedOwner>,
    purpose: FilePurpose,
    extension: &str,
) -> PathBuf {
    let ext = ext_group(extension);

    // Sensitive/private/account/legal/security information gets a clear local bucket.
    // It is still only movable through assisted mode with backup + rollback.
    if matches!(
        purpose,
        FilePurpose::SensitiveDocument | FilePurpose::Receipt
    ) {
        return safesort_root.join("SensitiveInfo").join(ext);
    }

    // Things that look like executable code or unknown/system-risk stay review-only,
    // except loose harmless JSON/XML files can be organized by extension when the
    // manifest safety gates also approve them.
    if matches!(purpose, FilePurpose::Code) {
        match extension.to_lowercase().as_str() {
            "json" => return safesort_root.join("JSONs"),
            "xml" => return safesort_root.join("XMLs"),
            _ => return safesort_root.join("Other").join("Review Needed"),
        }
    }

    if matches!(purpose, FilePurpose::Unknown) {
        return safesort_root.join("Other").join("Review Needed");
    }

    // Known owner/project/client/book wins: owner first, extension second,
    // optional purpose subfolder third.
    if let Some(o) = owner {
        let mut path = safesort_root
            .join(clean_owner_folder_name(&o.canonical))
            .join(ext);

        if let Some(sub) = subcategory_for(purpose) {
            for part in sub.split('/') {
                path = path.join(part);
            }
        }

        return path;
    }

    // Unknown but safe files fall back to simple extension/type folders directly
    // under ./safesort, for example safesort/PDFs, safesort/MP3s, safesort/JPGs.
    safesort_root.join(fallback_folder(extension))
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
        assert_eq!(ext_group("wav"), "WAVs");
        assert_eq!(ext_group("mp4"), "MP4s");
        assert_eq!(ext_group("mov"), "MOVs");
        assert_eq!(ext_group("docx"), "DOCX");
        assert_eq!(ext_group("zip"), "ZIPs");
        assert_eq!(ext_group("json"), "JSONs");
        assert_eq!(ext_group("txt"), "TXTs");
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
    fn test_unknown_png_goes_to_fallback_pngs() {
        let dest = local_destination(&root(), None, FilePurpose::Image, "png");
        let s = dest.to_string_lossy();
        assert!(s.ends_with("/PNGs"), "expected safesort/PNGs, got: {s}");
        assert!(!s.contains("Other"), "should not be under Other, got: {s}");
    }

    #[test]
    fn test_unknown_safe_pdf_goes_flat_pdfs() {
        let dest = local_destination(&root(), None, FilePurpose::Document, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.ends_with("/PDFs"), "expected safesort/PDFs, got: {s}");
        assert!(
            !s.contains("Reports"),
            "generic PDFs should not go to Reports, got: {s}"
        );
    }

    #[test]
    fn test_sensitive_doc_goes_to_sensitive_info() {
        let dest = local_destination(&root(), None, FilePurpose::SensitiveDocument, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.contains("SensitiveInfo"), "got: {s}");
        assert!(s.ends_with("/PDFs"), "got: {s}");
    }

    #[test]
    fn test_sensitive_txt_goes_to_sensitive_info_txts() {
        let dest = local_destination(&root(), None, FilePurpose::SensitiveDocument, "txt");
        let s = dest.to_string_lossy();
        assert!(s.contains("SensitiveInfo"), "got: {s}");
        assert!(s.ends_with("/TXTs"), "got: {s}");
    }

    #[test]
    fn test_audio_goes_to_flat_mp3s() {
        let dest = local_destination(&root(), None, FilePurpose::Audio, "mp3");
        let s = dest.to_string_lossy();
        assert!(s.ends_with("/MP3s"), "got: {s}");
    }

    #[test]
    fn test_video_goes_to_flat_mp4s() {
        let dest = local_destination(&root(), None, FilePurpose::Video, "mp4");
        let s = dest.to_string_lossy();
        assert!(s.ends_with("/MP4s"), "got: {s}");
    }

    #[test]
    fn test_loose_json_goes_flat_jsons() {
        let dest = local_destination(&root(), None, FilePurpose::Code, "json");
        let s = dest.to_string_lossy();
        assert!(s.ends_with("/JSONs"), "got: {s}");
    }

    #[test]
    fn test_script_stays_review_needed() {
        let dest = local_destination(&root(), None, FilePurpose::Code, "sh");
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
    fn test_receipts_fallback_to_extension_without_owner() {
        let dest = local_destination(&root(), None, FilePurpose::Receipt, "pdf");
        let s = dest.to_string_lossy();
        assert!(s.ends_with("/PDFs"), "got: {s}");
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
        let safe_root = PathBuf::from("/tmp/test/safesort");
        let evil = owner("../../etc/passwd", OwnerCategory::Unknown);
        let dest = local_destination(&safe_root, Some(&evil), FilePurpose::Image, "png");
        assert!(
            dest.starts_with(&safe_root),
            "Path traversal must be prevented, got: {}",
            dest.display()
        );
        let relative = dest.strip_prefix(&safe_root).unwrap();
        for part in relative.components() {
            let text = part.as_os_str().to_string_lossy();
            assert!(
                !text.contains(".."),
                "Path component must not contain .., got: {text}"
            );
            assert!(
                !text.contains('/'),
                "Path component must not contain slash, got: {text}"
            );
            assert!(
                !text.contains('\\'),
                "Path component must not contain backslash, got: {text}"
            );
        }
    }
}
