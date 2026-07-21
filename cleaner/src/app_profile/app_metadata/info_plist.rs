// Copyright 2026 ziprangga
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Parsed application identity information.
//!
//! Doc:
//! Stores the subset of `Info.plist` values required by the
//! scanning and cleanup system.
//!
//! Captured values include:
//!
//! - Application display name.
//! - Bundle identifier.
//! - Executable name.
//! - Organization identifier.
//!
//! These values are later used to construct matching rules
//! for locating files, processes, containers, and receipts.
//!
//! Example:
//!
//! ```text
//! Name: Safari
//! Bundle ID: com.apple.Safari
//! Executable: Safari
//! Organization: apple
//! ```
//!
//! Note:
//! This type intentionally stores only the fields currently
//! required by the application scanner.
//!..

use crate::error::{ErrorKind, Result};
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

    /// Parses application information from an Info.plist file.
    ///
    /// Doc:
    /// Reads a plist file and extracts the metadata required by
    /// the scanning system.
    ///
    /// Required fields:
    ///
    /// - `CFBundleIdentifier`
    /// - `CFBundleExecutable`
    ///
    /// Application name is resolved using:
    ///
    /// 1. `CFBundleDisplayName`
    /// 2. Application bundle filename
    ///
    /// The organization value is derived from the bundle
    /// identifier.
    ///
    /// Design:
    ///
    /// This value is used as an additional matching signal when
    /// searching for associated files.
    ///
    /// Example:
    ///
    ///     com.apple.Safari      -> apple
    ///     com.google.Chrome    -> google
    ///     org.mozilla.firefox  -> mozilla
    ///
    /// ```text
    /// com.apple.Safari
    ///     └── apple
    /// ```
    ///
    /// Returns an error when required fields are missing or the
    /// plist structure is invalid.
    ///
    /// Note:
    /// Only a subset of available plist fields is parsed because
    /// the scanner requires application identity rather than
    /// complete bundle metadata.
    pub fn from_plist(plist_path: &Path, app_path: &Path) -> Result<Self> {
        let plist = Value::from_file(plist_path).map_err(|e| {
            ErrorKind::failed()
                .with_summary("Failed to read plist file")
                .with_reason(format!("{}: {}", plist_path.display(), e))
        })?;

        let dict = plist.as_dictionary().ok_or_else(|| {
            ErrorKind::failed()
                .with_summary("Invalid plist structure")
                .with_reason("The parsed plist root is not a dictionary mapping")
        })?;

        let bundle_id = dict
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                ErrorKind::failed()
                    .with_summary("Missing bundle identifier")
                    .with_reason("The required field 'CFBundleIdentifier' was missing or invalid")
            })?
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
            .ok_or_else(|| {
                ErrorKind::failed()
                    .with_summary("Application identity resolution failed")
                    .with_reason("Failed to determine application name from plist or bundle path")
            })?;

        let bundle_executable_name = dict
            .get("CFBundleExecutable")
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                ErrorKind::failed()
                    .with_summary("Missing executable configuration")
                    .with_reason("The required field 'CFBundleExecutable' was missing or invalid")
            })?
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
