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
//! macOS platform utility helpers.
//!
//! Provides utilities for interacting with macOS-specific
//! system tools and metadata formats.
//!
//! Note:
//! These helpers depend on system utilities like native macOS command-line tools.
//!..

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Exports a BOM file to a text log.
///
/// Doc:
/// Executes the macOS `lsbom` utility and writes the resulting
/// file listing to the specified output file.
///
/// Design:
/// The implementation delegates BOM parsing to Apple's
/// official tooling rather than attempting to interpret the
/// BOM format directly.
///
/// This ensures compatibility with system-generated BOM files
/// while keeping maintenance requirements low.
///
/// Note:
/// An error is returned when:
///
/// - `lsbom` cannot be executed.
/// - The BOM file is invalid.
/// - The output file cannot be written.
///
pub fn run_lsbom_command(bom_file: &Path, output_file: &Path) -> Result<()> {
    let bom_file_str = bom_file.to_string_lossy();

    let output = Command::new("lsbom")
        .args(["-f", "-l", "-s", "-p", "f", &bom_file_str])
        .output()
        .with_context(|| format!("Failed to run lsbom on {}", bom_file.display()))?;

    if output.status.success() {
        let mut f = File::create(output_file)
            .with_context(|| format!("Failed to create file: {}", output_file.display()))?;
        f.write_all(&output.stdout)
            .with_context(|| format!("Failed to write BOM log: {}", output_file.display()))?;
        println!("Saved BOM log: {}", output_file.display());
        Ok(())
    } else {
        anyhow::bail!(
            "lsbom failed for {}: {}",
            bom_file.display(),
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

// pub fn run_cmd_as_root(command: &str) -> Result<(), String> {
//     let escaped_command = command.replace("\"", "\\\"");
//     let script = format!(
//         "do shell script \"{}\" with administrator privileges",
//         escaped_command
//     );

//     let output = Command::new("osascript")
//         .arg("-e")
//         .arg(&script)
//         .output()
//         .map_err(|e| format!("Failed to execute osascript: {}", e))?;

//     if output.status.success() {
//         Ok(())
//     } else {
//         let error_msg = String::from_utf8_lossy(&output.stderr);
//         Err(format!(
//             "Authorization failed or user cancelled: {}",
//             error_msg.trim()
//         ))
//     }
// }
