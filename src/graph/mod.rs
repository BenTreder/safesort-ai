//! Dependency Graph module — Phase 2 foundation.
//!
//! This module provides the building blocks for tracking dependencies between
//! filesystem paths and analyzing impact of potential moves.

pub mod dependency_graph;
pub mod edge;
pub mod impact;
pub mod node;

pub use dependency_graph::DependencyGraph;
pub use edge::{Edge, EdgeKind};
pub use impact::{ImpactAnalysis, ImpactLevel};
pub use node::{
    Node, NodeKind, PathNode, ProjectNode, ScriptNode, SensitiveNode, ServiceNode, SymlinkNode,
};
