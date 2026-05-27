use anyhow::{Context, Result, anyhow};
use mini_logger::debug;
use plist::Value;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
struct PlistReader {
    plist_value: Value,
}

impl PlistReader {
    /// Read plist from path
    fn new(plist_path: &Path) -> Result<Self> {
        let plist = Value::from_file(&plist_path)
            .with_context(|| format!("Failed to read plist: {}", plist_path.display()))?;
        Ok(Self { plist_value: plist })
    }

    /// Get CFBundleIdentifier
    fn get_bundle_id(&self) -> Option<String> {
        self.plist_value
            .as_dictionary()
            .and_then(|d| d.get("CFBundleIdentifier"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// Get CFBundleDisplayName
    fn get_display_name(&self) -> Option<String> {
        self.plist_value
            .as_dictionary()
            .and_then(|d| d.get("CFBundleDisplayName"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// Get CFBundleExecutable
    fn get_executable_name(&self) -> Option<String> {
        self.plist_value
            .as_dictionary()
            .and_then(|d| d.get("CFBundleExecutable"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// get Organization
    fn get_organization(&self) -> Option<String> {
        self.get_bundle_id()
            .and_then(|bundle_id| bundle_id.split('.').nth(1).map(|s| s.to_string()))
    }
}

#[derive(Debug, Default, Clone)]
pub struct AppInfo {
    path: PathBuf,
    name: String,
    bundle_id: String,
    bundle_executable_name: String,
    organization: String,
}

impl AppInfo {
    /// new contruct
    pub fn new(
        path: PathBuf,
        name: String,
        bundle_id: String,
        bundle_executable_name: String,
        organization: String,
    ) -> Self {
        Self {
            path,
            name,
            bundle_id,
            bundle_executable_name,
            organization,
        }
    }
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

            debug!("Info.plist selected from: {}", selected.to_string_lossy());

            plist_path = selected;
        }

        let plist = PlistReader::new(&plist_path)?;
        let bundle_id = plist.get_bundle_id().ok_or_else(|| {
            anyhow::anyhow!("CFBundleIdentifier not found in {}", plist_path.display())
        })?;
        let app_name = plist
            .get_display_name()
            .or_else(|| {
                // fallback to file stem if CFBundleDisplayName is missing
                Some(app_path.file_stem()?.to_string_lossy().into_owned())
            })
            .ok_or_else(|| anyhow!("Failed to determine app name for {}", app_path.display()))?;

        let bundle_executable_name = plist
            .get_executable_name()
            .ok_or_else(|| anyhow!("CFBundleExecutable not found in {}", plist_path.display()))?;

        let organization = plist.get_organization().unwrap_or_default();

        debug!(
            "path: {}, name: {}, bundle_id: {}, bundle_name: {}, organization: {}",
            app_path.display(),
            app_name,
            bundle_id,
            bundle_executable_name,
            organization,
        );

        Ok(Self {
            path: app_path.to_path_buf(),
            name: app_name,
            bundle_id,
            bundle_executable_name,
            organization,
        })
    }

    //// get path reference
    pub fn as_path(&self) -> &PathBuf {
        &self.path
    }

    //// get name reference
    pub fn as_name(&self) -> &str {
        &self.name
    }

    //// get bundle_id reference
    pub fn as_bundle_id(&self) -> &str {
        &self.bundle_id
    }

    //// get bundle executable name reference
    pub fn as_bundle_executable_name(&self) -> &str {
        &self.bundle_executable_name
    }

    //// get organization reference
    pub fn as_organization(&self) -> &str {
        &self.organization
    }
}
