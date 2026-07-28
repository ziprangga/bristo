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
//! Running process discovery and storage.
//!
//! This module is responsible for locating and storing processes
//! associated with an application.
//!
//! The module is built around two primary types:
//!
//! - `AppProcs` stores discovered processes.
//! - `Proc` represents a single running process.
//!
//! Process discovery uses application metadata to identify
//! potentially related processes currently running on the system.
//!
//! Matching is performed using information derived from the
//! target application, including:
//!
//! - Executable name.
//! - Bundle identifier.
//! - Organization identifier.
//! - Common helper process names.
//!
//! Runtime process information is used by higher-level cleanup
//! operations to determine whether an application is currently
//! active and whether running processes should be terminated
//! before removal.
//!
//! Note:
//! The information returned by this module reflects
//! the current process table and may change between scans.
//!..

use crate::app_profile::metadata::AppMetadata;
use mini_logger::debug;
use rayon::prelude::*;
use std::ffi::OsString;
use sysinfo::{ProcessesToUpdate, System};

/// Snapshot of a running process.
///
/// Doc:
/// Represents a single process discovered during runtime
/// scanning.
///
/// Each process stores:
///
/// - Process identifier (PID).
/// - Full command line.
/// - Process name.
///
/// The command line is retained because it often contains
/// application identifiers that are not present in the
/// process name alone.
///
/// Note:
/// `Proc` is a lightweight snapshot of process information
/// captured during scanning and does not maintain a live
/// connection to the operating system.
#[derive(Debug, Default, Clone)]
pub struct Proc {
    pid: i32,
    command: String,
    name: String,
}

impl Proc {
    /// Construct a Proc.
    pub fn new(pid: i32, command: String, name: String) -> Self {
        Self { pid, command, name }
    }
    /// get the copy of pid
    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// get the reference of command
    pub fn as_command(&self) -> &str {
        &self.command
    }

    /// get the reference of process name
    pub fn as_name(&self) -> &str {
        &self.name
    }
}

/// Collection of running application processes.
///
/// Doc:
/// Stores processes believed to belong to a specific
/// application.
///
/// Processes are discovered by matching runtime process
/// information against metadata extracted from the target
/// application.
///
/// The collection may contain:
///
/// - Main application processes.
/// - Helper processes.
/// - Background service processes.
/// - Child processes that expose matching identifiers.
///
/// Note:
/// The contents of this collection represent a snapshot of
/// the system at the time the scan was performed.
#[derive(Debug, Default, Clone)]
pub struct AppProcs {
    processes: Vec<Proc>,
}

impl AppProcs {
    /// Returns all discovered processes.
    pub fn list(&self) -> &[Proc] {
        &self.processes
    }

    /// Returns true when no matching processes were found.
    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    /// Discovers running processes associated with an application.
    ///
    /// Doc:
    /// Scans the current process table and attempts to identify
    /// processes belonging to the provided application.
    ///
    /// Matching is performed using:
    ///
    /// - Executable name.
    /// - Bundle identifier.
    /// - Organization identifier.
    /// - Common helper process names.
    ///
    /// Both the process name and complete command line are
    /// inspected when evaluating matches.
    ///
    /// Matching processes are collected into a new
    /// `AppProcs` instance and returned to the caller.
    ///
    /// Design:
    /// Process identification relies on multiple metadata-derived
    /// patterns rather than executable names alone.
    ///
    /// Many macOS applications launch helper processes,
    /// background services, or child processes whose names may
    /// differ from the primary application executable.
    ///
    /// Examples:
    ///
    /// ```text
    /// Google Chrome
    /// Google Chrome Helper
    ///
    /// Visual Studio Code
    /// Code Helper
    /// ```
    ///
    /// Matching against bundle identifiers, organization names,
    /// and command-line arguments improves detection accuracy
    /// across different application architectures.
    ///
    /// Note:
    /// Process discovery is inherently heuristic-based and may
    /// produce false positives when unrelated processes contain
    /// similar identifiers.
    pub fn find_app_processes(app_metadata: &AppMetadata) -> Self {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        // Design:
        // Many macOS applications spawn helper processes using a
        // naming convention similar to:
        //
        //     "<Executable> Helper"
        //
        // These helper processes often remain active even when the
        // primary application process is not immediately visible.
        //
        // A helper pattern is therefore included as an additional
        // matching signal during process discovery.
        let helper = format!("{} Helper", app_metadata.as_bundle_executable_name());

        let patterns = [
            app_metadata.as_bundle_executable_name(),
            app_metadata.as_bundle_id(),
            app_metadata.as_organization(),
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
}
