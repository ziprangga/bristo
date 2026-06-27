mod receipt_data;
pub use receipt_data::ReceiptData;

use std::path::{Path, PathBuf};

use crate::app_profile::app_metadata::AppMetadata;
use crate::locations_scan::ReceiptsLocations;
use crate::rules::MatchRules;
use crate::scanner::construct_scanner_result;
use crate::scanner::scan_general;

#[derive(Debug, Default, Clone)]
pub struct AppLogReceipt {
    bom_files: Vec<ReceiptData>,
}

impl AppLogReceipt {
    /// New contruct
    pub fn new(bom_files: &[ReceiptData]) -> Self {
        Self {
            bom_files: bom_files.to_vec(),
        }
    }

    //// get bom file as reference
    pub fn as_bom_files(&self) -> &[ReceiptData] {
        &self.bom_files
    }

    //// total of bom_file count
    pub fn count(&self) -> usize {
        self.bom_files.len()
    }

    //// check if bom_file is empty or not
    pub fn is_empty(&self) -> bool {
        self.bom_files.is_empty()
    }

    /// Update btm files with given list
    pub fn set_bom_files(&mut self, btm_data: Vec<ReceiptData>) {
        self.bom_files = btm_data;
    }

    /// Find BOM files for the given app
    pub fn scan_bom_files<F>(
        &mut self,
        app_metadata: &AppMetadata,
        locations: &ReceiptsLocations,
        in_progress: F,
    ) where
        F: Fn(usize, &Path) + Send + Sync,
    {
        self.bom_files.clear();
        let results: Vec<ReceiptData> = scan_general(
            locations.as_paths(),
            1,
            |n, path| in_progress(n, path),
            |path| {
                path.extension().map(|ext| ext == "bom").unwrap_or(false)
                    && MatchRules::new()
                        .contain(app_metadata.as_info().as_name())
                        .contain(app_metadata.as_info().as_bundle_executable_name())
                        .contain(app_metadata.as_info().as_organization())
                        .contain(app_metadata.as_info().as_bundle_id())
                        .check(&path)
            },
            |path_buf: PathBuf| {
                let name = path_buf
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                ReceiptData::new(path_buf, name)
            },
        );

        let filtered = construct_scanner_result(results, None, |item: &ReceiptData| item.as_path());

        self.set_bom_files(filtered);
    }
}
