mod asc_data;
pub use asc_data::AscData;

use crate::app_profile::app_metadata::AppMetadata;
use crate::locations_scan::{SandboxLocations, ScanLocations};
use crate::rules::MatchRules;

use crate::scanner::construct_scanner_result;
use crate::scanner::scan_container;
use crate::scanner::scan_general;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct AppAscFiles {
    asc_files: Vec<AscData>,
}

impl AppAscFiles {
    /// Contruct AscFiles
    pub fn new(asc_files: &[AscData]) -> Self {
        Self {
            asc_files: asc_files.to_vec(),
        }
    }
    //// reference of associate files
    pub fn as_asc_files(&self) -> &[AscData] {
        &self.asc_files
    }

    /// Update associate_files with given list
    pub fn set_asc_files(&mut self, asc_data: Vec<AscData>) {
        self.asc_files = asc_data;
    }

    // Scan all file associate from list of location
    // for huge directory and try using walkdir + rayon
    // use in_progress as emitter status to caller
    pub fn scan_asc_files<F>(
        &mut self,
        app_metadata: &AppMetadata,
        locations: &ScanLocations,
        in_progress: F,
    ) where
        F: Fn(usize, &Path) + Send + Sync,
    {
        // let results = self.find_and_add_asc_files(app_metadata, locations, in_progress);

        // // remove child path if the parent in the list
        // // so it not mess with the list when move to trash
        // let mut sorted = results;
        // sorted.sort_by_key(|file| file.as_path().components().count());

        // let mut filtered: Vec<AscData> = Vec::new();

        // 'parent_filter: for file in sorted {
        //     for existing_file in &filtered {
        //         if file.as_path().starts_with(existing_file.as_path()) {
        //             continue 'parent_filter;
        //         }
        //     }
        //     filtered.push(file);
        // }

        // // from sandbox location
        // let mut merged = filtered;

        // let container_matches = self.find_container_dirs(app_metadata);
        // merged.extend(container_matches);

        // // Deduplicate once after merge
        // use std::collections::HashSet;
        // let mut seen_unique = HashSet::new();
        // merged.retain(|file| seen_unique.insert(file.as_path().to_path_buf()));

        // // Build the indexed list
        // self.set_asc_files(merged);
        //
        // the new logic was use scanner module that produce template logic for scan
        // so not needed to write same logic everywhere
        //
        let main_results: Vec<AscData> = scan_general(
            locations.as_paths(),
            3,
            |n, path| in_progress(n, path),
            |path| {
                MatchRules::new()
                    .equal(app_metadata.as_info().as_name())
                    .equal(app_metadata.as_info().as_bundle_executable_name())
                    .equal(app_metadata.as_info().as_organization())
                    .contain(app_metadata.as_info().as_bundle_id())
                    .check(path)
            },
            |path_buf: PathBuf| {
                vec![AscData::new(
                    path_buf.clone(),
                    path_buf.file_name().unwrap().to_string_lossy().to_string(),
                )]
            },
        );

        let containers_dir = SandboxLocations::new();
        let patterns = containers_dir.sandbox_pattern();

        let container_results: Vec<AscData> = scan_container(
            containers_dir.as_paths(),
            &patterns,
            |path| {
                MatchRules::new()
                    .contain(app_metadata.as_info().as_bundle_id())
                    .check(path)
            },
            |container_dir, _file_path| {
                let folder_name = container_dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let display_name = if folder_name == app_metadata.as_info().as_bundle_id() {
                    folder_name
                } else {
                    app_metadata.as_info().as_name().to_string()
                };

                AscData::new(container_dir.to_path_buf(), display_name)
            },
        );

        let results =
            construct_scanner_result(main_results, Some(container_results), |item: &AscData| {
                item.as_path()
            });

        self.set_asc_files(results);
    }

    // fn find_and_add_asc_files<F>(
    //     &self,
    //     app_metadata: &AppMetadata,
    //     locations: &ScanLocations,
    //     in_progress: F,
    // ) -> Vec<AscData>
    // where
    //     F: Fn(usize, &Path) + Send + Sync,
    // {
    //     let counter = Arc::new(AtomicUsize::new(0));
    //     let progress = Arc::new(in_progress);

    //     // Parallel
    //     let results: Vec<AscData> = locations
    //         .as_paths()
    //         .par_iter()
    //         .filter(|base| base.exists())
    //         .flat_map_iter(|base| {
    //             WalkDir::new(base)
    //                 .max_depth(3)
    //                 .into_iter()
    //                 .filter_map(|e| e.ok())
    //                 .flat_map(|entry| {
    //                     let path_buf = entry.path().to_path_buf();
    //                     let mut matches = Vec::new();
    //                     let rules = MatchRules::new()
    //                         .equal(app_metadata.as_info().as_name())
    //                         .equal(app_metadata.as_info().as_bundle_executable_name())
    //                         .equal(app_metadata.as_info().as_organization())
    //                         .contain(app_metadata.as_info().as_bundle_id())
    //                         .check(&path_buf);

    //                     if rules {
    //                         matches.push(AscData::new(
    //                             path_buf.clone(),
    //                             path_buf.file_name().unwrap().to_string_lossy().to_string(),
    //                         ));
    //                     }

    //                     // Batched atomic progress every 256 files
    //                     let n = counter.fetch_add(1, Ordering::Relaxed) + 1;
    //                     if n.is_multiple_of(256) {
    //                         progress(n, &path_buf);
    //                     }

    //                     matches.into_iter()
    //                 })
    //                 .collect::<Vec<_>>()
    //         })
    //         .collect();

    //     results
    // }

    // // Special sandbox container scanner for app that using uuid folder name
    // // or for app that install from app store
    // fn find_container_dirs(&self, app_metadata: &AppMetadata) -> Vec<AscData> {
    //     let containers_dir = SandboxLocations::new();
    //     let patterns = containers_dir.sandbox_pattern();

    //     let results = containers_dir
    //         .as_paths()
    //         .par_iter()
    //         .filter(|base| base.exists())
    //         .flat_map_iter(|base| {
    //             WalkDir::new(base)
    //                 .max_depth(1)
    //                 .into_iter()
    //                 .filter_map(|e| e.ok())
    //                 .filter(|entry| entry.depth() == 1 && entry.file_type().is_dir())
    //                 .filter_map(|entry| {
    //                     let path = entry.path().to_path_buf();

    //                     patterns.par_iter().find_map_any(|pattern| {
    //                         let pattern_dir = path.join(pattern);

    //                         if !pattern_dir.is_dir() {
    //                             return None;
    //                         }
    //                         std::fs::read_dir(&pattern_dir)
    //                             .ok()?
    //                             .filter_map(|e| e.ok())
    //                             .find_map(|entry| {
    //                                 let file_path = entry.path();
    //                                 let rules = MatchRules::new()
    //                                     .contain(app_metadata.as_info().as_bundle_id())
    //                                     .check(&file_path);

    //                                 if rules {
    //                                     let folder_name = path
    //                                         .file_name()
    //                                         .map(|n| n.to_string_lossy().to_string())
    //                                         .unwrap_or_default();

    //                                     let display_name = if folder_name
    //                                         == app_metadata.as_info().as_bundle_id()
    //                                     {
    //                                         folder_name
    //                                     } else {
    //                                         app_metadata.as_info().as_name().to_string()
    //                                     };

    //                                     Some(AscData::new(path.clone(), display_name))
    //                                 } else {
    //                                     None
    //                                 }
    //                             })
    //                     })
    //                 })
    //                 .collect::<Vec<_>>()
    //         })
    //         .collect();

    //     results
    // }
}
