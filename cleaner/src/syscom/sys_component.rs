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
//! System process, filesystem, and Finder integration helpers.
//!
//! Provides low-level wrappers around macOS APIs and Unix
//! system calls used throughout the application.
//!
//! Responsibilities include:
//!
//! - Process termination.
//! - Trash operations.
//! - Finder integration.
//! - User cache discovery.
//! - User temporary directory discovery.
//!
//! Note:
//! Most callers should access these helpers through higher-
//! level abstractions such as `Cleaner`.
//!..

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
use objc2_foundation::{NSArray, NSAutoreleasePool, NSFileManager, NSString, NSURL};

// ============
use libc::SIGTERM;
use libc::confstr;
use libc::kill;

pub const DARWIN_USER_CACHE_DIR: i32 = libc::_CS_DARWIN_USER_CACHE_DIR;
pub const DARWIN_USER_TEMP_DIR: i32 = libc::_CS_DARWIN_USER_TEMP_DIR;

/// Resolves a system configuration path.
///
/// Doc:
/// Retrieves a filesystem path from a libc `confstr` entry.
///
/// Design:
/// Buffer sizing is performed using the standard two-step
/// `confstr` pattern:
///
/// 1. Query required size.
/// 2. Allocate exact storage.
/// 3. Retrieve value.
///
/// This avoids fixed-size buffers and truncation risks.
///
/// Note:
/// Returns `None` when the requested configuration value is
/// unavailable.
pub fn sysconf_path(name: i32) -> Option<PathBuf> {
    // get required buffer size
    let len = unsafe { confstr(name, std::ptr::null_mut(), 0) };
    if len == 0 {
        return None;
    }

    // Allocate the exact buffer size
    let mut buf = vec![0u8; len as usize];

    let written = unsafe { confstr(name, buf.as_mut_ptr() as *mut _, len) };
    if written == 0 {
        return None;
    }

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

/// Terminates one or more processes.
///
/// Doc:
/// Sends `SIGTERM` to each provided process identifier.
///
/// Design:
/// `SIGTERM` is used instead of stronger termination signals
/// because it allows applications an opportunity to perform
/// graceful shutdown and cleanup operations.
///
/// Failures are accumulated and returned as a single error.
///
/// Note:
/// Processes that have already exited are ignored.
pub fn kill_pids(pids: &str) -> Result<()> {
    // collect error
    let mut errors = Vec::new();

    for pid_str in pids.split_whitespace() {
        // parse PID
        let pid = pid_str
            // can change it with .parse::<i32>()
            .parse::<libc::pid_t>()
            .map_err(|_| anyhow!("Invalid PID: {}", pid_str))?;

        // Invoke the direct Unix kill system call via the libc crate
        // Using libc::SIGTERM guarantees the correct platform signal macro code
        // can use let ret = unsafe { kill(pid as libc::c_int, SIGTERM) }; when using i32 in parsing pid
        let ret = unsafe { kill(pid, SIGTERM) };

        if ret != 0 {
            // errno contains the error code
            let err = std::io::Error::last_os_error();
            // Ignore if the process has already exited.
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

/// Moves files to the macOS Trash.
///
/// Doc:
/// Attempts to move each provided path into the user's Trash
/// using native macOS filesystem APIs.
///
/// Design:
/// File removal is delegated to `NSFileManager` rather than
/// direct filesystem deletion.
///
/// This preserves standard macOS behavior:
///
/// - Files are recoverable.
/// - Finder remains synchronized.
/// - Trash metadata is maintained.
/// - User expectations are respected.
///
/// The implementation intentionally avoids permanent deletion.
///
/// Note:
/// The returned vector contains only failed operations and
/// their associated reasons.
pub fn trash_files_nsfilemanager(paths: &[PathBuf]) -> Result<Vec<(PathBuf, String)>> {
    let mut failed_paths = Vec::new();

    if paths.is_empty() {
        return Ok(failed_paths);
    }

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

            let reason = match (domain == "NSCocoaErrorDomain", code) {
                (true, 513) => {
                    "Permission not allowed by macOS privacy protection (TCC)".to_string()
                }
                _ => format!("Failed with {} ({})", domain, code),
            };

            failed_paths.push((path.clone(), reason));
        }
    }

    // Safely drain the pooled memory structure now that your array iterations have completed
    drop(pool);

    Ok(failed_paths)
}

/// Reveals a file in Finder.
///
/// Doc:
/// Opens Finder and selects the specified filesystem item.
///
/// Design:
/// Uses `NSWorkspace` so behavior matches the native
/// "Reveal in Finder" experience provided by macOS
/// applications.
///
/// Note:
/// The target path must be representable as a valid UTF-8
/// string.
pub fn show_in_finder(path: &Path) -> Result<()> {
    let s = path
        .to_str()
        .ok_or_else(|| anyhow!("Path is not valid UTF-8"))?;

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
