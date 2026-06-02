mod proc;
pub use proc::Proc;

use crate::app_profile::app_metadata::AppMetadata;
use mini_logger::debug;
use rayon::prelude::*;
use std::ffi::OsString;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug, Default, Clone)]
pub struct AppProcs {
    processes: Vec<Proc>,
}

impl AppProcs {
    // Scan process running for app
    pub fn find_app_processes(app_metadata: &AppMetadata) -> Self {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let helper = format!(
            "{} Helper",
            app_metadata.as_info().as_bundle_executable_name()
        );

        let patterns = [
            app_metadata.as_info().as_bundle_executable_name(),
            app_metadata.as_info().as_bundle_id(),
            app_metadata.as_info().as_organization(),
            helper.as_str(),
        ];

        let processes = sys
            .processes()
            .par_iter()
            .filter_map(|(&pid, process)| {
                // Join full command line for debug
                let cmd_line = process
                    .cmd()
                    .iter()
                    .map(|s: &OsString| s.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(" ");

                // Convert process.name() to string for pattern matching
                let process_name = process.name().to_string_lossy().to_string();

                debug!(
                    "PID {}: cmd_line = '{}', process = '{}', checking patterns {:?}",
                    pid, cmd_line, process_name, patterns
                );

                // Match if command line contains pattern OR process name contains pattern
                let is_match = patterns
                    .iter()
                    .any(|pat| cmd_line.contains(pat) || process_name.contains(pat));

                if is_match {
                    // Contruct the result
                    Some(Proc::new(pid.as_u32() as i32, cmd_line, process_name))
                } else {
                    None
                }
            })
            .collect();

        Self { processes }
    }

    pub fn list(&self) -> &[Proc] {
        &self.processes
    }

    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }
}
