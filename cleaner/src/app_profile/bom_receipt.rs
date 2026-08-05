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
//! Package receipt and BOM file discovery.
//!
//! This module is responsible for locating and storing package
//! receipt information associated with an application.
//!
//! The module is built around two primary types:
//!
//! - `AppLogReceipt` stores discovered receipt records.
//! - `PathData` represents a single receipt or BOM entry.
//!
//! Discovery focuses on macOS package receipts, particularly
//! BOM (Bill of Materials) files generated during package
//! installation.
//!
//! At present, discovery is limited to BOM receipt files.
//!
//! Receipt information is useful for:
//!
//! - Installation auditing.
//! - Package inspection.
//! - Debugging cleanup operations.
//! - Exporting installation manifests.
//!
//! Matching is performed using metadata derived from the target
//! application, including:
//!
//! - Application name.
//! - Executable name.
//! - Organization identifier.
//! - Bundle identifier.
//!
//! Note:
//! Receipt discovery provides installation records only.
//! It does not indicate whether files currently exist on disk
//! or whether they are actively used by the application.
//!..

use std::path::{Path, PathBuf};

use crate::app_profile::metadata::AppMetadata;
use crate::path_data::PathData;
use crate::utility::MatchRules;
use crate::utility::ReceiptsLocations;
use crate::utility::construct_and_deduplicate_paths;
use crate::utility::scan_general;

/// Collection of application receipt records.
///
/// Doc:
/// Stores receipt and BOM files associated with an application.
///
/// Receipt records represent installation metadata created by
/// package installers and system package management tools.
///
/// The collection is populated through `scan_bom_files()` and
/// can later be used for reporting, auditing, or BOM export
/// operations.
///
/// Note:
/// Receipt records are independent from associated files,
/// BTM files, and runtime processes.
#[derive(Debug, Default, Clone)]
pub struct AppLogReceipt {
    bom_files: Vec<PathData>,
}

impl AppLogReceipt {
    /// Creates a new receipt collection.
    pub fn new(bom_files: &[PathData]) -> Self {
        Self {
            bom_files: bom_files.to_vec(),
        }
    }

    /// Returns all discovered BOM files.
    pub fn as_bom_files(&self) -> &[PathData] {
        &self.bom_files
    }

    /// Returns the number of discovered BOM files.
    pub fn count(&self) -> usize {
        self.bom_files.len()
    }

    /// Returns true when no BOM files have been discovered.
    pub fn is_empty(&self) -> bool {
        self.bom_files.is_empty()
    }

    /// Updates the collection with the provided BOM files.
    pub fn set_bom_files(&mut self, btm_data: Vec<PathData>) {
        self.bom_files = btm_data;
    }

    /// Discovers BOM receipt files.
    ///
    /// Doc:
    /// Scans known package receipt locations and attempts to locate
    /// BOM files associated with the provided application.
    ///
    /// Only files with the `.bom` extension are considered.
    ///
    /// Matching is performed using application metadata,
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
    /// Design:
    /// Receipt scanning is intentionally separate from associated
    /// file and BTM scanning.
    ///
    /// Receipt records describe installation history rather than
    /// application state.
    ///
    /// A receipt may exist even when the application itself has
    /// already been removed.
    ///
    /// Likewise, an application may exist without a corresponding
    /// receipt when it was installed manually rather than through
    /// a package installer.
    ///
    /// Separating receipt discovery from other scanning categories
    /// allows callers to treat installation metadata independently
    /// from filesystem cleanup data.
    ///
    /// Note:
    /// Existing receipt records are replaced when scanning
    /// completes.
    pub fn scan_bom_files<F>(&mut self, app_metadata: &AppMetadata, progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        self.bom_files.clear();

        let locations_dir = ReceiptsLocations::new();
        let locations_scan = locations_dir.as_paths();

        let matcher = |path: &Path| {
            path.extension().map(|ext| ext == "bom").unwrap_or(false)
                && MatchRules::new()
                    .contain(app_metadata.as_name())
                    .contain(app_metadata.as_bundle_executable_name())
                    .contain(app_metadata.as_organization())
                    .contain(app_metadata.as_bundle_id())
                    .check_path(&path)
        };

        let builder = |path_buf: PathBuf| {
            let name = path_buf
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            PathData::new(path_buf, name)
        };

        let results: Vec<PathData> = scan_general(locations_scan, 1, progress, matcher, builder);

        let filtered =
            construct_and_deduplicate_paths(results, None, |item: &PathData| item.as_path());

        self.set_bom_files(filtered);
    }
}
