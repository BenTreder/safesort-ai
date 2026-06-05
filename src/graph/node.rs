//! Node types for the dependency graph.

use serde::Serialize;
use std::path::PathBuf;

/// Types of nodes in the dependency graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum NodeKind {
    /// A filesystem path (file or directory).
    Path,
    /// A systemd service unit.
    Service,
    /// A script or executable.
    Script,
    /// A project (git repo, Rust project, etc.).
    Project,
    /// A sensitive path (credentials, .env, private keys).
    Sensitive,
    /// A symbolic link.
    Symlink,
}

impl NodeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::Path => "Path",
            NodeKind::Service => "Service",
            NodeKind::Script => "Script",
            NodeKind::Project => "Project",
            NodeKind::Sensitive => "Sensitive",
            NodeKind::Symlink => "Symlink",
        }
    }
}

/// A node in the dependency graph representing a path or entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Node {
    /// Unique identifier for this node.
    pub id: String,
    /// The kind of node.
    pub kind: NodeKind,
    /// Display name.
    pub name: String,
    /// Full path (for path nodes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl Node {
    pub fn new_path(id: impl Into<String>, path: PathBuf) -> Self {
        Self {
            id: id.into(),
            kind: NodeKind::Path,
            name: path.display().to_string(),
            path: Some(path),
        }
    }

    pub fn new_service(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: NodeKind::Service,
            name: name.into(),
            path: None,
        }
    }

    pub fn new_script(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: NodeKind::Script,
            name: name.into(),
            path: None,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Specialized node types for ergonomic access
// ──────────────────────────────────────────────────────────────

/// A path node representing a filesystem entry.
pub struct PathNode {
    pub node: Node,
    pub is_directory: bool,
}

impl PathNode {
    pub fn new(path: PathBuf, is_directory: bool) -> Self {
        let id = path.to_string_lossy().into_owned();
        Self {
            node: Node::new_path(id.clone(), path),
            is_directory,
        }
    }
}

/// A service node representing a systemd service or similar.
pub struct ServiceNode {
    pub node: Node,
    pub unit_name: String,
    pub references: Vec<String>,
}

impl ServiceNode {
    pub fn new(unit_name: impl Into<String>, references: Vec<String>) -> Self {
        let unit_name = unit_name.into();
        Self {
            node: Node::new_service(format!("service:{}", unit_name), unit_name.clone()),
            unit_name,
            references,
        }
    }
}

/// A script node representing a script with path references.
pub struct ScriptNode {
    pub node: Node,
    pub script_path: PathBuf,
    pub path_references: Vec<String>,
}

impl ScriptNode {
    pub fn new(script_path: PathBuf, path_references: Vec<String>) -> Self {
        Self {
            node: Node::new_script(
                format!("script:{}", script_path.to_string_lossy()),
                script_path.display().to_string(),
            ),
            script_path,
            path_references,
        }
    }
}

/// A project node representing a code project.
pub struct ProjectNode {
    pub node: Node,
    pub project_type: ProjectType,
    pub marker_files: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    WordPress,
    Docker,
    Unknown,
}

impl ProjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::Node => "Node.js",
            ProjectType::Python => "Python",
            ProjectType::WordPress => "WordPress",
            ProjectType::Docker => "Docker",
            ProjectType::Unknown => "Unknown",
        }
    }
}

impl ProjectNode {
    pub fn new(
        project_path: PathBuf,
        project_type: ProjectType,
        marker_files: Vec<String>,
    ) -> Self {
        Self {
            node: Node::new_path(
                format!("project:{}", project_path.to_string_lossy()),
                project_path,
            ),
            project_type,
            marker_files,
        }
    }
}

/// A sensitive node representing credentials, secrets, etc.
pub struct SensitiveNode {
    pub node: Node,
    pub reason: SensitiveReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum SensitiveReason {
    EnvFile,
    PrivateKey,
    SecretFile,
    PrivateFolder,
    SensitivePath,
}

impl SensitiveReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            SensitiveReason::EnvFile => ".env file",
            SensitiveReason::PrivateKey => "private key",
            SensitiveReason::SecretFile => "secret file",
            SensitiveReason::PrivateFolder => "private folder",
            SensitiveReason::SensitivePath => "sensitive path",
        }
    }
}

impl SensitiveNode {
    pub fn new(path: PathBuf, reason: SensitiveReason) -> Self {
        Self {
            node: Node::new_path(format!("sensitive:{}", path.to_string_lossy()), path),
            reason,
        }
    }
}

/// A symlink node representing a symbolic link.
pub struct SymlinkNode {
    pub node: Node,
    pub target: PathBuf,
}

impl SymlinkNode {
    pub fn new(link_path: PathBuf, target: PathBuf) -> Self {
        Self {
            node: Node::new_path(
                format!("symlink:{}", link_path.to_string_lossy()),
                link_path,
            ),
            target,
        }
    }
}
