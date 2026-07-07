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

//! Associated application file entry.
//!
//! Doc:
//! Represents a single filesystem entry discovered during
//! associated-file scanning.
//!
//! Each entry stores:
//!
//! - The filesystem path.
//! - A display name.
//!
//! The display name is intended for reporting and user interfaces,
//! while the path uniquely identifies the underlying resource.
//!
//! Examples:
//!
//! ```text
//! ~/Library/Application Support/MyApp
//! ~/Library/Preferences/com.example.myapp.plist
//! ~/Library/Caches/com.example.myapp
//! ```
//!
//! Note:
//! `AscData` is a lightweight data container and does not perform
//! any filesystem operations itself.
//!..

use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct AscData {
    path: PathBuf,
    name: String,
}

impl AscData {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self { path, name }
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn as_name(&self) -> &str {
        &self.name
    }
}
