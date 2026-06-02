use anyhow::{Context, Result, anyhow};
use plist::Value;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct InfoPlist {
    name: String,
    bundle_id: String,
    bundle_executable_name: String,
    organization: String,
}

impl InfoPlist {
    pub fn new(
        name: String,
        bundle_id: String,
        bundle_executable_name: String,
        organization: String,
    ) -> Self {
        Self {
            name,
            bundle_id,
            bundle_executable_name,
            organization,
        }
    }
    pub fn from_plist(plist_path: &Path, app_path: &Path) -> Result<Self> {
        let plist = Value::from_file(plist_path)
            .with_context(|| format!("Failed to read plist: {}", plist_path.display()))?;

        let dict = plist
            .as_dictionary()
            .ok_or_else(|| anyhow!("Invalid plist format"))?;

        let bundle_id = dict
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_string())
            .ok_or_else(|| anyhow!("CFBundleIdentifier not found"))?
            .to_string();

        let name = dict
            .get("CFBundleDisplayName")
            .and_then(|v| v.as_string())
            .map(ToOwned::to_owned)
            .or_else(|| {
                app_path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
            })
            .ok_or_else(|| anyhow!("Failed to determine app name"))?;

        let bundle_executable_name = dict
            .get("CFBundleExecutable")
            .and_then(|v| v.as_string())
            .ok_or_else(|| anyhow!("CFBundleExecutable not found"))?
            .to_string();

        let organization = bundle_id.split('.').nth(1).unwrap_or_default().to_string();

        Ok(Self {
            name,
            bundle_id,
            bundle_executable_name,
            organization,
        })
    }

    /// get name reference
    pub fn as_name(&self) -> &str {
        &self.name
    }

    /// get bundle_id reference
    pub fn as_bundle_id(&self) -> &str {
        &self.bundle_id
    }

    /// get bundle executable name reference
    pub fn as_bundle_executable_name(&self) -> &str {
        &self.bundle_executable_name
    }

    /// get organization reference
    pub fn as_organization(&self) -> &str {
        &self.organization
    }
}
