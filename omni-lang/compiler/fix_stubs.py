import re
import os

def fix_monitor():
    p = 'src/monitor.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    content = content.replace("add_token", "inc_tokens")
    content = content.replace("add_item", "inc_items")
    with open(p, 'w') as f: f.write(content)

def fix_profiler():
    p = 'src/runtime/profiler.rs'
    if not os.path.exists(p): return
    stub = """
pub fn start_profile(name: &str) {}
pub fn end_profile(name: &str) {}

pub struct OmniProfiler;
impl OmniProfiler {
    pub fn new(_name: &str) -> Self { Self }
    pub fn end(&self) {}
}
"""
    with open(p, 'w') as f: f.write(stub)

def fix_reqwest():
    p = 'src/runtime/native.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    content = re.sub(r'match reqwest::blocking::get.*?\}\s*\}', 'Err(anyhow::anyhow!("reqwest disabled"))', content, flags=re.DOTALL)
    with open(p, 'w') as f: f.write(content)

os.chdir("d:/Project/Helios/omni-lang/compiler")
fix_monitor()
fix_profiler()
fix_reqwest()
print("Stubs fixed.")
