use rayon::prelude::*;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

/// Post-process scanner results.
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
/// - Returns a clean, deduplicated result set.
///
/// Typical usage:
/// - Associate scan results
/// - BTM scan results
/// - Receipt scan results
/// - Any scanner that may produce overlapping paths
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
/// This is the primary scanner for normal filesystem traversal.
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
    FBuild: Fn(PathBuf) -> Vec<T> + Send + Sync,
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
                        build(path_buf).into_iter()
                    } else {
                        Vec::new().into_iter()
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Generic filesystem scanner.
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
/// This is the primary scanner for normal filesystem traversal.
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
