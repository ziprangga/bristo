use std::path::Path;
use unicode_normalization::UnicodeNormalization;

pub enum Rules {
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

    /// Normalize & lowercase string case-insensitively for macOS APFS-safe comparison
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

pub struct MatchRules<'a> {
    rules: Vec<(Rules, &'a str)>,
}

impl<'a> MatchRules<'a> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn contain(mut self, value: &'a str) -> Self {
        self.rules.push((Rules::Contain, value));
        self
    }

    pub fn equal(mut self, value: &'a str) -> Self {
        self.rules.push((Rules::Equal, value));
        self
    }

    pub fn check(&self, path: &Path) -> bool {
        self.rules
            .iter()
            .any(|(rule, value)| rule.match_path(path, value))
    }
}
