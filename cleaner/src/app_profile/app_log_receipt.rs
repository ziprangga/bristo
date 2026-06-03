use std::path::PathBuf;

use crate::app_profile::app_metadata::AppMetadata;
use crate::locations_scan::ScanLocations;
use crate::rules::MatchRules;

#[derive(Debug, Default, Clone)]
pub struct AppLogReceipt {
    bom_file: Vec<PathBuf>,
}

impl AppLogReceipt {
    /// New contruct
    pub fn new(bom_file: &[PathBuf]) -> Self {
        Self {
            bom_file: bom_file.to_vec(),
        }
    }
    /// Find BOM files for the given app
    pub fn find_bom_files(&mut self, app_metadata: &AppMetadata, locations: &ScanLocations) {
        self.bom_file.clear();
        for dir in locations.receipts_dirs() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map(|ext| ext == "bom").unwrap_or(false)
                        && MatchRules::new()
                            .contain(app_metadata.as_info().as_name())
                            .contain(app_metadata.as_info().as_bundle_executable_name())
                            .contain(app_metadata.as_info().as_organization())
                            .contain(app_metadata.as_info().as_bundle_id())
                            .check(&path)
                    {
                        self.bom_file.push(path);
                    }
                }
            }
        }
    }

    //// total of bom_file count
    pub fn count(&self) -> usize {
        self.bom_file.len()
    }

    //// check if bom_file is empty or not
    pub fn is_empty(&self) -> bool {
        self.bom_file.is_empty()
    }

    //// get bom file as reference
    pub fn as_bom_file(&self) -> &Vec<PathBuf> {
        &self.bom_file
    }
}
