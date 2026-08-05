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

//! Trash operation tracking.
//!
//! Doc:
//! Represents the result of moving application-related files
//! to the system Trash.
//!
//! The module is responsible for recording which paths were
//! successfully moved and which paths failed during a trash
//! operation.
//!
//! The module is built around two primary types:
//!
//! - `TrashItem` stores information about a successfully
//!   trashed entry.
//! - `TrashEntry` stores the complete result of a trash
//!   operation.
//!
//! A trash operation may partially succeed.
//!
//! Some files can be moved successfully while others fail due
//! to permission issues, filesystem restrictions, missing
//! files, or operating system errors.
//!
//! Rather than treating the operation as entirely failed,
//! individual results are preserved and reported separately.
//!
//! Design:
//! Successful and failed entries are tracked independently.
//!
//! This allows callers to:
//!
//! - Report partial failures.
//! - Update application state incrementally.
//! - Present detailed error information to users.
//! - Support future restore functionality.
//!
//! Note:
//! Moving a file to Trash does not immediately delete it.
//!
//! Files remain recoverable until permanently removed from
//! the Trash by the user or operating system.
//!..

use crate::errors::{ErrorKind, Result};
use crate::path_data::PathData;
use crate::syscom::trash_files_nsfilemanager;

use std::path::Path;
use std::path::PathBuf;

/// Successfully trashed item.
///
/// Doc:
/// Stores information about a path that was successfully
/// moved to the system Trash.
///
/// The original source path is retained for reporting and
/// potential restore operations.
///
/// The trashed path represents the location assigned by the
/// operating system after the item was moved.
///
/// Design:
/// Source and trashed locations are stored separately because
/// operating systems may rename or relocate items during the
/// trash process.
///
/// Note:
/// Some implementations may not immediately provide the final
/// trash location.
/// In such cases the trashed path may be populated later.
#[derive(Debug, Clone)]
pub struct TrashItem {
    source_path: PathData,
    trashed_path: PathBuf,
}

impl TrashItem {
    pub fn new(source_path: PathData, trashed_path: PathBuf) -> Self {
        Self {
            source_path,
            trashed_path,
        }
    }

    pub fn as_source_path(&self) -> &PathData {
        &self.source_path
    }

    pub fn as_trashed_path(&self) -> &Path {
        &self.trashed_path
    }
}

/// Result of a trash operation.
///
/// Doc:
/// Stores the complete outcome of attempting to move one or
/// more paths to the system Trash.
///
/// Results are divided into two categories:
///
/// - Successfully moved items.
/// - Failed items with associated error information.
///
/// This structure allows callers to inspect partial success
/// scenarios and react accordingly.
///
/// Design:
/// A single operation may contain both successful and failed
/// entries.
///
/// Tracking them separately avoids losing information and
/// provides a better user experience when reporting cleanup
/// results.
///
/// Note:
/// An empty failure list does not necessarily mean the
/// operation succeeded completely.
///
/// Callers should inspect both moved and failed collections
/// when determining the final outcome.
#[derive(Debug, Default, Clone)]
pub struct TrashEntry {
    moved_path: Vec<TrashItem>,
    failed_path: Vec<(PathData, ErrorKind)>,
}

impl TrashEntry {
    pub fn new(moved_path: Vec<TrashItem>, failed_path: Vec<(PathData, ErrorKind)>) -> Self {
        Self {
            moved_path,
            failed_path,
        }
    }
    /// Moves all paths in the entry to the system Trash.
    ///
    /// Doc:
    /// Attempts to move every `PathData` in the provided slice into
    /// the operating system Trash.
    ///
    /// The operation records both successful and failed results.
    ///
    /// Failed paths are paired with the error that prevented the
    /// move operation from completing.
    ///
    /// Successfully moved paths are stored as `TrashItem` records.
    ///
    /// Design:
    /// Partial success is supported.
    ///
    /// Individual failures do not automatically discard
    /// successfully moved items.
    ///
    /// This behaviour allows cleanup operations to continue even
    /// when a subset of files cannot be removed.
    ///
    /// Note:
    /// The final trash location is determined by the operating
    /// system and may differ from the original path.
    pub fn moved_path_to_trash(paths: &[PathData]) -> Result<Self> {
        let mut result = Self::default();

        let path_bufs: Vec<PathBuf> = paths.iter().map(|p| p.as_path().to_path_buf()).collect();

        // let failed = trash_files_nsfilemanager(&path_bufs)?;
        let (moved, failed) = trash_files_nsfilemanager(&path_bufs)?;

        for (failed_path, reason) in failed {
            if let Some(item) = paths.iter().find(|p| p.as_path() == failed_path) {
                result.failed_path.push((item.clone(), reason));
            }
        }

        for (source_path, trashed_path) in moved {
            if let Some(item) = paths.iter().find(|p| p.as_path() == source_path) {
                result
                    .moved_path
                    .push(TrashItem::new(item.clone(), trashed_path));
            }
        }

        Ok(result)
    }

    // /// Restores trashed items.
    // ///
    // /// Doc:
    // /// Attempts to restore previously trashed items back to
    // /// their original locations.
    // ///
    // /// Design:
    // /// Restoration support is planned but not currently
    // /// implemented.
    // ///
    // /// Future implementations may use the stored trash location
    // /// together with the original source path to perform the
    // /// restore operation.
    // ///
    // /// Note:
    // /// Calling this function currently performs no restore
    // /// operation.
    // pub fn put_back(&self, _from_trash: &[TrashItem]) -> Result<()> {
    //     Ok(println!("to do"))
    // }

    pub fn moved_path(&self) -> &[TrashItem] {
        &self.moved_path
    }

    pub fn failed_path(&self) -> &[(PathData, ErrorKind)] {
        &self.failed_path
    }

    pub fn failed_paths(&self) -> Vec<PathData> {
        self.failed_path()
            .iter()
            .map(|(path, _)| path.clone())
            .collect()
    }

    pub fn set_moved_path(&mut self, moved_path: Vec<TrashItem>) {
        self.moved_path = moved_path;
    }

    pub fn set_failed_path(&mut self, failed_path: Vec<(PathData, ErrorKind)>) {
        self.failed_path = failed_path;
    }

    pub fn moved_path_mut(&mut self) -> &mut Vec<TrashItem> {
        &mut self.moved_path
    }

    pub fn failed_path_mut(&mut self) -> &mut Vec<(PathData, ErrorKind)> {
        &mut self.failed_path
    }
}
