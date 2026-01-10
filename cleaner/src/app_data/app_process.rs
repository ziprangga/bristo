use anyhow::Result;
use rayon::prelude::*;
use std::ffi::OsString;
use sysinfo::{ProcessesToUpdate, System};

use crate::AppInfo;
use crate::foundation::kill_pids;
use common_debug::debug_dev;

#[derive(Debug, Clone)]
pub struct AppProcess {
    pub pid: i32,
    pub command: String,
    pub process_name: String,
}

impl AppProcess {
    pub fn new(pid: i32, command: String, process_name: String) -> Self {
        Self {
            pid,
            command,
            process_name,
        }
    }

    pub fn find_app_processes(app: &AppInfo) -> Vec<Self> {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let patterns = [
            app.bundle_name.clone(),
            app.bundle_id.clone(),
            app.organization.clone(),
            format!("{} Helper", app.bundle_name),
        ];

        sys.processes()
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
                let process_name = process.name().to_string_lossy();

                debug_dev!(
                    "PID {}: cmd_line = '{}', process = '{}', checking patterns {:?}",
                    pid,
                    cmd_line,
                    process_name,
                    patterns
                );

                // Match if command line contains pattern OR process name contains pattern
                let is_match = patterns
                    .iter()
                    .any(|pat| cmd_line.contains(pat) || process_name.contains(pat));

                if is_match {
                    Some(Self::new(
                        pid.as_u32() as i32,
                        cmd_line,
                        process_name.into_owned(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn kill_app_processes(app_name: &str, processes: &[AppProcess]) -> Result<()> {
        if processes.is_empty() {
            println!("No running processes found for {}", app_name);
            return Ok(());
        }

        let pids = processes
            .iter()
            .map(|p| p.pid.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        kill_pids(&pids)?;

        Ok(())
    }
}
