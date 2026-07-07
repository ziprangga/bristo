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
//! Generic filesystem scanning utilities.
//!
//! This module provides the low-level scanning primitives used
//! throughout the application.
//!
//! The scanning system is responsible for:
//!
//! - Traversing filesystem locations.
//! - Applying application matching rules.
//! - Building typed scan results.
//! - Reporting scan progress.
//! - Normalizing and deduplicating results.
//!
//! Design:
//! Scanning responsibilities are intentionally separated into
//! three stages:
//!
//! - Discovery (`scan_general`, `scan_container`).
//! - Matching (caller-provided predicates).
//! - Result normalization (`construct_scanner_result`).
//!
//! This separation allows scanners to reuse the same traversal
//! infrastructure while applying different matching rules and
//! result types.
//!
//! Two scanning strategies are provided:
//!
//! - General filesystem traversal.
//! - Sandbox container discovery.
//!
//! Sandbox containers require specialized handling because
//! application identifiers are often stored inside container
//! metadata rather than directly in the container directory
//! name.
//!
//! Note:
//! This module is intentionally generic and does not contain
//! application-specific matching logic.
//!..

use rayon::prelude::*;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

/// Post-process scanner results.
///
/// Doc:
/// Normalizes scanner output into a stable result set.
///
/// Capabilities:
/// - Sorts paths by depth (parent directories first).
/// - Removes child paths when a parent path already exists.
///   Example:
///     /Library/Application Support/MyApp
///     /Library/Application Support/MyApp/cache
///   Only the parent path is kept.
/// - Merges additional results from another scanner.
///   Example:
///     General scan + Sandbox container scan.
/// - Removes duplicate paths after merging.
/// - Produces a clean final result.
///
/// Typical usage:
/// - Associate scan results
/// - BTM scan results
/// - Receipt scan results
/// - Any scanner that may produce overlapping paths
///
/// Design:
/// Parent directories are preferred over child paths.
///
/// For example:
///
/// ```text
/// /Library/Application Support/MyApp
/// /Library/Application Support/MyApp/cache
/// ```
///
/// Keeping both entries would create redundant cleanup targets.
/// Retaining only the parent path allows removal operations to
/// act on the highest meaningful filesystem boundary.
///
/// This behavior also prevents duplicate reporting when
/// multiple scanners discover files within the same directory.
///
/// Note:
/// Parent-path filtering occurs before duplicate removal.
pub fn construct_scanner_result<T, FPath>(
    mut results: Vec<T>,
    extra: Option<Vec<T>>,
    get_path: FPath,
) -> Vec<T>
where
    FPath: Fn(&T) -> &Path,
{
    results.sort_by_key(|item| get_path(item).components().count());

    let mut filtered = Vec::new();

    'parent_filter: for item in results {
        for existing in &filtered {
            if get_path(&item).starts_with(get_path(existing)) {
                continue 'parent_filter;
            }
        }
        filtered.push(item);
    }

    if let Some(extra) = extra {
        filtered.extend(extra);
    }

    let mut seen = HashSet::new();
    filtered.retain(|item| seen.insert(get_path(item).to_path_buf()));

    filtered
}

/// Generic filesystem scanner.
///
/// Doc:
/// This is the primary scanner for normal filesystem traversal.
///
/// Capabilities:
/// - Parallel directory traversal using Rayon.
/// - Recursive scanning using WalkDir.
/// - Configurable maximum depth.
/// - Custom match logic through a closure.
/// - Custom result construction through a closure.
/// - Progress callback support.
/// - Can return any output type.
///
/// Scan flow:
///   Locations
///       ↓
///   WalkDir
///       ↓
///   Match Rules
///       ↓
///   Build Result
///
/// Typical usage:
/// - Application support file scan
/// - LaunchAgent scan
/// - LaunchDaemon scan
/// - PrivilegedHelperTool scan
/// - Receipt scan
/// - Cache scan
/// - Log scan
///
/// Design:
///
/// This scanner is optimized for conventional filesystem
/// layouts where application ownership can be determined
/// directly from filenames or directory names.
///
/// Traversal is performed in parallel using Rayon to improve
/// performance across large directory trees.
///
/// Matching and result construction are delegated to caller
/// supplied closures so the traversal engine remains reusable
/// across different scanner types.
///
/// Note:
/// Progress callbacks are invoked periodically rather than
/// for every filesystem entry to reduce synchronization and
/// callback overhead during large scans.
pub fn scan_general<T, FProgress, FMatch, FBuild>(
    locations: &[PathBuf],
    max_depth: usize,
    in_progress: FProgress,
    is_match: FMatch,
    build: FBuild,
) -> Vec<T>
where
    T: Send,
    FProgress: Fn(usize, &Path) + Send + Sync,
    FMatch: Fn(&Path) -> bool + Send + Sync,
    FBuild: Fn(PathBuf) -> T + Send + Sync,
{
    let counter = Arc::new(AtomicUsize::new(0));
    let progress = Arc::new(in_progress);

    locations
        .par_iter()
        .filter(|base| base.exists())
        .flat_map_iter(|base| {
            WalkDir::new(base)
                .max_depth(max_depth)
                .into_iter()
                .filter_map(|e| e.ok())
                .flat_map(|entry| {
                    let path_buf = entry.path().to_path_buf();

                    let n = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if n.is_multiple_of(256) {
                        progress(n, &path_buf);
                    }

                    if is_match(&path_buf) {
                        Some(build(path_buf))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Scans sandbox container directories.
///
/// Doc:
/// Searches sandbox containers for files associated with an
/// application.
///
/// Design:
/// Sandbox containers require specialized scanning because
/// application ownership is often represented by files stored
/// inside the container rather than by the container directory
/// name itself.
///
/// Instead of recursively traversing the entire container,
/// the scanner inspects a predefined set of known locations
/// inside each container.
///
/// This significantly reduces filesystem traversal while
/// still providing reliable application identification.
///
/// Note:
/// Container scanning is intentionally separate from
/// `scan_general()` because its discovery strategy differs
/// substantially from normal filesystem traversal.
pub fn scan_container<T, FMatch, FBuild>(
    locations: &[PathBuf],
    patterns: &[PathBuf],
    is_match: FMatch,
    build: FBuild,
) -> Vec<T>
where
    T: Send,
    FMatch: Fn(&Path) -> bool + Send + Sync,
    FBuild: Fn(&Path, &Path) -> T + Send + Sync,
{
    locations
        .par_iter()
        .filter(|base| base.exists())
        .flat_map_iter(|base| {
            WalkDir::new(base)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|entry| entry.depth() == 1 && entry.file_type().is_dir())
                .filter_map(|entry| {
                    let container_dir = entry.path().to_path_buf();

                    patterns.par_iter().find_map_any(|pattern| {
                        let pattern_dir = container_dir.join(pattern);

                        if !pattern_dir.is_dir() {
                            return None;
                        }

                        std::fs::read_dir(&pattern_dir)
                            .ok()?
                            .filter_map(|e| e.ok())
                            .find_map(|file| {
                                let file_path = file.path();

                                if is_match(&file_path) {
                                    Some(build(&container_dir, &file_path))
                                } else {
                                    None
                                }
                            })
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
