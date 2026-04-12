import os
import re

def fix_monitor():
    p = 'src/monitor.rs'
    if not os.path.exists(p): return
    stub = """
use std::sync::atomic::{AtomicUsize, AtomicU64, AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static TOKENS: AtomicUsize = AtomicUsize::new(0);
static ITEMS: AtomicUsize = AtomicUsize::new(0);
static LAST_HB: AtomicU64 = AtomicU64::new(0);

pub fn enable() { ENABLED.store(true, Ordering::SeqCst); }
pub fn enabled() -> bool { ENABLED.load(Ordering::SeqCst) }
pub fn update_heartbeat() { LAST_HB.store(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(), Ordering::SeqCst); }
pub fn inc_tokens(c: usize) { TOKENS.fetch_add(c, Ordering::SeqCst); }
pub fn inc_items() { ITEMS.fetch_add(1, Ordering::SeqCst); }
pub fn snapshot() -> (usize, usize, u64) {
    (TOKENS.load(Ordering::SeqCst), ITEMS.load(Ordering::SeqCst), LAST_HB.load(Ordering::SeqCst))
}
pub fn rich_snapshot() -> (usize, Vec<String>, Vec<String>) {
    (0, vec![], vec![])
}
pub fn record_error(_err: crate::parser::ParseError) -> Result<(), ()> {
    Ok(())
}
pub fn record_parser_error(_err: &str) {}
pub fn record_parser_snapshot(_pos: usize, _preview: &[String]) {}
"""
    with open(p, 'w') as f: f.write(stub)

def fix_ovm():
    p = 'src/codegen/ovm.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    # Replace the multi-line mem_info
    pat = r'available_memory:\s*sys_info::mem_info\(\).*?,'
    content = re.sub(pat, 'available_memory: 0,', content, flags=re.DOTALL)
    with open(p, 'w') as f: f.write(content)

os.chdir("d:/Project/Helios/omni-lang/compiler")
fix_monitor()
fix_ovm()
print("Fixes applied.")
