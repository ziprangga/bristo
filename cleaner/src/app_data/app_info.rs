use anyhow::{Result, anyhow};
use common_debug::debug_dev;

use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::app_data::plist_reader::PlistReader;

#[derive(Debug, Default, Clone)]
pub struct AppInfo {
    pub path: PathBuf,
    pub name: String,
    pub bundle_id: String,
    pub bundle_name: String,
    pub organization: String,
}

impl AppInfo {
    /// Construct AppInfo from .app path
    pub fn from_path(app_path: &Path) -> Result<Self> {
        let mut plist_path = app_path.join("Contents").join("Info.plist");

        if !plist_path.exists() {
            let found = WalkDir::new(app_path)
                .into_iter()
                .par_bridge()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_file() && entry.file_name() == "Info.plist")
                .collect::<Vec<_>>();

            let upper = found
                .into_par_iter()
                .min_by_key(|entry| entry.depth())
                .map(|entry| entry.path().to_path_buf());

            let selected = upper
                .ok_or_else(|| anyhow::anyhow!("Info.plist not found in {}", app_path.display()))?;

            debug_dev!("Info.plist selected from: {}", selected.to_string_lossy());

            plist_path = selected;
        }

        let plist = PlistReader::new(&plist_path)?;
        let bundle_id = plist.bundle_id().ok_or_else(|| {
            anyhow::anyhow!("CFBundleIdentifier not found in {}", plist_path.display())
        })?;
        let app_name = plist
            .display_name()
            .or_else(|| {
                // fallback to file stem if CFBundleDisplayName is missing
                Some(app_path.file_stem()?.to_string_lossy().into_owned())
            })
            .ok_or_else(|| anyhow!("Failed to determine app name for {}", app_path.display()))?;

        let executable_name = plist
            .executable_name()
            .ok_or_else(|| anyhow!("CFBundleExecutable not found in {}", plist_path.display()))?;

        let organization = plist.organization().unwrap_or_default();

        debug_dev!(
            "path: {}, name: {}, bundle_id: {}, bundle_name: {}, organization: {}",
            app_path.to_string_lossy(),
            app_name.to_string(),
            bundle_id.to_string(),
            executable_name.to_string(),
            organization,
        );

        Ok(Self {
            path: app_path.to_path_buf(),
            name: app_name.to_string(),
            bundle_id: bundle_id.to_string(),
            bundle_name: executable_name.to_string(),
            organization,
        })
    }
}
