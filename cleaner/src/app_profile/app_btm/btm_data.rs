use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct BtmData {
    path: PathBuf,
    name: String,
}

impl BtmData {
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
