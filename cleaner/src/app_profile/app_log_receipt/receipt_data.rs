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

//! Package receipt entry.
//!
//! Doc:
//! Represents a single discovered receipt or BOM file.
//!
//! Each entry stores:
//!
//! - The filesystem path.
//! - A display name.
//!
//! Receipt entries are primarily used for reporting,
//! inspection, and BOM export functionality.
//!
//! Examples:
//!
//! ```text
//! com.apple.pkg.Safari.bom
//! com.vendor.application.bom
//! ```
//!
//! Note:
//! `ReceiptData` is a lightweight data container and performs
//! no filesystem or package management operations.
//!..

use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct ReceiptData {
    path: PathBuf,
    name: String,
}

impl ReceiptData {
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
