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
