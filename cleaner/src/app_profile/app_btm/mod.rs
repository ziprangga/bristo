mod btm_data;
pub use btm_data::BtmData;

use crate::app_profile::app_metadata::AppMetadata;
use crate::locations_scan::BtmLocations;
use crate::rules::MatchRules;

use crate::scanner::construct_scanner_result;
use crate::scanner::scan_general;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct AppBtmFiles {
    btm_files: Vec<BtmData>,
}

impl AppBtmFiles {
    /// Contruct BtmFiles
    pub fn new(btm_files: &[BtmData]) -> Self {
        Self {
            btm_files: btm_files.to_vec(),
        }
    }

    //// reference of btm files
    pub fn as_btm_files(&self) -> &[BtmData] {
        &self.btm_files
    }

    /// Update btm files with given list
    pub fn set_btm_files(&mut self, btm_data: Vec<BtmData>) {
        self.btm_files = btm_data;
    }

    pub fn scan_btm_files<F>(
        &mut self,
        app_metadata: &AppMetadata,
        locations: &BtmLocations,
        in_progress: F,
    ) where
        F: Fn(usize, &Path) + Send + Sync,
    {
        // let results = self.find_and_add_btm_files(app_metadata, locations, in_progress);

        // // remove child path if the parent in the list
        // // so it not mess with the list when move to trash
        // let mut sorted = results;
        // sorted.sort_by_key(|file| file.as_path().components().count());

        // let mut filtered: Vec<BtmData> = Vec::new();

        // 'parent_filter: for file in sorted {
        //     for existing_file in &filtered {
        //         if file.as_path().starts_with(existing_file.as_path()) {
        //             continue 'parent_filter;
        //         }
        //     }
        //     filtered.push(file);
        // }

        // // Deduplicate once after merge
        // let mut seen_unique = HashSet::new();
        // filtered.retain(|file| seen_unique.insert(file.as_path().to_path_buf()));

        // // Build the indexed list
        // self.set_btm_files(filtered);
        //
        // the new logic was use scanner module that produce template logic for scan
        // so not needed to write same logic everywhere
        //
        let results: Vec<BtmData> = scan_general(
            &locations.all_paths(),
            2,
            |n, path| in_progress(n, path),
            |path| {
                MatchRules::new()
                    .equal(app_metadata.as_info().as_name())
                    .equal(app_metadata.as_info().as_bundle_executable_name())
                    .contain(app_metadata.as_info().as_organization())
                    .contain(app_metadata.as_info().as_bundle_id())
                    .check(path)
            },
            |path_buf: PathBuf| {
                vec![BtmData::new(
                    path_buf.clone(),
                    path_buf.file_name().unwrap().to_string_lossy().to_string(),
                )]
            },
        );

        let filtered = construct_scanner_result(results, None, |item: &BtmData| item.as_path());

        self.set_btm_files(filtered);
    }

    // fn find_and_add_btm_files<F>(
    //     &self,
    //     app_metadata: &AppMetadata,
    //     locations: &BtmLocations,
    //     in_progress: F,
    // ) -> Vec<BtmData>
    // where
    //     F: Fn(usize, &Path) + Send + Sync,
    // {
    //     let counter = Arc::new(AtomicUsize::new(0));
    //     let progress = Arc::new(in_progress);

    //     // Parallel
    //     let results: Vec<BtmData> = locations
    //         .all_paths()
    //         .par_iter()
    //         .filter(|base| base.exists())
    //         .flat_map_iter(|base| {
    //             WalkDir::new(base)
    //                 .max_depth(2)
    //                 .into_iter()
    //                 .filter_map(|e| e.ok())
    //                 .flat_map(|entry| {
    //                     let path_buf = entry.path().to_path_buf();
    //                     let mut matches = Vec::new();
    //                     let rules = MatchRules::new()
    //                         .equal(app_metadata.as_info().as_name())
    //                         .equal(app_metadata.as_info().as_bundle_executable_name())
    //                         .contain(app_metadata.as_info().as_organization())
    //                         .contain(app_metadata.as_info().as_bundle_id())
    //                         .check(&path_buf);

    //                     if rules {
    //                         matches.push(BtmData::new(
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
}
