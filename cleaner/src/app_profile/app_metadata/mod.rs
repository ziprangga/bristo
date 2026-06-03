mod info_plist;
pub use info_plist::InfoPlist;

use anyhow::Result;
use mini_logger::debug;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Default, Clone)]
pub struct AppMetadata {
    path: PathBuf,
    info: InfoPlist,
}

impl AppMetadata {
    /// new contruct
    pub fn new(path: PathBuf, info: InfoPlist) -> Self {
        Self { path, info }
    }

    /// Construct AppInfo from .app path
    pub fn from_path(app_path: &Path) -> Result<Self> {
        let mut plist_path = app_path.join("Contents").join("Info.plist");

        if !plist_path.exists() {
            let found = WalkDir::new(app_path)
                .into_iter()
                .par_bridge()
                .filter_map(|e| e.ok())
                .filter(|entry| entry.file_type().is_file() && entry.file_name() == "Info.plist")
                .collect::<Vec<_>>();

            let upper = found
                .into_par_iter()
                .min_by_key(|entry| entry.depth())
                .map(|entry| entry.path().to_path_buf());

            let selected = upper
                .ok_or_else(|| anyhow::anyhow!("Info.plist not found in {}", app_path.display()))?;

            debug!("Info.plist selected from: {}", selected.to_string_lossy());

            plist_path = selected;
        }

        let info = InfoPlist::from_plist(&plist_path, app_path)?;

        debug!(
            "path: {}, name: {}, bundle_id: {}, bundle_name: {}, organization: {}",
            app_path.display(),
            info.as_name(),
            info.as_bundle_id(),
            info.as_bundle_executable_name(),
            info.as_organization(),
        );

        Ok(Self {
            path: app_path.to_path_buf(),
            info,
        })
    }

    //// get path reference
    pub fn as_path(&self) -> &PathBuf {
        &self.path
    }

    /// get info reference
    pub fn as_info(&self) -> &InfoPlist {
        &self.info
    }
}
