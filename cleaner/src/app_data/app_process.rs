use anyhow::Result;
use rayon::prelude::*;
use std::ffi::OsString;
use sysinfo::{ProcessesToUpdate, System};

use crate::AppInfo;
use crate::syscom::kill_pids;
use mini_logger::debug;

#[derive(Debug, Default, Clone)]
pub struct AppProcess {
    pub pid: i32,
    pub command: String,
    pub process_name: String,
}

impl AppProcess {
    // Scan process running for app from AppInfo data
    pub fn find_app_processes(app: &AppInfo) -> Vec<Self> {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let patterns = [
            app.bundle_executable_name.clone(),
            app.bundle_id.clone(),
            app.organization.clone(),
            format!("{} Helper", app.bundle_executable_name),
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
                    Some(Self {
                        pid: pid.as_u32() as i32,
                        command: cmd_line,
                        process_name: process_name.into_owned(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn kill_app_processes(app_name: &str, processes: &[AppProcess]) -> Result<usize> {
        if processes.is_empty() {
            println!("No running processes found for {}", app_name);
            return Ok(0);
        }

        let mut killed_count = 0;

        for p in processes {
            if kill_pids(&p.pid.to_string()).is_ok() {
                killed_count += 1;
            } else {
                eprintln!("Failed to kill PID {} for {}", p.pid, app_name);
            }
        }

        Ok(killed_count)
    }
}
