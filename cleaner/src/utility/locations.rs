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
use std::path::{Path, PathBuf};
// =======
use crate::syscom::sysconf_path;
use crate::syscom::{DARWIN_USER_CACHE_DIR, DARWIN_USER_TEMP_DIR};

/// Scan location description.
///
/// Doc:
/// Represents a filesystem location used during scanning.
///
/// A location consists of:
///
/// - A root directory used as the scan entry point.
/// - Zero or more relative patterns used to validate or
///   specialize traversal within that root.
///
/// Design:
/// Most scanners only require a root directory, while some
/// scanners—such as sandbox discovery—also require expected
/// directory patterns.
///
/// Combining both pieces of information into a single type
/// allows scanner configuration to remain self-contained and
/// extensible without introducing scanner-specific metadata.
///
/// Note:
/// Pattern paths are interpreted relative to the location
/// root and are not absolute filesystem paths.
#[derive(Debug, Clone)]
pub struct Location {
    root: PathBuf,
    patterns: Vec<PathBuf>,
}

impl Location {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            patterns: Vec::new(),
        }
    }

    pub fn with_patterns(
        root: impl Into<PathBuf>,
        patterns: impl IntoIterator<Item = PathBuf>,
    ) -> Self {
        Self {
            root: root.into(),
            patterns: patterns.into_iter().collect(),
        }
    }

    pub fn as_root(&self) -> &Path {
        &self.root
    }

    pub fn as_patterns(&self) -> &[PathBuf] {
        &self.patterns
    }
}

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
/// Each location consists of a scan root and may optionally
/// include additional traversal metadata.
#[derive(Debug, Default, Clone)]
pub struct GeneralLocations {
    locations: Vec<Location>,
}

impl GeneralLocations {
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
        let mut locations = Vec::new();

        // Get home directory
        let home = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users/Unknown"));

        let mut push_root = |root| locations.push(Location::new(root));

        // User library directory
        push_root(home.join("Library"));
        push_root(home.join("Library/Application Scripts"));
        push_root(home.join("Library/Application Support"));
        push_root(home.join("Library/Application Support/CrashReporter"));
        // push_root(home.join("Library/Containers"));
        push_root(home.join("Library/Caches"));
        push_root(home.join("Library/HTTPStorages"));
        // push_root(home.join("Library/Group Containers"));
        push_root(home.join("Library/Internet Plug-Ins"));
        // push_root(home.join("Library/LaunchAgents"));
        push_root(home.join("Library/Logs"));
        push_root(home.join("Library/Preferences"));
        push_root(home.join("Library/Preferences/ByHost"));
        push_root(home.join("Library/Saved Application State"));
        push_root(home.join("Library/WebKit"));

        // System-wide directory
        push_root(PathBuf::from("/Library"));
        push_root(PathBuf::from("/Library/Application Support"));
        push_root(PathBuf::from("/Library/Application Support/CrashReporter"));
        push_root(PathBuf::from("/Library/Caches"));
        push_root(PathBuf::from("/Library/Extensions"));
        push_root(PathBuf::from("/Library/Internet Plug-Ins"));
        // push_root(PathBuf::from("/Library/LaunchAgents"));
        // push_root(PathBuf::from("/Library/LaunchDaemons"));
        push_root(PathBuf::from("/Library/Logs"));
        push_root(PathBuf::from("/Library/Preferences"));
        // push_root(PathBuf::from("/Library/PrivilegedHelperTools"));
        // push_root(PathBuf::from("/private/var/db/receipts"));
        push_root(PathBuf::from("/usr/local/bin"));
        push_root(PathBuf::from("/usr/local/etc"));
        push_root(PathBuf::from("/usr/local/opt"));
        push_root(PathBuf::from("/usr/local/sbin"));
        push_root(PathBuf::from("/usr/local/share"));
        push_root(PathBuf::from("/usr/local/var"));

        // Optional: macOS cache/temp directories
        if let Some(p) = sysconf_path(DARWIN_USER_CACHE_DIR) {
            push_root(p);
        }
        if let Some(p) = sysconf_path(DARWIN_USER_TEMP_DIR) {
            push_root(p);
        }

        Self { locations }
    }

    pub fn as_locations(&self) -> &[Location] {
        &self.locations
    }

    pub fn location_roots(&self) -> Vec<PathBuf> {
        self.locations
            .iter()
            .map(|location| location.as_root().to_path_buf())
            .collect()
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
/// Receipt locations currently consist of scan roots only and
/// do not define additional traversal patterns.
#[derive(Debug, Default, Clone)]
pub struct ReceiptsLocations {
    locations: Vec<Location>,
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
            locations: vec![Location::new("/private/var/db/receipts")],
        }
    }

    pub fn as_locations(&self) -> &[Location] {
        &self.locations
    }

    pub fn location_roots(&self) -> Vec<PathBuf> {
        self.locations
            .iter()
            .map(|location| location.as_root().to_path_buf())
            .collect()
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
/// Sandbox-specific validation patterns are stored together
/// with each scan location, allowing callers to obtain both
/// the scan root and its expected container layout from a
/// single source.
///
/// Note:
/// Container discovery is primarily consumed by
/// `AppAscFiles`.
#[derive(Debug, Default, Clone)]
pub struct SandboxLocations {
    locations: Vec<Location>,
}

impl SandboxLocations {
    pub fn new() -> Self {
        let home = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users/Unknown"));

        let container = Location::with_patterns(
            home.join("Library/Containers"),
            [PathBuf::from("Data").join("Library").join("Preferences")],
        );

        let group_container = Location::with_patterns(
            home.join("Library/Group Containers"),
            [PathBuf::from("Library").join("Preferences")],
        );

        Self {
            locations: vec![container, group_container],
        }
    }

    pub fn as_locations(&self) -> &[Location] {
        &self.locations
    }

    /// Returns the expected relative directory patterns.
    ///
    /// Doc:
    /// Returns the validation patterns associated with the
    /// sandbox scan location.
    ///
    /// Design:
    /// These patterns identify directories expected to exist
    /// within a valid application container and help reduce
    /// false positives during discovery.
    ///
    /// Note:
    /// Returned paths are relative to the sandbox location root.
    pub fn as_pattern(&self) -> &[PathBuf] {
        self.locations
            .first()
            .map(|location| location.as_patterns())
            .unwrap_or(&[])
    }

    pub fn location_roots(&self) -> Vec<PathBuf> {
        self.locations
            .iter()
            .map(|location| location.as_root().to_path_buf())
            .collect()
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
pub struct BackgroundTaskLocations {
    legacy_dir: Vec<Location>,
    preference_dir: Vec<Location>,
    privileged_dir: Vec<Location>,
}

impl BackgroundTaskLocations {
    pub fn new() -> Self {
        let home = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users/Unknown"));

        let legacy_dir = vec![
            Location::new("/Library/LaunchAgents"),
            Location::new("/Library/LaunchDaemons"),
            Location::new(home.join("Library/LaunchAgents")),
        ];

        let preference_dir = vec![
            Location::new("/Library/PreferencePanes"),
            Location::new("/Library/Preferences"),
            Location::new(home.join("Library/PreferencePanes")),
            Location::new(home.join("Library/Preferences")),
        ];

        let privileged_dir = vec![Location::new("/Library/PrivilegedHelperTools")];

        Self {
            legacy_dir,
            preference_dir,
            privileged_dir,
        }
    }

    /// Returns all configured background task scan locations.
    ///
    /// Doc:
    /// Combines every Background Task Management location into a
    /// single collection suitable for scanning.
    ///
    /// Design:
    /// Locations remain categorized internally for organization,
    /// while callers typically require a unified collection when
    /// performing filesystem traversal.
    ///
    /// Note:
    /// Each returned location currently contains only a scan root,
    /// but additional metadata may be associated with locations in
    /// the future.
    pub fn all_locations(&self) -> Vec<Location> {
        self.legacy_dir
            .iter()
            .chain(self.privileged_dir.iter())
            .chain(self.preference_dir.iter())
            .cloned()
            .collect()
    }

    /// Returns the root directory of every configured background
    /// task scan location.
    ///
    /// Doc:
    /// Extracts the filesystem root from each configured location
    /// and returns them as a flat collection.
    ///
    /// Design:
    /// Some scanners require only directory roots and do not use
    /// any additional location metadata.
    ///
    /// This method provides that simplified view without exposing
    /// the internal organization of the location groups.
    ///
    /// Note:
    /// Only the root directories are returned. Any metadata
    /// associated with each location is omitted.
    pub fn all_location_roots(&self) -> Vec<PathBuf> {
        self.legacy_dir
            .iter()
            .chain(self.privileged_dir.iter())
            .chain(self.preference_dir.iter())
            .map(|location| location.as_root().to_path_buf())
            .collect()
    }
}
