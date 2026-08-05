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

//! Scanning and utility helpers.
//!
//! Doc:
//! Provides reusable utilities shared across application
//! scanning and discovery operations.
//!
//! The module exposes components for:
//!
//! - Building scan location collections.
//! - Defining filename and string matching rules.
//! - Performing generic filesystem scans.
//! - Performing sandbox container scans.
//! - Caching application icons.
//!
//! Design:
//! Common scanning functionality is centralized here to avoid
//! duplicating traversal, matching, and path construction logic
//! across individual scanners.
//!
//! The public API re-exports the primary utility types and
//! functions while keeping implementation details organized
//! into private submodules.
//!
//! Note:
//! The utilities in this module are intended to be generic and
//! reusable. They are not coupled to any specific application
//! scanner or cleanup workflow.
//!...

mod icon_cache;
mod locations_scan;
mod rules;
mod scanner;

pub use icon_cache::IconCache;
pub use locations_scan::{BtmLocations, ReceiptsLocations, SandboxLocations, ScanLocations};
pub use rules::MatchRules;
pub use scanner::{construct_and_deduplicate_paths, scan_container, scan_general};
