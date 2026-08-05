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

//! Path data and trash tracking.
//!
//! Doc:
//! Provides the common path model used throughout the application
//! together with types for tracking trash operations.
//!
//! The module is composed of two primary components:
//!
//! - `PathData`, which represents a discovered filesystem entry.
//! - `trash_entry`, which records the results of moving paths to
//!   the system Trash.
//!
//! `PathData` is shared across scanning, reporting, cleanup,
//! and user-interface layers, while `trash_entry` builds upon
//! it to preserve successful and failed trash operations.
//!
//! Design:
//! Centralizing path-related types in a single module ensures
//! consistent ownership, formatting, and reporting of filesystem
//! entries throughout the application.
//!
//! Note:
//! User-facing path formatting is implemented by `PathData`'s
//! `Display` implementation, while filesystem operations should
//! continue using `PathData::as_path()`.
//!..

pub mod trash_entry;

use std::path::{Path, PathBuf};

/// Path data entry.
///
/// Doc:
/// Represents a single filesystem entry discovered during
/// scanning operations.
///
/// The module provides a lightweight container used throughout
/// the application to represent files, directories, package
/// receipts, associated data, and background task resources.
///
/// Each entry stores:
///
/// - The filesystem path.
/// - A display name.
///
/// `PathData` acts as the common path model shared across
/// scanning, reporting, cleanup, trash operations, and
/// user-interface layers.
///
/// `PathData` acts as the common path model shared across
/// scanning, reporting, cleanup, and user-interface layers.
///
/// Design:
/// Path ownership and display formatting are centralized in
/// this type to avoid duplicating path handling logic across
/// the application.
///
/// User-facing path rendering is implemented through
/// `Display`, allowing callers to format paths consistently
/// without manually performing path transformations.
///
/// When possible, paths located inside the user's home
/// directory are rendered using a `~` prefix rather than the
/// full absolute home path.
///
/// Note:
/// The `Display` implementation is intended for user-facing
/// output only.
///
/// Callers that require the original filesystem path should
/// use `as_path()` instead of relying on formatted output.
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
