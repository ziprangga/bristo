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

/// Running process information.
///
/// Doc:
/// Represents a single process discovered during runtime
/// scanning.
///
/// Each process stores:
///
/// - Process identifier (PID).
/// - Full command line.
/// - Process name.
///
/// The command line is retained because it often contains
/// application identifiers that are not present in the
/// process name alone.
///
/// Note:
/// `Proc` is a lightweight snapshot of process information
/// captured during scanning and does not maintain a live
/// connection to the operating system.
#[derive(Debug, Default, Clone)]
pub struct Proc {
    pid: i32,
    command: String,
    name: String,
}

impl Proc {
    /// Contruct Proc
    pub fn new(pid: i32, command: String, name: String) -> Self {
        Self { pid, command, name }
    }
    /// get the copy of pid
    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// get the reference of command
    pub fn as_command(&self) -> &str {
        &self.command
    }

    /// get the reference of process name
    pub fn as_name(&self) -> &str {
        &self.name
    }
}
