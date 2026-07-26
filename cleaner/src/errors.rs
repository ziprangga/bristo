// Copyright 2026 ziprangga
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Error types and result utilities.
//!
//! Doc:
//! Provides the application's error model together with a
//! convenience `Result` alias.
//!
//! The module is built around two primary types:
//!
//! - `Kind` classifies the overall outcome of an operation.
//! - `ErrorKind` stores structured error information.
//!
//! Error reporting is intentionally lightweight and focuses on
//! communicating operation outcomes rather than capturing
//! detailed diagnostic information.
//!
//! An error may contain:
//!
//! - A classification kind.
//! - An optional summary.
//! - An optional reason.
//!
//! This error system is used throughout the application for:
//!
//! - Scanning operations.
//! - Cleanup operations.
//! - Trash operations.
//! - Status reporting.
//! - User-interface feedback.
//!
//! Supported classifications include:
//!
//! - `Failed` for operations that could not be completed.
//! - `Skipped` for operations that were intentionally not
//!   performed.
//!
//! Design:
//! Error classification is separated from descriptive text.
//!
//! This allows callers to reason about operation outcomes
//! using a small and stable set of categories while still
//! presenting meaningful information through summaries and
//! reasons.
//!
//! `ErrorKind` uses `Cow<'static, str>` to support both static
//! messages and dynamically generated content without
//! requiring separate APIs.
//!
//! The type also implements standard Rust error traits and
//! ordering behaviour so it can integrate naturally with
//! common error-handling and status-reporting workflows.
//!
//! Note:
//! `Skipped` does not necessarily represent a failure.
//!
//! Operations may be skipped intentionally due to filtering
//! rules, user choices, missing prerequisites, or other
//! expected conditions.
//!
//! This module is intended primarily for application-level
//! status reporting and user-facing feedback.
//!
//! It is not intended to replace richer diagnostic error
//! systems used for low-level debugging or detailed failure
//! analysis.
//!..

use std::borrow::Cow;

/// Application result type.
///
/// Doc:
/// Convenience alias that uses `ErrorKind` as the default
/// error type throughout the application.
///
/// Design:
/// Using a shared result type promotes consistency across
/// modules and reduces repetitive type declarations.
///
/// Note:
/// Modules may still define or convert from more specialised
/// error types when additional context is required.
pub type Result<T> = std::result::Result<T, ErrorKind>;

/// Top-level error classification.
///
/// Doc:
/// Represents the overall category of an operation result.
///
/// The variant describes what happened at a high level,
/// independent of any detailed explanation.
///
/// Design:
/// The enum is intentionally small and stable so it can be
/// used for sorting, filtering, prioritisation, status
/// reporting, and UI presentation.
///
/// Note:
/// Additional descriptive information should be stored in
/// higher-level error structures rather than added directly
/// to this enum.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// The operation could not be completed successfully.
    Failed,
    /// The operation was intentionally not performed.
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

/// Application error representation.
///
/// Doc:
/// Stores structured error information describing the outcome
/// of an operation.
///
/// Each error consists of:
///
/// - An error classification.
/// - An optional summary.
/// - An optional detailed reason.
///
/// The summary is intended to provide a concise explanation
/// suitable for status messages and UI presentation.
///
/// The reason provides additional context when available.
///
/// Design:
/// Construction follows a builder-style pattern so callers can
/// create concise errors while attaching optional details as
/// needed.
///
/// Examples:
///
/// - A failed file operation.
/// - A skipped cleanup step.
/// - A validation failure.
/// - A partially completed task.
///
/// Note:
/// An error may contain neither a summary nor a reason.
///
/// Callers should not assume descriptive text is always
/// available.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    pub fn priority(&self) -> u8 {
        match self.kind {
            Kind::Failed => 0,
            Kind::Skipped => 1,
        }
    }
}

/// Human-readable error formatter.
///
/// Doc:
/// Formats an error using its classification, summary,
/// and optional reason.
///
/// The resulting output follows a compact structure:
///
/// `[Kind: Summary] - Reason`
///
/// Components are omitted when not present.
///
/// Design:
/// Formatting is intended for user-facing status messages,
/// logs, and debugging output while remaining concise and
/// easy to scan.
///
/// Note:
/// The formatted representation is not considered a stable
/// serialisation format and should not be parsed
/// programmatically.
impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;

        f.write_str(self.kind.as_str())?;

        if let Some(summary) = self.summary() {
            write!(f, ": {}", summary)?;
        }

        f.write_str("]")?;

        if let Some(reason) = self.reason() {
            write!(f, " - {}", reason)?;
        }

        Ok(())
    }
}

/// Standard error integration.
///
/// Doc:
/// Enables interoperability with APIs and libraries that
/// operate on `std::error::Error`.
///
/// Design:
/// This allows `ErrorKind` to participate in common Rust
/// error-handling patterns without requiring additional
/// wrapper types.
///
/// Note:
/// `ErrorKind` does not currently expose an underlying source
/// error.
impl std::error::Error for ErrorKind {}

/// Error priority ordering.
///
/// Doc:
/// Provides deterministic ordering based on error
/// classification.
///
/// Design:
/// Ordering is delegated to `Kind`, allowing collections of
/// errors to be sorted according to their category.
///
/// Note:
/// Summaries and reasons are intentionally ignored when
/// comparing errors.
impl Ord for ErrorKind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.kind.cmp(&other.kind)
    }
}

impl PartialOrd for ErrorKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
