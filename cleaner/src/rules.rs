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
//! Matching rules used by scanner and discovery components.
//!
//! This module provides lightweight matching utilities for
//! determining whether a filesystem entry or text value may
//! belong to an application.
//!
//! Supported matching strategies include:
//!
//! - Exact matching.
//! - Partial matching.
//!
//! Supported targets include:
//!
//! - Filesystem paths.
//! - Filenames.
//! - Process names.
//! - Command lines.
//! - Application identifiers.
//!
//! Design:
//! Different scanners operate on different kinds of data.
//!
//! Filesystem scanners typically evaluate path names,
//! while process scanners evaluate process names and
//! command lines.
//!
//! Rather than implementing separate matching systems,
//! matching behavior is centralized in `MatchRules` and
//! applied consistently across both path-based and
//! string-based discovery.
//!
//! Path matching is intentionally performed against the
//! final path component (file or directory name) rather
//! than the complete path.
//!
//! This keeps comparisons predictable and avoids false
//! matches caused by unrelated parent directories.
//!
//! Matching behavior is implemented through composable
//! rules collected by `MatchRules`.
//!
//! Note:
//! Matching is case-insensitive and Unicode-normalized
//! to improve compatibility with macOS filesystem
//! behavior and process metadata comparisons.
//!..

use std::path::Path;
use unicode_normalization::UnicodeNormalization;

enum Rules {
    Equal,
    Contain,
}

impl Rules {
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

    /// Evaluates a path against a matching rule.
    ///
    /// Doc:
    /// Dispatches path matching to the appropriate
    /// comparison strategy associated with the rule.
    ///
    /// Note:
    /// Matching is performed against the final path
    /// component rather than the complete path.
    fn match_path(&self, path: &Path, value: &str) -> bool {
        match self {
            Rules::Equal => self.path_equals_ignore_case(path, value),
            Rules::Contain => self.path_contains_ignore_case(path, value),
        }
    }

    /// Evaluates a string against a matching rule.
    ///
    /// Doc:
    /// Dispatches string matching to the appropriate
    /// comparison strategy associated with the rule.
    ///
    /// Design:
    /// This method is primarily used by process and
    /// metadata scanners where matching targets are
    /// plain text rather than filesystem paths.
    ///
    /// Note:
    /// Matching is case-insensitive and Unicode-normalized.
    fn match_string(&self, text: &str, value: &str) -> bool {
        match self {
            Rules::Equal => self.string_equals_ignore_case(text, value),
            Rules::Contain => self.string_contains_ignore_case(text, value),
        }
    }

    /// Performs case-insensitive substring matching on a path name.
    ///
    /// Doc:
    /// Returns true when the final component of the provided
    /// path contains the specified value.
    ///
    /// Design:
    /// Matching is performed against `file_name()` rather
    /// than the complete path.
    ///
    /// This avoids matches caused solely by parent directory
    /// names and keeps comparisons focused on the actual
    /// filesystem entry being evaluated.
    ///
    /// Note:
    /// Matching is Unicode-normalized and case-insensitive.
    /// Empty search values never match.
    fn path_contains_ignore_case(&self, path: &Path, needle: &str) -> bool {
        if needle.trim().is_empty() {
            return false;
        }

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.normalize_lowercase(name)
                .contains(&self.normalize_lowercase(needle))
        } else {
            false
        }
    }

    /// Performs case-insensitive exact matching on a path name.
    ///
    /// Doc:
    /// Returns true when the final component of the provided
    /// path exactly matches the specified value.
    ///
    /// Design:
    /// Matching is performed against `file_name()` rather
    /// than the complete path.
    ///
    /// This keeps matching behavior predictable and avoids
    /// false positives originating from unrelated parent
    /// directories.
    ///
    /// Note:
    /// Matching is Unicode-normalized and case-insensitive.
    /// Empty values never match.
    fn path_equals_ignore_case(&self, path: &Path, value: &str) -> bool {
        if value.trim().is_empty() {
            return false;
        }

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.normalize_lowercase(name) == self.normalize_lowercase(value)
        } else {
            false
        }
    }

    /// Performs case-insensitive substring matching on text.
    ///
    /// Doc:
    /// Returns true when the provided text contains the
    /// specified value.
    ///
    /// Design:
    /// This method is used for matching non-filesystem
    /// values such as process names, command lines,
    /// bundle identifiers, and other application metadata.
    ///
    /// Note:
    /// Matching is Unicode-normalized and case-insensitive.
    /// Empty search values never match.
    fn string_contains_ignore_case(&self, text: &str, needle: &str) -> bool {
        if needle.trim().is_empty() {
            return false;
        }

        self.normalize_lowercase(text)
            .contains(&self.normalize_lowercase(needle))
    }

    /// Performs case-insensitive exact matching on text.
    ///
    /// Doc:
    /// Returns true when the provided text exactly matches
    /// the specified value.
    ///
    /// Design:
    /// This method is used for matching non-filesystem
    /// values such as process names, command lines,
    /// bundle identifiers, and other application metadata.
    ///
    /// Note:
    /// Matching is Unicode-normalized and case-insensitive.
    /// Empty values never match.
    fn string_equals_ignore_case(&self, text: &str, value: &str) -> bool {
        if value.trim().is_empty() {
            return false;
        }

        self.normalize_lowercase(text) == self.normalize_lowercase(value)
    }
}

/// Collection of filename matching rules.
///
/// Doc:
/// Stores a set of matching rules used to evaluate whether a
/// filesystem entry or text value appears related to an
/// application.
///
/// Rules can be combined using a builder-style API:
///
/// - `equal()`
/// - `contain()`
///
/// The collection is evaluated through:
///
/// - `check_path()`
/// - `check_string()`
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

    /// Returns the number of registered matching rules.
    ///
    /// Doc:
    /// Returns the total count of rules currently stored in
    /// the matcher.
    ///
    /// Design:
    /// Rules are added through the builder-style API using
    /// methods such as:
    ///
    /// - `equal()`
    /// - `contain()`
    ///
    /// Empty values are ignored and therefore do not
    /// contribute to the returned count.
    ///
    /// Note:
    /// The returned value reflects only valid stored rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns whether no matching rules are registered.
    ///
    /// Doc:
    /// Indicates whether the matcher currently contains
    /// any valid rules.
    ///
    /// Design:
    /// This method provides a convenient way for callers
    /// to detect when rule construction produced no usable
    /// matching criteria.
    ///
    /// This can occur when all supplied values were empty
    /// or whitespace-only and were therefore discarded by
    /// the builder methods.
    ///
    /// Example:
    ///
    /// ```text
    /// MatchRules::new()
    ///     .equal("")
    ///     .contain("")
    /// ```
    ///
    /// Produces an empty matcher.
    ///
    /// Note:
    /// Equivalent to `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
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
        // self.rules.push((Rules::Contain, value));
        // self
        if !value.trim().is_empty() {
            self.rules.push((Rules::Contain, value));
        }

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
        // self.rules.push((Rules::Equal, value));
        // self
        if !value.trim().is_empty() {
            self.rules.push((Rules::Equal, value));
        }

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
    pub fn check_path(&self, path: &Path) -> bool {
        self.rules
            .iter()
            .any(|(rule, value)| rule.match_path(path, value))
    }

    /// Evaluates all registered rules against a string.
    ///
    /// Doc:
    /// Tests the provided text against every configured
    /// matching rule.
    ///
    /// Design:
    /// Rules are evaluated using logical OR semantics.
    ///
    /// The comparison is:
    ///
    /// - Case-insensitive.
    /// - Unicode-normalized.
    ///
    /// A string is considered a match when at least one
    /// registered rule succeeds.
    ///
    /// This method is intended for matching non-filesystem
    /// values such as:
    ///
    /// - Process names.
    /// - Command lines.
    /// - Bundle identifiers.
    /// - Application metadata.
    ///
    /// Note:
    /// Empty rules never produce a match.
    pub fn check_string(&self, text: &str) -> bool {
        self.rules
            .iter()
            .any(|(rule, value)| rule.match_string(text, value))
    }
}
