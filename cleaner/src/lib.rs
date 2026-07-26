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
//! Core application cleanup and uninstall orchestration.
//!
//! This crate provides the high-level API used to inspect, analyze,
//! and remove macOS applications together with their associated files.
//!
//! The crate is built around three primary responsibilities:
//!
//! - `AppProfile` stores all discovered application information.
//! - `Cleaner` orchestrates scanning and removal operations.
//! - `IconCache` provides cached macOS icon assets for UI rendering.
//!
//! The cleanup workflow generally follows:
//!
//! 1. Build an `AppProfile` from an application path.
//! 2. Discover running processes.
//! 3. Locate package receipts and BOM logs.
//! 4. Locate associated files.
//! 5. Locate BTM (Background Task Management) files.
//! 6. Optionally terminate running processes.
//! 7. Move discovered files to Trash.
//!
//! The module intentionally separates discovery, state management,
//! and system interaction:
//!
//! - `AppProfile` owns discovered application state.
//! - `Cleaner` coordinates cleanup operations.
//! - `syscom` performs platform-specific system calls.
//!
//! Associated file discovery is divided into multiple categories:
//!
//! - Application bundle paths.
//! - Associated files (`AscFiles`).
//! - Background task management files (`BtmFiles`).
//! - Package receipt and BOM metadata.
//!
//! Failed cleanup operations are represented by `TrashEntry`,
//! allowing callers to inspect which files could not be removed
//! and why.
//!
//! The module is designed so both CLI and GUI frontends can share
//! the same scanning and cleanup logic while providing their own
//! presentation layer.
//!
//! Note:
//! Most applications should interact primarily with `Cleaner`.
//! Lower-level types such as `AppProfile`, scanning modules,
//! and platform-specific helpers exist to organize implementation
//! details and support advanced use cases.
//!..

mod app_profile;
mod errors;
mod icon_cache;
mod locations_scan;
mod path_data;
mod rules;
mod scanner;
mod syscom;
mod trash_entry;

pub use app_profile::AppLogReceipt;
pub use app_profile::AppMetadata;
pub use app_profile::AppProfile;
pub use app_profile::InfoPlist;
pub use app_profile::PathEntry;
pub use app_profile::{AppProcs, Proc};
pub use errors::{ErrorKind, Result};
pub use icon_cache::IconCache;
pub use locations_scan::{BtmLocations, ReceiptsLocations, SandboxLocations, ScanLocations};
pub use path_data::{PathData, SourceKind};
pub use rules::MatchRules;
pub use trash_entry::TrashEntry;

use mini_logger::debug;
use rayon::prelude::*;
use std::borrow::Cow;
use std::path::Path;

// /// Result classification for a trash operation.
// ///
// /// Doc:
// /// Represents the outcome of a file removal attempt.
// ///
// /// Variants:
// ///
// /// - `Failed` indicates a removal operation was attempted
// ///   but did not succeed.
// /// - `Skipped` indicates the operation was intentionally
// ///   not executed.
// ///
// /// Note:
// /// Successful removals are not represented because
// /// `trash_all_entry()` only returns entries requiring
// /// additional attention from the caller.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum TrashStatus {
//     Failed,
//     Skipped,
// }

/// Describes a file that could not be removed.
///
/// Doc:
/// Contains information about a failed or skipped cleanup
/// operation.
///
/// A `TrashEntry` stores:
///
/// - The resulting `TrashStatus`.
/// - The affected `FileEntry`.
/// - An optional human-readable reason.
///
/// Examples:
///
/// - Permission denied.
/// - File is currently in use.
/// - Application removal skipped because associated
///   files failed.
///
/// Note:
/// These entries are primarily intended for user-facing
/// reporting and cleanup diagnostics.
// #[derive(Debug, Clone)]
// pub struct TrashEntry {
//     status: TrashStatus,
//     entry: FileEntry,
//     reason: Option<String>,
// }

// impl TrashEntry {
//     pub fn new(status: TrashStatus, entry: FileEntry, reason: Option<String>) -> Self {
//         Self {
//             status: status,
//             entry,
//             reason: reason,
//         }
//     }

//     pub fn status(&self) -> TrashStatus {
//         self.status
//     }

//     pub fn reason(&self) -> Option<&str> {
//         self.reason.as_deref()
//     }

//     pub fn entry(&self) -> &FileEntry {
//         &self.entry
//     }

//     pub fn into_entry(self) -> FileEntry {
//         self.entry
//     }

//     pub fn failed(entry: FileEntry, reason: String) -> Self {
//         Self {
//             status: TrashStatus::Failed,
//             entry,
//             reason: Some(reason),
//         }
//     }

//     pub fn skipped(entry: FileEntry, reason: String) -> Self {
//         Self {
//             status: TrashStatus::Skipped,
//             entry,
//             reason: Some(reason),
//         }
//     }
// }
// #[derive(Debug, Clone)]
// pub struct TrashEntry {
//     entry: FileEntry,
//     error: Option<ErrorKind>,
// }

// impl TrashEntry {
//     pub fn new(entry: FileEntry, error: Option<ErrorKind>) -> Self {
//         Self { entry, error }
//     }

//     pub fn error(&self) -> Option<&ErrorKind> {
//         self.error.as_ref()
//     }

//     pub fn entry(&self) -> &FileEntry {
//         &self.entry
//     }

//     pub fn into_entry(self) -> FileEntry {
//         self.entry
//     }

//     pub fn failed(entry: FileEntry, error: ErrorKind) -> Self {
//         Self {
//             entry,
//             error: Some(error),
//         }
//     }

//     pub fn skipped(entry: FileEntry, error: ErrorKind) -> Self {
//         Self {
//             entry,
//             error: Some(error),
//         }
//     }
// }

/// Application cleanup coordinator.
///
/// Doc:
/// `Cleaner` is the primary entry point for application scanning,
/// inspection, and removal.
///
/// A `Cleaner` owns a single `AppProfile` and provides operations
/// for:
///
/// - Process discovery.
/// - Process termination.
/// - Receipt scanning.
/// - Associated file discovery.
/// - BTM file discovery.
/// - BOM log export.
/// - Trash operations.
///
/// Typical workflow:
///
/// ```text
/// Application Path
///       │
///       ▼
/// Cleaner::new_profile()
///       │
///       ▼
/// find_app_process()
///       │
///       ▼
/// scan_app_profile()
///       │
///       ├─ save_bom_logs()
///       ├─ print_summary()
///       ├─ trash_all_entry()
///       └─ reset()
/// ```
///
/// Note:
/// `Cleaner` acts as an orchestration layer and delegates
/// platform-specific operations to `syscom` while storing
/// discovered state inside `AppProfile`.
#[derive(Debug, Default, Clone)]
pub struct Cleaner {
    app_profile: AppProfile,
    trash_entry: TrashEntry,
}

impl Cleaner {
    pub fn new(app_profile: AppProfile) -> Self {
        Self {
            app_profile,
            trash_entry: TrashEntry::default(),
        }
    }

    pub fn as_app_profile(&self) -> &AppProfile {
        &self.app_profile
    }

    pub fn as_trash_entry(&self) -> &TrashEntry {
        &self.trash_entry
    }

    pub fn as_trash_entry_mut(&mut self) -> &mut TrashEntry {
        &mut self.trash_entry
    }

    pub fn new_profile<F>(path: &Path, progress: Option<F>) -> Result<Self>
    where
        F: Fn(Cow<'static, str>) + Send + Sync + Clone,
    {
        let app_profile = AppProfile::from_path(path)?;

        if let Some(ref progress_hook) = progress {
            let app_name = app_profile.as_app_metadata().as_info().as_name();
            progress_hook(Cow::Owned(format!("Found profile for '{}'", app_name)));
        }

        Ok(Self::new(app_profile))
    }

    pub fn find_app_process<F>(&mut self, progress: Option<F>) -> Result<&Self>
    where
        F: Fn(Cow<'static, str>) + Send + Sync + Clone,
    {
        self.app_profile.find_pid_and_command();

        if let Some(ref progress_hook) = progress {
            let process_count = self.app_profile.as_app_procs().list().len();
            progress_hook(Cow::Owned(format!("Found process {}", process_count)));
        }

        Ok(self)
    }

    pub fn kill_app_process<F>(&self, progress: Option<F>) -> Result<()>
    where
        F: Fn(usize, usize) + Send + Sync + Clone,
    {
        let processes = self.app_profile.as_app_procs();

        if processes.is_empty() {
            return Ok(());
        }

        let total = processes.list().len();
        let mut errors = Vec::new();
        let mut killed_count = 0;

        for (current, p) in processes.list().iter().enumerate() {
            match syscom::kill_pid(p.pid()) {
                Ok(_) => {
                    killed_count += 1;
                }

                Err(err) => {
                    errors.push(err);
                }
            }

            if let Some(ref progress_hook) = progress {
                progress_hook(current + 1, total);
            }
        }

        if !errors.is_empty() {
            return Err(ErrorKind::failed()
                .with_summary(format!("Killed {}/{} processes", killed_count, total))
                .with_reason(
                    errors
                        .iter()
                        .filter_map(|e| e.reason())
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                        .join("\n"),
                ));
        }

        Ok(())
    }

    /// Scan an app at the given path and return AppProfile
    pub fn scan_app_profile<F>(&mut self, progress: F) -> Result<&Self>
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        let locations = ScanLocations::new();

        let btm_locations = BtmLocations::new();

        let receipts_locations = ReceiptsLocations::new();

        self.app_profile
            .find_log_bom(&receipts_locations, progress.clone());

        self.app_profile
            .find_associate_files(&locations, progress.clone());

        self.app_profile.find_btm_files(&btm_locations, progress);

        Ok(self)
    }

    /// Save BOM logs of the current app to the given folder
    pub fn save_bom_logs(&self, log_dir: &Path) -> Result<()> {
        // Determine the folder
        let app_log_folder = Path::new(log_dir).join(format!(
            "{}_bom_log",
            self.app_profile.as_app_metadata().as_info().as_name()
        ));
        debug!("Creating folder: {}", app_log_folder.display());

        std::fs::create_dir_all(&app_log_folder).map_err(|e| {
            ErrorKind::failed()
                .with_summary("Failed to prepare logging target")
                .with_reason(format!(
                    "Failed to create log folder {}: {}",
                    app_log_folder.display(),
                    e
                ))
        })?;

        // Use par_iter() for parallel processing
        let results: Vec<Result<()>> = self
            .app_profile
            .as_app_log_receipt()
            .as_bom_files()
            .par_iter()
            .map(|bom_file| {
                let output_file = app_log_folder
                    .join(bom_file.as_name())
                    .with_extension("log");
                syscom::run_lsbom_command(bom_file.as_path(), &output_file)
            })
            .collect();

        // Collect all errors, return the first one if any
        results.into_iter().collect::<Result<()>>()
    }

    /// listed of FileEntry, all path that associate to the app
    pub fn all_entries_enumerate(&self) -> Vec<(usize, PathData)> {
        self.app_profile
            .path_entry()
            .all_paths()
            .into_iter()
            .enumerate()
            .collect()
    }

    pub fn move_to_trash(&mut self) -> Result<()> {
        let trash_entry = TrashEntry::move_to_trash(self.app_profile.path_entry())?;

        let failed: Vec<PathData> = trash_entry
            .failed_path()
            .iter()
            .map(|(path, _)| path.clone())
            .collect();

        if !failed.is_empty() {
            self.app_profile.update_path_entry(&failed);
        }

        self.trash_entry = trash_entry;

        Ok(())
    }

    pub fn restore_moved_path(&self) -> Result<()> {
        Ok(println!("to do"))
    }

    pub fn show_in_finder(path: &Path) -> Result<()> {
        syscom::show_in_finder(path)
    }

    pub fn reset(&mut self) {
        self.app_profile.reset();
    }
}
