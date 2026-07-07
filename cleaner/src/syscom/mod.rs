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
//! macOS system integration utilities.
//!
//! This module provides platform-specific functionality used by
//! the scanning, cleanup, and UI layers.
//!
//! The module acts as a boundary between application logic and
//! operating system services.
//!
//! Responsibilities include:
//!
//! - Process management.
//! - Finder integration.
//! - Trash operations.
//! - BOM inspection.
//! - System path resolution.
//! - Native icon retrieval.
//! - Image conversion utilities.
//!
//! The implementation relies on a combination of:
//!
//! - Native macOS frameworks.
//! - Objective-C bindings.
//! - libc system calls.
//! - Standard system utilities.
//!
//! The module is intentionally isolated so higher-level
//! components can remain focused on application discovery and
//! cleanup workflows rather than platform-specific details.
//!
//! Submodules:
//!
//! - `sys_component` provides system and filesystem operations.
//! - `sys_bom_log` provides BOM inspection helpers.
//! - `sys_asset` provides icon and image utilities.
//!
//! Design:
//! This module intentionally contains most operating-system
//! dependencies used by the project.
//!
//! Centralizing platform-specific code reduces coupling and
//! makes it easier to adapt higher-level logic to future
//! platforms or alternative implementations.
//!
//! Note:
//! All APIs in this module are currently macOS-specific.
//!..

mod sys_bom_log;
mod sys_component;

pub use sys_bom_log::run_lsbom_command;
pub use sys_component::{
    DARWIN_USER_CACHE_DIR, DARWIN_USER_TEMP_DIR, kill_pids, show_in_finder, sysconf_path,
    trash_files_nsfilemanager,
};

// =============================

mod sys_asset;
pub use sys_asset::{
    get_default_file_icon, get_default_folder_icon, get_installed_app_icon_by_path,
    ns_image_to_rgba_bytes,
};
