use std::borrow::Cow;

pub type Result<T> = std::result::Result<T, ErrorKind>;

#[derive(Debug, Clone, Default)]
pub struct ErrorData {
    summary: Option<Cow<'static, str>>,
    reason: Option<Cow<'static, str>>,
}

impl ErrorData {
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn is_empty(&self) -> bool {
        self.summary.is_none() && self.reason.is_none()
    }

    pub fn render(&self) -> String {
        let mut parts = String::new();

        if let Some(summary) = self.summary() {
            parts.push_str(summary);
            parts.push_str(" :");
        }

        if let Some(reason) = self.reason() {
            parts.push_str(reason);
        }

        parts
    }
}

impl std::fmt::Display for ErrorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self.render();
        f.write_str(&text)
    }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Failed(ErrorData),
    Skipped(ErrorData),
}

impl ErrorKind {
    fn data_mut(&mut self) -> &mut ErrorData {
        match self {
            Self::Failed(data) | Self::Skipped(data) => data,
        }
    }

    pub fn failed() -> Self {
        Self::Failed(ErrorData::default())
    }

    pub fn skipped() -> Self {
        Self::Skipped(ErrorData::default())
    }

    pub fn with_summary(mut self, summary: impl Into<Cow<'static, str>>) -> Self {
        self.data_mut().summary = Some(summary.into());
        self
    }

    pub fn with_reason(mut self, reason: impl Into<Cow<'static, str>>) -> Self {
        self.data_mut().reason = Some(reason.into());
        self
    }

    pub fn data(&self) -> &ErrorData {
        match self {
            Self::Failed(data) | Self::Skipped(data) => data,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failed(data) => match data.is_empty() {
                true => f.write_str("[FAILED]"),
                false => write!(f, "[FAILED] {}", data),
            },

            Self::Skipped(data) => match data.is_empty() {
                true => f.write_str("[SKIPPED]"),
                false => write!(f, "[SKIPPED] {}", data),
            },
        }
    }
}
