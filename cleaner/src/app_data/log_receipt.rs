use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use crate::app_data::app_info::AppInfo;
use crate::locations_scan::LocationsScan;
use crate::rules::MatchRules;
use crate::syscom::run_lsbom_command;

#[derive(Debug, Default, Clone)]
pub struct LogReceipt {
    bom_file: Vec<PathBuf>,
}

impl LogReceipt {
    /// Find BOM files for the given app
    pub fn find_bom_files(&mut self, app: &AppInfo, locations: &LocationsScan) {
        self.bom_file.clear();
        for dir in locations.receipts_dirs() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map(|ext| ext == "bom").unwrap_or(false)
                        && MatchRules::new()
                            .contain(app.as_name())
                            .contain(app.as_bundle_executable_name())
                            .contain(app.as_organization())
                            .contain(app.as_bundle_id())
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

    //// Save all BOM files to the given log directory in parallel
    pub fn save_bom_log(&self, log_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(log_dir)
            .with_context(|| format!("Failed to create log folder: {}", log_dir.display()))?;

        // Use par_iter() for parallel processing
        let results: Vec<Result<()>> = self
            .bom_file
            .par_iter()
            .map(|bom_file| {
                let output_file = bom_file
                    .file_name()
                    .map(|n| log_dir.join(n).with_extension("log"))
                    .context("BOM file has no filename")?;

                run_lsbom_command(bom_file, &output_file)
            })
            .collect();

        // Collect all errors, return the first one if any
        results.into_iter().collect::<Result<()>>()
    }
}
