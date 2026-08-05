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

//! Icon cache and image storage.
//!
//! Doc:
//! Provides a lightweight cache for storing platform-generated
//! file, folder, and application icons.
//!
//! Icons are resolved using operating-system APIs and stored
//! as raw RGBA image buffers for later reuse by UI components.
//!
//! The cache is keyed by icon identity rather than individual
//! filesystem entries.
//!
//! Common cache categories include:
//!
//! - Application bundle icons.
//! - System folder icons.
//! - Generic file icons.
//!
//! Stored icon data contains:
//!
//! - Image width.
//! - Image height.
//! - RGBA pixel bytes.
//!
//! Consumers may access icon data as:
//!
//! - Borrowed RGBA slices.
//! - Owned RGBA buffers.
//! - Generated RGB buffers.
//!
//! Design:
//! Icon retrieval can be significantly more expensive than
//! rendering previously cached image data.
//!
//! This module centralizes icon acquisition and storage so
//! multiple UI views can reuse icon resources without
//! repeatedly invoking platform-specific icon APIs.
//!
//! Application bundles use their full path as a cache key
//! because each application may provide a unique icon.
//!
//! Generic files and folders share common cache entries,
//! reducing memory usage and duplicate icon generation.
//!
//! Note:
//! Icon generation depends on platform-specific system APIs
//! and may not be available on all operating systems.
//!
//! Cached image data is intended for presentation purposes
//! only and should not be treated as a persistent asset
//! store.
//!..

use std::collections::HashMap;
use std::path::Path;

use crate::syscom::get_default_file_icon;
use crate::syscom::get_default_folder_icon;
use crate::syscom::get_installed_app_icon_by_path;
use crate::syscom::ns_image_to_rgba_bytes;
/// Cached icon storage for UI consumers.
///
/// Doc:
/// Stores platform-generated icon images as raw RGBA buffers
/// keyed by file type or application path.
///
/// Supported icon categories:
///
/// - Application bundles (`*.app`).
/// - System folder icons.
/// - Generic file icons.
///
/// The cache stores:
///
/// - Width.
/// - Height.
/// - RGBA pixel data.
///
/// Consumers can retrieve icon data as:
///
/// - Borrowed RGBA slices.
/// - Owned RGBA buffers.
/// - Generated RGB buffers.
///
/// Note:
/// Icon generation may require platform-specific system APIs.
/// This type exists separately from `Cleaner` so that UI-related
/// functionality remains independent from application cleanup logic.
#[derive(Debug, Clone)]
pub struct IconCache {
    icon_cache: HashMap<String, (usize, usize, Vec<u8>)>,
}

impl IconCache {
    pub fn new(path: &Path, target_size: f64) -> Option<Self> {
        let path_str = path.to_str().unwrap_or("");

        // 1. Determine the appropriate cache key dynamically
        let cache_key = if path_str.ends_with(".app") {
            path_str.to_string()
        } else if path.is_dir() {
            "__system_folder__".to_string()
        } else {
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("__system_generic_file__")
                .to_string()
        };

        // Generate and load the owned icon data instantly
        let icon = Self::load_icon_for_key(&cache_key, target_size)?;

        // Initialize the HashMap and insert the resolved icon
        let mut map = HashMap::new();
        map.insert(cache_key, icon);

        // Wrap the map in Self and return it
        Some(Self { icon_cache: map })
    }

    pub fn icon_cache_owned(self) -> HashMap<String, (usize, usize, Vec<u8>)> {
        self.icon_cache
    }

    // Get width for a specific file path icon
    pub fn width(&self, path: &Path) -> Option<usize> {
        let key = Self::get_cache_key(path);
        self.icon_cache.get(&key).map(|(w, _, _)| *w)
    }

    // Get height for a specific file path icon
    pub fn height(&self, path: &Path) -> Option<usize> {
        let key = Self::get_cache_key(path);
        self.icon_cache.get(&key).map(|(_, h, _)| *h)
    }

    // Get an immutable reference to the raw RGBA slice
    pub fn rgba_bytes(&self, path: &Path) -> Option<&[u8]> {
        let key = Self::get_cache_key(path);
        self.icon_cache
            .get(&key)
            .map(|(_, _, bytes)| bytes.as_slice())
    }

    // Consume the cache and extract a specific icon's raw vector allocation
    pub fn into_rgba_bytes(mut self, path: &Path) -> Option<Vec<u8>> {
        let key = Self::get_cache_key(path);
        self.icon_cache.remove(&key).map(|(_, _, bytes)| bytes)
    }

    // Build an RGB vector on the fly from the stored RGBA tuple data
    pub fn rgb_bytes(&self, path: &Path) -> Option<Vec<u8>> {
        let key = Self::get_cache_key(path);
        let (width, height, rgba_bytes) = self.icon_cache.get(&key)?;

        let mut rgb = Vec::with_capacity(width * height * 3);

        // Chunk through data 4 bytes at a time (R, G, B, A)
        for chunk in rgba_bytes.chunks_exact(4) {
            rgb.push(chunk[0]); // R
            rgb.push(chunk[1]); // G
            rgb.push(chunk[2]); // B
            // chunk[3] (Alpha) is intentionally skipped
        }

        Some(rgb)
    }

    pub fn get_cache_key(path: &Path) -> String {
        let path_str = path.to_str().unwrap_or("");

        if path_str.ends_with(".app") {
            path_str.to_string()
        } else if path.is_dir() {
            "__system_folder__".to_string()
        } else {
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("__system_generic_file__")
                .to_string()
        }
    }

    fn load_icon_for_key(key: &str, target_size: f64) -> Option<(usize, usize, Vec<u8>)> {
        let ns_image = if key.ends_with(".app") {
            get_installed_app_icon_by_path(key)
        } else if key == "__system_folder__" {
            get_default_folder_icon()
        } else {
            get_default_file_icon()
        };

        let (width, height, bytes) = ns_image_to_rgba_bytes(&ns_image, target_size)?;
        Some((width, height, bytes))
    }
}
