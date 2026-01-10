use std::path::Path;
use unicode_normalization::UnicodeNormalization;

/// Normalize & lowercase string case-insensitively for macOS APFS-safe comparison
fn normalize_lowercase(s: &str) -> String {
    s.nfd() // normalize to NFD (decomposed)
        .collect::<String>()
        .to_lowercase() // lowercase for case-insensitive comparison
}

/// Compare PathBuf or filenames using contains
pub fn path_contains_ignore_case(path: &Path, needle: &str) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        normalize_lowercase(name).contains(&normalize_lowercase(needle))
    } else {
        false
    }
}

/// Compare PathBuf or filenames using equals value
pub fn path_equals_ignore_case(path: &Path, value: &str) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        normalize_lowercase(name) == normalize_lowercase(value)
    } else {
        false
    }
}
