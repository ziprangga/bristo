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
//! Path matching rules used by scanner components.
//!
//! This module provides lightweight matching utilities for
//! determining whether a filesystem entry may belong to an
//! application.
//!
//! Supported matching strategies include:
//!
//! - Exact filename matching.
//! - Partial filename matching.
//!
//! Design:
//! Scanner components operate on large collections of files and
//! directories.
//!
//! Matching is intentionally performed against the final path
//! component (file or directory name) rather than the entire
//! path.
//!
//! This keeps comparisons predictable and avoids false matches
//! caused by unrelated parent directories.
//!
//! Matching behavior is implemented through composable rules
//! collected by `MatchRules`.
//!
//! Note:
//! Matching is case-insensitive and Unicode-normalized to
//! improve compatibility with macOS filesystem behavior.
//!..

use std::path::Path;
use unicode_normalization::UnicodeNormalization;

enum Rules {
    Equal,
    Contain,
}

impl Rules {
    fn match_path(&self, path: &Path, value: &str) -> bool {
        match self {
            Rules::Equal => self.path_equals_ignore_case(path, value),
            Rules::Contain => self.path_contains_ignore_case(path, value),
        }
    }

    /// Normalizes text for filesystem comparison.
    ///
    /// Doc:
    /// Converts a string into a normalized lowercase form suitable
    /// for case-insensitive filename matching.
    ///
    /// Design:
    /// macOS filesystems commonly store and compare filenames using
    /// Unicode normalization rules.
    ///
    /// Performing normalization before comparison improves
    /// matching reliability for filenames containing accented or
    /// non-ASCII characters.
    ///
    /// Example:
    ///
    /// ```text
    /// Café
    /// Café
    /// ```
    ///
    /// These visually identical strings may have different Unicode
    /// representations but normalize to the same form.
    ///
    /// Note:
    /// NFD normalization is used to align with common macOS
    /// filesystem behavior.
    fn normalize_lowercase(&self, s: &str) -> String {
        s.nfd() // normalize to NFD (decomposed)
            .collect::<String>()
            .to_lowercase() // lowercase for case-insensitive comparison
    }

    /// Compare PathBuf or filenames using contains
    fn path_contains_ignore_case(&self, path: &Path, needle: &str) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.normalize_lowercase(name)
                .contains(&self.normalize_lowercase(needle))
        } else {
            false
        }
    }

    /// Compare PathBuf or filenames using equals value
    fn path_equals_ignore_case(&self, path: &Path, value: &str) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.normalize_lowercase(name) == self.normalize_lowercase(value)
        } else {
            false
        }
    }
}

/// Collection of filename matching rules.
///
/// Doc:
/// Stores a set of matching rules used to evaluate whether a
/// filesystem entry appears related to an application.
///
/// Rules can be combined using a builder-style API:
///
/// - `equal()`
/// - `contain()`
///
/// The collection is evaluated through `check()`.
///
/// Design:
/// Matching requirements vary across scanners.
///
/// Some applications are identified by:
///
/// - Application names.
/// - Executable names.
/// - Bundle identifiers.
/// - Organization identifiers.
///
/// Rather than hardcoding matching logic into each scanner,
/// scanners compose the rules they require and delegate the
/// evaluation to this type.
///
/// Note:
/// Rules are evaluated using logical OR semantics.
pub struct MatchRules<'a> {
    rules: Vec<(Rules, &'a str)>,
}

impl<'a> MatchRules<'a> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a substring matching rule.
    ///
    /// Doc:
    /// Registers a rule that succeeds when a filename contains
    /// the provided value.
    ///
    /// Note:
    /// Matching is case-insensitive and Unicode-normalized.
    pub fn contain(mut self, value: &'a str) -> Self {
        self.rules.push((Rules::Contain, value));
        self
    }

    /// Adds an exact matching rule.
    ///
    /// Doc:
    /// Registers a rule that succeeds when a filename exactly
    /// matches the provided value.
    ///
    /// Note:
    /// Matching is case-insensitive and Unicode-normalized.
    pub fn equal(mut self, value: &'a str) -> Self {
        self.rules.push((Rules::Equal, value));
        self
    }

    /// Evaluates all registered rules.
    ///
    /// Doc:
    /// Tests the provided path against every configured rule.
    ///
    /// Design:
    /// Rules are evaluated using logical OR semantics.
    ///
    /// A path is considered a match when at least one rule
    /// succeeds.
    ///
    /// This behavior intentionally favors discovery coverage
    /// during scanning. Application-owned files frequently
    /// reference only a single identifier, such as:
    ///
    /// - Application name.
    /// - Executable name.
    /// - Bundle identifier.
    /// - Organization identifier.
    ///
    /// Requiring all identifiers to appear simultaneously
    /// would significantly reduce discovery coverage and
    /// cause many legitimate files to be missed.
    ///
    /// Example:
    ///
    /// A file may match because its filename contains:
    ///
    /// - The application name.
    /// - The executable name.
    /// - The bundle identifier.
    ///
    /// Only one successful match is required.
    ///
    /// Note:
    /// Matching is performed against the final path component
    /// (`file_name`) rather than the complete path.
    pub fn check(&self, path: &Path) -> bool {
        self.rules
            .iter()
            .any(|(rule, value)| rule.match_path(path, value))
    }
}
