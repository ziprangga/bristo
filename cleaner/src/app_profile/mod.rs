// Copyright 2026 ziprangga
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Application profile and discovery state.
//!
//! This module defines the core data structures used to represent
//! an application and all information discovered during scanning.
//!
//! The module is built around two primary concepts:
//!
//! - `AppProfile` stores the complete application discovery state.
//! - `PathEntry` stores and manages discovered filesystem entries.
//!
//! An `AppProfile` acts as the central aggregation point for:
//!
//! - Application metadata.
//! - Running processes.
//! - Application-related filesystem paths, including package receipt BOM files.
//!
//! The typical lifecycle is:
//!
//! 1. Create an `AppProfile` from an application path.
//! 2. Discover running processes.
//! 3. Scan application filesystem entries.
//! 4. Retrieve discovered filesystem entries.
//!
//! The module separates application information into dedicated
//! containers:
//!
//! - `Metadata` stores application information.
//! - `AppProcs` stores discovered running processes.
//! - `PathEntry` stores the application bundle together with
//!   discovered filesystem entries, including associated files,
//!   sandbox containers, background task files, and package
//!   receipt BOM files.
//!
//! `AppProfile` provides a single aggregate state used by the
//! cleanup workflow while delegating discovery responsibilities
//! to the specialized components it owns.
//!
//! Note:
//! Most callers should interact with `AppProfile` rather than the
//! individual storage types directly. Lower-level types exist to
//! organize discovery results and scanning logic.
//!..

mod metadata;
mod path_entry;
mod process_entry;

pub use metadata::Metadata;
pub use path_entry::PathEntry;
pub use process_entry::ProcessEntry;

use crate::errors::Result;
use crate::path_data::PathData;
use mini_logger::debug;
use std::path::Path;

/// Aggregated application discovery state.
///
/// Doc:
/// Stores all information discovered about an application during
/// the scanning workflow.
///
/// An `AppProfile` acts as the central model used throughout
/// application inspection and cleanup operations.
///
/// Stored information includes:
///
/// - Application metadata.
/// - Running processes.
/// - Discovered filesystem entries, including associated
///   application files, sandbox containers, background task
///   files, and package receipt BOM files.
///
/// Discovery operations progressively populate the profile:
///
/// ```text
/// Metadata
///      │
///      ▼
/// AppProcs
///      │
///      ▼
/// PathEntry
///      │
///      ├─ Application bundle
///      ├─ General associated files
///      ├─ Sandbox containers
///      ├─ Background task files
///      └─ Package receipt BOM files
/// ```
///
/// Once populated, `PathEntry` provides access to every discovered
/// filesystem entry related to the application, organized by
/// category.
///
/// Note:
/// `AppProfile` represents mutable discovery state. It is commonly
/// owned by `Cleaner`, which provides the higher-level cleanup
/// orchestration.
#[derive(Debug, Default, Clone)]
pub struct AppProfile {
    metadata: Metadata,
    process_entry: ProcessEntry,
    path_entry: PathEntry,
}

impl AppProfile {
    pub fn new(metadata: Metadata, process_entry: ProcessEntry, path_entry: PathEntry) -> Self {
        Self {
            metadata,
            process_entry,
            path_entry,
        }
    }

    pub fn from_path(app_path: &Path) -> Result<Self> {
        let metadata = Metadata::from_path(app_path)?;
        let path_entry = PathEntry::from_metadata(&metadata);

        Ok(Self {
            metadata: metadata,
            process_entry: ProcessEntry::default(),
            path_entry: path_entry,
        })
    }

    pub fn as_metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn as_process_entry(&self) -> &ProcessEntry {
        &self.process_entry
    }

    pub fn as_path_entry(&self) -> &PathEntry {
        &self.path_entry
    }

    // =================Scanner========================================
    // Scan processed
    pub fn find_pid_and_command(&mut self) {
        self.process_entry = ProcessEntry::find_app_processes(&self.metadata);

        // debug list of the app process
        for _p in self.process_entry.list() {
            debug!(
                "list of process app: PID {}: cmd_line = '{}' name = '{}'",
                _p.pid(),
                _p.as_command(),
                _p.as_name()
            );
        }
    }

    // /// Scans package receipts and BOM metadata for the application.
    // ///
    // /// The progress callback reports the current scanning progress.
    // pub fn find_log_bom<F>(&mut self, progress: F)
    // where
    //     F: Fn(usize, &Path) + Send + Sync + Clone,
    // {
    //     self.app_log_receipt
    //         .scan_bom_files(&self.metadata, progress);
    // }

    /// Scans for filesystem paths associated with the application.
    ///
    /// Discovery includes traditional application files,
    /// sandbox containers, and BTM-related entries.
    ///
    /// The discovered paths are normalized, deduplicated,
    /// and stored inside `PathEntry`.
    ///
    /// The progress callback reports the current scanning progress.
    pub fn find_path_entry<F>(&mut self, progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        self.path_entry.scan_path_entry(&self.metadata, progress)
    }

    // ========================Setter==========================================
    /// Updates stored path information after a cleanup attempt.
    ///
    /// Paths provided in `failed` represent entries that could not
    /// be removed and are used to rebuild the remaining discovery
    /// state.
    pub fn update_path_entry(&mut self, failed: &[PathData]) {
        self.path_entry.update_entry(failed);
    }

    /// Clears all stored application state.
    ///
    /// Doc:
    /// Resets the profile back to its default empty state.
    ///
    /// All discovery results are discarded, including:
    ///
    /// - Application metadata.
    /// - Running processes.
    /// - Discovered filesystem entries including Package receipt BOM files.
    ///
    /// Note:
    /// After calling this method the profile behaves as if it
    /// had never been scanned.
    pub fn reset(&mut self) {
        self.metadata = Metadata::default();
        self.process_entry = ProcessEntry::default();
        self.path_entry = PathEntry::default()
    }
}
