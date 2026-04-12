import os
import re

def fix_main_cfg_bug():
    p = 'src/main.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    # Replace dangling cfg
    content = re.sub(r'#\[cfg\(unix\)\]\s*use anyhow::Result;', 'use anyhow::Result;', content)
    with open(p, 'w') as f: f.write(content)

def fix_profiler():
    p = 'src/runtime/profiler.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    if "RuntimeProfiler" not in content:
        content += """
pub struct RuntimeProfiler;
impl RuntimeProfiler {
    pub fn start_profiling() {}
    pub fn stop_profiling() -> String { String::new() }
}
"""
    with open(p, 'w') as f: f.write(content)

def fix_monitor():
    p = 'src/monitor.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    if "fn disable" not in content:
        content += "\npub fn disable() { ENABLED.store(false, Ordering::SeqCst); }"
    with open(p, 'w') as f: f.write(content)

os.chdir("d:/Project/Helios/omni-lang/compiler")
fix_main_cfg_bug()
fix_profiler()
fix_monitor()
print("Final stubs applied.")
