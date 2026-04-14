mod app_data;
mod rules;
mod syscom;
pub use app_data::*;
pub use rules::*;

use anyhow::Result;
use mini_logger::debug;
use simple_status::{Emitter, status_emit};
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct Cleaner {
    pub app_data: AppData,
}

impl Cleaner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_app(path: &Path, status: Option<&Emitter>) -> Result<Self> {
        let mut app_data = AppData::new(path)?;

        status_emit!(
            status,
            "Scanning running processes for '{}'",
            app_data.app.name
        );

        // Find running processes
        app_data.find_pid_and_command();

        status_emit!(status, "Found process {}", app_data.app_process.len());

        Ok(Self { app_data })
    }

    pub fn kill_app_process(&self, status: Option<&Emitter>) -> Result<()> {
        if self.app_data.app_process.is_empty() {
            return Ok(());
        }

        let killed_count =
            AppProcess::kill_app_processes(&self.app_data.app.name, &self.app_data.app_process)?;

        status_emit!(
            status,
            stage: "Completed",
            total: killed_count,
            message: "All processes killed",);

        Ok(())
    }

    /// Scan an app at the given path and return AppData
    pub fn scan_app_data(&mut self, status: Option<&Emitter>) -> Result<&Self> {
        status_emit!(
            status,
            "Scanning logs and associated files for '{}'",
            self.app_data.app.name
        );

        status_emit!(
            status,
            stage: "Started",
            message: "Finding BOM logs...",
        );

        let locations = LocationsScan::new();

        self.app_data.find_log_bom(&locations);

        let total_bom_file = self.app_data.log.bom_file.len();

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

        self.app_data
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

    /// Save BOM logs of the current app to the given folder
    pub fn save_bom_logs(&self, log_dir: &Path) -> Result<()> {
        // Determine the folder
        let app_log_folder =
            Path::new(log_dir).join(format!("{}_bom_log", &self.app_data.app.name));
        debug!("Creating folder: {}", app_log_folder.display());

        // Call the LogReceipt function
        self.app_data.save_bom_log_app(&app_log_folder)
    }

    /// Move all associated files including the app itself to trash
    pub fn trash_all(&self) -> Result<Vec<(PathBuf, String)>> {
        // get all path in the associate_files field with enumerate
        let paths: Vec<PathBuf> = self
            .app_data
            .all_associate_entries_enumerate()
            .iter()
            .map(|(_i, (path, _label))| path.clone())
            .collect();

        // delete all associate_files
        let failed_paths = syscom::trash_files_nsfilemanager(&paths)?;

        Ok(failed_paths)
    }

    /// Print a summary of the app data
    /// For CLI
    pub fn print_summary(&self) {
        println!("App Name: {}", self.app_data.app.name);
        println!("Bundle ID: {}", self.app_data.app.bundle_id);
        println!("Bundle Name: {}", self.app_data.app.bundle_executable_name);

        println!("\nRunning processes:");
        for p in &self.app_data.app_process {
            println!("PID {}: {}", p.pid, p.command);
        }

        println!("\nLog BOM files:");
        for log in &self.app_data.log.bom_file {
            println!("{}", log.display());
        }

        println!("\nAssociated files:");
        for (_i, (path, label)) in &self.app_data.all_associate_entries_enumerate() {
            println!("{} -> {}", label, path.display());
        }
    }

    pub fn show_in_finder(path: &Path) -> Result<()> {
        syscom::show_in_finder(path)
    }

    pub fn reset(&mut self) {
        self.app_data.reset();
    }
}
