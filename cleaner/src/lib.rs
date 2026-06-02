mod app_profile;
mod locations_scan;
mod rules;
mod syscom;
pub use app_profile::AppLogReceipt;
pub use app_profile::{AppAscFiles, AscData};
pub use app_profile::{AppMetadata, InfoPlist};
pub use app_profile::{AppProcs, Proc};
pub use app_profile::{AppProfile, FileEntry};
pub use locations_scan::{LocationsScan, SandboxContainerLocation};
pub use rules::MatchRules;

use anyhow::{Context, Result};
use mini_logger::debug;
use rayon::prelude::*;
use simple_status::{Emitter, status_emit};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrashStatus {
    Failed,
    Skipped,
}

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

    pub fn new_profile(path: &Path, status: Option<&Emitter>) -> Result<Self> {
        let app_profile = AppProfile::from_path(path)?;

        status_emit!(
            status,
            "Scanning running processes for '{}'",
            app_profile.as_app_metadata().as_info().as_name()
        );

        Ok(Self { app_profile })
    }

    pub fn find_app_process(&mut self, status: Option<&Emitter>) -> Result<&Self> {
        self.app_profile.find_pid_and_command();
        status_emit!(
            status,
            "Found process {}",
            self.app_profile.as_app_procs().list().len()
        );

        Ok(self)
    }

    pub fn kill_app_process(&self, status: Option<&Emitter>) -> Result<()> {
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
            status,
            stage: "Completed",
            total: killed_count,
            message: "All processes killed",);

        Ok(())
    }

    /// Scan an app at the given path and return AppProfile
    pub fn scan_app_profile(&mut self, status: Option<&Emitter>) -> Result<&Self> {
        status_emit!(
            status,
            "Scanning logs and associated files for '{}'",
            self.app_profile.as_app_metadata().as_info().as_name()
        );

        status_emit!(
            status,
            stage: "Started",
            message: "Finding BOM logs...",
        );

        let locations = LocationsScan::new();

        self.app_profile.find_log_bom(&locations);

        let total_bom_file = self.app_profile.as_app_log_receipt().count();

        status_emit!(
            status,
            stage: "Completed",
            total: total_bom_file,
            message: "BOM logs scan completed",
        );

        status_emit!(
            status,
            stage: "Started",
            message: "Finding associated files...",
        );

        self.app_profile
            .find_associate_files(&locations, |cur, _path| {
                status_emit!(
                    status,
                    stage: "Searching",
                    current: cur,
                );
            });

        status_emit!(
            status,
            stage: "Completed",
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
            .as_bom_file()
            .par_iter()
            .map(|bom_file| {
                let output_file = bom_file
                    .file_name()
                    .map(|n| app_log_folder.join(n).with_extension("log"))
                    .context("BOM file has no filename")?;

                syscom::run_lsbom_command(bom_file, &output_file)
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
        let mut app_paths = Vec::new();

        for entry in &entries {
            match entry {
                FileEntry::AscFiles(_) => {
                    asc_paths.push(entry.as_path().to_path_buf());
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

        // trash AppPath only when other have no failures
        let can_trash_app = asc_failed.is_empty();
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
        for log in self.app_profile.as_app_log_receipt().as_bom_file() {
            println!("{}", log.display());
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
