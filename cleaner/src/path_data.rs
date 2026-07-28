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

//! Path data entry.
//!
//! Doc:
//! Represents a single filesystem entry discovered during
//! scanning operations.
//!
//! The module provides a lightweight container used throughout
//! the application to represent files, directories, package
//! receipts, associated data, and background task resources.
//!
//! Each entry stores:
//!
//! - The filesystem path.
//! - A display name.
//! - An optional source category.
//!
//! Source categories identify where a path originated from
//! during discovery.
//!
//! Common categories include:
//!
//! - Application bundle paths.
//! - Background Task Management (BTM) files.
//! - Associated application data.
//!
//! `PathData` acts as the common path model shared across
//! scanning, reporting, cleanup, and user-interface layers.
//!
//! Design:
//! Path ownership and display formatting are centralized in
//! this type to avoid duplicating path handling logic across
//! the application.
//!
//! User-facing path rendering is implemented through
//! `Display`, allowing callers to format paths consistently
//! without manually performing path transformations.
//!
//! When possible, paths located inside the user's home
//! directory are rendered using a `~` prefix rather than the
//! full absolute home path.
//!
//! Note:
//! The `Display` implementation is intended for user-facing
//! output only.
//!
//! Callers that require the original filesystem path should
//! use `as_path()` instead of relying on formatted output.
//!..

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct PathData {
    path: PathBuf,
    name: String,
}

impl PathData {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self { path, name }
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn as_name(&self) -> &str {
        &self.name
    }
}

/// User-facing path formatter.
///
/// Doc:
/// Formats a path for presentation within the application UI.
///
/// If the path resides inside the current user's home
/// directory, the home prefix is replaced with `~` to improve
/// readability and reduce visual noise.
///
/// Examples:
///
/// - `/Users/alice/Documents/file.txt`
///   becomes `~/Documents/file.txt`.
///
/// - `/Library/Application Support/App`
///   remains unchanged.
///
/// Design:
/// Formatting is centralized here so all views, status
/// messages, logs, and reports present paths consistently.
///
/// Note:
/// The formatted output is intended for display purposes and
/// should not be used for filesystem operations.
impl std::fmt::Display for PathData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::env::var_os("HOME") {
            Some(home) => {
                let home = Path::new(&home);

                match self.path.strip_prefix(home) {
                    Ok(rest) => write!(f, "~/{}", rest.display()),
                    Err(_) => write!(f, "{}", self.path.display()),
                }
            }
            None => write!(f, "{}", self.path.display()),
        }
    }
}
