use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct AscData {
    path: PathBuf,
    name: String,
}

impl AscData {
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
