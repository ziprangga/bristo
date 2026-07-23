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

use crate::errors::{ErrorKind, Result};

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
        // .with_context(|| format!("Failed to run lsbom on {}", bom_file.display()))?;
        .map_err(|e| {
            ErrorKind::failed()
                .with_summary("Utility execution failed")
                .with_reason(format!(
                    "Failed to run lsbom on {}: {}",
                    bom_file.display(),
                    e
                ))
        })?;

    if output.status.success() {
        let mut f = File::create(output_file).map_err(|e| {
            ErrorKind::failed()
                .with_summary("File creation failed")
                .with_reason(format!(
                    "Failed to create file {}: {}",
                    output_file.display(),
                    e
                ))
        })?;
        f.write_all(&output.stdout).map_err(|e| {
            ErrorKind::failed()
                .with_summary("File writing failed")
                .with_reason(format!(
                    "Failed to write BOM log to {}: {}",
                    output_file.display(),
                    e
                ))
        })?;
        println!("Saved BOM log: {}", output_file.display());
        Ok(())
    } else {
        Err(ErrorKind::failed()
            .with_summary("BOM parsing utility failure")
            .with_reason(format!(
                "lsbom failed for {}: {}",
                bom_file.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            )))
    }
}
