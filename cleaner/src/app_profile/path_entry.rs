use std::path::Path;
use std::path::PathBuf;

use crate::app_profile::metadata::AppMetadata;
use crate::locations_scan::BtmLocations;
use crate::locations_scan::SandboxLocations;
use crate::locations_scan::ScanLocations;
use crate::path_data::{PathData, SourceKind};
use crate::rules::MatchRules;

use crate::scanner::construct_scanner_result;
use crate::scanner::scan_container;
use crate::scanner::scan_general;

#[derive(Debug, Clone, Default)]
pub struct PathEntry {
    app_path: Option<PathData>,
    associated: Vec<PathData>,
    btm: Vec<PathData>,
}

impl PathEntry {
    pub fn from_path_and_metadata(app_path: &Path, metadata: &AppMetadata) -> Self {
        let app_path = PathData::new(
            app_path.to_path_buf(),
            metadata.as_info().as_name().to_string(),
            SourceKind::App,
        );
        Self {
            app_path: Some(app_path),
            associated: Vec::new(),
            btm: Vec::new(),
        }
    }

    pub fn all_paths(&self) -> Vec<PathData> {
        let mut paths = Vec::new();

        if let Some(app_path) = &self.app_path {
            paths.push(app_path.clone());
        }
        paths.extend(self.associated.iter().cloned());
        paths.extend(self.btm.iter().cloned());

        paths
    }

    pub fn update_entry(&mut self, failed: &[PathData]) {
        let app_path = failed
            .iter()
            .find(|item| matches!(item.as_kind(), Some(SourceKind::App)))
            .cloned();

        let associated = failed
            .iter()
            .filter(|item| matches!(item.as_kind(), Some(SourceKind::Associated)))
            .cloned()
            .collect();

        let btm = failed
            .iter()
            .filter(|item| matches!(item.as_kind(), Some(SourceKind::Btm)))
            .cloned()
            .collect();

        self.app_path = app_path;
        self.set_associated(associated);
        self.set_btm(btm);
    }

    //// get path reference
    pub fn as_app_path(&self) -> Option<&PathData> {
        self.app_path.as_ref()
    }

    //// reference of btm files
    pub fn as_btm(&self) -> &[PathData] {
        &self.btm
    }

    //// reference of associate files
    pub fn as_associated(&self) -> &[PathData] {
        &self.associated
    }

    /// Update path app
    pub fn set_app_path(&mut self, path: PathData) {
        self.app_path = Some(path);
    }

    /// Update btm files with given list
    pub fn set_btm(&mut self, btm_data: Vec<PathData>) {
        self.btm = btm_data;
    }

    /// Update associate_files with given list
    pub fn set_associated(&mut self, asc_data: Vec<PathData>) {
        self.associated = asc_data;
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
        progress: F,
    ) where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let results: Vec<PathData> = scan_general(
            &locations.all_paths(),
            2,
            |n, path| progress(n, path),
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

                PathData::new(path_buf, name, SourceKind::Btm)
            },
        );

        let filtered = construct_scanner_result(results, None, |item: &PathData| item.as_path());

        self.set_btm(filtered);
    }

    // Scan all file associate from list of location
    // for huge directory and try using walkdir + rayon
    // use progress as emitter status to caller
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
        progress: F,
    ) where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let main_results: Vec<PathData> = scan_general(
            locations.as_paths(),
            3,
            |n, path| progress(n, path),
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

                PathData::new(path_buf, name, SourceKind::Associated)
            },
        );

        let containers_dir = SandboxLocations::new();
        let patterns = containers_dir.sandbox_pattern();

        let container_results: Vec<PathData> = scan_container(
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

                PathData::new(container_dir.to_path_buf(), display_name, SourceKind::App)
            },
        );

        let results =
            construct_scanner_result(main_results, Some(container_results), |item: &PathData| {
                item.as_path()
            });

        self.set_associated(results);
    }
}
