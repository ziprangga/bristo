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
//! The module is built around two primary types:
//!
//! - `AppMetadata` represents an application bundle.
//! - `InfoPlist` stores metadata extracted from `Info.plist`.
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
//! Metadata is primarily derived from `Info.plist`, with sensible
//! fallbacks used when specific fields are unavailable.
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

mod info_plist;
pub use info_plist::InfoPlist;

use anyhow::Result;
use mini_logger::debug;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Application bundle metadata.
///
/// Doc:
/// Represents a macOS application bundle together with its
/// parsed identity information.
///
/// An `AppMetadata` contains:
///
/// - The application bundle path.
/// - Parsed `Info.plist` data.
///
/// The application path identifies the bundle on disk while
/// the associated `InfoPlist` provides information used by
/// discovery and matching operations.
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
    path: PathBuf,
    info: InfoPlist,
}

impl AppMetadata {
    /// new contruct
    pub fn new(path: PathBuf, info: InfoPlist) -> Self {
        Self { path, info }
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
    /// associated `InfoPlist`.
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

    /// Update path app
    pub fn set_app_path(&mut self, path: PathBuf) {
        self.path = path;
    }
}
