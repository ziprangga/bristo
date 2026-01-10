mod app_data;
mod foundation;
mod helpers;
pub use app_data::*;
pub use helpers::*;

use anyhow::Result;
use status::StatusEmitter;
use std::path::Path;
use std::path::PathBuf;

use common_debug::debug_dev;

#[derive(Debug, Default, Clone)]
pub struct Cleaner {
    pub app_data: AppData,
}

impl Cleaner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_app(path: &Path, status: Option<&StatusEmitter>) -> Result<Self> {
        let mut app_data = AppData::new(path)?;

        if let Some(s) = status {
            s.with_message(format!(
                "Scanning running processes for '{}'",
                app_data.app.name
            ))
            .emit();
        }

        // Find running processes
        app_data.find_pid_and_command();

        if let Some(s) = status {
            let total_process = app_data.app_process.len();
            s.with_message(format!("Found process {}", total_process))
                .emit();
        }

        // // If any processes found, show confirmation dialog
        // if !app_data.app_process.is_empty() {
        //     let user_confirmed = foundation::modal_kill_dialog(&app_data.app.name)?;

        //     if user_confirmed {
        //         // User chose Yes → kill processes
        //         AppProcess::kill_app_processes(&app_data.app.name, &app_data.app_process)?;

        //         if let Some(s) = status {
        //             s.with_stage("Completed")
        //                 .with_message("All processes killed")
        //                 .with_total(total_process)
        //                 .emit();
        //         }
        //     }
        // }

        Ok(Self { app_data })
    }

    pub fn confirm_and_kill_process(&self, status: Option<&StatusEmitter>) -> Result<()> {
        if self.app_data.app_process.is_empty() {
            return Ok(());
        }

        let user_confirmed = foundation::modal_kill_dialog(&self.app_data.app.name)?;

        if user_confirmed {
            let total_process = self.app_data.app_process.len();
            // User chose Yes → kill processes
            AppProcess::kill_app_processes(&self.app_data.app.name, &self.app_data.app_process)?;

            if let Some(s) = status {
                s.with_stage("Completed")
                    .with_message("All processes killed")
                    .with_total(total_process)
                    .emit();
            }
        }

        Ok(())
    }

    /// Scan an app at the given path and return AppData
    pub fn scan_app_data(&mut self, status: Option<&StatusEmitter>) -> Result<&Self> {
        if let Some(s) = status {
            s.with_message(format!(
                "Scanning logs and associated files for '{}'",
                self.app_data.app.name
            ))
            .emit();
        }

        if let Some(s) = status {
            s.with_stage("started")
                .with_message("Finding BOM logs...")
                .emit();
        }

        let locations = LocationsScan::new();

        self.app_data.find_log_bom(&locations);

        let total_bom_file = self.app_data.log.bom_file.len();

        if let Some(s) = status {
            s.with_stage("completed")
                .with_total(total_bom_file)
                .with_message("BOM logs scan completed")
                .emit();
        }

        if let Some(s) = status {
            s.with_stage("started")
                .with_message("Finding associated files...")
                .emit();
        }

        self.app_data
            .find_associate_files(&locations, |cur, _path| {
                if let Some(s) = status {
                    s.with_stage("Searching").with_current(cur).emit();
                }
            });

        if let Some(s) = status {
            s.with_stage("completed")
                .with_message("Associated files scan completed")
                .emit();
        }

        Ok(self)
    }

    /// Save BOM logs of the current app to the given folder
    pub fn save_bom_logs(&self, log_dir: &Path) -> Result<()> {
        // Determine the folder
        let app_log_folder =
            Path::new(log_dir).join(format!("{}_bom_log", &self.app_data.app.name));
        debug_dev!("Creating folder: {}", app_log_folder.display());

        // Call the LogReceipt function
        self.app_data.log.save_bom_log(&app_log_folder)
    }

    /// Move all associated files including the app itself to trash
    pub fn trash_all(&self) -> Result<Vec<(PathBuf, String)>> {
        // get all path in the associate_files field with enumerate
        let paths: Vec<PathBuf> = self
            .app_data
            .all_found_entries_enumerate()
            .iter()
            .map(|(_i, (path, _label))| path.clone())
            .collect();

        // delete all associate_files
        let failed_paths = foundation::trash_files_nsfilemanager(&paths)?;

        Ok(failed_paths)
    }

    /// Print a summary of the app data
    /// For CLI
    pub fn print_summary(&self) {
        println!("App Name: {}", self.app_data.app.name);
        println!("Bundle ID: {}", self.app_data.app.bundle_id);
        println!("Bundle Name: {}", self.app_data.app.bundle_name);

        println!("\nRunning processes:");
        for p in &self.app_data.app_process {
            println!("PID {}: {}", p.pid, p.command);
        }

        println!("\nLog BOM files:");
        for log in &self.app_data.log.bom_file {
            println!("{}", log.display());
        }

        println!("\nAssociated files:");
        for (_i, (path, label)) in &self.app_data.all_found_entries_enumerate() {
            println!("{} -> {}", label, path.display());
        }
    }

    pub fn show_in_finder(path: &Path) -> Result<()> {
        foundation::show_in_finder(path)
    }

    // fn confirm_kill_dialog(app_name: &str) -> Result<bool> {
    //     // let confirmation = foundation::kill_dialog(app_name);
    //     // Ok(confirmation)
    //     foundation::modal_kill_dialog(app_name)
    // }

    pub fn reset(&mut self) {
        self.app_data.reset();
    }
}
