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
//! Background Task Management (BTM) file discovery and storage.
//!
//! This module is responsible for locating and storing files that
//! may be related to an application's background services,
//! login items, launch agents, launch daemons, and other
//! persistent system integrations.
//!
//! The module is built around two primary types:
//!
//! - `AppBtmFiles` stores discovered BTM-related entries.
//! - `BtmData` represents a single discovered entry.
//!
//! Discovery is performed by scanning known BTM-related locations
//! and applying application-specific matching rules.
//!
//! Matching rules are derived from application metadata such as:
//!
//! - Application name.
//! - Executable name.
//! - Organization identifier.
//! - Bundle identifier.
//!
//! Results are normalized and deduplicated before being stored.
//!
//! BTM files are tracked separately from general associated files
//! because they often represent application persistence mechanisms
//! rather than user-generated data.
//!
//! Note:
//! This module is responsible only for discovery and storage.
//! Cleanup operations are performed by higher-level components such
//! as `AppProfile` and `Cleaner`.
//!..

mod btm_data;
pub use btm_data::BtmData;

use crate::app_profile::app_metadata::AppMetadata;
use crate::locations_scan::BtmLocations;
use crate::rules::MatchRules;

use crate::scanner::construct_scanner_result;
use crate::scanner::scan_general;
use std::path::Path;
use std::path::PathBuf;

/// Collection of discovered BTM-related files.
///
/// Doc:
/// Stores filesystem entries associated with an application's
/// background execution and persistence mechanisms.
///
/// Examples may include:
///
/// - Login item registrations.
/// - LaunchAgent files.
/// - LaunchDaemon files.
/// - Background service configuration files.
/// - Other system-managed persistence entries.
///
/// The collection is populated through `scan_btm_files()` and
/// later consumed by cleanup and reporting operations.
///
/// Note:
/// Entries are stored as `BtmData` values and represent
/// discovered filesystem resources rather than running
/// processes.
#[derive(Debug, Default, Clone)]
pub struct AppBtmFiles {
    btm_files: Vec<BtmData>,
}

impl AppBtmFiles {
    /// Contruct BtmFiles
    pub fn new(btm_files: &[BtmData]) -> Self {
        Self {
            btm_files: btm_files.to_vec(),
        }
    }

    //// reference of btm files
    pub fn as_btm_files(&self) -> &[BtmData] {
        &self.btm_files
    }

    /// Update btm files with given list
    pub fn set_btm_files(&mut self, btm_data: Vec<BtmData>) {
        self.btm_files = btm_data;
    }

    /// Discovers BTM-related files.
    ///
    /// Doc:
    /// Scans known Background Task Management locations and
    /// attempts to identify entries belonging to the provided
    /// application.
    ///
    /// Design:
    ///
    /// BTM scanning is intentionally separated from associated-file
    /// scanning.
    ///
    /// Associated files primarily represent user/application data,
    /// while BTM files represent persistence mechanisms that allow
    /// applications to execute automatically or integrate with
    /// system background services.
    ///
    /// Keeping these categories separate allows cleanup operations
    /// to apply different policies if needed in the future.
    ///
    /// Matching is performed using metadata-derived rules,
    /// including:
    ///
    /// - Application name.
    /// - Executable name.
    /// - Organization identifier.
    /// - Bundle identifier.
    ///
    /// Matching results are normalized, deduplicated, and stored
    /// internally.
    ///
    /// The provided callback is invoked periodically to report
    /// scanning progress.
    ///
    /// Note:
    /// Existing BTM results are replaced when the scan completes.
    pub fn scan_btm_files<F>(
        &mut self,
        app_metadata: &AppMetadata,
        locations: &BtmLocations,
        in_progress: F,
    ) where
        F: Fn(usize, &Path) + Send + Sync,
    {
        let results: Vec<BtmData> = scan_general(
            &locations.all_paths(),
            2,
            |n, path| in_progress(n, path),
            |path| {
                MatchRules::new()
                    .equal(app_metadata.as_info().as_name())
                    .equal(app_metadata.as_info().as_bundle_executable_name())
                    .contain(app_metadata.as_info().as_organization())
                    .contain(app_metadata.as_info().as_bundle_id())
                    .check(path)
            },
            |path_buf: PathBuf| {
                let name = path_buf
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                BtmData::new(path_buf, name)
            },
        );

        let filtered = construct_scanner_result(results, None, |item: &BtmData| item.as_path());

        self.set_btm_files(filtered);
    }
}
