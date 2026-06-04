mod btm_data;
pub use btm_data::BtmData;

use crate::app_profile::app_metadata::AppMetadata;
use crate::locations_scan::BtmLocations;
use crate::rules::MatchRules;

use crate::scanner::construct_scanner_result;
use crate::scanner::scan_general;
use std::path::Path;
use std::path::PathBuf;

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
                vec![BtmData::new(
                    path_buf.clone(),
                    path_buf.file_name().unwrap().to_string_lossy().to_string(),
                )]
            },
        );

        let filtered = construct_scanner_result(results, None, |item: &BtmData| item.as_path());

        self.set_btm_files(filtered);
    }
}
