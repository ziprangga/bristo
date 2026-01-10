use anyhow::Result;
use anyhow::anyhow;
use std::path::Path;
use std::path::PathBuf;
// use std::process::Command;
// ============
use objc2::rc::Retained;
use objc2::{ClassType, msg_send};
use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSArray;
use objc2_foundation::{NSError, NSFileManager, NSString, NSURL};

pub fn trash_files_nsfilemanager(paths: &[PathBuf]) -> Result<Vec<(PathBuf, String)>> {
    let mut failed_paths = Vec::new();

    if paths.is_empty() {
        return Ok(failed_paths);
    }

    unsafe {
        // NSFileManager *fm = [NSFileManager defaultManager]
        let fm: Retained<NSFileManager> = msg_send![NSFileManager::class(), defaultManager];

        let urls: Vec<Retained<NSURL>> = paths
            .iter()
            .filter_map(|path| {
                let s = path.to_str()?;
                let ns_string = NSString::from_str(s);
                let url: Retained<NSURL> = msg_send![NSURL::class(), fileURLWithPath: &*ns_string];
                Some(url)
            })
            .collect();

        for (i, url) in urls.iter().enumerate() {
            let mut resulting_url: *mut NSURL = std::ptr::null_mut();
            let mut error: *mut NSError = std::ptr::null_mut();

            let success: bool = msg_send![
                &*fm,
                trashItemAtURL: &**url,
                resultingItemURL: &mut resulting_url,
                error: &mut error
            ];

            if !success {
                let reason = if !error.is_null() {
                    let domain = (*error).domain().to_string();
                    let code = (*error).code();
                    if domain == "NSCocoaErrorDomain" && code == 513 {
                        "Permission not allowed by macOS privacy protection (TCC)".to_string()
                    } else {
                        format!("Failed with {} ({})", domain, code)
                    }
                } else {
                    "unknown reason".to_string()
                };

                failed_paths.push((paths[i].clone(), reason));
            }
        }
    }

    Ok(failed_paths)
}

pub fn show_in_finder(path: &Path) -> Result<()> {
    let s = path
        .to_str()
        .ok_or_else(|| anyhow!("Path is not valid UTF-8"))?;

    let ns_path = NSString::from_str(s);
    let url = NSURL::fileURLWithPath(&ns_path);

    let urls = NSArray::from_slice(&[&*url]);

    let workspace = NSWorkspace::sharedWorkspace();

    unsafe {
        let _: () = msg_send![&workspace, activateFileViewerSelectingURLs: &*urls];
    }

    Ok(())
}

// pub fn show_in_finder(path: &Path) -> Result<()> {
//     Command::new("open").arg("-R").arg(path).status()?;

//     Ok(())
// }

// /// Move  paths to trash
// pub fn trash_files(paths: &[PathBuf]) -> Result<()> {
//     if paths.is_empty() {
//         return Ok(());
//     }

//     let script = paths
//         .iter()
//         .map(|p| format!("POSIX file \"{}\"", p.display()))
//         .collect::<Vec<_>>()
//         .join(", ");

//     let applescript = format!(
//         "tell application \"Finder\" to move {{{}}} to trash",
//         script
//     );

//     Command::new("osascript")
//         .arg("-e")
//         .arg(applescript)
//         .status()?;

//     Ok(())
// }
