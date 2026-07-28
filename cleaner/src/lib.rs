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

//! Core application cleanup and uninstall orchestration.
//!
//! This crate provides the high-level API used to inspect, analyze,
//! and remove macOS applications together with their related files.
//!
//! The crate is organized around three primary responsibilities:
//!
//! - `AppProfile` stores discovered application information and scan results.
//! - `Cleaner` coordinates application analysis and cleanup workflows.
//! - `TrashEntry` stores the result of trash operations, including moved
//!   paths and paths that failed to move.
//!
//! The cleanup workflow generally follows:
//!
//! 1. Build an `AppProfile` from an application path.
//! 2. Discover running application processes.
//! 3. Locate package receipts and BOM metadata.
//! 4. Locate associated files.
//! 5. Locate BTM (Background Task Management) files.
//! 6. Optionally terminate running processes.
//! 7. Move discovered files to Trash.
//!
//! The crate separates application state, orchestration, and
//! platform-specific operations:
//!
//! - `AppProfile` owns discovered application data.
//! - `Cleaner` coordinates scanning, process handling, and cleanup.
//! - `syscom` provides macOS system command integration.
//! - `TrashEntry` represents trash operation results.
//!
//! Application files are discovered from multiple sources:
//!
//! - Application bundle paths.
//! - Associated files (`AscFiles`).
//! - Sandbox container data.
//! - Background Task Management files (`BtmFiles`).
//! - Package receipts and BOM metadata.
//!
//! Cleanup results are preserved through `TrashEntry`, allowing callers
//! to inspect which paths were successfully moved and which paths failed
//! together with their associated errors.
//!
//! The crate is designed to be shared by different frontends, such as
//! CLI or GUI applications, while keeping scanning and cleanup logic
//! independent from presentation concerns.
//!
//! Note:
//! Most applications should interact primarily with `Cleaner`.
//! Lower-level types such as `AppProfile`, scanning modules, and
//! platform-specific helpers exist to organize implementation details
//! and support advanced workflows.
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
pub use app_profile::PathEntry;
pub use app_profile::{AppProcs, Proc};
pub use errors::{ErrorKind, Result};
pub use icon_cache::IconCache;
pub use locations_scan::{BtmLocations, ReceiptsLocations, SandboxLocations, ScanLocations};
pub use path_data::PathData;
pub use rules::MatchRules;
pub use trash_entry::TrashEntry;

use mini_logger::debug;
use rayon::prelude::*;
use std::borrow::Cow;
use std::path::Path;

/// Application cleanup coordinator.
///
/// Doc:
/// `Cleaner` is the primary entry point for application scanning,
/// inspection, process handling, and removal operations.
///
/// A `Cleaner` owns an `AppProfile` containing discovered application
/// information and a `TrashEntry` containing the latest trash operation
/// result.
///
/// Responsibilities include:
///
/// - Application profile creation.
/// - Process discovery.
/// - Process termination.
/// - Receipt scanning.
/// - Associated file discovery.
/// - BTM file discovery.
/// - BOM log export.
/// - Moving discovered files to Trash.
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
///       ├─ move_to_trash()
///       └─ reset()
/// ```
///
/// Note:
/// `Cleaner` acts as an orchestration layer. Platform-specific
/// operations are delegated to `syscom`, while discovered state and
/// cleanup results are stored internally.
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
            let app_name = app_profile.as_app_metadata().as_name();
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

    /// Scan the current application profile and discover related files.
    pub fn scan_app_profile<F>(&mut self, progress: F) -> Result<&Self>
    where
        F: Fn(usize, &Path) + Send + Sync + Clone,
    {
        self.app_profile.find_associated_paths(progress);

        Ok(self)
    }

    /// Export discovered BOM metadata into log files inside the given folder.
    pub fn save_bom_logs(&self, log_dir: &Path) -> Result<()> {
        // Determine the folder
        let app_log_folder = Path::new(log_dir).join(format!(
            "{}_bom_log",
            self.app_profile.as_app_metadata().as_name()
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

    /// Returns all discovered application paths with their index.
    pub fn all_entries_enumerate(&self) -> Vec<(usize, PathData)> {
        self.app_profile
            .as_path_entry()
            .all_paths()
            .into_iter()
            .enumerate()
            .collect()
    }

    /// Move discovered application paths to Trash.
    ///
    /// Associated paths are moved first. The application bundle
    /// is moved only when all associated paths were successfully
    /// moved.
    pub fn move_to_trash(&mut self) -> Result<&Self> {
        let path_entry = self.app_profile.as_path_entry();

        let mut moved = Vec::new();
        let mut failed = Vec::new();

        // Associated paths
        let asc_trash = TrashEntry::moved_path_to_trash(path_entry.as_associated_paths())?;

        moved.extend(asc_trash.moved_path().iter().cloned());
        failed.extend(asc_trash.failed_path().iter().cloned());

        // App bundle only if associated succeeded
        match failed.is_empty() {
            true => {
                if let Some(app_path) = path_entry.as_app_path() {
                    let app_trash =
                        TrashEntry::moved_path_to_trash(std::slice::from_ref(app_path))?;

                    moved.extend(app_trash.moved_path().iter().cloned());
                    failed.extend(app_trash.failed_path().iter().cloned());
                }
            }

            false => {
                if let Some(app_path) = path_entry.as_app_path() {
                    failed.push((
                        app_path.clone(),
                        ErrorKind::skipped()
                            .with_reason("because some associated files failed to move"),
                    ));
                }
            }
        }

        let trash_entry = TrashEntry::new(moved, failed);

        self.app_profile
            .update_path_entry(&trash_entry.failed_paths());

        self.trash_entry = trash_entry;

        Ok(self)
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
