use super::destination::PlacementDestination;
use std::collections::HashMap;

/// A user-defined rule for automatic placement.
#[derive(Debug, Clone)]
pub struct PlacementRule {
    /// Pattern to match (e.g. "bentreder + logo").
    pub pattern: String,
    /// The destination for matching files.
    pub destination: PlacementDestination,
    /// Whether this rule is active.
    pub active: bool,
}

/// Local rules storage. In Phase 2, this is in-memory only.
/// In Phase 3+, it will be backed by ~/.safesort/rules.toml.
pub struct RulesEngine {
    /// Pattern → rule mapping.
    rules: HashMap<String, PlacementRule>,
    /// Whether to persist rules to disk (disabled in Phase 2).
    persist: bool,
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RulesEngine {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            persist: false,
        }
    }

    /// Enable persistence (requires explicit flag).
    pub fn enable_persistence(&mut self) {
        self.persist = true;
    }

    /// Add a rule.
    pub fn add_rule(&mut self, pattern: &str, destination: PlacementDestination) {
        let pattern = pattern.to_lowercase();
        self.rules.insert(
            pattern.clone(),
            PlacementRule {
                pattern,
                destination,
                active: true,
            },
        );
    }

    /// Look up a rule by pattern.
    pub fn lookup(&self, pattern: &str) -> Option<&PlacementRule> {
        self.rules.get(&pattern.to_lowercase())
    }

    /// Check if a filename matches any active rule.
    pub fn match_file(&self, filename: &str) -> Option<&PlacementRule> {
        let lower = filename.to_lowercase();
        for rule in self.rules.values() {
            if rule.active && lower.contains(&rule.pattern) {
                return Some(rule);
            }
        }
        None
    }

    /// List all rules.
    pub fn list_rules(&self) -> Vec<&PlacementRule> {
        self.rules.values().collect()
    }

    /// Number of rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_rules_engine() {
        let mut engine = RulesEngine::new();
        assert!(engine.is_empty());

        engine.add_rule(
            "bentreder_logo",
            PlacementDestination {
                path: PathBuf::from("~/Workspace/Brand Assets/BenTreder/Logos"),
                description: "BenTreder Logos".to_string(),
                is_staging: true,
                risk: super::super::destination::DestinationRisk::Safe,
            },
        );

        assert_eq!(engine.len(), 1);

        let rule = engine.match_file("bentreder_logo_final.png");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().pattern, "bentreder_logo");

        let no_match = engine.match_file("random_file.txt");
        assert!(no_match.is_none());
    }
}
