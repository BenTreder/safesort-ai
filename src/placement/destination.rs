use super::file_purpose::FilePurpose;
use super::ownership::{DetectedOwner, OwnerCategory};
use std::path::PathBuf;

/// A recommended safe destination for a file.
#[derive(Debug, Clone)]
pub struct PlacementDestination {
    /// The recommended path (relative to home or absolute).
    pub path: PathBuf,
    /// Human-readable description of this destination.
    pub description: String,
    /// Whether this is a safe staging area (not a live project path).
    pub is_staging: bool,
    /// Risk level of placing a file here.
    pub risk: DestinationRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationRisk {
    /// Safe staging area, no live impact.
    Safe,
    /// Near a live area but still safe.
    LowRisk,
    /// Could affect a live system — needs review.
    NeedsReview,
}

/// Plans safe destinations based on ownership + purpose + profile.
pub struct DestinationPlanner {
    home: PathBuf,
}

impl DestinationPlanner {
    pub fn new(home: PathBuf) -> Self {
        Self { home }
    }

    /// Generate candidate destinations for a file.
    /// Returns a sorted list: best match first.
    pub fn plan(
        &self,
        owner: Option<&DetectedOwner>,
        purpose: FilePurpose,
        _is_in_safe_zone: bool,
    ) -> Vec<PlacementDestination> {
        let mut destinations = Vec::new();

        let owner_name = owner.map(|o| o.canonical.as_str()).unwrap_or("Unknown");
        let owner_cat = owner.map(|o| o.category);

        match purpose {
            FilePurpose::Logo | FilePurpose::Icon | FilePurpose::Favicon => {
                // Brand Assets > {Owner} > Logos
                destinations.push(self.make_dest(
                    &format!("Workspace/06_Business/Brand Assets/{owner_name}/Logos"),
                    &format!("Brand Assets → {owner_name} → Logos"),
                    true,
                    DestinationRisk::Safe,
                ));
                // Websites > {Owner} > Incoming Assets > Logos
                if matches!(owner_cat, Some(OwnerCategory::Website)) {
                    let domain = owner_name;
                    destinations.push(self.make_dest(
                        &format!("Workspace/03_Websites/{domain}/Incoming Assets/Logos"),
                        &format!("Websites → {domain} → Incoming Assets → Logos"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::Banner | FilePurpose::Cover => {
                destinations.push(self.make_dest(
                    &format!("Workspace/06_Business/Brand Assets/{owner_name}/Banners"),
                    &format!("Brand Assets → {owner_name} → Banners"),
                    true,
                    DestinationRisk::Safe,
                ));
                if matches!(owner_cat, Some(OwnerCategory::Website)) {
                    let domain = owner_name;
                    destinations.push(self.make_dest(
                        &format!("Workspace/03_Websites/{domain}/Incoming Assets/Banners"),
                        &format!("Websites → {domain} → Incoming Assets → Banners"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::Screenshot => {
                destinations.push(self.make_dest(
                    "Workspace/07_Media/Screenshots/Web QA",
                    "Media → Screenshots → Web QA",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::ErrorScreenshot => {
                destinations.push(self.make_dest(
                    "Workspace/07_Media/Screenshots/Errors",
                    "Media → Screenshots → Errors",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::QaScreenshot => {
                destinations.push(self.make_dest(
                    "Workspace/07_Media/Screenshots/Plugin QA",
                    "Media → Screenshots → Plugin QA",
                    true,
                    DestinationRisk::Safe,
                ));
                if matches!(owner_cat, Some(OwnerCategory::Plugin)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/04_WordPress/Plugins/{owner_name}/Screenshots"),
                        &format!("WordPress → Plugins → {owner_name} → Screenshots"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::SocialProof => {
                destinations.push(self.make_dest(
                    "Workspace/07_Media/Screenshots/Social Proof",
                    "Media → Screenshots → Social Proof",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Report => {
                destinations.push(self.make_dest(
                    "Workspace/09_Reports/Website Audits",
                    "Reports → Website Audits",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Audit => {
                destinations.push(self.make_dest(
                    "Workspace/09_Reports/Website Audits",
                    "Reports → Website Audits",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Invoice => {
                destinations.push(self.make_dest(
                    "Workspace/06_Business/Invoices",
                    "Business → Invoices",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Receipt => {
                destinations.push(self.make_dest(
                    "Workspace/06_Business/Receipts",
                    "Business → Receipts",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Proposal => {
                destinations.push(self.make_dest(
                    "Workspace/02_Client Work/Proposals",
                    "Client Work → Proposals",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Contract => {
                destinations.push(self.make_dest(
                    "Workspace/02_Client Work/Proposals",
                    "Client Work → Proposals",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Backup => {
                destinations.push(self.make_dest(
                    "Workspace/08_Archives/Website Backups",
                    "Archives → Website Backups",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::ReleaseZip => {
                if matches!(owner_cat, Some(OwnerCategory::Plugin)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/04_WordPress/Plugins/{owner_name}/Release Zips"),
                        &format!("WordPress → Plugins → {owner_name} → Release Zips"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
                destinations.push(self.make_dest(
                    "Workspace/08_Archives/ZIP Archives",
                    "Archives → ZIP Archives",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::PluginAsset => {
                if matches!(owner_cat, Some(OwnerCategory::Plugin)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/04_WordPress/Plugins/{owner_name}/Assets"),
                        &format!("WordPress → Plugins → {owner_name} → Assets"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::WebsiteAsset => {
                if matches!(owner_cat, Some(OwnerCategory::Website)) {
                    let domain = owner_name;
                    destinations.push(self.make_dest(
                        &format!("Workspace/03_Websites/{domain}/Incoming Assets"),
                        &format!("Websites → {domain} → Incoming Assets"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::SocialPost => {
                destinations.push(self.make_dest(
                    "Workspace/06_Business/Social Posts",
                    "Business → Social Posts",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Document => {
                destinations.push(self.make_dest(
                    "Workspace/09_Reports/Client Reports",
                    "Reports → Client Reports",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Archive => {
                destinations.push(self.make_dest(
                    "Workspace/08_Archives/ZIP Archives",
                    "Archives → ZIP Archives",
                    true,
                    DestinationRisk::Safe,
                ));
                destinations.push(self.make_dest(
                    "Workspace/08_Archives/Tarballs",
                    "Archives → Tarballs",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Image => {
                destinations.push(self.make_dest(
                    "Workspace/07_Media/Product Images",
                    "Media → Product Images",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Video => {
                destinations.push(self.make_dest(
                    "Workspace/07_Media/Video Assets",
                    "Media → Video Assets",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Installer => {
                destinations.push(self.make_dest(
                    "Workspace/08_Archives",
                    "Archives",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::JobApplication => {
                destinations.push(self.make_dest(
                    "Workspace/06_Business/Career/Job Applications",
                    "Career → Job Applications",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Resume => {
                destinations.push(self.make_dest(
                    "Workspace/06_Business/Career/Resumes",
                    "Career → Resumes",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::CoverLetter => {
                destinations.push(self.make_dest(
                    "Workspace/06_Business/Career/Cover Letters",
                    "Career → Cover Letters",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::Soq => {
                if matches!(owner_cat, Some(OwnerCategory::Client)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/02_Client Work/{owner_name}/Proposals/SOQ"),
                        &format!("Client Work → {owner_name} → Proposals → SOQ"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
                destinations.push(self.make_dest(
                    "Workspace/02_Client Work/Proposals/SOQ",
                    "Client Work → Proposals → SOQ",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::NfcInsert => {
                if matches!(owner_cat, Some(OwnerCategory::Client)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/02_Client Work/{owner_name}/Deliverables/Print Assets/NFC Inserts"),
                        &format!("Client Work → {owner_name} → Print Assets → NFC Inserts"),
                        true,
                        DestinationRisk::Safe,
                    ));
                } else {
                    destinations.push(self.make_dest(
                        &format!(
                            "Workspace/06_Business/Brand Assets/{owner_name}/Print Assets/NFC Inserts"
                        ),
                        &format!("Brand Assets → {owner_name} → Print Assets → NFC Inserts"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::PrintInsert => {
                if matches!(owner_cat, Some(OwnerCategory::Client)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/02_Client Work/{owner_name}/Deliverables/Print Assets/Inserts"),
                        &format!("Client Work → {owner_name} → Print Assets → Inserts"),
                        true,
                        DestinationRisk::Safe,
                    ));
                } else {
                    destinations.push(self.make_dest(
                        &format!(
                            "Workspace/06_Business/Brand Assets/{owner_name}/Print Assets/Inserts"
                        ),
                        &format!("Brand Assets → {owner_name} → Print Assets → Inserts"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::Mailer => {
                if matches!(owner_cat, Some(OwnerCategory::Client)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/02_Client Work/{owner_name}/Deliverables/Print Assets/Mailers"),
                        &format!("Client Work → {owner_name} → Print Assets → Mailers"),
                        true,
                        DestinationRisk::Safe,
                    ));
                } else {
                    destinations.push(self.make_dest(
                        &format!(
                            "Workspace/06_Business/Brand Assets/{owner_name}/Print Assets/Mailers"
                        ),
                        &format!("Brand Assets → {owner_name} → Print Assets → Mailers"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::Postcard => {
                if matches!(owner_cat, Some(OwnerCategory::Client)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/02_Client Work/{owner_name}/Deliverables/Print Assets/Postcards"),
                        &format!("Client Work → {owner_name} → Print Assets → Postcards"),
                        true,
                        DestinationRisk::Safe,
                    ));
                } else {
                    destinations.push(self.make_dest(
                        &format!(
                            "Workspace/06_Business/Brand Assets/{owner_name}/Print Assets/Postcards"
                        ),
                        &format!("Brand Assets → {owner_name} → Print Assets → Postcards"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::Flyer => {
                if matches!(owner_cat, Some(OwnerCategory::Client)) {
                    destinations.push(self.make_dest(
                        &format!(
                            "Workspace/02_Client Work/{owner_name}/Deliverables/Print Assets/Flyers"
                        ),
                        &format!("Client Work → {owner_name} → Print Assets → Flyers"),
                        true,
                        DestinationRisk::Safe,
                    ));
                } else {
                    destinations.push(self.make_dest(
                        &format!(
                            "Workspace/06_Business/Brand Assets/{owner_name}/Print Assets/Flyers"
                        ),
                        &format!("Brand Assets → {owner_name} → Print Assets → Flyers"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::SalesSheet => {
                if matches!(owner_cat, Some(OwnerCategory::Client)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/02_Client Work/{owner_name}/Deliverables/Print Assets/Sales Sheets"),
                        &format!("Client Work → {owner_name} → Print Assets → Sales Sheets"),
                        true,
                        DestinationRisk::Safe,
                    ));
                } else {
                    destinations.push(self.make_dest(
                        &format!(
                            "Workspace/06_Business/Brand Assets/{owner_name}/Print Assets/Sales Sheets"
                        ),
                        &format!("Brand Assets → {owner_name} → Print Assets → Sales Sheets"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::StickerSheet | FilePurpose::Sticker => {
                if matches!(owner_cat, Some(OwnerCategory::Client)) {
                    destinations.push(self.make_dest(
                        &format!("Workspace/02_Client Work/{owner_name}/Deliverables/Print Assets/Stickers"),
                        &format!("Client Work → {owner_name} → Print Assets → Stickers"),
                        true,
                        DestinationRisk::Safe,
                    ));
                } else {
                    destinations.push(self.make_dest(
                        &format!(
                            "Workspace/06_Business/Brand Assets/{owner_name}/Print Assets/Stickers"
                        ),
                        &format!("Brand Assets → {owner_name} → Print Assets → Stickers"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
            }
            FilePurpose::BookInterior | FilePurpose::BookManuscript => {
                destinations.push(self.make_dest(
                    &format!("Workspace/06_Business/Books/{owner_name}/Interior Drafts"),
                    &format!("Books → {owner_name} → Interior Drafts"),
                    true,
                    DestinationRisk::Safe,
                ));
                destinations.push(self.make_dest(
                    "Workspace/06_Business/Books/Drafts",
                    "Books → Drafts",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            FilePurpose::CannabisImage => {
                if matches!(owner_cat, Some(OwnerCategory::Client)) {
                    destinations.push(self.make_dest(
                        &format!(
                            "Workspace/02_Client Work/{owner_name}/Deliverables/Product Images"
                        ),
                        &format!("Client Work → {owner_name} → Product Images"),
                        true,
                        DestinationRisk::Safe,
                    ));
                }
                destinations.push(self.make_dest(
                    "Workspace/07_Media/Product Images/Cannabis",
                    "Media → Product Images → Cannabis",
                    true,
                    DestinationRisk::Safe,
                ));
            }
            _ => {
                // Unknown or unhandled purpose → Review Needed
                destinations.push(self.make_dest(
                    "Workspace/99_Review Needed",
                    "Review Needed",
                    true,
                    DestinationRisk::NeedsReview,
                ));
            }
        }

        destinations
    }

    fn make_dest(
        &self,
        relative: &str,
        description: &str,
        is_staging: bool,
        risk: DestinationRisk,
    ) -> PlacementDestination {
        PlacementDestination {
            path: self.home.join(relative),
            description: description.to_string(),
            is_staging,
            risk,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ownership::OwnerCategory;
    use super::*;

    fn planner() -> DestinationPlanner {
        DestinationPlanner::new(PathBuf::from("/home/user"))
    }

    #[test]
    fn test_logo_destination() {
        let p = planner();
        let owner = DetectedOwner {
            canonical: "BenTreder.com".to_string(),
            display: "Ben Treder Digital".to_string(),
            category: OwnerCategory::Website,
        };
        let dests = p.plan(Some(&owner), FilePurpose::Logo, true);
        assert!(!dests.is_empty());
        assert!(dests[0].path.to_string_lossy().contains("Brand Assets"));
        assert!(dests[0].path.to_string_lossy().contains("Logos"));
        assert!(dests[0].is_staging);
    }

    #[test]
    fn test_release_zip_destination() {
        let p = planner();
        let owner = DetectedOwner {
            canonical: "Website Fix Finder".to_string(),
            display: "Website Fix Finder".to_string(),
            category: OwnerCategory::Plugin,
        };
        let dests = p.plan(Some(&owner), FilePurpose::ReleaseZip, true);
        assert!(
            dests
                .iter()
                .any(|d| d.path.to_string_lossy().contains("Release Zips"))
        );
    }

    #[test]
    fn test_unknown_owner_gets_review() {
        let p = planner();
        let dests = p.plan(None, FilePurpose::Unknown, true);
        assert!(
            dests
                .iter()
                .any(|d| d.path.to_string_lossy().contains("Review Needed"))
        );
    }

    #[test]
    fn test_error_screenshot_destination() {
        let p = planner();
        let dests = p.plan(None, FilePurpose::ErrorScreenshot, true);
        assert!(dests[0].path.to_string_lossy().contains("Errors"));
    }

    #[test]
    fn test_all_destinations_are_staging() {
        let p = planner();
        let owner = DetectedOwner {
            canonical: "TestBrand".to_string(),
            display: "Test Brand".to_string(),
            category: OwnerCategory::Brand,
        };
        for purpose in &[
            FilePurpose::Logo,
            FilePurpose::Banner,
            FilePurpose::Screenshot,
            FilePurpose::Report,
        ] {
            let dests = p.plan(Some(&owner), *purpose, true);
            for dest in &dests {
                assert!(
                    dest.is_staging,
                    "Destination for {:?} should be staging",
                    purpose
                );
            }
        }
    }
}
