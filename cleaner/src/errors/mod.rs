mod kind;
pub use kind::Kind;

use std::borrow::Cow;

pub type Result<T> = std::result::Result<T, ErrorKind>;

#[derive(Debug, Clone)]
pub struct ErrorKind {
    kind: Kind,
    summary: Option<Cow<'static, str>>,
    reason: Option<Cow<'static, str>>,
}

impl ErrorKind {
    pub fn failed() -> Self {
        Self {
            kind: Kind::Failed,
            summary: None,
            reason: None,
        }
    }

    pub fn skipped() -> Self {
        Self {
            kind: Kind::Skipped,
            summary: None,
            reason: None,
        }
    }

    pub fn with_summary(mut self, summary: impl Into<Cow<'static, str>>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn with_reason(mut self, reason: impl Into<Cow<'static, str>>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn is_empty(&self) -> bool {
        self.summary.is_none() && self.reason.is_none()
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.kind {
            Kind::Failed => "[FAILED]",
            Kind::Skipped => "[SKIPPED]",
        };

        f.write_str(prefix)?;

        if let Some(summary) = self.summary() {
            write!(f, " {} :", summary)?;

            if let Some(reason) = self.reason() {
                f.write_str(reason)?;
            }
        } else if let Some(reason) = self.reason() {
            write!(f, " {}", reason)?;
        }

        Ok(())
    }
}

// Implement standard Error trait so it plays nice with standard tools and tasks
impl std::error::Error for ErrorKind {}
