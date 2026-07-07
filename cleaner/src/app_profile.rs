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

//! Doc:
//! Application profile and discovery state.
//!
//! This module defines the core data structures used to represent
//! an application and all information discovered during scanning.
//!
//! The module is built around two primary concepts:
//!
//! - `AppProfile` stores the complete application state.
//! - `FileEntry` provides a unified representation of discovered files.
//!
//! An `AppProfile` acts as the central aggregation point for:
//!
//! - Application metadata.
//! - Running processes.
//! - Package receipt information.
//! - Associated files.
//! - Background task management (BTM) files.
//!
//! The typical lifecycle is:
//!
//! 1. Create an `AppProfile` from an application path.
//! 2. Discover running processes.
//! 3. Scan package receipts and BOM files.
//! 4. Scan associated files.
//! 5. Scan BTM files.
//! 6. Retrieve a merged list of discovered entries.
//!
//! The module intentionally separates discovered data into dedicated
//! containers (`AppMetadata`, `AppProcs`, `AppLogReceipt`,
//! `AppAscFiles`, and `AppBtmFiles`) while exposing a single
//! aggregate view through `AppProfile`.
//!
//! `FileEntry` exists as a common abstraction for application bundles,
//! associated files, and BTM files, allowing callers to process all
//! discovered items through a unified interface.
//!
//! Note:
//! Most callers should interact with `AppProfile` rather than the
//! individual storage types directly. The lower-level types primarily
//! exist to organize discovery results and scanning logic.
//!..

mod app_asc_files;
mod app_btm;
mod app_log_receipt;
mod app_metadata;
mod app_proc;

pub use app_asc_files::{AppAscFiles, AscData};
pub use app_btm::{AppBtmFiles, BtmData};
pub use app_log_receipt::AppLogReceipt;
pub use app_metadata::{AppMetadata, InfoPlist};
pub use app_proc::{AppProcs, Proc};

use crate::locations_scan::BtmLocations;
use crate::locations_scan::ReceiptsLocations;
use crate::locations_scan::ScanLocations;
use crate::scanner::construct_scanner_result;
use anyhow::Result;
use mini_logger::debug;
use std::path::Path;

/// Unified representation of a discovered filesystem entry.
///
/// Doc:
/// Represents a file or directory associated with an application.
///
/// Variants:
///
/// - `AppPath` represents the application bundle itself.
/// - `AscFiles` represents associated application files.
/// - `BtmFiles` represents background task management files.
///
/// The type provides a common interface for retrieving names
/// and filesystem paths regardless of the underlying source.
///
/// This abstraction allows scanning, reporting, UI rendering,
/// and cleanup operations to operate on a single collection of
/// entries without needing to know their specific category.
///
/// Note:
/// `FileEntry` is primarily used as the merged output of
/// `AppProfile::all_entries()`.
#[derive(Debug, Clone)]
pub enum FileEntry {
    AppPath(AppMetadata),
    AscFiles(AscData),
    BtmFiles(BtmData),
}

impl FileEntry {
    pub fn as_path(&self) -> &Path {
        match self {
            Self::AppPath(v) => v.as_path(),
            Self::AscFiles(v) => v.as_path(),
            Self::BtmFiles(v) => v.as_path(),
        }
    }

    pub fn as_name(&self) -> &str {
        match self {
            Self::AppPath(v) => v.as_info().as_name(),
            Self::AscFiles(v) => v.as_name(),
            Self::BtmFiles(v) => v.as_name(),
        }
    }
}

/// Aggregated application discovery state.
///
/// Doc:
/// Stores all information known about a scanned application.
///
/// An `AppProfile` acts as the central model used throughout
/// the scanning and cleanup workflow.
///
/// Stored information includes:
///
/// - Application metadata.
/// - Running processes.
/// - Package receipt information.
/// - Associated files.
/// - Background task management files.
///
/// Discovery operations progressively populate the profile:
///
/// ```text
/// AppMetadata
///      │
///      ▼
/// AppProcs
///      │
///      ▼
/// AppLogReceipt
///      │
///      ▼
/// AppAscFiles
///      │
///      ▼
/// AppBtmFiles
/// ```
///
/// Once populated, the profile can provide a merged view of
/// all discovered filesystem entries through `all_entries()`.
///
/// The profile itself performs discovery coordination by
/// delegating scanning work to the specialized storage types
/// responsible for each category.
///
/// Note:
/// `AppProfile` is designed to be a mutable scanning state.
/// It is commonly owned by `Cleaner`, which provides the
/// higher-level cleanup workflow.
#[derive(Debug, Default, Clone)]
pub struct AppProfile {
    app_metadata: AppMetadata,
    app_procs: AppProcs,
    app_log_receipt: AppLogReceipt,
    app_asc_files: AppAscFiles,
    app_btm_files: AppBtmFiles,
}

impl AppProfile {
    pub fn new(
        app_metadata: AppMetadata,
        app_procs: AppProcs,
        app_log_receipt: AppLogReceipt,
        app_asc_files: AppAscFiles,
        app_btm_files: AppBtmFiles,
    ) -> Self {
        Self {
            app_metadata,
            app_procs,
            app_log_receipt,
            app_asc_files,
            app_btm_files,
        }
    }

    pub fn from_path(app_path: &Path) -> Result<Self> {
        let app_metadata = AppMetadata::from_path(app_path)?;

        Ok(Self {
            app_metadata: app_metadata,
            app_procs: AppProcs::default(),
            app_log_receipt: AppLogReceipt::default(),
            app_asc_files: AppAscFiles::default(),
            app_btm_files: AppBtmFiles::default(),
        })
    }

    pub fn as_app_metadata(&self) -> &AppMetadata {
        &self.app_metadata
    }

    pub fn as_app_procs(&self) -> &AppProcs {
        &self.app_procs
    }

    pub fn as_app_log_receipt(&self) -> &AppLogReceipt {
        &self.app_log_receipt
    }

    pub fn as_app_asc_files(&self) -> &AppAscFiles {
        &self.app_asc_files
    }

    pub fn as_app_btm_files(&self) -> &AppBtmFiles {
        &self.app_btm_files
    }

    pub fn find_pid_and_command(&mut self) {
        self.app_procs = AppProcs::find_app_processes(&self.app_metadata);

        // debug list of the app process
        for _p in self.app_procs.list() {
            debug!(
                "list of process app: PID {}: cmd_line = '{}' name = '{}'",
                _p.pid(),
                _p.as_command(),
                _p.as_name()
            );
        }
    }

    pub fn find_log_bom<F>(&mut self, locations: &ReceiptsLocations, in_progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync,
    {
        self.app_log_receipt
            .scan_bom_files(&self.app_metadata, locations, in_progress);
    }

    // Scan all file associate from list of location
    // use in_progress as emitter status to caller
    pub fn find_associate_files<F>(&mut self, locations: &ScanLocations, in_progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync,
    {
        self.app_asc_files
            .scan_asc_files(&self.app_metadata, locations, in_progress);
    }

    // Scan all file btm from list of location
    // use in_progress as emitter status to caller
    pub fn find_btm_files<F>(&mut self, locations: &BtmLocations, in_progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync,
    {
        self.app_btm_files
            .scan_btm_files(&self.app_metadata, locations, in_progress);
    }

    /// Returns all discovered filesystem entries.
    ///
    /// Doc:
    /// Produces a merged collection containing:
    ///
    /// - Associated files.
    /// - BTM files.
    /// - The application bundle itself.
    ///
    /// The resulting collection is normalized through
    /// `construct_scanner_result()` before being returned.
    ///
    /// This method is typically used by:
    ///
    /// - Cleanup operations.
    /// - Reporting utilities.
    /// - User interfaces.
    /// - Export functionality.
    ///
    /// Note:
    /// The returned collection represents the current state of
    /// the profile and reflects any modifications made through
    /// `replace_file_entries()`.
    pub fn all_entries(&self) -> Vec<FileEntry> {
        let mut entries = Vec::new();

        // AscFiles
        entries.extend(
            self.app_asc_files
                .as_asc_files()
                .iter()
                .cloned()
                .map(FileEntry::AscFiles),
        );

        // BtmFiles
        entries.extend(
            self.app_btm_files
                .as_btm_files()
                .iter()
                .cloned()
                .map(FileEntry::BtmFiles),
        );

        // AppPath
        entries.push(FileEntry::AppPath(self.app_metadata.clone()));

        construct_scanner_result(entries, None, |entry| entry.as_path())
    }

    /// Replaces discovered file entries.
    ///
    /// Doc:
    /// Updates the profile using a new collection of `FileEntry`
    /// values.
    ///
    /// Entries are automatically separated into their respective
    /// storage containers:
    ///
    /// - Application metadata.
    /// - Associated files.
    /// - BTM files.
    ///
    /// This method is primarily used after cleanup operations
    /// when only a subset of entries remain available.
    ///
    /// Note:
    /// Entries not present in the provided collection are removed
    /// from the profile state.
    pub fn replace_file_entries(&mut self, entries: Vec<FileEntry>) {
        let mut app_metadata = None;
        let mut asc_files = Vec::new();
        let mut btm_files = Vec::new();

        for entry in entries {
            match entry {
                FileEntry::AppPath(app) => {
                    app_metadata = Some(app);
                }

                FileEntry::BtmFiles(file) => {
                    btm_files.push(file);
                }

                FileEntry::AscFiles(file) => {
                    asc_files.push(file);
                }
            }
        }

        if let Some(app) = app_metadata {
            self.app_metadata = app;
        }

        self.app_asc_files.set_asc_files(asc_files);
        self.app_btm_files.set_btm_files(btm_files);
    }

    /// Clears all stored application state.
    ///
    /// Doc:
    /// Resets the profile back to its default empty state.
    ///
    /// All discovery results are discarded, including:
    ///
    /// - Metadata.
    /// - Processes.
    /// - Receipts.
    /// - Associated files.
    /// - BTM files.
    ///
    /// Note:
    /// After calling this method the profile behaves as if it
    /// had never been scanned.
    pub fn reset(&mut self) {
        self.app_metadata = AppMetadata::default();
        self.app_procs = AppProcs::default();
        self.app_log_receipt = AppLogReceipt::default();
        self.app_asc_files = AppAscFiles::default();
        self.app_btm_files = AppBtmFiles::default();
    }
}
