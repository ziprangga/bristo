mod app_asc_files;
mod app_btm;
mod app_log_receipt;
mod app_metadata;
mod app_proc;

pub use app_asc_files::{AppAscFiles, AscData};
pub use app_btm::{AppBtmFiles, BtmData};
pub use app_log_receipt::AppLogReceipt;
pub use app_metadata::{AppMetadata, InfoPlist};
pub use app_proc::{AppProcs, Proc};

use crate::locations_scan::BtmLocations;
use crate::locations_scan::ScanLocations;
use anyhow::Result;
use mini_logger::debug;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum FileEntry {
    AppPath(AppMetadata),
    AscFiles(AscData),
    BtmFiles(BtmData),
}

impl FileEntry {
    pub fn as_path(&self) -> &Path {
        match self {
            Self::AppPath(v) => v.as_path(),
            Self::AscFiles(v) => v.as_path(),
            Self::BtmFiles(v) => v.as_path(),
        }
    }

    pub fn as_name(&self) -> &str {
        match self {
            Self::AppPath(v) => v.as_info().as_name(),
            Self::AscFiles(v) => v.as_name(),
            Self::BtmFiles(v) => v.as_name(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AppProfile {
    app_metadata: AppMetadata,
    app_procs: AppProcs,
    app_log_receipt: AppLogReceipt,
    app_asc_files: AppAscFiles,
    app_btm_files: AppBtmFiles,
}

impl AppProfile {
    pub fn new(
        app_metadata: AppMetadata,
        app_procs: AppProcs,
        app_log_receipt: AppLogReceipt,
        app_asc_files: AppAscFiles,
        app_btm_files: AppBtmFiles,
    ) -> Self {
        Self {
            app_metadata,
            app_procs,
            app_log_receipt,
            app_asc_files,
            app_btm_files,
        }
    }

    pub fn from_path(app_path: &Path) -> Result<Self> {
        let app_metadata = AppMetadata::from_path(app_path)?;

        Ok(Self {
            app_metadata: app_metadata,
            app_procs: AppProcs::default(),
            app_log_receipt: AppLogReceipt::default(),
            app_asc_files: AppAscFiles::default(),
            app_btm_files: AppBtmFiles::default(),
        })
    }

    pub fn as_app_metadata(&self) -> &AppMetadata {
        &self.app_metadata
    }

    pub fn as_app_procs(&self) -> &AppProcs {
        &self.app_procs
    }

    pub fn as_app_log_receipt(&self) -> &AppLogReceipt {
        &self.app_log_receipt
    }

    pub fn as_app_asc_files(&self) -> &AppAscFiles {
        &self.app_asc_files
    }

    pub fn as_app_btm_files(&self) -> &AppBtmFiles {
        &self.app_btm_files
    }

    // method to replace path of associate file when failed moved to trash
    pub fn replace_file_entries(&mut self, entries: Vec<FileEntry>) {
        let mut app_metadata = None;
        let mut asc_files = Vec::new();
        let mut btm_files = Vec::new();

        for entry in entries {
            match entry {
                FileEntry::AppPath(app) => {
                    app_metadata = Some(app);
                }

                FileEntry::BtmFiles(file) => {
                    btm_files.push(file);
                }

                FileEntry::AscFiles(file) => {
                    asc_files.push(file);
                }
            }
        }

        if let Some(app) = app_metadata {
            self.app_metadata = app;
        }

        self.app_asc_files.set_asc_files(asc_files);
        self.app_btm_files.set_btm_files(btm_files);
    }

    pub fn find_pid_and_command(&mut self) {
        self.app_procs = AppProcs::find_app_processes(&self.app_metadata);

        // debug list of the app process
        for _p in self.app_procs.list() {
            debug!(
                "list of process app: PID {}: cmd_line = '{}' name = '{}'",
                _p.pid(),
                _p.as_command(),
                _p.as_name()
            );
        }
    }

    pub fn find_log_bom(&mut self, locations: &ScanLocations) {
        self.app_log_receipt
            .find_bom_files(&self.app_metadata, locations);
    }

    // Scan all file associate from list of location
    // use in_progress as emitter status to caller
    pub fn find_associate_files<F>(&mut self, locations: &ScanLocations, in_progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync,
    {
        self.app_asc_files
            .scan_asc_files(&self.app_metadata, locations, in_progress);
    }

    // Scan all file btm from list of location
    // use in_progress as emitter status to caller
    pub fn find_btm_files<F>(&mut self, locations: &BtmLocations, in_progress: F)
    where
        F: Fn(usize, &Path) + Send + Sync,
    {
        self.app_btm_files
            .scan_btm_files(&self.app_metadata, locations, in_progress);
    }

    // merged all entry
    pub fn all_entries(&self) -> Vec<FileEntry> {
        let mut entries = Vec::new();

        // AscFiles
        entries.extend(
            self.app_asc_files
                .as_asc_files()
                .iter()
                .cloned()
                .map(FileEntry::AscFiles),
        );

        // BtmFiles
        entries.extend(
            self.app_btm_files
                .as_btm_files()
                .iter()
                .cloned()
                .map(FileEntry::BtmFiles),
        );

        // AppPath
        entries.push(FileEntry::AppPath(self.app_metadata.clone()));

        entries
    }

    pub fn reset(&mut self) {
        self.app_metadata = AppMetadata::default();
        self.app_procs = AppProcs::default();
        self.app_log_receipt = AppLogReceipt::default();
        self.app_asc_files = AppAscFiles::default();
        self.app_btm_files = AppBtmFiles::default();
    }
}
