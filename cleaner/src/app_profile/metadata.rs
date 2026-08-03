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

//! Doc:
//! Application metadata and Info.plist parsing.
//!
//! This module is responsible for identifying an application and
//! extracting the metadata required by the scanning system.
//!
//! The module is built around a single primary type:
//!
//! - `AppMetadata` represents an application bundle together
//!   with its parsed identity information.
//!
//! Application metadata serves as the foundation for all discovery
//! operations throughout the crate.
//!
//! Information extracted from the application bundle is later used to:
//!
//! - Locate running processes.
//! - Match associated files.
//! - Match sandbox containers.
//! - Match package receipts.
//! - Generate user-facing application information.
//!
//! Metadata is derived from the application's `Info.plist`
//! and used throughout the scanning system.
//!
//! Typical workflow:
//!
//! 1. Create `AppMetadata` from an application path.
//! 2. Locate the application's `Info.plist`.
//! 3. Parse application identity information.
//! 4. Use the resulting metadata for scanning operations.
//!
//! Note:
//! This module performs metadata discovery only. It does not scan
//! associated files, discover processes, or perform cleanup
//! operations.
//!..

use mini_logger::debug;
use plist::Value;
use rayon::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::errors::{ErrorKind, Result};

/// Application bundle metadata.
///
/// Doc:
/// Represents a macOS application bundle together with its
/// parsed identity information.
///
/// An `AppMetadata` contains:
///
/// - The application bundle path.
/// - Application display name.
/// - Bundle identifier.
/// - Executable name.
/// - Organization identifier.
///
/// The application path identifies the bundle on disk while
/// the parsed metadata fields are used by discovery and
/// matching operations.
///
/// Examples:
///
/// ```text
/// /Applications/Safari.app
/// /Applications/Visual Studio Code.app
/// ~/Applications/MyApp.app
/// ```
///
/// Note:
/// This type acts as the primary identity source for the
/// scanning system.
#[derive(Debug, Default, Clone)]
pub struct AppMetadata {
    bundle_path: PathBuf,
    name: String,
    bundle_id: String,
    bundle_executable_name: String,
    organization: String,
    alias_name: String,
}

impl AppMetadata {
    /// new contruct
    pub fn new(
        bundle_path: PathBuf,
        name: String,
        bundle_id: String,
        bundle_executable_name: String,
        organization: String,
        alias_name: String,
    ) -> Self {
        Self {
            bundle_path,
            name,
            bundle_id,
            bundle_executable_name,
            organization,
            alias_name,
        }
    }

    /// Constructs application metadata from an application bundle.
    ///
    /// Doc:
    /// Creates an `AppMetadata` by locating and parsing the
    /// application's `Info.plist`.
    ///
    /// Design:
    ///
    /// Most macOS applications store metadata in:
    ///
    ///     Contents/Info.plist
    ///
    /// However, some applications ship with unusual bundle layouts.
    /// When the expected file is missing, a recursive search is
    /// performed and the nearest Info.plist is selected.
    ///
    ///
    /// The method first attempts to locate:
    ///
    /// ```text
    /// Contents/Info.plist
    /// ```
    ///
    /// using the standard macOS bundle layout.
    ///
    /// If the file cannot be found at the expected location,
    /// a recursive search is performed and the closest matching
    /// `Info.plist` is selected.
    ///
    /// The parsed metadata is then used to construct the
    /// resulting `AppMetadata`.
    ///
    /// Returns an error if:
    ///
    /// - No `Info.plist` can be found.
    /// - The plist cannot be parsed.
    /// - Required metadata fields are missing.
    ///
    /// Note:
    /// Some applications use non-standard bundle layouts, which
    /// is why a fallback search is performed.
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

            let selected = upper.ok_or_else(|| {
                ErrorKind::failed()
                    .with_summary("Missing metadata configuration")
                    .with_reason(format!(
                        "Info.plist not found in bundle: {}",
                        app_path.display()
                    ))
            })?;

            debug!("Info.plist selected from: {}", selected.to_string_lossy());

            plist_path = selected;
        }

        let metadata = Self::parse_info_plist(&plist_path, app_path)?;

        debug!(
            "path: {}, name: {}, bundle_id: {}, bundle_name: {}, organization: {}",
            app_path.display(),
            metadata.as_name(),
            metadata.as_bundle_id(),
            metadata.as_bundle_executable_name(),
            metadata.as_organization(),
        );

        Ok(metadata)
    }

    /// get bundle path reference
    pub fn as_bundle_path(&self) -> &Path {
        &self.bundle_path
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

    /// get alias name reference
    pub fn as_alias_name(&self) -> &str {
        &self.alias_name
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
    fn parse_info_plist(plist_path: &Path, app_path: &Path) -> Result<Self> {
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

        let bundle_executable_name = match dict
            .get("CFBundleExecutable")
            .and_then(|v| v.as_string())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(executable) => executable.to_string(),
            None => {
                let binary_dir = app_path.join("Contents").join("MacOS");

                let executable = std::fs::read_dir(&binary_dir)
                    .ok()
                    .and_then(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .find(|e| e.path().is_file())
                    })
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .ok_or_else(|| {
                        ErrorKind::failed()
                            .with_summary("Missing executable configuration")
                            .with_reason(
                                "CFBundleExecutable was missing or empty and no executable could be inferred",
                            )
                    })?;

                executable
            }
        };

        let organization = bundle_id.split('.').nth(1).unwrap_or_default().to_string();

        let alias_name = bundle_id
            .rsplit_once('.')
            .map(|(_, last)| last)
            .unwrap_or_default()
            .to_string();

        Ok(Self {
            bundle_path: app_path.to_path_buf(),
            name,
            bundle_id,
            bundle_executable_name,
            organization,
            alias_name,
        })
    }
}
