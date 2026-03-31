// Copyright 2024 Shreyash Jagtap
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

// Minimal diagnostics module used by the compiler crate.
// This provides lightweight types so the crate compiles while the
// full diagnostics implementation is developed.

use std::fmt;

#[derive(Debug, Clone)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: Option<String>,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:?}] {}: {}",
            self.level,
            self.code.as_deref().unwrap_or(""),
            self.message
        )
    }
}

#[derive(Debug, Clone)]
pub enum ErrorCode {
    Generic,
}

#[derive(Debug, Clone)]
pub enum QualityStandards {
    Default,
}

pub struct DiagnosticCollector {
    pub items: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, d: Diagnostic) {
        self.items.push(d);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.items
    }
}

impl Default for DiagnosticCollector {
    fn default() -> Self {
        Self::new()
    }
}
