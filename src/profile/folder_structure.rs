use super::user_profile::UserProfile;
use std::fmt::Write;

/// Generate a recommended folder structure based on the detected user profile.
pub fn recommend(profile: &UserProfile) -> String {
    let mut output = String::new();

    // Determine the dominant profile type(s)
    let sorted = profile.sorted_scores();
    let top_profiles: Vec<&str> = sorted
        .iter()
        .filter(|(_, s)| s.score >= 2.0)
        .map(|(name, _)| name.as_str())
        .collect();

    let has_dev = top_profiles.iter().any(|p| *p == "Developer");
    let has_wp = top_profiles
        .iter()
        .any(|p| *p == "WordPress Plugin Builder");
    let has_web = top_profiles.iter().any(|p| *p == "Website Owner");
    let has_ai = top_profiles.iter().any(|p| *p == "AI Power User");
    let has_seo = top_profiles.iter().any(|p| *p == "SEO/Content Creator");

    let has_freelancer = top_profiles
        .iter()
        .any(|p| *p == "Client-Service Freelancer")
        || top_profiles.iter().any(|p| *p == "Business Owner");

    if has_dev || has_wp || has_web || has_ai || has_seo {
        writeln!(output, "  ~/Workspace/").unwrap();
        writeln!(output, "    00_Inbox/").unwrap();
        writeln!(output, "      Needs Review/").unwrap();
        writeln!(output, "      From Downloads/").unwrap();
        writeln!(output, "      Temporary Holding/").unwrap();
        writeln!(output, "").unwrap();
        writeln!(output, "    01_Active Projects/").unwrap();
        if has_ai {
            writeln!(output, "      AI Apps/").unwrap();
        }
        if has_dev {
            writeln!(output, "      Rust Tools/").unwrap();
            writeln!(output, "      Python Tools/").unwrap();
        }
        if has_wp {
            writeln!(output, "      WordPress Plugins/").unwrap();
        }
        if has_web {
            writeln!(output, "      Websites/").unwrap();
        }
        writeln!(output, "      Local Business Tools/").unwrap();
        writeln!(output, "      Trading Tools/").unwrap();
        writeln!(output, "      Experiments/").unwrap();
        writeln!(output, "").unwrap();

        if has_freelancer || has_seo {
            writeln!(output, "    02_Client Work/").unwrap();
            writeln!(output, "      Active Clients/").unwrap();
            writeln!(output, "      Completed Clients/").unwrap();
            writeln!(output, "      Proposals/").unwrap();
            writeln!(output, "      Reports/").unwrap();
            writeln!(output, "      Assets Received/").unwrap();
            writeln!(output, "      Deliverables/").unwrap();
            writeln!(output, "").unwrap();
        }

        if has_web || has_wp {
            writeln!(output, "    03_Websites/").unwrap();
            writeln!(output, "      Live Sites/").unwrap();
            writeln!(output, "      Static Sites/").unwrap();
            writeln!(output, "      WordPress Sites/").unwrap();
            writeln!(output, "      Backups/").unwrap();
            writeln!(output, "      SEO Pages/").unwrap();
            writeln!(output, "      Legal Pages/").unwrap();
            writeln!(output, "").unwrap();
        }

        if has_wp {
            writeln!(output, "    04_WordPress/").unwrap();
            writeln!(output, "      Plugins/").unwrap();
            writeln!(output, "        Free Versions/").unwrap();
            writeln!(output, "        Pro Versions/").unwrap();
            writeln!(output, "        WordPress.org Ready/").unwrap();
            writeln!(output, "        Release Zips/").unwrap();
            writeln!(output, "        Assets/").unwrap();
            writeln!(output, "        Screenshots/").unwrap();
            writeln!(output, "      Themes/").unwrap();
            writeln!(output, "      Snippets/").unwrap();
            writeln!(output, "      QA Checklists/").unwrap();
            writeln!(output, "").unwrap();
        }

        if has_ai {
            writeln!(output, "    05_AI Workflow/").unwrap();
            writeln!(output, "      Claude Prompts/").unwrap();
            writeln!(output, "      ChatGPT Prompts/").unwrap();
            writeln!(output, "      OpenRouter Prompts/").unwrap();
            writeln!(output, "      Project Checkpoints/").unwrap();
            writeln!(output, "      Debugging Logs/").unwrap();
            writeln!(output, "      Prompt Packs/").unwrap();
            writeln!(output, "      Generated Reports/").unwrap();
            writeln!(output, "").unwrap();
        }

        writeln!(output, "    06_Business/").unwrap();
        writeln!(output, "      Brand Assets/").unwrap();
        writeln!(output, "      Outreach/").unwrap();
        writeln!(output, "      Leads/").unwrap();
        writeln!(output, "      Social Posts/").unwrap();
        writeln!(output, "      Google Business/").unwrap();
        writeln!(output, "      Facebook Posts/").unwrap();
        writeln!(output, "      Alignable/").unwrap();
        writeln!(output, "      Invoices/").unwrap();
        writeln!(output, "      Receipts/").unwrap();
        writeln!(output, "").unwrap();
        writeln!(output, "    07_Media/").unwrap();
        writeln!(output, "      Screenshots/").unwrap();
        writeln!(output, "        Web QA/").unwrap();
        writeln!(output, "        Plugin QA/").unwrap();
        writeln!(output, "        Social Proof/").unwrap();
        writeln!(output, "        Errors/").unwrap();
        writeln!(output, "      Logos/").unwrap();
        writeln!(output, "      Icons/").unwrap();
        writeln!(output, "      Banners/").unwrap();
        writeln!(output, "      Product Images/").unwrap();
        writeln!(output, "      Video Assets/").unwrap();
        writeln!(output, "").unwrap();
        writeln!(output, "    08_Archives/").unwrap();
        writeln!(output, "      Old Projects/").unwrap();
        writeln!(output, "      Old Releases/").unwrap();
        writeln!(output, "      Website Backups/").unwrap();
        writeln!(output, "      Plugin Backups/").unwrap();
        writeln!(output, "      Monthly Archives/").unwrap();
        writeln!(output, "      ZIP Archives/").unwrap();
        writeln!(output, "      Tarballs/").unwrap();
        writeln!(output, "").unwrap();
        writeln!(output, "    09_Reports/").unwrap();
        writeln!(output, "      Website Audits/").unwrap();
        writeln!(output, "      SEO Reports/").unwrap();
        writeln!(output, "      Security Reports/").unwrap();
        writeln!(output, "      Client Reports/").unwrap();
        writeln!(output, "      Scan Reports/").unwrap();
        writeln!(output, "").unwrap();
        writeln!(output, "    10_Learning/").unwrap();
        if has_dev {
            writeln!(output, "      Rust/").unwrap();
            writeln!(output, "      Linux/").unwrap();
        }
        if has_wp {
            writeln!(output, "      WordPress/").unwrap();
        }
        if has_ai {
            writeln!(output, "      AI/").unwrap();
        }
        writeln!(output, "      Cybersecurity/").unwrap();
        writeln!(output, "").unwrap();
        writeln!(output, "    99_Review Needed/").unwrap();
        writeln!(output, "      Unknown Projects/").unwrap();
        writeln!(output, "      Mixed Folders/").unwrap();
        writeln!(output, "      Possible Secrets/").unwrap();
        writeln!(output, "      Possible Apps/").unwrap();
        writeln!(output, "      Manual Decision Required/").unwrap();
    } else {
        writeln!(output, "  ~/Workspace/").unwrap();
        writeln!(output, "    00_Inbox/").unwrap();
        writeln!(output, "      Needs Review/").unwrap();
        writeln!(output, "      From Downloads/").unwrap();
        writeln!(output, "").unwrap();
        writeln!(output, "    01_Active Projects/").unwrap();
        writeln!(output, "").unwrap();
        writeln!(output, "    02_Documents/").unwrap();
        writeln!(output, "    03_Media/").unwrap();
        writeln!(output, "      Screenshots/").unwrap();
        writeln!(output, "      Photos/").unwrap();
        writeln!(output, "").unwrap();
        writeln!(output, "    04_Archives/").unwrap();
        writeln!(output, "    99_Review Needed/").unwrap();
    }

    // Explain overlay concept
    writeln!(output, "").unwrap();
    writeln!(output, "  NOTE: This is a recommendation only.").unwrap();
    writeln!(output, "  Actual paths are NOT moved.").unwrap();
    writeln!(output, "  SafeSort AI uses a 'Workspace Overlay' concept:").unwrap();
    writeln!(output, "  your folders stay where they are, but SafeSort").unwrap();
    writeln!(output, "  categorizes them mentally for you.").unwrap();

    output
}
