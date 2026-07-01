use anyhow::Result;
use anyhow::anyhow;
use std::ffi::CStr;
use std::path::Path;
use std::path::PathBuf;
// ============
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
// ============
use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
// use objc2_foundation::NSArray;
// use objc2_foundation::{NSError, NSFileManager, NSString, NSURL};
use objc2_foundation::{NSArray, NSAutoreleasePool, NSFileManager, NSString, NSURL};

// ============
use libc::confstr;
use libc::{SIGTERM, c_int, kill};

pub const DARWIN_USER_CACHE_DIR: i32 = libc::_CS_DARWIN_USER_CACHE_DIR;
pub const DARWIN_USER_TEMP_DIR: i32 = libc::_CS_DARWIN_USER_TEMP_DIR;

/// System configuration path resolver (Safe libc buffer sizing)
pub fn sysconf_path(name: i32) -> Option<PathBuf> {
    // get required buffer size
    // let len = confstr(name, std::ptr::null_mut(), 0);
    let len = unsafe { confstr(name, std::ptr::null_mut(), 0) };
    if len == 0 {
        return None;
    }

    // Allocate the exact buffer size
    let mut buf = vec![0u8; len as usize];

    // let written = confstr(name, buf.as_mut_ptr() as *mut _, len);
    let written = unsafe { confstr(name, buf.as_mut_ptr() as *mut _, len) };
    if written == 0 {
        return None;
    }

    // let s = CStr::from_ptr(buf.as_ptr() as *const _)
    //     .to_string_lossy()
    //     .trim() // remove newline if any
    //     .trim_end_matches('/') // match bash sed
    //     .to_string();

    // Some(PathBuf::from(s))

    // preventing out-of-bounds pointer scanning if trailing bytes change.
    let c_str = CStr::from_bytes_with_nul(&buf).ok()?;

    // Extract raw OS bytes directly into PathBuf (bypasses lossy UTF-8 conversion)
    let raw_bytes = c_str.to_bytes();
    let os_str = OsStr::from_bytes(raw_bytes);

    // Clean trailing spaces or path delimiters matching standard Unix shell scripts
    let trimmed_bytes = os_str.as_bytes();
    let mut end = trimmed_bytes.len();
    while end > 0 && (trimmed_bytes[end - 1] == b'/') {
        end -= 1;
    }

    Some(PathBuf::from(OsStr::from_bytes(&trimmed_bytes[..end])))
}

/// Closes running system processes safely via PIDs
pub fn kill_pids(pids: &str) -> Result<()> {
    // collect error
    let mut errors = Vec::new();

    for pid_str in pids.split_whitespace() {
        // parse PID
        let pid = pid_str
            // can change it with .parse::<libc::pid_t>()
            .parse::<i32>()
            .map_err(|_| anyhow!("Invalid PID: {}", pid_str))?;

        // Invoke the direct Unix kill system call via the libc crate
        // Using libc::SIGTERM guarantees the correct platform signal macro code
        // can use let ret = unsafe { kill(pid, SIGTERM) }; too
        let ret = unsafe { kill(pid as c_int, SIGTERM) };

        if ret != 0 {
            // errno contains the error code
            let err = std::io::Error::last_os_error();
            // return Err(anyhow!("Failed to kill PID {}: {}", pid, err));

            // Ignore if the process has already exited.
            // if err.kind() != std::io::ErrorKind::NotFound {
            //     errors.push(format!("PID {}: {}", pid, err));
            // }
            if err.raw_os_error() != Some(libc::ESRCH) {
                errors.push(format!("PID {}: {}", pid, err));
            }
        }
    }

    // If any signals failed to deliver, collect them all into an aggregate error reports
    if !errors.is_empty() {
        return Err(anyhow!(
            "Failed to stop some processes:\n{}",
            errors.join("\n")
        ));
    }

    Ok(())
}

/// Trashes a collection of files using native Apple NSFileManager mechanisms
pub fn trash_files_nsfilemanager(paths: &[PathBuf]) -> Result<Vec<(PathBuf, String)>> {
    let mut failed_paths = Vec::new();

    if paths.is_empty() {
        return Ok(failed_paths);
    }

    // let _mtm = MainThreadMarker::new()
    //     .ok_or_else(|| anyhow!("Trashing files must be executed on the Main Thread"))?;

    // unsafe {
    //     let pool = NSAutoreleasePool::new();
    //     // NSFileManager *fm = [NSFileManager defaultManager]
    //     // let fm: Retained<NSFileManager> = msg_send![NSFileManager::class(), defaultManager];
    //     // Use the safe high-level binding defaultManager method instead of msg_send!
    //     let fm = NSFileManager::defaultManager();

    //     let urls: Vec<Retained<NSURL>> = paths
    //         .iter()
    //         .filter_map(|path| {
    //             let s = path.to_str()?;
    //             let ns_string = NSString::from_str(s);
    //             // let url: Retained<NSURL> = msg_send![NSURL::class(), fileURLWithPath: &*ns_string];
    //             // Some(url)
    //             // Standard binding call
    //             let url: Retained<NSURL> = NSURL::fileURLWithPath(&ns_string);
    //             Some(url)
    //         })
    //         .collect();

    //     // for (i, url) in urls.iter().enumerate() {
    //     //     let mut resulting_url: *mut NSURL = std::ptr::null_mut();
    //     //     let mut error: *mut NSError = std::ptr::null_mut();

    //     //     let success: bool = msg_send![
    //     //         &*fm,
    //     //         trashItemAtURL: &**url,
    //     //         resultingItemURL: &mut resulting_url,
    //     //         error: &mut error
    //     //     ];

    //     //     if !success {
    //     //         let reason = if !error.is_null() {
    //     //             let domain = (*error).domain().to_string();
    //     //             let code = (*error).code();
    //     //             if domain == "NSCocoaErrorDomain" && code == 513 {
    //     //                 "Permission not allowed by macOS privacy protection (TCC)".to_string()
    //     //             } else {
    //     //                 format!("Failed with {} ({})", domain, code)
    //     //             }
    //     //         } else {
    //     //             "unknown reason".to_string()
    //     //         };

    //     //         failed_paths.push((paths[i].clone(), reason));
    //     //     }
    //     // }
    //     for (i, url) in urls.iter().enumerate() {
    //         // FIXED: Call the correct method name ending in _error
    //         // It automatically captures failures and returns a standard Rust Result enum!
    //         if let Err(error) = fm.trashItemAtURL_resultingItemURL_error(url, None) {
    //             let domain = error.domain().to_string();
    //             let code = error.code();

    //             let reason = if domain == "NSCocoaErrorDomain" && code == 513 {
    //                 "Permission not allowed by macOS privacy protection (TCC)".to_string()
    //             } else {
    //                 format!("Failed with {} ({})", domain, code)
    //             };

    //             failed_paths.push((paths[i].clone(), reason));
    //         }
    //     }

    //     drop(pool);
    // }

    // Initialize an isolated Autorelease Pool instance block safely
    let pool = unsafe { NSAutoreleasePool::new() };

    // Fetch the standard filesystem workspace context reference
    let fm = NSFileManager::defaultManager();

    // Map system strings cleanly into Objective-C managed instances
    let urls: Vec<Retained<NSURL>> = paths
        .iter()
        .filter_map(|path| {
            let s = path.to_str()?;
            let ns_string = NSString::from_str(s);
            // Autogenerated wrapper for [NSURL fileURLWithPath:]
            Some(NSURL::fileURLWithPath(&ns_string))
        })
        .collect();

    // Execute the deletion sequences sequentially
    for (path, url) in paths.iter().zip(urls.iter()) {
        // Pass a null mutable pointer to skip tracking the final destination path inside the Trash
        // let destination_ptr = std::ptr::null_mut();
        let mut resulting_url: Option<Retained<NSURL>> = None;
        let out_url = Some(&mut resulting_url);

        // The autogenerated binding method returns a standard Rust Result enum!
        if let Err(error) = fm.trashItemAtURL_resultingItemURL_error(url, out_url) {
            let domain = error.domain().to_string();
            let code = error.code();

            let reason = if domain == "NSCocoaErrorDomain" && code == 513 {
                "Permission not allowed by macOS privacy protection (TCC)".to_string()
            } else {
                format!("Failed with {} ({})", domain, code)
            };

            failed_paths.push((path.clone(), reason));
        }
    }

    // Safely drain the pooled memory structure now that your array iterations have completed
    drop(pool);

    Ok(failed_paths)
}

pub fn show_in_finder(path: &Path) -> Result<()> {
    let s = path
        .to_str()
        .ok_or_else(|| anyhow!("Path is not valid UTF-8"))?;

    // Enforce main thread constraint for interacting with Finder UI elements
    // let _mtm = MainThreadMarker::new()
    //     .ok_or_else(|| anyhow!("Showing file in Finder must run on the Main Thread"))?;

    // let ns_path = NSString::from_str(s);
    // let url = NSURL::fileURLWithPath(&ns_path);
    // let urls = NSArray::from_slice(&[&*url]);
    // let workspace = NSWorkspace::sharedWorkspace();

    // unsafe {
    //     let _: () = msg_send![&workspace, activateFileViewerSelectingURLs: &*urls];
    // }

    // unsafe {
    //     let pool = NSAutoreleasePool::new();

    //     let ns_path = NSString::from_str(s);
    //     let url: Retained<NSURL> = NSURL::fileURLWithPath(&ns_path);

    //     // Construct standard NSArray vector slices cleanly
    //     let urls = {
    //         let slice: &[&NSURL] = &[&url];
    //         NSArray::from_slice(slice)
    //     };

    //     let workspace = NSWorkspace::sharedWorkspace();

    //     // 5. FIXED: No more unsafe msg_send! needed.
    //     // This high-level binding is fully exposed by objc2-app-kit!
    //     workspace.activateFileViewerSelectingURLs(&urls);

    //     drop(pool);
    // }

    // Initialize an isolated Autorelease Pool instance safely
    let pool = unsafe { NSAutoreleasePool::new() };

    // Generate standard Objective-C strings and native file URLs
    let ns_path = NSString::from_str(s);
    let url: Retained<NSURL> = NSURL::fileURLWithPath(&ns_path);

    // Construct a standard NSArray cleanly using a slice of Retained items
    // Using from_slice with an explicit slice array of the Retained pointer type
    // let urls_slice: &[Retained<NSURL>] = &[url];
    // let urls: Retained<NSArray<NSURL>> = NSArray::from_slice(urls_slice);

    // NSArray::from_slice expects &[&NSURL] for objc2-foundation 0.3
    let slice: &[&NSURL] = &[&url];
    let urls = NSArray::from_slice(slice);

    // Safely call the workspace binding without any unsafe block
    let workspace = NSWorkspace::sharedWorkspace();
    workspace.activateFileViewerSelectingURLs(&urls);

    // Clean up the pooled allocation layers
    drop(pool);

    Ok(())
}
