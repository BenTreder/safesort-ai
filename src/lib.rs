//! SafeSort AI — Safety-First Folder Organizer
//!
//! This library provides the core scanning, classification, and safety engine
//! for the SafeSort AI CLI application.

pub mod config;
pub mod detectors;
pub mod error;
pub mod graph;
pub mod manifest;
pub mod placement;
pub mod preflight;
pub mod profile;
pub mod reports;
pub mod rules_file;
pub mod safety;
pub mod scan;
