use std::env;
use std::path::PathBuf;
// =======
use crate::syscom::sysconf_path;
use crate::syscom::{DARWIN_USER_CACHE_DIR, DARWIN_USER_TEMP_DIR};

#[derive(Debug, Clone)]
pub struct LocationsScan {
    paths: Vec<PathBuf>,
}

impl LocationsScan {
    /// Build a default list of app-related locations
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

    /// Return only receipts directories
    pub fn receipts_dirs(&self) -> Vec<PathBuf> {
        self.paths
            .iter()
            .filter(|p| p.ends_with("receipts") || p == &&PathBuf::from("/private/var/db/receipts"))
            .cloned()
            .collect()
    }
}

impl Default for LocationsScan {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SandboxContainerLocation {
    paths: Vec<PathBuf>,
}

impl SandboxContainerLocation {
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

    // Pattern to check the container directory
    pub fn sandbox_pattern(&self) -> Vec<PathBuf> {
        let mut pattern = Vec::new();
        pattern.push(PathBuf::from("Data").join("Library").join("Preferences"));
        pattern
    }
}

impl Default for SandboxContainerLocation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BtmLocations {
    legacy_dir: Vec<PathBuf>,
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

        let mut privileged_dir = Vec::new();
        privileged_dir.push(PathBuf::from("/Library/PrivilegedHelperTools"));

        Self {
            legacy_dir,
            privileged_dir,
        }
    }

    // Returns a clone of legacy directories
    pub fn as_legacy_dir(&self) -> &Vec<PathBuf> {
        &self.legacy_dir
    }

    // Returns a clone of privileged directories
    pub fn as_privileged_dir(&self) -> &Vec<PathBuf> {
        &self.privileged_dir
    }

    pub fn all_paths(&self) -> Vec<PathBuf> {
        self.legacy_dir
            .iter()
            .chain(self.privileged_dir.iter())
            .cloned()
            .collect()
    }
}

impl Default for BtmLocations {
    fn default() -> Self {
        Self::new()
    }
}
