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
//! Known filesystem locations used during application discovery.
//!
//! This module provides curated collections of macOS directories
//! commonly associated with:
//!
//! - Application data.
//! - Installer receipts.
//! - Sandboxed applications.
//! - Background task management.
//!
//! The location groups are used by scanner components to reduce
//! search scope and improve performance.
//!
//! Design:
//! Rather than scanning the entire filesystem, the application
//! searches a known set of locations where macOS applications
//! typically store data and configuration.
//!
//! This approach significantly reduces scan time while still
//! covering the majority of files relevant to application
//! cleanup.
//!
//! Location categories are separated according to their purpose:
//!
//! - `ScanLocations` for general application data.
//! - `ReceiptsLocations` for installer receipts.
//! - `SandboxLocations` for application containers.
//! - `BtmLocations` for persistence-related components.
//!
//! The module intentionally centralizes all filesystem search
//! roots used by the application.
//!
//! Keeping scan locations in one place makes it easier to:
//!
//! - Audit search coverage.
//! - Add support for new macOS storage conventions.
//! - Debug missing scan results.
//! - Maintain consistent behavior across scanners.
//!
//! New scanners should prefer reusing existing location
//! providers rather than introducing ad-hoc search paths.
//!
//! Note:
//! All paths in this module are macOS-specific.
//!..

use std::env;
use std::path::PathBuf;
// =======
use crate::syscom::sysconf_path;
use crate::syscom::{DARWIN_USER_CACHE_DIR, DARWIN_USER_TEMP_DIR};

/// General application-related scan locations.
///
/// Doc:
/// Stores directories commonly used by applications for
/// configuration, caches, logs, extensions, helper tools,
/// and user data.
///
/// These locations represent the primary search scope used
/// when discovering associated files.
///
/// Design:
/// The goal is to balance coverage and performance.
///
/// Scanning the entire filesystem would be expensive and
/// would produce many unrelated matches.
///
/// Instead, only locations known to commonly contain
/// application-owned resources are included.
///
/// The collection contains both:
///
/// - User-specific locations.
/// - System-wide locations.
///
/// Dynamic cache and temporary directories are resolved using
/// system configuration APIs when available.
///
/// Note:
/// The location list is intentionally opinionated and may be
/// expanded as additional macOS storage conventions emerge.
#[derive(Debug, Default, Clone)]
pub struct ScanLocations {
    paths: Vec<PathBuf>,
}

impl ScanLocations {
    /// Constructs the default application scan locations.
    ///
    /// Doc:
    /// Builds a collection of directories commonly used by
    /// macOS applications.
    ///
    /// Included locations cover:
    ///
    /// - Application Support.
    /// - Caches.
    /// - Logs.
    /// - Preferences.
    /// - Containers.
    /// - Launch agents.
    /// - Launch daemons.
    /// - Plugin directories.
    /// - Homebrew-style installation paths.
    ///
    /// Design:
    /// The selected locations were chosen based on common
    /// application installation and storage patterns observed
    /// across native, sandboxed, and third-party applications.
    ///
    /// The resulting list intentionally favors practical
    /// application cleanup coverage over exhaustive filesystem
    /// traversal.
    ///
    /// Note:
    /// Nonexistent paths are not filtered here and may be handled
    /// by scanner implementations.
    pub fn new() -> Self {
        let mut paths = Vec::new();

        // Get home directory
        let home = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users/Unknown"));

        // User library locations
        paths.push(home.join("Library"));
        paths.push(home.join("Library/Application Scripts"));
        paths.push(home.join("Library/Application Support"));
        paths.push(home.join("Library/Application Support/CrashReporter"));
        paths.push(home.join("Library/Containers"));
        paths.push(home.join("Library/Caches"));
        paths.push(home.join("Library/HTTPStorages"));
        paths.push(home.join("Library/Group Containers"));
        paths.push(home.join("Library/Internet Plug-Ins"));
        paths.push(home.join("Library/LaunchAgents"));
        paths.push(home.join("Library/Logs"));
        paths.push(home.join("Library/Preferences"));
        paths.push(home.join("Library/Preferences/ByHost"));
        paths.push(home.join("Library/Saved Application State"));
        paths.push(home.join("Library/WebKit"));

        // System-wide locations
        paths.push(PathBuf::from("/Library"));
        paths.push(PathBuf::from("/Library/Application Support"));
        paths.push(PathBuf::from("/Library/Application Support/CrashReporter"));
        paths.push(PathBuf::from("/Library/Caches"));
        paths.push(PathBuf::from("/Library/Extensions"));
        paths.push(PathBuf::from("/Library/Internet Plug-Ins"));
        paths.push(PathBuf::from("/Library/LaunchAgents"));
        paths.push(PathBuf::from("/Library/LaunchDaemons"));
        paths.push(PathBuf::from("/Library/Logs"));
        paths.push(PathBuf::from("/Library/Preferences"));
        paths.push(PathBuf::from("/Library/PrivilegedHelperTools"));
        paths.push(PathBuf::from("/private/var/db/receipts"));
        paths.push(PathBuf::from("/usr/local/bin"));
        paths.push(PathBuf::from("/usr/local/etc"));
        paths.push(PathBuf::from("/usr/local/opt"));
        paths.push(PathBuf::from("/usr/local/sbin"));
        paths.push(PathBuf::from("/usr/local/share"));
        paths.push(PathBuf::from("/usr/local/var"));

        // Optional: macOS cache/temp directories
        if let Some(p) = sysconf_path(DARWIN_USER_CACHE_DIR) {
            paths.push(p);
        }
        if let Some(p) = sysconf_path(DARWIN_USER_TEMP_DIR) {
            paths.push(p);
        }

        Self { paths }
    }

    pub fn as_paths(&self) -> &Vec<PathBuf> {
        &self.paths
    }
}

/// Installer receipt locations.
///
/// Doc:
/// Stores directories known to contain macOS installer
/// receipts and BOM files.
///
/// Design:
/// Receipt scanning is isolated from general application
/// scanning because receipt files have a distinct purpose:
/// recording files installed by packages.
///
/// Restricting receipt searches to known receipt locations
/// avoids unnecessary filesystem traversal.
///
/// Note:
/// Receipt locations are primarily consumed by
/// `AppLogReceipt`.
#[derive(Debug, Default, Clone)]
pub struct ReceiptsLocations {
    paths: Vec<PathBuf>,
}

impl ReceiptsLocations {
    /// Constructs the default receipt locations.
    ///
    /// Doc:
    /// Returns the standard macOS installer receipt directory.
    ///
    /// Design:
    /// Modern macOS systems store package receipts in a central
    /// location, making a broader search unnecessary.
    ///
    /// Note:
    /// Additional receipt locations can be added in the future if
    /// macOS storage conventions change.
    pub fn new() -> Self {
        Self {
            paths: vec![PathBuf::from("/private/var/db/receipts")],
        }
    }

    pub fn as_paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

/// Sandboxed application container locations.
///
/// Doc:
/// Stores directories used by macOS application sandboxing.
///
/// These locations are used to discover containerized
/// application data that may not appear in general scans.
///
/// Design:
/// Sandbox containers have a well-defined structure that
/// differs from traditional application storage locations.
///
/// Keeping container scanning separate allows specialized
/// matching and traversal strategies to be applied.
///
/// Note:
/// Container discovery is primarily used by
/// `AppAscFiles`.
#[derive(Debug, Default, Clone)]
pub struct SandboxLocations {
    paths: Vec<PathBuf>,
}

impl SandboxLocations {
    pub fn new() -> Self {
        let mut paths = Vec::new();
        let home = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users/Unknown"));

        paths.push(home.join("Library/Containers"));

        Self { paths }
    }

    pub fn as_paths(&self) -> &Vec<PathBuf> {
        &self.paths
    }

    /// Returns expected sandbox directory patterns.
    ///
    /// Doc:
    /// Provides relative paths used to validate whether a
    /// container directory appears to belong to a real
    /// application sandbox.
    ///
    /// Design:
    /// Container directories may exist without containing
    /// meaningful application data.
    ///
    /// The returned patterns identify well-known locations that
    /// should exist within a functional sandbox.
    ///
    /// This reduces false positives during container discovery.
    ///
    /// Note:
    /// Patterns are evaluated relative to each container root.
    pub fn sandbox_pattern(&self) -> Vec<PathBuf> {
        let mut pattern = Vec::new();
        pattern.push(PathBuf::from("Data").join("Library").join("Preferences"));
        pattern
    }
}

/// Background Task Management locations.
///
/// Doc:
/// Stores directories commonly associated with application
/// persistence mechanisms and background execution.
///
/// The locations are organized into categories:
///
/// - Legacy launch services.
/// - Preferences.
/// - Privileged helper tools.
///
/// Design:
/// Modern macOS applications may install components outside
/// their primary application bundle to support:
///
/// - Automatic startup.
/// - Background services.
/// - Privileged operations.
/// - System integration.
///
/// These files require different cleanup considerations than
/// ordinary application data, so they are grouped separately.
///
/// Note:
/// This structure is primarily consumed by
/// `AppBtmFiles`.
#[derive(Debug, Default, Clone)]
pub struct BtmLocations {
    legacy_dir: Vec<PathBuf>,
    preference_dir: Vec<PathBuf>,
    privileged_dir: Vec<PathBuf>,
}

impl BtmLocations {
    pub fn new() -> Self {
        let home = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users/Unknown"));

        let mut legacy_dir = Vec::new();
        legacy_dir.push(PathBuf::from("/Library/LaunchAgents"));
        legacy_dir.push(PathBuf::from("/Library/LaunchDaemons"));
        legacy_dir.push(home.join("Library/LaunchAgents"));

        let mut preference_dir = Vec::new();
        preference_dir.push(PathBuf::from("/Library/PreferencePanes"));
        preference_dir.push(PathBuf::from("/Library/Preferences"));
        preference_dir.push(home.join("Library/PreferencePanes"));
        preference_dir.push(home.join("Library/Preferences"));

        let mut privileged_dir = Vec::new();
        privileged_dir.push(PathBuf::from("/Library/PrivilegedHelperTools"));

        Self {
            legacy_dir,
            preference_dir,
            privileged_dir,
        }
    }

    /// Returns all BTM search locations.
    ///
    /// Doc:
    /// Combines all persistence-related location groups into a
    /// single collection suitable for scanning.
    ///
    /// Design:
    /// The internal categorization is preserved for readability
    /// and future expansion, while scanners typically require a
    /// flat list of directories.
    ///
    /// This method provides that flattened view without exposing
    /// implementation details.
    pub fn all_paths(&self) -> Vec<PathBuf> {
        self.legacy_dir
            .iter()
            .chain(self.privileged_dir.iter())
            .chain(self.preference_dir.iter())
            .cloned()
            .collect()
    }
}
