import re
import os

def process_main():
    p = 'src/main.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    
    # 1. Remove sysinfo, chrono, pprof imports
    content = re.sub(r'use sysinfo::\{.*?\};\n', '', content)
    content = re.sub(r'use pprof::ProfilerGuard;\n', '', content)
    
    # 2. Patch the sysinfo / chrono logic in the monitor thread
    content = content.replace("let mut sys = System::new_all();", "let mut sys_fake = 1;")
    content = content.replace("let pid = sysinfo::get_current_pid().unwrap_or_else(|_| sysinfo::Pid::from(0));", "let pid = std::process::id();")
    
    # 3. Patch the sys.refresh process loops
    content = re.sub(r'sys\.refresh_process\(pid\);', '', content)
    
    # 4. Patch `if let Some(p) = sys.process(pid) {` to a blunt `if true {`
    content = content.replace("if let Some(p) = sys.process(pid) {", "if true {")
    content = content.replace("p.cpu_usage()", "0.0")
    content = content.replace("p.memory()", "0")
    content = content.replace("p.virtual_memory()", "0")
    
    # 5. Patch `chrono::Utc::now().format(...)` to basic unix timestamp string
    chrono_replace = 'std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()'
    content = content.replace('chrono::Utc::now().format("%Y%m%dT%H%M%S")', chrono_replace)
    
    # 6. Disable the flamegraph profiling
    content = re.sub(r'#\[cfg\(unix\)\]\s*\{\s*if let Some\(g\) = &guard \{.*?\}\s*\}', r'#[cfg(unix)] { /* pprof removed */ }', content, flags=re.DOTALL)
    
    with open(p, 'w') as f: f.write(content)

def process_native():
    p = 'src/runtime/native.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    
    # Num CPUS
    content = content.replace("num_cpus::get() as i64", "1 /* num_cpus stubbed */")
    
    # Reqwest
    reqwest_pat = r'match reqwest::blocking::get\(url\)\s*\{\s*Ok\(resp\)\s*=>\s*match\s*resp\.text\(\).*?_ => Err.*?\}\s*\}'
    content = re.sub(reqwest_pat, r'Err(anyhow::anyhow!("reqwest disabled"))', content, flags=re.DOTALL|re.MULTILINE)
    
    # Ndarray
    content = content.replace("let tensor = ndarray::Array1::<f32>::zeros(size);", "return Err(anyhow::anyhow!(\"ndarray disabled\"));")
    content = content.replace("Ok(RuntimeValue::Vector(tensor))", "/* unreachable */")
    
    with open(p, 'w') as f: f.write(content)

def process_interpreter():
    p = 'src/runtime/interpreter.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    
    # Vector variant of zero usage except definitions -> just substitute it out or make it a stub list
    content = content.replace("Vector(ndarray::Array1<f32>),", "Vector(Vec<f32>),")
    with open(p, 'w') as f: f.write(content)

def process_ovm():
    p = 'src/codegen/ovm.rs'
    if not os.path.exists(p): return
    with open(p, 'r') as f: content = f.read()
    content = content.replace("num_cpus::get() as u32", "1")
    with open(p, 'w') as f: f.write(content)

os.chdir("d:/Project/Helios/omni-lang/compiler")
process_main()
process_native()
process_interpreter()
process_ovm()
print("Cleanup complete.")
