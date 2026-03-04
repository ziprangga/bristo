use crate::app_data::app_info::AppInfo;
use crate::app_data::locations_scan::{LocationsScan, SandboxContainerLocation};
use crate::rules::MatchRules;

use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

#[derive(Debug, Default, Clone)]
pub struct AssociateFiles {
    pub associate_files: Vec<(PathBuf, String)>,
}

impl AssociateFiles {
    // Add and replace existing path from list of paths
    // convenient way where some of paths not moved when all path exist try to move
    // like move to trash
    pub fn replace(&mut self, files: Vec<(PathBuf, String)>) {
        self.associate_files = files;
    }
    // Scan all file associate from list of location
    // for huge directory and try using walkdir + rayon
    // use in_progress as emitter status to caller
    pub fn scan_associate_files<F>(
        &mut self,
        app_info: &AppInfo,
        locations: &LocationsScan,
        in_progress: F,
    ) where
        F: Fn(usize, &Path) + Send + Sync,
    {
        let unique_results = self.find_and_add_associate_files(app_info, locations, in_progress);

        // remove child path if the parent in the list
        // so it not mess with the list when move to trash
        let mut sorted = unique_results;
        sorted.sort_by_key(|(p, _)| p.components().count());

        let mut filtered: Vec<(PathBuf, String)> = Vec::new();

        'parent_filter: for (path, name) in sorted {
            for (existing_path, _) in &filtered {
                if path.starts_with(existing_path) {
                    continue 'parent_filter;
                }
            }
            filtered.push((path, name));
        }

        // from sandbox location
        let mut merged = filtered;

        let container_matches = self.find_container_dirs(app_info);
        merged.extend(container_matches);

        // Build the indexed list including the app itself
        self.set_all_associate_file(app_info, merged);
    }

    fn find_and_add_associate_files<F>(
        &self,
        app_info: &AppInfo,
        locations: &LocationsScan,
        in_progress: F,
    ) -> Vec<(PathBuf, String)>
    where
        F: Fn(usize, &Path) + Send + Sync,
    {
        let counter = Arc::new(AtomicUsize::new(0));
        let progress = Arc::new(in_progress);

        // Parallel
        let results: Vec<(PathBuf, String)> = locations
            .paths
            .par_iter()
            .filter(|base| base.exists())
            .map(|base| {
                WalkDir::new(base)
                    .max_depth(3)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|entry| entry.file_type().is_file() || entry.file_type().is_dir())
                    .flat_map(|entry| {
                        let path_buf = entry.path().to_path_buf();
                        let mut matches = Vec::new();
                        let rules = MatchRules::new()
                            .equal(&app_info.name)
                            .equal(&app_info.bundle_executable_name)
                            .equal(&app_info.organization)
                            .contain(&app_info.bundle_id)
                            .check(&path_buf);

                        if rules {
                            matches.push((
                                path_buf.clone(),
                                path_buf.file_name().unwrap().to_string_lossy().to_string(),
                            ));
                        }

                        // Batched atomic progress every 256 files
                        let n = counter.fetch_add(1, Ordering::Relaxed) + 1;
                        if n.is_multiple_of(256) {
                            progress(n, &path_buf);
                        }

                        matches.into_iter()
                    })
                    .collect::<Vec<_>>()
            })
            .reduce(Vec::new, |mut acc, v| {
                acc.extend(v);
                acc
            }); // Collect directly without per-base Vec

        // Deduplicate once at the end
        let mut seen = HashSet::new();

        let unique_results: Vec<(PathBuf, String)> = results
            .into_iter()
            .filter(|(p, _)| seen.insert(p.clone()))
            .collect();

        unique_results
    }

    // Special sandbox container scanner for app that using uuid folder name
    fn find_container_dirs(&self, app_info: &AppInfo) -> Vec<(PathBuf, String)> {
        let containers_dir = SandboxContainerLocation::new();
        let patterns = containers_dir.sandbox_pattern();

        let results = containers_dir
            .paths
            .par_iter()
            .filter(|base| base.exists())
            .flat_map_iter(|base| {
                WalkDir::new(base)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|entry| entry.depth() == 1 && entry.file_type().is_dir())
                    .filter_map(|entry| {
                        let path = entry.path().to_path_buf();

                        patterns.par_iter().find_map_any(|pattern| {
                            let pattern_dir = path.join(pattern);

                            if !pattern_dir.is_dir() {
                                return None;
                            }
                            std::fs::read_dir(&pattern_dir)
                                .ok()?
                                .filter_map(Result::ok)
                                .find_map(|entry| {
                                    let file_path = entry.path();
                                    let rules = MatchRules::new()
                                        .contain(&app_info.bundle_id)
                                        .check(&file_path);

                                    if file_path.is_file() && rules {
                                        Some((path.clone(), app_info.name.clone()))
                                    } else {
                                        None
                                    }
                                })
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        results
    }

    /// Update associate_files with given list and include app itself
    fn set_all_associate_file(&mut self, app_info: &AppInfo, files: Vec<(PathBuf, String)>) {
        // Start with enumerated files
        let mut path_asc: Vec<(PathBuf, String)> = files.into_iter().collect();

        // Append the app itself
        path_asc.push((app_info.path.clone(), app_info.name.clone()));

        self.associate_files = path_asc;
    }
}
