//! Dependency graph for tracking path relationships.

use crate::graph::{Edge, EdgeKind, impact::ImpactAnalysis};
use crate::scan::{evidence::EvidenceKind, item::ScanItem};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Dependency graph tracking relationships between paths.
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    /// All nodes in the graph.
    pub nodes: HashSet<String>,
    /// Edges grouped by from-node.
    pub edges: HashMap<String, Vec<Edge>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, id: impl Into<String>) -> &mut Self {
        self.nodes.insert(id.into());
        self
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) -> &mut Self {
        self.nodes.insert(edge.from.clone());
        self.nodes.insert(edge.to.clone());
        self.edges.entry(edge.from.clone()).or_default().push(edge);
        self
    }

    /// Get all edges from a node.
    pub fn edges_from(&self, node_id: &str) -> Vec<&Edge> {
        self.edges
            .get(node_id)
            .map(|e| e.iter().collect())
            .unwrap_or_default()
    }

    /// Analyze the impact of moving a path.
    pub fn analyze_impact(&self, path: &Path) -> ImpactAnalysis {
        let path_str = path.to_string_lossy().into_owned();

        // Check if any edges point to this path (something depends on it)
        let mut dependencies = Vec::new();
        let mut max_level = crate::graph::ImpactLevel::None;

        for edge in self.all_edges() {
            if edge.to == path_str {
                let level = match edge.kind {
                    EdgeKind::References
                    | EdgeKind::UsesWorkingDirectory
                    | EdgeKind::UsesEnvFile => crate::graph::ImpactLevel::Critical,
                    EdgeKind::Executes => crate::graph::ImpactLevel::High,
                    EdgeKind::SymlinksTo => crate::graph::ImpactLevel::High,
                    EdgeKind::ContainsProjectMarker => crate::graph::ImpactLevel::Medium,
                    EdgeKind::ContainsSecretMarker => crate::graph::ImpactLevel::Medium,
                    EdgeKind::MayBreakIfMoved => crate::graph::ImpactLevel::Critical,
                };

                max_level = std::cmp::max(max_level, level);
                dependencies.push(crate::graph::impact::Dependency {
                    dependent: edge.from.clone(),
                    kind: edge.kind,
                    reason: edge
                        .description
                        .clone()
                        .unwrap_or_else(|| format!("{:?}", edge.kind)),
                });
            }
        }

        ImpactAnalysis {
            path: path_str,
            level: max_level,
            dependencies,
        }
    }

    /// Analyze impact based on evidence found during scanning.
    pub fn analyze_impact_from_evidence(
        &self,
        path: &str,
        evidence: &[crate::scan::evidence::Evidence],
    ) -> ImpactAnalysis {
        let mut max_level = crate::graph::ImpactLevel::None;
        let mut dependencies = Vec::new();

        for ev in evidence {
            let _level = match ev.kind {
                EvidenceKind::SystemdReference => {
                    max_level = std::cmp::max(max_level, crate::graph::ImpactLevel::Critical);
                    continue;
                }
                EvidenceKind::CronReference => {
                    max_level = std::cmp::max(max_level, crate::graph::ImpactLevel::Critical);
                    continue;
                }
                EvidenceKind::ScriptPathRef => {
                    max_level = std::cmp::max(max_level, crate::graph::ImpactLevel::High);
                }
                EvidenceKind::SensitiveFile | EvidenceKind::SensitivePath => {
                    max_level = std::cmp::max(max_level, crate::graph::ImpactLevel::High);
                }
                EvidenceKind::ProjectMarker | EvidenceKind::ContainsRust => {
                    max_level = std::cmp::max(max_level, crate::graph::ImpactLevel::Medium);
                }
                EvidenceKind::Symlink | EvidenceKind::SymlinkTarget => {
                    max_level = std::cmp::max(max_level, crate::graph::ImpactLevel::High);
                }
                _ => continue,
            };

            dependencies.push(crate::graph::impact::Dependency {
                dependent: ev.path.clone(),
                kind: EdgeKind::MayBreakIfMoved,
                reason: ev.description.clone(),
            });
        }

        ImpactAnalysis {
            path: path.to_string(),
            level: max_level,
            dependencies,
        }
    }

    /// Analyze impact for a Rust project directory.
    pub fn analyze_project_impact(
        &self,
        item: &ScanItem,
        evidence: &[crate::scan::evidence::Evidence],
    ) -> ImpactAnalysis {
        // Check for .git or Cargo.toml
        let has_project_markers = evidence.iter().any(|e| {
            matches!(
                e.kind,
                EvidenceKind::ProjectMarker | EvidenceKind::ContainsRust
            )
        });

        if !has_project_markers {
            return ImpactAnalysis::none(item.path.to_string_lossy().into_owned());
        }

        // Projects with .git/Cargo.toml are REVIEW at minimum
        // Check for additional evidence to determine impact level
        let has_sensitive = evidence.iter().any(|e| {
            matches!(
                e.kind,
                EvidenceKind::SensitiveFile | EvidenceKind::SensitivePath
            )
        });

        let has_symlink = evidence
            .iter()
            .any(|e| matches!(e.kind, EvidenceKind::Symlink | EvidenceKind::SymlinkTarget));

        let level = if has_sensitive {
            crate::graph::ImpactLevel::Critical
        } else if has_symlink {
            crate::graph::ImpactLevel::High
        } else {
            crate::graph::ImpactLevel::Medium
        };

        let mut deps = Vec::new();
        for ev in evidence.iter().filter(|e| {
            matches!(
                e.kind,
                EvidenceKind::ProjectMarker | EvidenceKind::ContainsRust
            )
        }) {
            deps.push(crate::graph::impact::Dependency {
                dependent: ev.path.clone(),
                kind: EdgeKind::ContainsProjectMarker,
                reason: ev.description.clone(),
            });
        }

        ImpactAnalysis {
            path: item.path.to_string_lossy().into_owned(),
            level,
            dependencies: deps,
        }
    }

    /// Check if this is a sensitive .env folder.
    pub fn analyze_sensitive_folder_impact(
        &self,
        item: &ScanItem,
        evidence: &[crate::scan::evidence::Evidence],
    ) -> ImpactAnalysis {
        let has_env = evidence
            .iter()
            .any(|e| matches!(e.kind, EvidenceKind::SensitiveFile));

        if has_env {
            let mut deps = Vec::new();
            for ev in evidence.iter() {
                deps.push(crate::graph::impact::Dependency {
                    dependent: ev.path.clone(),
                    kind: EdgeKind::ContainsSecretMarker,
                    reason: "Contains .env file".to_string(),
                });
            }
            ImpactAnalysis {
                path: item.path.to_string_lossy().into_owned(),
                level: crate::graph::ImpactLevel::Critical,
                dependencies: deps,
            }
        } else {
            ImpactAnalysis::none(item.path.to_string_lossy().into_owned())
        }
    }

    /// Collect all edges in the graph.
    pub fn all_edges(&self) -> Vec<&Edge> {
        self.edges.values().flat_map(|v| v.iter()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_add_node() {
        let mut g = DependencyGraph::new();
        g.add_node("test-path").add_node("another-path");
        assert!(g.nodes.contains("test-path"));
        assert!(g.nodes.contains("another-path"));
    }

    #[test]
    fn test_graph_add_edge() {
        let mut g = DependencyGraph::new();
        g.add_edge(Edge::references(
            "/home/user/script.sh",
            "/home/user/target",
        ));
        let edges = g.edges_from("/home/user/script.sh");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].to, "/home/user/target");
    }

    #[test]
    fn test_impact_none() {
        let g = DependencyGraph::new();
        let analysis = g.analyze_impact_from_evidence("/safe/path", &[]);
        assert_eq!(analysis.level, crate::graph::ImpactLevel::None);
    }

    #[test]
    fn test_impact_project_markers() {
        use crate::scan::evidence::{Evidence, EvidenceKind};

        let g = DependencyGraph::new();
        let evidence = vec![Evidence {
            kind: EvidenceKind::ContainsRust,
            path: "/path/to/project/Cargo.toml".to_string(),
            description: "Rust project marker".to_string(),
            note: None,
        }];
        let analysis = g.analyze_impact_from_evidence("/path/to/project", &evidence);
        assert_eq!(analysis.level, crate::graph::ImpactLevel::Medium);
    }

    #[test]
    fn test_impact_project_markers_dot_git() {
        use crate::scan::evidence::{Evidence, EvidenceKind};
        let g = DependencyGraph::new();
        let evidence = vec![Evidence {
            kind: EvidenceKind::ProjectMarker,
            path: "/path/to/project/.git/config".to_string(),
            description: ".git directory found".to_string(),
            note: None,
        }];
        let analysis = g.analyze_impact_from_evidence("/path/to/project", &evidence);
        assert_eq!(
            analysis.level,
            crate::graph::ImpactLevel::Medium,
            ".git marker should produce Medium impact"
        );
    }

    #[test]
    fn test_impact_project_markers_package_json() {
        use crate::scan::evidence::{Evidence, EvidenceKind};
        let g = DependencyGraph::new();
        let evidence = vec![Evidence {
            kind: EvidenceKind::ProjectMarker,
            path: "/path/to/webapp/package.json".to_string(),
            description: "Node.js project marker".to_string(),
            note: None,
        }];
        let analysis = g.analyze_impact_from_evidence("/path/to/webapp", &evidence);
        assert_eq!(
            analysis.level,
            crate::graph::ImpactLevel::Medium,
            "package.json marker should produce Medium impact"
        );
    }

    #[test]
    fn test_impact_project_markers_composer_json() {
        use crate::scan::evidence::{Evidence, EvidenceKind};
        let g = DependencyGraph::new();
        let evidence = vec![Evidence {
            kind: EvidenceKind::ProjectMarker,
            path: "/path/to/plugin/composer.json".to_string(),
            description: "PHP/Composer project marker".to_string(),
            note: None,
        }];
        let analysis = g.analyze_impact_from_evidence("/path/to/plugin", &evidence);
        assert_eq!(
            analysis.level,
            crate::graph::ImpactLevel::Medium,
            "composer.json marker should produce Medium impact"
        );
    }

    #[test]
    fn test_impact_project_markers_pyproject_toml() {
        use crate::scan::evidence::{Evidence, EvidenceKind};
        let g = DependencyGraph::new();
        let evidence = vec![Evidence {
            kind: EvidenceKind::ProjectMarker,
            path: "/path/to/data-tool/pyproject.toml".to_string(),
            description: "Python project marker".to_string(),
            note: None,
        }];
        let analysis = g.analyze_impact_from_evidence("/path/to/data-tool", &evidence);
        assert_eq!(
            analysis.level,
            crate::graph::ImpactLevel::Medium,
            "pyproject.toml marker should produce Medium impact"
        );
    }

    #[test]
    fn test_impact_env_marker_is_critical() {
        use crate::scan::evidence::{Evidence, EvidenceKind};
        use crate::scan::item::ScanItem;
        let g = DependencyGraph::new();
        let evidence = vec![Evidence {
            kind: EvidenceKind::SensitiveFile,
            path: "/path/to/app/.env".to_string(),
            description: ".env file found".to_string(),
            note: None,
        }];
        let item = ScanItem {
            path: std::path::PathBuf::from("/path/to/app"),
            name: "app".to_string(),
            is_dir: true,
            is_symlink: false,
            symlink_target: None,
            extension: None,
            depth: 0,
            is_hidden: false,
        };
        let analysis = g.analyze_sensitive_folder_impact(&item, &evidence);
        assert_eq!(
            analysis.level,
            crate::graph::ImpactLevel::Critical,
            ".env should produce Critical impact"
        );
    }

    #[test]
    fn test_impact_systemd_reference_is_critical() {
        use crate::scan::evidence::{Evidence, EvidenceKind};
        let g = DependencyGraph::new();
        let evidence = vec![Evidence {
            kind: EvidenceKind::SystemdReference,
            path: "/path/to/app".to_string(),
            description: "Referenced in systemd unit".to_string(),
            note: None,
        }];
        let analysis = g.analyze_impact_from_evidence("/path/to/app", &evidence);
        assert_eq!(
            analysis.level,
            crate::graph::ImpactLevel::Critical,
            "systemd reference should produce Critical impact"
        );
    }

    #[test]
    fn test_active_project_analyze_produces_medium_or_higher() {
        use crate::scan::evidence::{Evidence, EvidenceKind};
        use crate::scan::item::ScanItem;
        let g = DependencyGraph::new();
        let evidence = vec![Evidence {
            kind: EvidenceKind::ProjectMarker,
            path: "/path/to/project/.git".to_string(),
            description: ".git directory".to_string(),
            note: None,
        }];
        let item = ScanItem {
            path: std::path::PathBuf::from("/path/to/project"),
            name: "project".to_string(),
            is_dir: true,
            is_symlink: false,
            symlink_target: None,
            extension: None,
            depth: 0,
            is_hidden: false,
        };
        let analysis = g.analyze_project_impact(&item, &evidence);
        assert!(
            analysis.level >= crate::graph::ImpactLevel::Medium,
            "Active project should be Medium or higher, not None/Low — got {:?}",
            analysis.level
        );
    }

    #[test]
    fn test_impact_sensitive_file() {
        use crate::scan::evidence::{Evidence, EvidenceKind};

        let g = DependencyGraph::new();
        let evidence = vec![Evidence {
            kind: EvidenceKind::SensitiveFile,
            path: "/path/to/.env".to_string(),
            description: ".env file found".to_string(),
            note: None,
        }];
        let analysis = g.analyze_sensitive_folder_impact(
            &ScanItem {
                path: std::path::PathBuf::from("/path/to"),
                name: "to".to_string(),
                is_dir: true,
                is_symlink: false,
                symlink_target: None,
                extension: None,
                depth: 0,
                is_hidden: false,
            },
            &evidence,
        );
        assert_eq!(analysis.level, crate::graph::ImpactLevel::Critical);
    }
}
