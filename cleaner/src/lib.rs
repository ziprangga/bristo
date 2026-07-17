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
mod locations_scan;
mod rules;
mod scanner;
mod syscom;
pub use app_profile::AppLogReceipt;
pub use app_profile::{AppAscFiles, AscData};
pub use app_profile::{AppBtmFiles, BtmData};
pub use app_profile::{AppMetadata, InfoPlist};
pub use app_profile::{AppProcs, Proc};
pub use app_profile::{AppProfile, FileEntry};
pub use locations_scan::{BtmLocations, ReceiptsLocations, SandboxLocations, ScanLocations};
pub use rules::MatchRules;

use anyhow::{Context, Result};
use mini_logger::debug;
use rayon::prelude::*;
use simple_status::{Emitter, status_emit};
use std::collections::HashMap;
use std::path::Path;

/// Result classification for a trash operation.
///
/// Doc:
/// Represents the outcome of a file removal attempt.
///
/// Variants:
///
/// - `Failed` indicates a removal operation was attempted
///   but did not succeed.
/// - `Skipped` indicates the operation was intentionally
///   not executed.
///
/// Note:
/// Successful removals are not represented because
/// `trash_all_entry()` only returns entries requiring
/// additional attention from the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrashStatus {
    Failed,
    Skipped,
}

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
#[derive(Debug, Clone)]
pub struct TrashEntry {
    status: TrashStatus,
    entry: FileEntry,
    reason: Option<String>,
}

impl TrashEntry {
    pub fn new(status: TrashStatus, entry: FileEntry, reason: Option<String>) -> Self {
        Self {
            status: status,
            entry,
            reason: reason,
        }
    }

    pub fn status(&self) -> TrashStatus {
        self.status
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn entry(&self) -> &FileEntry {
        &self.entry
    }

    pub fn into_entry(self) -> FileEntry {
        self.entry
    }

    pub fn failed(entry: FileEntry, reason: String) -> Self {
        Self {
            status: TrashStatus::Failed,
            entry,
            reason: Some(reason),
        }
    }

    pub fn skipped(entry: FileEntry, reason: String) -> Self {
        Self {
            status: TrashStatus::Skipped,
            entry,
            reason: Some(reason),
        }
    }
}

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
}

impl Cleaner {
    pub fn new(app_profile: AppProfile) -> Self {
        Self { app_profile }
    }

    pub fn as_app_profile(&self) -> &AppProfile {
        &self.app_profile
    }

    pub fn new_profile(path: &Path, emitter: Option<&Emitter>) -> Result<Self> {
        let app_profile = AppProfile::from_path(path)?;

        status_emit!(
            emitter,
            "Scanning running processes for '{}'",
            app_profile.as_app_metadata().as_info().as_name()
        );

        Ok(Self { app_profile })
    }

    pub fn find_app_process(&mut self, emitter: Option<&Emitter>) -> Result<&Self> {
        self.app_profile.find_pid_and_command();
        status_emit!(
            emitter,
            "Found process {}",
            self.app_profile.as_app_procs().list().len()
        );

        Ok(self)
    }

    pub fn kill_app_process(&self, emitter: Option<&Emitter>) -> Result<()> {
        let processes = self.app_profile.as_app_procs();

        if processes.is_empty() {
            println!(
                "No running processes found for {}",
                self.app_profile.as_app_metadata().as_info().as_name()
            );
            return Ok(());
        }

        let mut killed_count = 0;

        for p in processes.list() {
            if syscom::kill_pids(&p.pid().to_string()).is_ok() {
                killed_count += 1;
            } else {
                eprintln!(
                    "Failed to kill PID {} for {}",
                    p.pid(),
                    self.app_profile.as_app_metadata().as_info().as_name()
                );
            }
        }

        status_emit!(
            emitter,
            action: "Completed",
            total: killed_count,
            message: "All processes killed",);

        Ok(())
    }

    /// Scan an app at the given path and return AppProfile
    pub fn scan_app_profile(&mut self, emitter: Option<&Emitter>) -> Result<&Self> {
        status_emit!(
            emitter,
            "Scanning logs and associated files for '{}'",
            self.app_profile.as_app_metadata().as_info().as_name()
        );

        status_emit!(
            emitter,
            action: "Started",
            message: "Finding BOM logs...",
        );

        let locations = ScanLocations::new();

        let btm_locations = BtmLocations::new();

        let receipts_locations = ReceiptsLocations::new();

        status_emit!(
            emitter,
            action: "Started",
            message: "Finding BOM files...",
        );

        self.app_profile
            .find_log_bom(&receipts_locations, |cur, _path| {
                status_emit!(
                    emitter,
                    action: "Searching",
                    current: cur,
                );
            });

        let total_bom_file = self.app_profile.as_app_log_receipt().count();

        status_emit!(
            emitter,
            action: "Completed",
            total: total_bom_file,
            message: "BOM logs scan completed",
        );

        status_emit!(
            emitter,
            action: "Started",
            message: "Finding associated files...",
        );

        self.app_profile
            .find_associate_files(&locations, |cur, _path| {
                status_emit!(
                    emitter,
                    action: "Searching",
                    current: cur,
                );
            });

        status_emit!(
            emitter,
            action: "Started",
            message: "Finding btm files...",
        );

        self.app_profile
            .find_btm_files(&btm_locations, |cur, _path| {
                status_emit!(
                    emitter,
                    action: "Searching",
                    current: cur,
                );
            });

        status_emit!(
            emitter,
            action: "Completed",
            message: "Associated files scan completed",
        );

        Ok(self)
    }

    /// Replace Associate Files
    pub fn replace_remaining_entries(&mut self, entries: Vec<TrashEntry>) {
        let file_entries: Vec<FileEntry> =
            entries.into_iter().map(TrashEntry::into_entry).collect();

        self.app_profile.replace_file_entries(file_entries);
    }

    /// Save BOM logs of the current app to the given folder
    pub fn save_bom_logs(&self, log_dir: &Path) -> Result<()> {
        // Determine the folder
        let app_log_folder = Path::new(log_dir).join(format!(
            "{}_bom_log",
            self.app_profile.as_app_metadata().as_info().as_name()
        ));
        debug!("Creating folder: {}", app_log_folder.display());

        std::fs::create_dir_all(&app_log_folder).with_context(|| {
            format!("Failed to create log folder: {}", app_log_folder.display())
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
    pub fn all_entries_enumerate(&self) -> Vec<(usize, FileEntry)> {
        self.app_profile
            .all_entries()
            .into_iter()
            .enumerate()
            .collect()
    }

    /// Move all associated files including the app itself to trash
    pub fn trash_all_entry(&self) -> Result<Vec<TrashEntry>> {
        let entries = self.app_profile.all_entries();

        let mut asc_paths = Vec::new();
        let mut btm_paths = Vec::new();
        let mut app_paths = Vec::new();

        for entry in &entries {
            match entry {
                FileEntry::AscFiles(_) => {
                    asc_paths.push(entry.as_path().to_path_buf());
                }

                FileEntry::BtmFiles(_) => {
                    btm_paths.push(entry.as_path().to_path_buf());
                }

                FileEntry::AppPath(_) => {
                    app_paths.push(entry.as_path().to_path_buf());
                }
            }
        }

        let mut results: Vec<TrashEntry> = Vec::new();

        // trash ASC
        let asc_failed = syscom::trash_files_nsfilemanager(&asc_paths)?;
        for (failed_path, reason) in &asc_failed {
            if let Some(entry) = entries.iter().find(|e| e.as_path() == failed_path) {
                results.push(TrashEntry::failed(entry.clone(), reason.clone()));
            }
        }

        // trash BTM
        let btm_failed = syscom::trash_files_nsfilemanager(&btm_paths)?;
        for (failed_path, reason) in &btm_failed {
            if let Some(entry) = entries.iter().find(|e| e.as_path() == failed_path) {
                results.push(TrashEntry::failed(entry.clone(), reason.clone()));
            }
        }

        // trash AppPath only when other have no failures
        let can_trash_app = asc_failed.is_empty() && btm_failed.is_empty();
        if can_trash_app {
            let app_failed = syscom::trash_files_nsfilemanager(&app_paths)?;

            for (failed_path, reason) in &app_failed {
                if let Some(entry) = entries.iter().find(|e| e.as_path() == failed_path) {
                    results.push(TrashEntry::failed(entry.clone(), reason.clone()));
                }
            }
        } else {
            for entry in &entries {
                if matches!(entry, FileEntry::AppPath(_)) {
                    results.push(TrashEntry::skipped(
                        entry.clone(),
                        "because some associated files failed to move".to_string(),
                    ));
                }
            }
        }

        Ok(results)
    }

    /// Print a summary of the app data
    /// For CLI
    pub fn print_summary(&self) {
        println!(
            "App Name: {}",
            self.app_profile.as_app_metadata().as_info().as_name()
        );
        println!(
            "Bundle ID: {}",
            self.app_profile.as_app_metadata().as_info().as_bundle_id()
        );
        println!(
            "Bundle Name: {}",
            self.app_profile
                .as_app_metadata()
                .as_info()
                .as_bundle_executable_name()
        );

        println!("\nRunning processes:");
        for p in self.app_profile.as_app_procs().list() {
            println!("PID {}: {}", p.pid(), p.as_command());
        }

        println!("\nLog BOM files:");
        for log in self.app_profile.as_app_log_receipt().as_bom_files() {
            println!("{}", log.as_path().display());
        }

        println!("\nAll associated files:");
        for (_i, entry) in self.all_entries_enumerate() {
            println!("{} -> {}", entry.as_name(), entry.as_path().display());
        }
    }

    pub fn show_in_finder(path: &Path) -> Result<()> {
        syscom::show_in_finder(path)
    }

    pub fn reset(&mut self) {
        self.app_profile.reset();
    }
}

/// Cached icon storage for UI consumers.
///
/// Doc:
/// Stores platform-generated icon images as raw RGBA buffers
/// keyed by file type or application path.
///
/// Supported icon categories:
///
/// - Application bundles (`*.app`).
/// - System folder icons.
/// - Generic file icons.
///
/// The cache stores:
///
/// - Width.
/// - Height.
/// - RGBA pixel data.
///
/// Consumers can retrieve icon data as:
///
/// - Borrowed RGBA slices.
/// - Owned RGBA buffers.
/// - Generated RGB buffers.
///
/// Note:
/// Icon generation may require platform-specific system APIs.
/// This type exists separately from `Cleaner` so that UI-related
/// functionality remains independent from application cleanup logic.
#[derive(Debug, Clone)]
pub struct IconCache {
    icon_cache: HashMap<String, (usize, usize, Vec<u8>)>,
}

impl IconCache {
    pub fn new(path: &Path, target_size: f64) -> Option<Self> {
        let path_str = path.to_str().unwrap_or("");

        // 1. Determine the appropriate cache key dynamically
        let cache_key = if path_str.ends_with(".app") {
            path_str.to_string()
        } else if path.is_dir() {
            "__system_folder__".to_string()
        } else {
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("__system_generic_file__")
                .to_string()
        };

        // Generate and load the owned icon data instantly
        let icon = Self::load_icon_for_key(&cache_key, target_size)?;

        // Initialize the HashMap and insert the resolved icon
        let mut map = HashMap::new();
        map.insert(cache_key, icon);

        // Wrap the map in Self and return it
        Some(Self { icon_cache: map })
    }

    pub fn icon_cache_owned(self) -> HashMap<String, (usize, usize, Vec<u8>)> {
        self.icon_cache
    }

    // Get width for a specific file path icon
    pub fn width(&self, path: &Path) -> Option<usize> {
        let key = Self::get_cache_key(path);
        self.icon_cache.get(&key).map(|(w, _, _)| *w)
    }

    // Get height for a specific file path icon
    pub fn height(&self, path: &Path) -> Option<usize> {
        let key = Self::get_cache_key(path);
        self.icon_cache.get(&key).map(|(_, h, _)| *h)
    }

    // Get an immutable reference to the raw RGBA slice
    pub fn rgba_bytes(&self, path: &Path) -> Option<&[u8]> {
        let key = Self::get_cache_key(path);
        self.icon_cache
            .get(&key)
            .map(|(_, _, bytes)| bytes.as_slice())
    }

    // Consume the cache and extract a specific icon's raw vector allocation
    pub fn into_rgba_bytes(mut self, path: &Path) -> Option<Vec<u8>> {
        let key = Self::get_cache_key(path);
        self.icon_cache.remove(&key).map(|(_, _, bytes)| bytes)
    }

    // Build an RGB vector on the fly from the stored RGBA tuple data
    pub fn rgb_bytes(&self, path: &Path) -> Option<Vec<u8>> {
        let key = Self::get_cache_key(path);
        let (width, height, rgba_bytes) = self.icon_cache.get(&key)?;

        let mut rgb = Vec::with_capacity(width * height * 3);

        // Chunk through data 4 bytes at a time (R, G, B, A)
        for chunk in rgba_bytes.chunks_exact(4) {
            rgb.push(chunk[0]); // R
            rgb.push(chunk[1]); // G
            rgb.push(chunk[2]); // B
            // chunk[3] (Alpha) is intentionally skipped
        }

        Some(rgb)
    }

    pub fn get_cache_key(path: &Path) -> String {
        let path_str = path.to_str().unwrap_or("");

        if path_str.ends_with(".app") {
            path_str.to_string()
        } else if path.is_dir() {
            "__system_folder__".to_string()
        } else {
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("__system_generic_file__")
                .to_string()
        }
    }

    fn load_icon_for_key(key: &str, target_size: f64) -> Option<(usize, usize, Vec<u8>)> {
        let ns_image = if key.ends_with(".app") {
            syscom::get_installed_app_icon_by_path(key)
        } else if key == "__system_folder__" {
            syscom::get_default_folder_icon()
        } else {
            syscom::get_default_file_icon()
        };

        let (width, height, bytes) = syscom::ns_image_to_rgba_bytes(&ns_image, target_size)?;
        Some((width, height, bytes))
    }
}
