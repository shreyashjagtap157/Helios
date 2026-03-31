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

use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref ENABLED: AtomicBool = AtomicBool::new(false);
    static ref TOKENS: AtomicUsize = AtomicUsize::new(0);
    static ref ITEMS: AtomicUsize = AtomicUsize::new(0);
    // last heartbeat as epoch ms
    static ref LAST_HB_MS: AtomicU64 = AtomicU64::new(0);
    // Lightweight parser state for debug dumps
    static ref PARSER_CUR: AtomicUsize = AtomicUsize::new(0);
    static ref PARSER_PREVIEW: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref PARSER_ERRORS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn enable() {
    ENABLED.store(true, Ordering::SeqCst);
    update_heartbeat();
}

pub fn disable() {
    ENABLED.store(false, Ordering::SeqCst);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::SeqCst)
}

pub fn inc_tokens(n: usize) {
    if enabled() {
        TOKENS.fetch_add(n, Ordering::SeqCst);
        update_heartbeat();
    }
}

pub fn inc_items() {
    if enabled() {
        ITEMS.fetch_add(1, Ordering::SeqCst);
        update_heartbeat();
    }
}

/// Record a compact parser snapshot: current token index and a small preview
pub fn record_parser_snapshot(current: usize, preview: &[String]) {
    if !enabled() {
        return;
    }
    PARSER_CUR.store(current, Ordering::SeqCst);
    if let Ok(mut p) = PARSER_PREVIEW.lock() {
        p.clear();
        for s in preview.iter().take(8) {
            p.push(s.clone());
        }
    }
    update_heartbeat();
}

/// Record a recent parser error message for diagnostics
pub fn record_parser_error(msg: &str) {
    if !enabled() {
        return;
    }
    if let Ok(mut e) = PARSER_ERRORS.lock() {
        e.push(msg.to_string());
        let excess = if e.len() > 50 { e.len() - 50 } else { 0 };
        if excess > 0 {
            e.drain(0..excess);
        }
    }
    update_heartbeat();
}

pub fn update_heartbeat() {
    LAST_HB_MS.store(now_ms(), Ordering::SeqCst);
}

pub fn snapshot() -> (usize, usize, u64) {
    (
        TOKENS.load(Ordering::SeqCst),
        ITEMS.load(Ordering::SeqCst),
        LAST_HB_MS.load(Ordering::SeqCst),
    )
}

/// Return a rich parser state snapshot for diagnostics
pub fn rich_snapshot() -> (usize, Vec<String>, Vec<String>) {
    let cur = PARSER_CUR.load(Ordering::SeqCst);
    let preview = PARSER_PREVIEW.lock().map(|p| p.clone()).unwrap_or_default();
    let errors = PARSER_ERRORS.lock().map(|e| e.clone()).unwrap_or_default();
    (cur, preview, errors)
}
