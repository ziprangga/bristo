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
//! - Package receipt and BOM information.
//! - Application related filesystem paths.
//!
//! The typical lifecycle is:
//!
//! 1. Create an `AppProfile` from an application path.
//! 2. Discover running processes.
//! 3. Scan package receipts and BOM files.
//! 4. Scan associated application files.
//! 5. Scan BTM (Background Task Management) files.
//! 6. Retrieve discovered filesystem entries.
//!
//! The module separates application information into dedicated
//! containers:
//!
//! - `AppMetadata` stores application information.
//! - `AppProcs` stores discovered running processes.
//! - `AppLogReceipt` stores package receipt and BOM metadata.
//! - `PathEntry` stores application, associated, and BTM paths.
//!
//! `AppProfile` provides a single aggregate state used by the
//! cleanup workflow while delegating discovery details to the
//! specialized containers responsible for each data category.
//!
//! Note:
//! Most callers should interact with `AppProfile` rather than the
//! individual storage types directly. Lower-level types exist to
//! organize discovery results and scanning logic.
//!..

mod bom_receipt;
mod info_plist;
mod metadata;
mod path_entry;
mod processed;

pub use bom_receipt::AppLogReceipt;
pub use info_plist::InfoPlist;
pub use metadata::AppMetadata;
pub use path_entry::PathEntry;

pub use processed::{AppProcs, Proc};

use crate::errors::Result;
use crate::locations_scan::BtmLocations;
use crate::locations_scan::ReceiptsLocations;
use crate::locations_scan::ScanLocations;
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
/// - Package receipt information.
/// - BOM metadata.
/// - Discovered filesystem paths.
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
/// PathEntry
///      │
///      ├─ Application bundle
///      ├─ Associated files
///      └─ BTM files
/// ```
///
/// Once populated, `PathEntry` provides access to all discovered
/// filesystem locations related to the application.
///
/// Note:
/// `AppProfile` represents mutable discovery state. It is commonly
/// owned by `Cleaner`, which provides the higher-level cleanup
/// orchestration.
#[derive(Debug, Default, Clone)]
pub struct AppProfile {
    app_metadata: AppMetadata,
    app_procs: AppProcs,
    app_log_receipt: AppLogReceipt,
    path_entry: PathEntry,
}

impl AppProfile {
    pub fn new(
        app_metadata: AppMetadata,
        app_procs: AppProcs,
        app_log_receipt: AppLogReceipt,
        path_entry: PathEntry,
    ) -> Self {
        Self {
            app_metadata,
            app_procs,
            app_log_receipt,
            path_entry,
        }
    }

    pub fn from_path(app_path: &Path) -> Result<Self> {
        let app_metadata = AppMetadata::from_path(app_path)?;
        let path_entry = PathEntry::from_path_and_metadata(app_path, &app_metadata);

        Ok(Self {
            app_metadata: app_metadata,
            app_procs: AppProcs::default(),
            app_log_receipt: AppLogReceipt::default(),
            path_entry: path_entry,
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

    /// Scans package receipts and BOM metadata for the application.
    ///
    /// The progress callback reports the current scanning progress.
    pub fn find_log_bom<F>(&mut self, locations: &ReceiptsLocations, progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        self.app_log_receipt
            .scan_bom_files(&self.app_metadata, locations, progress);
    }

    /// Scans filesystem locations for files associated with the application.
    ///
    /// The discovered paths are stored inside `PathEntry`.
    ///
    /// The progress callback reports the current scanning progress.
    pub fn find_associate_files<F>(&mut self, locations: &ScanLocations, progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        self.path_entry
            .scan_asc_files(&self.app_metadata, locations, progress);
    }

    /// Scans Background Task Management (BTM) locations.
    ///
    /// The discovered paths are stored inside `PathEntry`.
    ///
    /// The progress callback reports the current scanning progress.
    pub fn find_btm_files<F>(&mut self, locations: &BtmLocations, progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        self.path_entry
            .scan_btm_files(&self.app_metadata, locations, progress);
    }

    pub fn path_entry(&self) -> &PathEntry {
        &self.path_entry
    }

    /// Updates stored filesystem entries using failed cleanup paths.
    ///
    /// Paths provided in `failed` are used to rebuild the remaining
    /// application entries after a cleanup operation.
    pub fn update_path_entry(&mut self, failed: &[PathData]) {
        self.path_entry.update_entry(failed);
    }
    // pub fn path_entry_mut(&mut self) -> &mut PathEntry {
    //     &mut self.path_entry
    // }

    /// Clears all stored application state.
    ///
    /// Doc:
    /// Resets the profile back to its default empty state.
    ///
    /// All discovery results are discarded, including:
    ///
    /// - Metadata.
    /// - Processes.
    /// - BOM Receipts.
    /// - Discovered filesystem paths.
    ///
    /// Note:
    /// After calling this method the profile behaves as if it
    /// had never been scanned.
    pub fn reset(&mut self) {
        self.app_metadata = AppMetadata::default();
        self.app_procs = AppProcs::default();
        self.app_log_receipt = AppLogReceipt::default();
        self.path_entry = PathEntry::default()
    }
}
