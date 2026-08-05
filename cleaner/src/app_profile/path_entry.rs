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

//! Application path discovery and storage.
//!
//! This module defines `PathEntry`, the container responsible for
//! storing filesystem paths discovered for an application.
//!
//! The discovery system collects information from multiple sources,
//! including:
//!
//! - Application-data locations.
//! - Sandbox container directories.
//! - Background Task Management (BTM) locations.
//!
//! All discovered paths are merged into a single associated-path
//! collection and deduplicated before being stored.
//!
//! `PathEntry` stores:
//!
//! - The application bundle path.
//! - Discovered associated paths.
//!
//! The stored information is later used by cleanup workflows to
//! determine which filesystem locations should be removed.
//!
//! Note:
//! This module performs path discovery and storage only.
//! It does not perform file removal, Trash operations, or process
//! management.
//!..

use std::path::Path;
use std::path::PathBuf;

use crate::app_profile::metadata::AppMetadata;
use crate::path_data::PathData;

use crate::utility::BtmLocations;
use crate::utility::MatchRules;
use crate::utility::SandboxLocations;
use crate::utility::ScanLocations;
use crate::utility::construct_and_deduplicate_paths;
use crate::utility::scan_container;
use crate::utility::scan_general;

/// Application path inventory.
///
/// Doc:
/// Stores the application bundle together with all discovered
/// paths associated with that application.
///
/// A `PathEntry` typically contains:
///
/// - The application bundle itself.
/// - Associated application files.
/// - Sandbox container directories.
/// - Background Task Management (BTM) entries.
///
/// Associated paths are collected from multiple scanners and
/// normalized into a single deduplicated list.
///
/// Note:
/// This type stores discovery results only.
/// It does not perform file deletion or cleanup operations.
/// Sandbox containers, BTM entries, and traditional application
/// files are stored together inside `associated_paths` after
/// discovery and deduplication.
#[derive(Debug, Clone, Default)]
pub struct PathEntry {
    app_path: Option<PathData>,
    associated_paths: Vec<PathData>,
}

impl PathEntry {
    /// Creates a new path entry for an application.
    ///
    /// Doc:
    /// Initializes a `PathEntry` using the application bundle path
    /// and parsed application metadata.
    ///
    /// The application bundle is stored immediately while
    /// associated paths remain empty until a scan is performed.
    pub fn from_metadata(metadata: &AppMetadata) -> Self {
        let app_path = PathData::new(
            metadata.as_bundle_path().to_path_buf(),
            metadata.as_name().to_string(),
        );

        Self {
            app_path: Some(app_path),
            associated_paths: Vec::new(),
        }
    }

    //// get path reference
    pub fn as_app_path(&self) -> Option<&PathData> {
        self.app_path.as_ref()
    }

    //// get associated paths reference
    pub fn as_associated_paths(&self) -> &[PathData] {
        &self.associated_paths
    }

    /// Returns all discovered paths.
    ///
    /// Doc:
    /// Returns the application bundle together with every
    /// associated path currently stored in the entry.
    ///
    /// The returned list may contain:
    ///
    /// - Application bundles.
    /// - Associated files.
    /// - Sandbox containers.
    /// - BTM entries.
    pub fn all_paths(&self) -> Vec<PathData> {
        let mut paths = Vec::new();

        if let Some(app_path) = &self.app_path {
            paths.push(app_path.clone());
        }

        paths.extend(self.associated_paths.iter().cloned());

        let all_paths =
            construct_and_deduplicate_paths(paths, None, |item: &PathData| item.as_path());

        all_paths
    }

    // ===========================Scanner===============================
    /// Discovers paths associated with an application.
    ///
    /// Doc:
    /// Executes all path-discovery scanners and stores the
    /// combined results.
    ///
    /// Discovery currently includes:
    ///
    /// - Associated application files.
    /// - Sandbox container directories.
    /// - Background Task Management (BTM) entries.
    ///
    /// Results from all scanners are merged, normalized,
    /// deduplicated, and stored internally.
    ///
    /// The provided callback is invoked periodically to report
    /// scanning progress.
    ///
    /// Note:
    /// Existing associated-path results are replaced when the
    /// scan completes.
    pub fn scan_associated_paths<F>(&mut self, app_metadata: &AppMetadata, progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let mut associated_paths = Vec::new();

        let associated_files = self.scan_associated_files(app_metadata, progress.clone());
        let sandbox_container = self.scan_sandbox_container(app_metadata, progress.clone());
        let btm_files = self.scan_btm_files(app_metadata, progress);

        associated_paths.extend(associated_files);
        associated_paths.extend(sandbox_container);
        associated_paths.extend(btm_files);

        let filtered_associated_paths =
            construct_and_deduplicate_paths(associated_paths, None, |item: &PathData| {
                item.as_path()
            });

        self.set_associated_paths(filtered_associated_paths)
    }

    // ====================Setter====================

    /// Replaces the stored app path.
    pub fn set_app_path(&mut self, path: PathData) {
        self.app_path = Some(path);
    }

    /// Replaces the stored associated paths.
    pub fn set_associated_paths(&mut self, paths: Vec<PathData>) {
        self.associated_paths = paths;
    }

    /// Updates stored paths from a filtered result set.
    ///
    /// Doc:
    /// Replaces the current application and associated-path
    /// information using the provided list.
    ///
    /// This is primarily used after cleanup operations where
    /// some paths may have been removed while others remain.
    pub fn update_entry(&mut self, failed: &[PathData]) {
        let current_app_path = self.as_app_path().map(|p| p.as_path().to_path_buf());

        let app_path = current_app_path
            .as_ref()
            .and_then(|app| failed.iter().find(|item| item.as_path() == app).cloned());

        let associated_paths = failed
            .iter()
            .filter(|item| {
                current_app_path
                    .as_ref()
                    .is_none_or(|app| item.as_path() != app)
            })
            .cloned()
            .collect();

        self.app_path = app_path;
        self.associated_paths = associated_paths;
    }

    // ==================Internal Scanner=============

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
    fn scan_btm_files<F>(&mut self, app_metadata: &AppMetadata, progress: F) -> Vec<PathData>
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let locations_dir = BtmLocations::new();
        let locations_scan = locations_dir.all_paths();

        let matcher = |path: &Path| {
            MatchRules::new()
                .equal(app_metadata.as_name())
                .equal(app_metadata.as_bundle_executable_name())
                .contain(app_metadata.as_organization())
                .contain(app_metadata.as_bundle_id())
                .contain(app_metadata.as_alias_name())
                .check_path(path)
        };

        let builder = |path_buf: PathBuf| {
            let name = path_buf
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            PathData::new(path_buf, name)
        };

        let results: Vec<PathData> = scan_general(&locations_scan, 2, progress, matcher, builder);

        let filtered =
            construct_and_deduplicate_paths(results, None, |item: &PathData| item.as_path());

        filtered
    }

    /// Discovers traditional application files.
    ///
    /// Doc:
    /// Scans common application-data locations and attempts to
    /// identify files associated with the provided application.
    ///
    /// Typical matches may include:
    ///
    /// - Preferences.
    /// - Application Support data.
    /// - Cache files.
    /// - Logs.
    ///
    /// Matching is performed using application metadata.
    ///
    /// Note:
    /// Sandbox containers and BTM entries are discovered through
    /// separate scanners.
    fn scan_associated_files<F>(&mut self, app_metadata: &AppMetadata, progress: F) -> Vec<PathData>
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let locations_dir = ScanLocations::new();
        let locations_scan = locations_dir.as_paths();
        let matcher = |path: &Path| {
            MatchRules::new()
                .equal(app_metadata.as_name())
                .equal(app_metadata.as_bundle_executable_name())
                // .contain(app_metadata.as_organization())
                .contain(app_metadata.as_bundle_id())
                .contain(app_metadata.as_alias_name())
                .check_path(path)
        };
        let builder = |path_buf: PathBuf| {
            let name = path_buf
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            PathData::new(path_buf, name)
        };

        let asc_results: Vec<PathData> =
            scan_general(locations_scan, 3, progress, matcher, builder);

        let results =
            construct_and_deduplicate_paths(asc_results, None, |item: &PathData| item.as_path());

        results
    }

    /// Discovers sandbox container directories.
    ///
    /// Doc:
    /// Scans known sandbox container locations and attempts to
    /// identify containers belonging to the provided application.
    ///
    /// Matching is primarily performed using the application's
    /// bundle identifier.
    ///
    /// When a matching container is found, the container root
    /// directory is returned as the discovered path.
    ///
    /// Note:
    /// Container scanning is separate from general associated-file
    /// scanning because sandboxed applications use a different
    /// filesystem layout.
    fn scan_sandbox_container<F>(
        &mut self,
        app_metadata: &AppMetadata,
        progress: F,
    ) -> Vec<PathData>
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let containers_dir = SandboxLocations::new();
        let locations_scan = containers_dir.as_paths();
        let patterns = containers_dir.sandbox_pattern();

        let is_match = |path: &Path| {
            MatchRules::new()
                .contain(app_metadata.as_bundle_id())
                .contain(app_metadata.as_alias_name())
                .contain(app_metadata.as_bundle_executable_name())
                .check_path(path)
        };

        let builder = |container_dir: &Path, _file_path: &Path| {
            let folder_name = container_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let display_name = if folder_name == app_metadata.as_bundle_id() {
                folder_name
            } else {
                app_metadata.as_name().to_string()
            };

            PathData::new(container_dir.to_path_buf(), display_name)
        };

        let container_results: Vec<PathData> =
            scan_container(locations_scan, &patterns, progress, is_match, builder);

        let results =
            construct_and_deduplicate_paths(container_results, None, |item: &PathData| {
                item.as_path()
            });

        results
    }
}
