use crate::scan::evidence::EvidenceKind;

/// Signal weights for user profile detection.
#[derive(Debug, Clone)]
pub struct SignalWeight {
    pub profile: UserProfileType,
    /// Evidence kind that triggers this signal.
    pub evidence: EvidenceKind,
    /// Weight to add to the profile score.
    pub weight: f64,
}

/// All profile detection signals.
pub fn all_signals() -> Vec<SignalWeight> {
    vec![
        // Developer signals
        SignalWeight {
            profile: UserProfileType::Developer,
            evidence: EvidenceKind::ContainsRust,
            weight: 3.0,
        },
        SignalWeight {
            profile: UserProfileType::Developer,
            evidence: EvidenceKind::ContainsNodeJs,
            weight: 2.0,
        },
        SignalWeight {
            profile: UserProfileType::Developer,
            evidence: EvidenceKind::ContainsPython,
            weight: 2.0,
        },
        SignalWeight {
            profile: UserProfileType::Developer,
            evidence: EvidenceKind::ContainsDockerfile,
            weight: 1.5,
        },
        SignalWeight {
            profile: UserProfileType::Developer,
            evidence: EvidenceKind::ProjectMarker,
            weight: 1.0,
        },
        SignalWeight {
            profile: UserProfileType::Developer,
            evidence: EvidenceKind::ContainsScripts,
            weight: 1.0,
        },
        // WordPress signals
        SignalWeight {
            profile: UserProfileType::WordPressBuilder,
            evidence: EvidenceKind::ContainsWordPress,
            weight: 4.0,
        },
        SignalWeight {
            profile: UserProfileType::WordPressBuilder,
            evidence: EvidenceKind::ContainsPhp,
            weight: 2.0,
        },
        // Website Owner signals
        SignalWeight {
            profile: UserProfileType::WebsiteOwner,
            evidence: EvidenceKind::WebsiteFolder,
            weight: 3.0,
        },
        // AI Power User signals
        SignalWeight {
            profile: UserProfileType::AiPowerUser,
            evidence: EvidenceKind::ContainsPython,
            weight: 1.0,
        },
        // SEO/Content Creator signals
        SignalWeight {
            profile: UserProfileType::SeoContentCreator,
            evidence: EvidenceKind::DocumentFile,
            weight: 0.5,
        },
        SignalWeight {
            profile: UserProfileType::SeoContentCreator,
            evidence: EvidenceKind::MediaFile,
            weight: 0.5,
        },
        // Designer/Media Creator signals
        SignalWeight {
            profile: UserProfileType::DesignerMediaCreator,
            evidence: EvidenceKind::MediaFile,
            weight: 1.0,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserProfileType {
    Developer,
    WordPressBuilder,
    WebsiteOwner,
    AiPowerUser,
    SeoContentCreator,
    ClientServiceFreelancer,
    DesignerMediaCreator,
    BusinessOwner,
    DataReportsUser,
    GeneralUser,
}

impl UserProfileType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Developer => "Developer",
            Self::WordPressBuilder => "WordPress Plugin Builder",
            Self::WebsiteOwner => "Website Owner",
            Self::AiPowerUser => "AI Power User",
            Self::SeoContentCreator => "SEO/Content Creator",
            Self::ClientServiceFreelancer => "Client-Service Freelancer",
            Self::DesignerMediaCreator => "Designer/Media Creator",
            Self::BusinessOwner => "Business Owner",
            Self::DataReportsUser => "Data/Reports User",
            Self::GeneralUser => "General User",
        }
    }

    pub fn confidence(score: f64) -> &'static str {
        if score >= 5.0 {
            "high"
        } else if score >= 2.0 {
            "medium"
        } else if score > 0.0 {
            "low"
        } else {
            "—"
        }
    }
}
