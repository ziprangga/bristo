use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct ReceiptData {
    path: PathBuf,
    name: String,
}

impl ReceiptData {
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
