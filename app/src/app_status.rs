use cleaner::ErrorKind;
use simple_status::StatusEvent;

use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum StatusResult {
    Success(Cow<'static, str>),
    Error(ErrorKind),
}

impl StatusResult {
    pub fn success(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Success(msg.into())
    }

    pub fn error(error: ErrorKind) -> Self {
        Self::Error(error)
    }
}

impl std::fmt::Display for StatusResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success(msg) => match msg.is_empty() {
                true => f.write_str("[SUCCESS]"),
                false => write!(f, "[SUCCESS] {}", msg),
            },

            Self::Error(error) => write!(f, "{}", error),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Status {
    status_event: Option<StatusEvent>,
    status_result: Option<StatusResult>,
}

impl Status {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status_event(mut self, status_event: StatusEvent) -> Self {
        self.status_event = Some(status_event);
        self
    }

    pub fn with_status_error(mut self, error_kind: ErrorKind) -> Self {
        self.status_result = Some(StatusResult::Error(error_kind));
        self
    }

    pub fn with_status_success(mut self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.status_result = Some(StatusResult::Success(msg.into()));
        self
    }

    pub fn status_event(&self) -> Option<&StatusEvent> {
        self.status_event.as_ref()
    }

    pub fn status_result(&self) -> Option<&StatusResult> {
        self.status_result.as_ref()
    }

    pub fn status_success(&self) -> Option<&str> {
        match &self.status_result {
            Some(StatusResult::Success(msg)) => Some(msg),
            _ => None,
        }
    }

    pub fn status_error(&self) -> Option<&ErrorKind> {
        match &self.status_result {
            Some(StatusResult::Error(error)) => Some(error),
            _ => None,
        }
    }

    pub fn update_status(&mut self, status: Status) {
        if let Some(event) = status.status_event {
            self.status_event = Some(event);
        }
        if let Some(result) = status.status_result {
            self.status_result = Some(result);
        }
    }

    pub fn clear(&mut self) {
        self.status_event = None;
        self.status_result = None;
    }
}
