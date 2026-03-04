use anyhow::{Context, Result};
use plist::Value;
use std::path::Path;

#[derive(Debug)]
pub struct PlistReader {
    plist_value: Value,
}

impl PlistReader {
    /// Read plist from path
    pub fn new(plist_path: &Path) -> Result<Self> {
        let plist = Value::from_file(&plist_path)
            .with_context(|| format!("Failed to read plist: {}", plist_path.display()))?;
        Ok(Self { plist_value: plist })
    }

    /// Get CFBundleIdentifier
    pub fn bundle_id(&self) -> Option<String> {
        self.plist_value
            .as_dictionary()
            .and_then(|d| d.get("CFBundleIdentifier"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// Get CFBundleDisplayName
    pub fn display_name(&self) -> Option<String> {
        self.plist_value
            .as_dictionary()
            .and_then(|d| d.get("CFBundleDisplayName"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// Get CFBundleExecutable
    pub fn executable_name(&self) -> Option<String> {
        self.plist_value
            .as_dictionary()
            .and_then(|d| d.get("CFBundleExecutable"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// get Organization
    pub fn organization(&self) -> Option<String> {
        self.bundle_id()
            .and_then(|bundle_id| bundle_id.split('.').nth(1).map(|s| s.to_string()))
    }
}
