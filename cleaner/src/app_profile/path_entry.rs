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
//! discovering and storing filesystem paths associated with an
//! application.
//!
//! The discovery system collects information from multiple sources,
//! including:
//!
//! - Application-data locations.
//! - Sandbox container directories.
//! - Background Task Management locations.
//! - Installer receipt BOM (Bill of Materials) files.
//!
//! Each discovery category is stored independently, allowing
//! cleanup and reporting operations to treat different resource
//! types separately.
//!
//! `PathEntry` stores:
//!
//! - The application bundle path.
//! - General associated files.
//! - Sandbox container directories.
//! - Background Task Management files.
//! - Installer receipt BOM files.
//!
//! Note:
//! This module performs path discovery and storage only.
//! It does not perform file removal, Trash operations, or process
//! management.
//!..

use std::path::Path;
use std::path::PathBuf;

use crate::app_profile::metadata::Metadata;
use crate::path_data::PathData;

use crate::utility::BackgroundTaskLocations;
use crate::utility::GeneralLocations;
use crate::utility::MatchRules;
use crate::utility::ReceiptsLocations;
use crate::utility::SandboxLocations;
use crate::utility::construct_and_deduplicate_paths;
use crate::utility::scan_container;
use crate::utility::scan_general;

/// Application path inventory.
///
/// Doc:
/// Stores the application bundle together with every filesystem
/// resource discovered for that application.
///
/// A `PathEntry` typically contains:
///
/// - The application bundle itself.
/// - General associated files.
/// - Sandbox container directories.
/// - Background Task Management files.
/// - Installer receipt BOM (Bill of Materials) files.
///
/// Each category is maintained independently to allow different
/// discovery, reporting, and cleanup behavior.
///
/// Note:
/// This type stores discovery results only.
/// It does not perform file deletion, Trash operations, or other
/// cleanup tasks.
#[derive(Debug, Clone, Default)]
pub struct PathEntry {
    /// Application bundle path.
    ///
    /// Doc:
    /// Stores the primary application bundle represented by this
    /// entry.
    ///
    /// Note:
    /// This field is `None` when the application bundle is no
    /// longer present or has been removed during cleanup.
    app_path: Option<PathData>,

    /// Installer receipt files.
    ///
    /// Doc:
    /// Stores discovered macOS package receipt files associated
    /// with the application.
    ///
    /// Discovery focuses on installer-generated BOM (Bill of
    /// Materials) files used to reconstruct the list of files
    /// originally installed by a package.
    ///
    /// Note:
    /// BOM files are installation metadata maintained by the
    /// operating system. They are stored separately from
    /// associated filesystem paths because they describe package
    /// installation rather than application resources.
    bom_files: Vec<PathData>,

    /// General associated filesystem paths.
    ///
    /// Doc:
    /// Stores traditional filesystem resources discovered as
    /// belonging to the application.
    ///
    /// Typical entries include:
    ///
    /// - Application Support files.
    /// - Preferences.
    /// - Caches.
    /// - Logs.
    ///
    /// Note:
    /// Sandbox containers, Background Task Management files, and
    /// installer receipt BOM files are stored separately because
    /// they represent distinct categories of application
    /// resources.
    general_associated_files: Vec<PathData>,

    /// Background Task Management files.
    ///
    /// Doc:
    /// Stores filesystem entries associated with Background Task
    /// Management.
    ///
    /// These files allow applications to participate in macOS
    /// background execution mechanisms and are discovered
    /// separately from general application data.
    ///
    /// Note:
    /// Background Task Management files are maintained as their
    /// own category because they may require different cleanup
    /// policies from ordinary application resources.
    background_task_files: Vec<PathData>,

    /// Sandbox container directories.
    ///
    /// Doc:
    /// Stores sandbox container directories associated with the
    /// application.
    ///
    /// Discovery includes standard application containers and
    /// group containers when applicable.
    ///
    /// Note:
    /// Sandbox containers use a different filesystem layout from
    /// traditional application data and are therefore discovered
    /// independently.
    sandbox_container: Vec<PathData>,
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
    pub fn from_metadata(metadata: &Metadata) -> Self {
        let app_path = PathData::new(
            metadata.as_bundle_path().to_path_buf(),
            metadata.as_name().to_string(),
        );

        Self {
            app_path: Some(app_path),
            bom_files: Vec::new(),
            general_associated_files: Vec::new(),
            background_task_files: Vec::new(),
            sandbox_container: Vec::new(),
        }
    }

    /// get path reference
    pub fn as_app_path(&self) -> Option<&PathData> {
        self.app_path.as_ref()
    }

    /// Returns all discovered BOM files.
    pub fn as_bom_files(&self) -> &[PathData] {
        &self.bom_files
    }

    /// get associated paths reference
    pub fn as_general_associated_files(&self) -> &[PathData] {
        &self.general_associated_files
    }

    /// get associated paths reference
    pub fn as_background_task_files(&self) -> &[PathData] {
        &self.background_task_files
    }

    /// get associated paths reference
    pub fn as_sandbox_container(&self) -> &[PathData] {
        &self.sandbox_container
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
    /// - background task management entries.
    /// - Installer receipt BOM (Bill of Materials) files.
    pub fn all_paths(&self) -> Vec<PathData> {
        let mut paths = Vec::new();

        if let Some(app_path) = &self.app_path {
            paths.push(app_path.clone());
        }

        paths.extend(self.bom_files.iter().cloned());
        paths.extend(self.general_associated_files.iter().cloned());
        paths.extend(self.background_task_files.iter().cloned());
        paths.extend(self.sandbox_container.iter().cloned());

        let all_paths = construct_and_deduplicate_paths(paths, |item: &PathData| item.as_path());

        all_paths
    }

    // ===========================Scanner===============================
    /// Executes every path-discovery scanner and stores the
    /// resulting path collections.
    ///
    /// Discovery currently includes:
    ///
    /// - General associated files.
    /// - Sandbox container directories.
    /// - Background Task Management files.
    /// - Installer receipt BOM (Bill of Materials) files.
    ///
    /// Each scanner returns a normalized and deduplicated result for
    /// its own category. Those results are then stored
    /// independently.
    ///
    /// The provided callback is invoked periodically to report
    /// scanning progress.
    ///
    /// Note:
    /// Existing discovery results are replaced when the scan
    /// completes.
    pub fn scan_path_entry<F>(&mut self, metadata: &Metadata, progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let bom_files = self.scan_bom_files(metadata, progress.clone());
        let general_associated_files =
            self.scan_general_associated_files(metadata, progress.clone());
        let background_task_files = self.scan_background_task_files(metadata, progress.clone());
        let sandbox_container = self.scan_sandbox_container(metadata, progress.clone());

        self.set_bom_files(bom_files);
        self.set_general_associated_files(general_associated_files);
        self.set_background_task_files(background_task_files);
        self.set_sandbox_container(sandbox_container);
    }

    // ====================Setter====================

    /// Replaces the stored app path.
    pub fn set_app_path(&mut self, path: PathData) {
        self.app_path = Some(path);
    }

    /// Updates the collection with the provided BOM files.
    pub fn set_bom_files(&mut self, btm_data: Vec<PathData>) {
        self.bom_files = btm_data;
    }

    /// Replaces the stored associated paths.
    pub fn set_general_associated_files(&mut self, paths: Vec<PathData>) {
        self.general_associated_files = paths;
    }

    /// Replaces the stored associated paths.
    pub fn set_background_task_files(&mut self, paths: Vec<PathData>) {
        self.background_task_files = paths;
    }

    /// Replaces the stored associated paths.
    pub fn set_sandbox_container(&mut self, paths: Vec<PathData>) {
        self.sandbox_container = paths;
    }

    /// Updates stored discovery results.
    ///
    /// Doc:
    /// Replaces the current application bundle and every
    /// discovery category using the provided list of remaining
    /// paths.
    ///
    /// This is primarily used after cleanup operations where
    /// some filesystem entries were successfully removed while
    /// others remain.
    ///
    /// Each stored category is updated independently, preserving
    /// the original classification of every remaining path.
    pub fn update_entry(&mut self, failed: &[PathData]) {
        let current_app_path = self.as_app_path().map(|p| p.as_path().to_path_buf());

        let current_bom_paths: Vec<PathBuf> = self
            .bom_files
            .iter()
            .map(|path| path.as_path().to_path_buf())
            .collect();

        let current_general_paths: Vec<PathBuf> = self
            .general_associated_files
            .iter()
            .map(|item| item.as_path().to_path_buf())
            .collect();

        let current_background_paths: Vec<PathBuf> = self
            .background_task_files
            .iter()
            .map(|item| item.as_path().to_path_buf())
            .collect();

        let current_sandbox_paths: Vec<PathBuf> = self
            .sandbox_container
            .iter()
            .map(|item| item.as_path().to_path_buf())
            .collect();

        let app_path = current_app_path
            .as_ref()
            .and_then(|app| failed.iter().find(|item| item.as_path() == app).cloned());

        let bom_files = failed
            .iter()
            .filter(|item| current_bom_paths.contains(&item.as_path().to_path_buf()))
            .cloned()
            .collect();

        let general_associated_files = failed
            .iter()
            .filter(|item| current_general_paths.contains(&item.as_path().to_path_buf()))
            .cloned()
            .collect();

        let background_task_files = failed
            .iter()
            .filter(|item| current_background_paths.contains(&item.as_path().to_path_buf()))
            .cloned()
            .collect();

        let sandbox_container = failed
            .iter()
            .filter(|item| current_sandbox_paths.contains(&item.as_path().to_path_buf()))
            .cloned()
            .collect();

        self.app_path = app_path;
        self.bom_files = bom_files;
        self.general_associated_files = general_associated_files;
        self.background_task_files = background_task_files;
        self.sandbox_container = sandbox_container;
    }

    // ==================Internal Scanner=============

    /// Discovers Background Task Management related files.
    ///
    /// Doc:
    /// Scans known Background Task Management locations and
    /// attempts to identify entries belonging to the provided
    /// application.
    ///
    /// Design:
    ///
    /// background task scanning is intentionally separated from associated-file
    /// scanning.
    ///
    /// Associated files primarily represent user/application data,
    /// while background task management files represent persistence mechanisms that allow
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
    /// Existing Background Task Management results are replaced when the scan completes.
    fn scan_background_task_files<F>(&mut self, metadata: &Metadata, progress: F) -> Vec<PathData>
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let locations_scan: Vec<PathBuf> = BackgroundTaskLocations::new().all_location_roots();

        let matcher = |path: &Path| {
            MatchRules::new()
                .equal(metadata.as_bundle_executable_name())
                .equal(metadata.as_name())
                .contain(metadata.as_bundle_id())
                .contain(metadata.as_alias_name())
                .contain(metadata.as_organization())
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

        let filtered = construct_and_deduplicate_paths(results, |item: &PathData| item.as_path());

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
    /// Sandbox containers and background task management entries are discovered through
    /// separate scanners.
    fn scan_general_associated_files<F>(
        &mut self,
        metadata: &Metadata,
        progress: F,
    ) -> Vec<PathData>
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let locations_scan: Vec<PathBuf> = GeneralLocations::new().location_roots();

        let matcher = |path: &Path| {
            MatchRules::new()
                .equal(metadata.as_bundle_executable_name())
                .equal(metadata.as_name())
                .contain(metadata.as_bundle_id())
                .contain(metadata.as_alias_name())
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
            scan_general(&locations_scan, 3, progress, matcher, builder);

        let results =
            construct_and_deduplicate_paths(asc_results, |item: &PathData| item.as_path());

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
    fn scan_sandbox_container<F>(&mut self, metadata: &Metadata, progress: F) -> Vec<PathData>
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let containers_dir = SandboxLocations::new();
        let locations_scan = containers_dir.location_roots();
        let patterns = containers_dir.as_pattern();

        let is_container_match = |path: &Path| {
            MatchRules::new()
                .contain(metadata.as_bundle_id())
                .contain(metadata.as_alias_name())
                .contain(metadata.as_bundle_executable_name())
                .check_path(path)
        };

        let is_file_match = |path: &Path| {
            MatchRules::new()
                .contain(metadata.as_bundle_id())
                .contain(metadata.as_alias_name())
                .check_path(path)
        };

        let builder = |container_dir: &Path, _file_path: &Path| {
            let folder_name = container_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let display_name = if folder_name == metadata.as_bundle_id() {
                folder_name
            } else {
                metadata.as_name().to_string()
            };

            PathData::new(container_dir.to_path_buf(), display_name)
        };

        let container_results: Vec<PathData> = scan_container(
            &locations_scan,
            1,
            &patterns,
            progress,
            is_container_match,
            is_file_match,
            builder,
        );

        let results =
            construct_and_deduplicate_paths(container_results, |item: &PathData| item.as_path());

        results
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
    fn scan_bom_files<F>(&mut self, metadata: &Metadata, progress: F) -> Vec<PathData>
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        self.bom_files.clear();

        let locations_scan: Vec<PathBuf> = ReceiptsLocations::new()
            .as_locations()
            .iter()
            .map(|location| location.as_root().to_path_buf())
            .collect();

        let matcher = |path: &Path| {
            path.extension().map(|ext| ext == "bom").unwrap_or(false)
                && MatchRules::new()
                    .contain(metadata.as_name())
                    .contain(metadata.as_bundle_executable_name())
                    .contain(metadata.as_organization())
                    .contain(metadata.as_bundle_id())
                    .check_path(&path)
        };

        let builder = |path_buf: PathBuf| {
            let name = path_buf
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            PathData::new(path_buf, name)
        };

        let results: Vec<PathData> = scan_general(&locations_scan, 1, progress, matcher, builder);

        let filtered = construct_and_deduplicate_paths(results, |item: &PathData| item.as_path());

        filtered
    }
}
