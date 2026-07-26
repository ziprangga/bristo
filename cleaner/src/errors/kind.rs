#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Failed,
    Skipped,
}

impl Kind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
        }
    }
}
