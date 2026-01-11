use anyhow::{Context, Result, anyhow};
use plist::Value;
use std::path::{Path, PathBuf};

use crate::helpers::path_contains_ignore_case;
use crate::helpers::path_equals_ignore_case;

pub enum MatchRules {
    Equal,
    Contain,
}

impl MatchRules {
    fn match_path(&self, path: &Path, value: &str) -> bool {
        match self {
            MatchRules::Equal => path_equals_ignore_case(path, value),
            MatchRules::Contain => path_contains_ignore_case(path, value),
        }
    }
}

#[derive(Debug, Clone)]
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
        let plist_path = Path::new(app_path).join("Contents").join("Info.plist");

        if !plist_path.exists() {
            anyhow::bail!("Info.plist not found in {}", app_path.display());
        }

        let plist = Value::from_file(&plist_path)
            .with_context(|| format!("Failed to read plist: {}", plist_path.display()))?;

        let bundle_id = plist
            .as_dictionary()
            .and_then(|d| d.get("CFBundleIdentifier"))
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                anyhow::anyhow!("CFBundleIdentifier not found in {}", plist_path.display())
            })?;

        let app_name = plist
            .as_dictionary()
            .and_then(|d| d.get("CFBundleDisplayName"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .or_else(|| {
                // fallback to file stem if CFBundleDisplayName is missing
                Some(app_path.file_stem()?.to_string_lossy().into_owned())
            })
            .ok_or_else(|| anyhow!("Failed to determine app name for {}", app_path.display()))?;

        let executable_name = plist
            .as_dictionary()
            .and_then(|d| d.get("CFBundleExecutable"))
            .and_then(|v| v.as_string())
            .ok_or_else(|| anyhow!("CFBundleExecutable not found in {}", plist_path.display()))?;

        let organization = bundle_id.split('.').nth(1).unwrap_or("").to_string();

        Ok(Self {
            path: app_path.to_path_buf(),
            name: app_name.to_string(),
            bundle_id: bundle_id.to_string(),
            bundle_name: executable_name.to_string(),
            organization,
        })
    }

    pub fn associate_path_matches(&self, path: &Path) -> bool {
        self.rules_matches(
            path,
            &[
                (MatchRules::Equal, &self.name),
                (MatchRules::Equal, &self.bundle_name),
                (MatchRules::Equal, &self.organization),
                (MatchRules::Contain, &self.bundle_id),
            ],
        )
        // path_equals_ignore_case(path, &self.name)
        //     || path_equals_ignore_case(path, &self.bundle_name)
        //     || path_equals_ignore_case(path, &self.organization)
        //     || path_contains_ignore_case(path, &self.bundle_id)
    }

    pub fn rules_matches(&self, path: &Path, rules: &[(MatchRules, &str)]) -> bool {
        rules
            .iter()
            .any(|(rule, value)| rule.match_path(path, value))
    }
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            name: String::new(),
            bundle_id: String::new(),
            bundle_name: String::new(),
            organization: String::new(),
        }
    }
}
