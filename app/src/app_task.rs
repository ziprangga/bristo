use rfd::AsyncFileDialog;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// use cleaner::TrashEntry;
use cleaner::{Cleaner, IconCache};
use cleaner::{ErrorKind, Result};
use simple_status::{StatusEmitter, status_emit};

pub async fn set_input_path() -> Result<PathBuf> {
    let file = AsyncFileDialog::new()
        .set_title("Browse App")
        .add_filter("Application", &["app"])
        .pick_file()
        .await
        .ok_or_else(|| {
            ErrorKind::skipped()
                .with_summary("Selection Canceled")
                .with_reason("No application selected")
        })?;

    Ok(file.path().to_path_buf())
}

pub async fn set_output_path() -> Result<PathBuf> {
    let folder = AsyncFileDialog::new()
        .set_title("Select Output Folder")
        .pick_folder()
        .await
        .ok_or_else(|| {
            ErrorKind::skipped()
                .with_summary("Selection Canceled")
                .with_reason("No folder selected")
        })?;

    Ok(folder.path().to_path_buf())
}

pub async fn process_app(path: PathBuf, emitter: Option<Arc<StatusEmitter>>) -> Result<Cleaner> {
    let cleaner = tokio::task::spawn_blocking(move || {
        let progress_hook = |msg: std::borrow::Cow<'static, str>| {
            status_emit!(
                emitter.as_deref(),
                message: msg,
            );
        };

        let cleaner = Cleaner::new_profile(&path, Some(progress_hook))?;

        Ok(cleaner)
    })
    .await
    .map_err(|e| {
        ErrorKind::failed()
            .with_summary("Add application failed")
            .with_reason(e.to_string())
    })??;

    Ok(cleaner)
}

pub async fn find_app_process_async(
    mut cleaner: Cleaner,
    emitter: Option<Arc<StatusEmitter>>,
) -> Result<Cleaner> {
    let cleaner = tokio::task::spawn_blocking(move || {
        let progress_hook = |msg: std::borrow::Cow<'static, str>| {
            status_emit!(
                emitter.as_deref(),
                message: msg,
            );
        };
        cleaner.find_app_process(Some(progress_hook))?;

        Ok(cleaner)
    })
    .await
    .map_err(|e| {
        ErrorKind::failed()
            .with_summary("Find process failed")
            .with_reason(e.to_string())
    })??;

    Ok(cleaner)
}

pub async fn kill_app_process_async(
    cleaner: Arc<Cleaner>,
    emitter: Option<Arc<StatusEmitter>>,
) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let progress_hook = |cur: usize, total: usize| {
            status_emit!(
                emitter.as_deref(),
                action: "Kill application process",
                current: cur,
                total: total,
            );
        };
        cleaner.kill_app_process(Some(progress_hook))?;

        Ok(cleaner)
    })
    .await
    .map_err(|e| {
        ErrorKind::failed()
            .with_summary("Confirm and kill process failed")
            .with_reason(e.to_string())
    })??;

    Ok(())
}

pub async fn scan_app_async(
    mut cleaner: Cleaner,
    emitter: Option<Arc<StatusEmitter>>,
) -> Result<Cleaner> {
    let app_name = cleaner
        .as_app_profile()
        .as_app_metadata()
        .as_name()
        .to_string();

    status_emit!(
        async,
        emitter.as_deref(),
        "Scanning logs and associated files for '{}'",
        app_name
    );

    status_emit!(
        async,
        emitter.as_deref(),
        action: "Started",
        message: "Finding BOM logs...",
    );

    let emitter_cln_block = emitter.clone();
    let cleaner = tokio::task::spawn_blocking(move || {
        let progress_hook = |cur: usize, _path: &Path| {
            status_emit!(
                emitter_cln_block.as_deref(),
                action: "Searching",
                current: cur,
            );
        };

        cleaner.scan_app_profile(progress_hook)?;

        Ok(cleaner)
    })
    .await
    .map_err(|e| {
        ErrorKind::failed()
            .with_summary("Scan failed")
            .with_reason(format!("Task execution panicked: {}", e))
    })??;

    let total_founded = cleaner.as_app_profile().as_path_entry().all_paths().len();
    status_emit!(
        async,
        emitter.as_deref(),
        action: "Completed",
        message: format!("{} items found", total_founded),
    );

    Ok(cleaner)
}

pub async fn open_loc_async(path: PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || Cleaner::show_in_finder(&path))
        .await
        .map_err(|e| {
            ErrorKind::failed()
                .with_summary("Open location failed")
                .with_reason(e.to_string())
        })?
}

pub async fn save_bom_logs_async(cleaner: Cleaner, log_dir: PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || cleaner.save_bom_logs(&log_dir))
        .await
        .map_err(|e| {
            ErrorKind::failed()
                .with_summary("Save bom logs failed")
                .with_reason(e.to_string())
        })?
}

pub async fn trash_app_async(mut cleaner: Cleaner) -> Result<Cleaner> {
    tokio::task::spawn_blocking(move || {
        cleaner.move_to_trash()?;
        Ok(cleaner)
    })
    .await
    .map_err(|e| {
        ErrorKind::failed()
            .with_summary("Move to trash failed")
            .with_reason(e.to_string())
    })?
}

pub async fn get_icon_asset_async(path: PathBuf, target_size: f64) -> Result<IconCache> {
    let path_for_error = path.clone();
    let cache_option = tokio::task::spawn_blocking(move || IconCache::new(&path, target_size))
        .await
        .map_err(|e| {
            ErrorKind::failed()
                .with_summary("Icon asset load failed")
                .with_reason(e.to_string())
        })?;

    cache_option.ok_or_else(|| {
        ErrorKind::failed()
            .with_summary("Get asset icon failed")
            .with_reason(format!(
                "Failed to load icon for path: {:?}",
                path_for_error
            ))
    })
}
