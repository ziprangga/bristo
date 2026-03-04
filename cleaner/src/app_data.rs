mod app_info;
mod app_process;
mod associate_files;
mod locations_scan;
mod log_receipt;
mod plist_reader;

pub use app_info::AppInfo;
pub use app_process::AppProcess;
pub use associate_files::AssociateFiles;
pub use locations_scan::{LocationsScan, SandboxContainerLocation};
pub use log_receipt::LogReceipt;
pub use plist_reader::PlistReader;

use anyhow::Result;
use std::path::{Path, PathBuf};

#[cfg(debug_assertions)]
use common_debug::debug_dev;

#[derive(Debug, Default, Clone)]
pub struct AppData {
    pub app: AppInfo,
    pub app_process: Vec<AppProcess>,
    pub log: LogReceipt,
    pub associate_files: AssociateFiles,
}

impl AppData {
    pub fn new(app_path: &Path) -> Result<Self> {
        // Create AppInfo from path
        let app_info = AppInfo::from_path(app_path)?;

        Ok(Self {
            app: app_info,
            app_process: Vec::new(),
            log: LogReceipt::default(),
            associate_files: AssociateFiles::default(),
        })
    }

    pub fn find_pid_and_command(&mut self) {
        self.app_process = AppProcess::find_app_processes(&self.app);

        // debug list of the app process
        #[cfg(debug_assertions)]
        for _p in &self.app_process {
            debug_dev!(
                "list of process app: PID {}: cmd_line = '{}' name = '{}'",
                _p.pid,
                _p.command,
                _p.process_name
            );
        }
    }

    pub fn find_log_bom(&mut self, locations: &LocationsScan) {
        self.log.find_bom_files(&self.app, locations);
    }

    // Scan all file associate from list of location
    // for huge directory and try using walkdir + rayon
    // use in_progress as emitter status to caller
    pub fn find_associate_files<F>(&mut self, locations: &LocationsScan, in_progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync,
    {
        self.associate_files
            .scan_associate_files(&self.app, locations, in_progress);
    }

    // ===============All Associate file with enumerate==================
    pub fn all_associate_entries_enumerate(&self) -> Vec<(usize, (PathBuf, String))> {
        self.associate_files
            .associate_files
            .iter()
            .enumerate()
            .map(|(i, (path, label))| (i, (path.clone(), label.clone())))
            .collect()
    }

    // =======Save All Bom Log that was founded==============
    pub fn save_bom_log_app(&self, log_dir: &Path) -> Result<()> {
        if self.log.bom_file.is_empty() {
            anyhow::bail!("No BOM files found for app: {}", self.app.name);
        }

        self.log.save_bom_log(log_dir)
    }

    pub fn reset(&mut self) {
        self.app = AppInfo::default();
        self.app_process.clear();
        self.log = LogReceipt::default();
        self.associate_files = AssociateFiles::default();
    }
}
