#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static TOKENS: AtomicUsize = AtomicUsize::new(0);
static ITEMS: AtomicUsize = AtomicUsize::new(0);
static LAST_HB: AtomicU64 = AtomicU64::new(0);

pub fn enable() {
    ENABLED.store(true, Ordering::SeqCst);
}
pub fn enabled() -> bool {
    ENABLED.load(Ordering::SeqCst)
}
pub fn update_heartbeat() {
    LAST_HB.store(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        Ordering::SeqCst,
    );
}
pub fn inc_tokens(c: usize) {
    TOKENS.fetch_add(c, Ordering::SeqCst);
}
pub fn inc_items() {
    ITEMS.fetch_add(1, Ordering::SeqCst);
}
pub fn snapshot() -> (usize, usize, u64) {
    (
        TOKENS.load(Ordering::SeqCst),
        ITEMS.load(Ordering::SeqCst),
        LAST_HB.load(Ordering::SeqCst),
    )
}
pub fn rich_snapshot() -> (usize, Vec<String>, Vec<String>) {
    (0, vec![], vec![])
}
pub fn record_error(_err: crate::parser::ParseError) -> Result<(), ()> {
    Ok(())
}
pub fn record_parser_error(_err: &str) {}
pub fn record_parser_snapshot(_pos: usize, _preview: &[String]) {}

pub fn disable() {
    ENABLED.store(false, Ordering::SeqCst);
}
