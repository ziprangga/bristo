use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// OS-dependent: calls `lsbom` for a single BOM file
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

// ===================================================
