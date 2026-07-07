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

//! Background Task Management file entry.
//!
//! Doc:
//! Represents a single filesystem entry discovered during
//! BTM scanning.
//!
//! Each entry stores:
//!
//! - The filesystem path.
//! - A display name.
//!
//! The display name is intended for user interfaces and
//! reporting, while the path uniquely identifies the
//! underlying filesystem resource.
//!
//! Examples:
//!
//! ```text
//! ~/Library/LaunchAgents/com.example.app.plist
//! /Library/LaunchDaemons/com.example.service.plist
//! ```
//!
//! Note:
//! `BtmData` is a lightweight data container and does not
//!..

use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct BtmData {
    path: PathBuf,
    name: String,
}

impl BtmData {
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
