// use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use rfd::AsyncFileDialog;

use cleaner::Cleaner;
use status::StatusEmitter;

pub async fn set_input_path() -> Result<Arc<PathBuf>> {
    let file = AsyncFileDialog::new()
        .set_title("Browse App")
        .add_filter("Application", &["app"])
        .pick_file()
        .await
        .ok_or_else(|| anyhow!("No application selected"))?;

    Ok(Arc::new(file.path().to_path_buf()))
}

pub async fn set_output_path() -> Result<Arc<PathBuf>> {
    let folder = AsyncFileDialog::new()
        .set_title("Select Output Folder")
        .pick_folder()
        .await
        .ok_or_else(|| anyhow!("No folder selected"))?;

    Ok(Arc::new(folder.path().to_path_buf()))
}

pub async fn add_app(path: PathBuf, status: Option<StatusEmitter>) -> Result<Cleaner> {
    tokio::task::spawn_blocking(move || Cleaner::new_app(&path, status.as_ref()))
        .await
        .map_err(|e| anyhow::anyhow!("Add application failed: {}", e))?
}

pub async fn confirm_kill_process(
    cleaner: Arc<Cleaner>,
    status: Option<StatusEmitter>,
) -> Result<()> {
    tokio::task::spawn_blocking(move || cleaner.confirm_and_kill_process(status.as_ref()))
        .await
        .map_err(|e| anyhow::anyhow!("Confirm and kill process failed: {}", e))?
}

pub async fn scan_app_async(
    mut cleaner: Cleaner,
    status: Option<StatusEmitter>,
) -> Result<Cleaner> {
    tokio::task::spawn_blocking(move || {
        let _ = cleaner.scan_app_data(status.as_ref());
        cleaner
    })
    .await
    .map_err(|e| anyhow::anyhow!("Scan failed: {}", e))
}

pub async fn open_loc_async(path: PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || Cleaner::show_in_finder(&path))
        .await
        .map_err(|e| anyhow::anyhow!("Open location failed: {}", e))?
}

pub async fn save_bom_logs_async(cleaner: Cleaner, log_dir: PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || cleaner.save_bom_logs(&log_dir))
        .await
        .map_err(|e| anyhow::anyhow!("Save bom  logs failed: {}", e))?
}

pub async fn trash_app_async(cleaner: Cleaner) -> Result<Vec<(PathBuf, String)>> {
    tokio::task::spawn_blocking(move || Cleaner::trash_all(&cleaner))
        .await
        .map_err(|e| anyhow::anyhow!("Move to trash failed: {}", e))?
}
