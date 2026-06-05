use std::collections::HashMap;
use std::path::Path;

/// Detected owner / project / brand from filename and path context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedOwner {
    /// Canonical name, e.g. "BenTreder.com".
    pub canonical: String,
    /// Display name, e.g. "Ben Treder Digital".
    pub display: String,
    /// Category of ownership.
    pub category: OwnerCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerCategory {
    /// A website or domain.
    Website,
    /// A brand or business name.
    Brand,
    /// A software project.
    Project,
    /// A client.
    Client,
    /// A plugin or tool.
    Plugin,
    /// Unknown / generic.
    Unknown,
}

/// Detects likely owner/project/brand from filename tokens and path context.
pub struct OwnershipDetector {
    /// Known aliases: lowercase token → canonical owner.
    aliases: HashMap<String, DetectedOwner>,
}

impl Default for OwnershipDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl OwnershipDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            aliases: HashMap::new(),
        };
        detector.load_builtin_aliases();
        detector
    }

    /// Register a custom alias.
    pub fn add_alias(
        &mut self,
        token: &str,
        canonical: &str,
        display: &str,
        category: OwnerCategory,
    ) {
        self.aliases.insert(
            token.to_lowercase(),
            DetectedOwner {
                canonical: canonical.to_string(),
                display: display.to_string(),
                category,
            },
        );
    }

    /// Detect ownership from a filename and its parent path.
    pub fn detect(&self, filename: &str, parent_path: &Path) -> Option<DetectedOwner> {
        let tokens = tokenize(filename);
        let parent_tokens = path_tokens(parent_path);

        // 1. Check filename tokens against aliases (exact match first)
        for token in &tokens {
            if let Some(owner) = self.aliases.get(token) {
                return Some(owner.clone());
            }
        }

        // 2. Check multi-token sequences (2- and 3-token windows)
        let all_tokens: Vec<String> = tokens.iter().chain(parent_tokens.iter()).cloned().collect();

        // Check 3-token windows first (more specific)
        for window in all_tokens.windows(3) {
            let combined = format!("{} {} {}", window[0], window[1], window[2]);
            if let Some(owner) = self.aliases.get(&combined) {
                return Some(owner.clone());
            }
        }

        // Check 2-token windows
        for window in all_tokens.windows(2) {
            let combined = format!("{} {}", window[0], window[1]);
            if let Some(owner) = self.aliases.get(&combined) {
                return Some(owner.clone());
            }
            // Also try hyphenated
            let hyphenated = format!("{}-{}", window[0], window[1]);
            if let Some(owner) = self.aliases.get(&hyphenated) {
                return Some(owner.clone());
            }
        }

        // 3. Check parent folder names against aliases
        for token in &parent_tokens {
            if let Some(owner) = self.aliases.get(token) {
                return Some(owner.clone());
            }
        }

        // 4. Heuristic: if filename has recognizable project-like structure, extract it
        if let Some(owner) = self.heuristic_detect(&tokens, &parent_tokens) {
            return Some(owner);
        }

        None
    }

    fn heuristic_detect(
        &self,
        tokens: &[String],
        parent_tokens: &[String],
    ) -> Option<DetectedOwner> {
        // Look for capitalized words that might be brand names
        // This is a simple heuristic: if a token looks like a proper noun
        // (starts with uppercase in original, or is camelCase), treat it as potential brand
        for token in tokens {
            if token.len() >= 3 && !is_common(token) {
                // Check if it looks like a brand/project name (camelCase or has uppercase)
                if token.chars().any(|c| c.is_uppercase())
                    && token.chars().any(|c| c.is_lowercase())
                {
                    return Some(DetectedOwner {
                        canonical: token.clone(),
                        display: token.clone(),
                        category: OwnerCategory::Unknown,
                    });
                }
            }
        }

        // Check parent tokens for project-like names
        for token in parent_tokens {
            if token.len() >= 3 && !is_common(token) {
                if token.chars().any(|c| c.is_uppercase())
                    && token.chars().any(|c| c.is_lowercase())
                {
                    return Some(DetectedOwner {
                        canonical: token.clone(),
                        display: token.clone(),
                        category: OwnerCategory::Unknown,
                    });
                }
            }
        }

        None
    }

    fn load_builtin_aliases(&mut self) {
        let entries: Vec<(&str, &str, &str, OwnerCategory)> = vec![
            // BenTreder
            (
                "bentreder",
                "BenTreder.com",
                "Ben Treder Digital",
                OwnerCategory::Website,
            ),
            (
                "ben treder",
                "BenTreder.com",
                "Ben Treder Digital",
                OwnerCategory::Website,
            ),
            (
                "ben-treder",
                "BenTreder.com",
                "Ben Treder Digital",
                OwnerCategory::Website,
            ),
            (
                "bentreder.com",
                "BenTreder.com",
                "Ben Treder Digital",
                OwnerCategory::Website,
            ),
            // QuickTapID
            (
                "quicktapid",
                "QuickTapID",
                "QuickTapID",
                OwnerCategory::Brand,
            ),
            ("quicktap", "QuickTapID", "QuickTapID", OwnerCategory::Brand),
            (
                "quick-tap-id",
                "QuickTapID",
                "QuickTapID",
                OwnerCategory::Brand,
            ),
            // Website Fix Finder
            (
                "websitefixfinder",
                "Website Fix Finder",
                "Website Fix Finder",
                OwnerCategory::Plugin,
            ),
            (
                "website-fix-finder",
                "Website Fix Finder",
                "Website Fix Finder",
                OwnerCategory::Plugin,
            ),
            (
                "website fix finder",
                "Website Fix Finder",
                "Website Fix Finder",
                OwnerCategory::Plugin,
            ),
            (
                "wff",
                "Website Fix Finder",
                "Website Fix Finder",
                OwnerCategory::Plugin,
            ),
            // Content Handoff Hub
            (
                "contenthandoffhub",
                "Content Handoff Hub",
                "Content Handoff Hub",
                OwnerCategory::Project,
            ),
            (
                "content-handoff-hub",
                "Content Handoff Hub",
                "Content Handoff Hub",
                OwnerCategory::Project,
            ),
            (
                "content handoff hub",
                "Content Handoff Hub",
                "Content Handoff Hub",
                OwnerCategory::Project,
            ),
            (
                "chh",
                "Content Handoff Hub",
                "Content Handoff Hub",
                OwnerCategory::Project,
            ),
            // LinuxPicker
            (
                "linuxpicker",
                "LinuxPicker",
                "LinuxPicker",
                OwnerCategory::Project,
            ),
            (
                "linux-picker",
                "LinuxPicker",
                "LinuxPicker",
                OwnerCategory::Project,
            ),
            (
                "linux picker",
                "LinuxPicker",
                "LinuxPicker",
                OwnerCategory::Project,
            ),
            // SafeSort
            (
                "safesort",
                "SafeSort AI",
                "SafeSort AI",
                OwnerCategory::Project,
            ),
            (
                "safesort-ai",
                "SafeSort AI",
                "SafeSort AI",
                OwnerCategory::Project,
            ),
            (
                "safe-sort",
                "SafeSort AI",
                "SafeSort AI",
                OwnerCategory::Project,
            ),
            // OptionsCommand
            (
                "optionscommand",
                "OptionsCommand",
                "OptionsCommand",
                OwnerCategory::Project,
            ),
            (
                "options-command",
                "OptionsCommand",
                "OptionsCommand",
                OwnerCategory::Project,
            ),
            (
                "options command",
                "OptionsCommand",
                "OptionsCommand",
                OwnerCategory::Project,
            ),
            // Paper Options
            (
                "paperoptions",
                "Paper Options",
                "Paper Options",
                OwnerCategory::Project,
            ),
            (
                "paper-options",
                "Paper Options",
                "Paper Options",
                OwnerCategory::Project,
            ),
            (
                "paper options",
                "Paper Options",
                "Paper Options",
                OwnerCategory::Project,
            ),
        ];

        for (token, canonical, display, category) in entries {
            self.aliases.insert(
                token.to_lowercase(),
                DetectedOwner {
                    canonical: canonical.to_string(),
                    display: display.to_string(),
                    category,
                },
            );
        }
    }
}

/// Tokenize a filename (without extension) into lowercase word tokens.
pub fn tokenize(filename: &str) -> Vec<String> {
    let without_ext = filename.split('.').next().unwrap_or(filename);
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in without_ext.chars() {
        if ch.is_alphanumeric() {
            current.push(ch);
        } else if ch == '_' || ch == '-' || ch == ' ' {
            if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }

    tokens
}

/// Extract lowercase tokens from path components.
fn path_tokens(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|c| c.as_os_str().to_str())
        .flat_map(|s| {
            s.split(|c: char| !c.is_alphanumeric())
                .filter(|t| !t.is_empty())
                .map(|t| t.to_lowercase())
        })
        .collect()
}

/// Common words that should not be treated as brand names.
fn is_common(word: &str) -> bool {
    matches!(
        word,
        "the"
            | "and"
            | "for"
            | "with"
            | "from"
            | "this"
            | "that"
            | "new"
            | "old"
            | "final"
            | "draft"
            | "copy"
            | "test"
            | "temp"
            | "tmp"
            | "file"
            | "files"
            | "doc"
            | "docs"
            | "image"
            | "images"
            | "pics"
            | "photo"
            | "photos"
            | "img"
            | "screen"
            | "shot"
            | "capture"
            | "export"
            | "import"
            | "download"
            | "upload"
            | "data"
            | "report"
            | "reports"
            | "document"
            | "documents"
            | "v"
            | "ver"
            | "version"
            | "vs"
            | "vs."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize("bentreder_logo.png"), vec!["bentreder", "logo"]);
        assert_eq!(
            tokenize("quicktapid-banner.png"),
            vec!["quicktapid", "banner"]
        );
        assert_eq!(
            tokenize("website-fix-finder-screenshot.jpg"),
            vec!["website", "fix", "finder", "screenshot"]
        );
        assert_eq!(
            tokenize("content_handoff_icon.png"),
            vec!["content", "handoff", "icon"]
        );
        assert_eq!(
            tokenize("linuxpicker_article.docx"),
            vec!["linuxpicker", "article"]
        );
    }

    #[test]
    fn test_detect_bentreder() {
        let detector = OwnershipDetector::new();
        let owner = detector.detect("bentreder_logo.png", Path::new("/tmp/Downloads"));
        assert!(owner.is_some());
        let owner = owner.unwrap();
        assert_eq!(owner.canonical, "BenTreder.com");
        assert_eq!(owner.category, OwnerCategory::Website);
    }

    #[test]
    fn test_detect_quicktapid() {
        let detector = OwnershipDetector::new();
        let owner = detector.detect("quicktapid_banner.png", Path::new("/tmp/Downloads"));
        assert!(owner.is_some());
        let owner = owner.unwrap();
        assert_eq!(owner.canonical, "QuickTapID");
    }

    #[test]
    fn test_detect_website_fix_finder() {
        let detector = OwnershipDetector::new();
        let owner = detector.detect("website-fix-finder-v1.0.zip", Path::new("/tmp/Downloads"));
        assert!(owner.is_some());
        let owner = owner.unwrap();
        assert_eq!(owner.canonical, "Website Fix Finder");
        assert_eq!(owner.category, OwnerCategory::Plugin);
    }

    #[test]
    fn test_detect_safesort() {
        let detector = OwnershipDetector::new();
        let owner = detector.detect("safesort-roadmap.pdf", Path::new("/tmp/Downloads"));
        assert!(owner.is_some());
        let owner = owner.unwrap();
        assert_eq!(owner.canonical, "SafeSort AI");
    }

    #[test]
    fn test_detect_linuxpicker() {
        let detector = OwnershipDetector::new();
        let owner = detector.detect("linuxpicker_article.docx", Path::new("/tmp/Downloads"));
        assert!(owner.is_some());
        let owner = owner.unwrap();
        assert_eq!(owner.canonical, "LinuxPicker");
    }

    #[test]
    fn test_no_match() {
        let detector = OwnershipDetector::new();
        let owner = detector.detect("random_file.txt", Path::new("/tmp/Downloads"));
        assert!(owner.is_none());
    }

    #[test]
    fn test_custom_alias() {
        let mut detector = OwnershipDetector::new();
        detector.add_alias("mybrand", "MyBrand", "My Brand", OwnerCategory::Brand);
        let owner = detector.detect("mybrand_logo.png", Path::new("/tmp/Downloads"));
        assert!(owner.is_some());
        assert_eq!(owner.unwrap().canonical, "MyBrand");
    }
}
