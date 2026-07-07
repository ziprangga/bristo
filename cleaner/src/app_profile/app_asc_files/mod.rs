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
//! Associated application file discovery and storage.
//!
//! This module is responsible for locating and storing files that are
//! associated with an application outside of the application bundle
//! itself.
//!
//! Associated files may include:
//!
//! - Preferences.
//! - Application Support data.
//! - Cache files.
//! - Logs.
//! - Sandbox containers.
//! - Vendor-specific data directories.
//!
//! The module is built around two primary types:
//!
//! - `AppAscFiles` stores discovered associated files.
//! - `AscData` represents a single discovered entry.
//!
//! Discovery is performed using a combination of:
//!
//! - General filesystem scanning.
//! - Sandbox container scanning.
//! - Application metadata matching.
//!
//! Matching rules are derived from application metadata such as:
//!
//! - Application name.
//! - Executable name.
//! - Organization name.
//! - Bundle identifier.
//!
//! Results from multiple scanning strategies are merged and
//! normalized before being stored.
//!
//! Note:
//! This module only discovers associated files. It does not perform
//! file deletion or cleanup operations. Those responsibilities belong
//! to higher-level components such as `AppProfile` and `Cleaner`.
//!..

mod asc_data;
pub use asc_data::AscData;

use crate::app_profile::app_metadata::AppMetadata;
use crate::locations_scan::{SandboxLocations, ScanLocations};
use crate::rules::MatchRules;

use crate::scanner::construct_scanner_result;
use crate::scanner::scan_container;
use crate::scanner::scan_general;
use std::path::Path;
use std::path::PathBuf;

/// Collection of associated application files.
///
/// Doc:
/// Stores all filesystem entries discovered during associated-file
/// scanning.
///
/// Associated files represent data that belongs to an application
/// but exists outside the application bundle itself.
///
/// Examples:
///
/// - `~/Library/Application Support/*`
/// - `~/Library/Preferences/*`
/// - `~/Library/Caches/*`
/// - Sandbox container directories
///
/// The collection is populated through `scan_asc_files()` and can
/// later be consumed by reporting, UI, or cleanup operations.
///
/// Note:
/// Entries are stored as `AscData` values and may originate from
/// multiple scanning strategies.
#[derive(Debug, Default, Clone)]
pub struct AppAscFiles {
    asc_files: Vec<AscData>,
}

impl AppAscFiles {
    /// Contruct AscFiles
    pub fn new(asc_files: &[AscData]) -> Self {
        Self {
            asc_files: asc_files.to_vec(),
        }
    }
    //// reference of associate files
    pub fn as_asc_files(&self) -> &[AscData] {
        &self.asc_files
    }

    /// Update associate_files with given list
    pub fn set_asc_files(&mut self, asc_data: Vec<AscData>) {
        self.asc_files = asc_data;
    }

    // Scan all file associate from list of location
    // for huge directory and try using walkdir + rayon
    // use in_progress as emitter status to caller
    //
    // Design:
    //
    // General scanning finds traditional application files
    // (Preferences, Caches, Application Support, etc).
    //
    // Container scanning exists separately because sandboxed
    // applications often store data under container directories
    // that require different matching and result construction.
    //
    // Both result sets are merged and deduplicated through
    // construct_scanner_result().
    pub fn scan_asc_files<F>(
        &mut self,
        app_metadata: &AppMetadata,
        locations: &ScanLocations,
        in_progress: F,
    ) where
        F: Fn(usize, &Path) + Send + Sync,
    {
        let main_results: Vec<AscData> = scan_general(
            locations.as_paths(),
            3,
            |n, path| in_progress(n, path),
            |path| {
                MatchRules::new()
                    .equal(app_metadata.as_info().as_name())
                    .equal(app_metadata.as_info().as_bundle_executable_name())
                    .equal(app_metadata.as_info().as_organization())
                    .contain(app_metadata.as_info().as_bundle_id())
                    .check(path)
            },
            |path_buf: PathBuf| {
                let name = path_buf
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                AscData::new(path_buf, name)
            },
        );

        let containers_dir = SandboxLocations::new();
        let patterns = containers_dir.sandbox_pattern();

        let container_results: Vec<AscData> = scan_container(
            containers_dir.as_paths(),
            &patterns,
            |path| {
                MatchRules::new()
                    .contain(app_metadata.as_info().as_bundle_id())
                    .check(path)
            },
            |container_dir, _file_path| {
                let folder_name = container_dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let display_name = if folder_name == app_metadata.as_info().as_bundle_id() {
                    folder_name
                } else {
                    app_metadata.as_info().as_name().to_string()
                };

                AscData::new(container_dir.to_path_buf(), display_name)
            },
        );

        let results =
            construct_scanner_result(main_results, Some(container_results), |item: &AscData| {
                item.as_path()
            });

        self.set_asc_files(results);
    }
}
