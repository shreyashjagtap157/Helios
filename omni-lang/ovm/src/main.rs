//! OVM Runner — Standalone Omni Virtual Machine
//! Loads and executes OVM bytecode files (.ovm).
//! Opcodes match the compiler's ovm.rs exactly.

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("OVM — Omni Virtual Machine v1.0.0");
        eprintln!("Usage: ovm-runner <program.ovm>");
        process::exit(1);
    }
    if args[1] == "--version" || args[1] == "-v" {
        println!("ovm-runner 1.0.0");
        return;
    }
    let bytes = match std::fs::read(&args[1]) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    if bytes.len() < 4 || &bytes[0..4] != b"OVM\0" {
        eprintln!(
            "Error: '{}' is not a valid OVM file (bad magic number)",
            args[1]
        );
        process::exit(1);
    }
    let mut vm = VM::new(&bytes, &args[2..]);
    match vm.run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("OVM Error: {}", e);
            process::exit(1);
        }
    }
}

#[derive(Clone, Debug)]
enum V {
    Null,
    I64(i64),
    F64(f64),
    Bool(bool),
    Str(String),
    Array(std::rc::Rc<std::cell::RefCell<Vec<V>>>),
    Object(std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, V>>>),
}

struct Frame {
    ret_pc: usize,
    base: usize,
    locals: [V; 128],
}

struct VM {
    code: Vec<u8>,
    consts: Vec<V>,
    funcs: Vec<(String, usize, u16)>,
    entry: usize,
    stack: Vec<V>,
    frames: Vec<Frame>,
    pc: usize,
    global_locals: [V; 256],
    heap: Vec<Option<Vec<u8>>>,
    heap_free: Vec<usize>,
    args: Vec<String>,
}

impl VM {
    fn new(data: &[u8], args: &[String]) -> Self {
        let mut vm = VM {
            code: Vec::new(),
            consts: Vec::new(),
            funcs: Vec::new(),
            entry: 0,
            stack: Vec::with_capacity(4096),
            frames: Vec::with_capacity(256),
            pc: 0,
            global_locals: std::array::from_fn(|_| V::Null),
            heap: Vec::new(),
            heap_free: Vec::new(),
            args: args.to_vec(),
        };
        if data.len() >= 84 && &data[0..4] == b"OVM\0" {
            vm.load(data);
        }
        vm
    }
    fn u64(data: &[u8], o: usize) -> u64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(&data[o..o + 8]);
        u64::from_le_bytes(b)
    }
    fn u32(data: &[u8], o: usize) -> u32 {
        let mut b = [0u8; 4];
        b.copy_from_slice(&data[o..o + 4]);
        u32::from_le_bytes(b)
    }
    fn u16(data: &[u8], o: usize) -> u16 {
        let mut b = [0u8; 2];
        b.copy_from_slice(&data[o..o + 2]);
        u16::from_le_bytes(b)
    }

    fn load(&mut self, data: &[u8]) {
        const MAX_CONSTANTS: usize = 1_000_000;
        const MAX_STRING_LEN: usize = 16 * 1024 * 1024; // 16MB max string
        const MAX_FUNCTIONS: usize = 1_000_000;
        const MAX_CODE_SIZE: usize = 64 * 1024 * 1024; // 64MB max code

        if data.len() > MAX_CODE_SIZE {
            eprintln!(
                "[ovm] Error: file too large ({} bytes, max {})",
                data.len(),
                MAX_CODE_SIZE
            );
            return;
        }

        self.entry = Self::u64(data, 12) as usize;
        let co = Self::u64(data, 20) as usize;
        let codelen = Self::u64(data, 44) as usize;
        let codeoff = Self::u64(data, 36) as usize;

        if codelen > MAX_CODE_SIZE || codeoff.saturating_add(codelen) > data.len() {
            eprintln!("[ovm] Error: invalid code section size or offset");
            return;
        }

        // Constants
        if co > 0 && co < data.len() {
            let n = Self::u32(data, co) as usize;
            if n > MAX_CONSTANTS {
                eprintln!(
                    "[ovm] Error: too many constants ({}, max {})",
                    n, MAX_CONSTANTS
                );
                return;
            }
            let mut p = co + 4;
            for _ in 0..n {
                if p >= data.len() {
                    break;
                }
                let tag = data[p];
                p += 1;
                match tag {
                    0x01 => {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(&data[p..p + 8]);
                        p += 8;
                        self.consts.push(V::I64(i64::from_le_bytes(b)));
                    }
                    0x02 => {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(&data[p..p + 8]);
                        p += 8;
                        self.consts.push(V::F64(f64::from_le_bytes(b)));
                    }
                    0x03 => {
                        let l = Self::u32(data, p) as usize;
                        p += 4;
                        if l > MAX_STRING_LEN {
                            eprintln!(
                                "[ovm] Error: string constant too large ({}, max {})",
                                l, MAX_STRING_LEN
                            );
                            return;
                        }
                        if p + l > data.len() {
                            eprintln!("[ovm] Error: string constant extends past file end");
                            return;
                        }
                        self.consts
                            .push(V::Str(String::from_utf8_lossy(&data[p..p + l]).to_string()));
                        p += l;
                    }
                    0x04 => {
                        let l = Self::u32(data, p) as usize;
                        p += 4 + l;
                        self.consts.push(V::Null);
                    }
                    _ => self.consts.push(V::Null),
                }
            }
        }
        // Functions from code section
        if codeoff > 0 && codelen > 0 {
            let cd = &data[codeoff..codeoff + codelen];
            let n = Self::u32(cd, 0) as usize;
            if n > MAX_FUNCTIONS {
                eprintln!(
                    "[ovm] Error: too many functions ({}, max {})",
                    n, MAX_FUNCTIONS
                );
                return;
            }
            let mut p = 4;
            let mut all = Vec::new();
            for _ in 0..n {
                if p + 14 > cd.len() {
                    break;
                }
                let start = all.len();
                let ni = Self::u32(cd, p) as usize;
                p += 4;
                let param_count = Self::u16(cd, p);
                p += 2;
                p += 2;
                p += 2; // param, local, stack counts
                let bl = Self::u32(cd, p) as usize;
                p += 4;
                if bl > MAX_CODE_SIZE {
                    eprintln!(
                        "[ovm] Error: function bytecode too large ({}, max {})",
                        bl, MAX_CODE_SIZE
                    );
                    return;
                }
                let name = match self.consts.get(ni) {
                    Some(V::Str(s)) => s.clone(),
                    _ => format!("f{}", ni),
                };
                if p + bl <= cd.len() {
                    all.extend_from_slice(&cd[p..p + bl]);
                    p += bl;
                }
                self.funcs.push((name, start, param_count));
            }
            self.code = all;
        }
    }

    fn run(&mut self) -> Result<(), String> {
        let main_idx = self
            .funcs
            .iter()
            .position(|(n, _, _)| n == "main")
            .unwrap_or(self.entry);
        if main_idx >= self.funcs.len() {
            return Err("No entry point".into());
        }
        self.pc = self.funcs[main_idx].1;
        self.exec()
    }

    fn exec(&mut self) -> Result<(), String> {
        let mut steps: u64 = 0;
        loop {
            steps += 1;
            if steps > 5_000_000 {
                return Err(format!(
                    "Infinite loop at pc={} sp={}",
                    self.pc,
                    self.stack.len()
                ));
            }
            if self.pc >= self.code.len() {
                return Ok(());
            }
            let op = self.code[self.pc];
            self.pc += 1;
            match op {
                // Stack (0x00-0x0F)
                0x00 => {} // Nop
                0x01 => {
                    let v = self.code[self.pc] as i8;
                    self.pc += 1;
                    self.push(V::I64(v as i64));
                }
                0x02 => {
                    let mut b = [0u8; 2];
                    b.copy_from_slice(&self.code[self.pc..self.pc + 2]);
                    self.pc += 2;
                    self.push(V::I64(i16::from_le_bytes(b) as i64));
                }
                0x03 => {
                    let mut b = [0u8; 4];
                    b.copy_from_slice(&self.code[self.pc..self.pc + 4]);
                    self.pc += 4;
                    self.push(V::I64(i32::from_le_bytes(b) as i64));
                }
                0x04 => {
                    let v = self.ri64();
                    self.push(V::I64(v));
                }
                0x06 => {
                    let v = self.rf64();
                    self.push(V::F64(v));
                }
                0x07 => {
                    let i = self.ru32() as usize;
                    if i < self.consts.len() {
                        self.push(self.consts[i].clone());
                    } else {
                        self.push(V::Null);
                    }
                }
                0x08 => self.push(V::Null),
                0x09 => self.push(V::Bool(true)),
                0x0A => self.push(V::Bool(false)),
                0x0B => {
                    let v = self.pk(0);
                    self.push(v);
                }
                0x0D => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(a);
                    self.push(b);
                }
                0x0E => {
                    // Rot: rotate top 3 stack values [a, b, c] → [c, a, b]
                    let c = self.pop();
                    let b = self.pop();
                    let a = self.pop();
                    self.push(b);
                    self.push(c);
                    self.push(a);
                }
                0x0F => {
                    self.pop();
                }

                // Arithmetic i64 (0x10-0x17)
                0x10 => {
                    let left = self.pop(); // left operand (on top, pushed last)
                    let right = self.pop(); // right operand (at bottom, pushed first)
                    match (&left, &right) {
                        (V::Str(a), V::Str(b)) => {
                            self.push(V::Str(format!("{}{}", a, b)));
                        }
                        (V::Str(a), _) => {
                            self.push(V::Str(format!("{}{}", a, self.vs(&right))));
                        }
                        (_, V::Str(b)) => {
                            self.push(V::Str(format!("{}{}", self.vs(&left), b)));
                        }
                        (V::I64(_), V::I64(_)) => {
                            let a = self.vi(&left);
                            let b = self.vi(&right);
                            match a.checked_add(b) {
                                Some(v) => self.push(V::I64(v)),
                                None => {
                                    return Err(format!(
                                        "integer overflow: {} + {} at pc={}",
                                        a,
                                        b,
                                        self.pc - 1
                                    ))
                                }
                            }
                        }
                        (V::F64(_), V::F64(_))
                        | (V::I64(_), V::F64(_))
                        | (V::F64(_), V::I64(_)) => {
                            self.push(V::F64(self.vf(&left) + self.vf(&right)));
                        }
                        _ => {
                            return Err(format!(
                                "type error: cannot add {} and {} at pc={}",
                                self.vt(&left),
                                self.vt(&right),
                                self.pc - 1
                            ));
                        }
                    }
                }
                0x11 => {
                    let l = self.pop();
                    let r = self.pop();
                    match (&l, &r) {
                        (V::I64(_), V::I64(_)) => {
                            let a = self.vi(&l);
                            let b = self.vi(&r);
                            match a.checked_sub(b) {
                                Some(v) => self.push(V::I64(v)),
                                None => {
                                    return Err(format!(
                                        "integer overflow: {} - {} at pc={}",
                                        a,
                                        b,
                                        self.pc - 1
                                    ))
                                }
                            }
                        }
                        (V::F64(_), V::F64(_))
                        | (V::I64(_), V::F64(_))
                        | (V::F64(_), V::I64(_)) => {
                            self.push(V::F64(self.vf(&l) - self.vf(&r)));
                        }
                        _ => {
                            return Err(format!(
                                "type error: cannot subtract {} and {} at pc={}",
                                self.vt(&l),
                                self.vt(&r),
                                self.pc - 1
                            ))
                        }
                    }
                }
                0x12 => {
                    let l = self.pop();
                    let r = self.pop();
                    match (&l, &r) {
                        (V::I64(_), V::I64(_)) => {
                            let a = self.vi(&l);
                            let b = self.vi(&r);
                            match a.checked_mul(b) {
                                Some(v) => self.push(V::I64(v)),
                                None => {
                                    return Err(format!(
                                        "integer overflow: {} * {} at pc={}",
                                        a,
                                        b,
                                        self.pc - 1
                                    ))
                                }
                            }
                        }
                        (V::F64(_), V::F64(_))
                        | (V::I64(_), V::F64(_))
                        | (V::F64(_), V::I64(_)) => {
                            self.push(V::F64(self.vf(&l) * self.vf(&r)));
                        }
                        _ => {
                            return Err(format!(
                                "type error: cannot multiply {} and {} at pc={}",
                                self.vt(&l),
                                self.vt(&r),
                                self.pc - 1
                            ))
                        }
                    }
                }
                0x13 => {
                    let l = self.pop();
                    let r = self.pop();
                    match (&l, &r) {
                        (V::I64(_), V::I64(_)) => {
                            let a = self.vi(&l);
                            let b = self.vi(&r);
                            if b == 0 {
                                return Err("division by zero".into());
                            }
                            if a == i64::MIN && b == -1 {
                                return Err(format!(
                                    "integer overflow: i64::MIN / -1 at pc={}",
                                    self.pc - 1
                                ));
                            }
                            self.push(V::I64(a / b));
                        }
                        (V::F64(_), V::F64(_))
                        | (V::I64(_), V::F64(_))
                        | (V::F64(_), V::I64(_)) => {
                            let a = self.vf(&l);
                            let b = self.vf(&r);
                            self.push(V::F64(a / b));
                        }
                        _ => {
                            return Err(format!(
                                "type error: cannot divide {} and {} at pc={}",
                                self.vt(&l),
                                self.vt(&r),
                                self.pc - 1
                            ))
                        }
                    }
                }
                0x14 => {
                    let l = self.pop();
                    let r = self.pop();
                    match (&l, &r) {
                        (V::I64(_), V::I64(_)) => {
                            let a = self.vi(&l);
                            let b = self.vi(&r);
                            if b == 0 {
                                return Err("modulo by zero".into());
                            }
                            if a == i64::MIN && b == -1 {
                                self.push(V::I64(0));
                            } else {
                                self.push(V::I64(a % b));
                            }
                        }
                        _ => {
                            return Err(format!(
                                "type error: cannot mod {} and {} at pc={}",
                                self.vt(&l),
                                self.vt(&r),
                                self.pc - 1
                            ))
                        }
                    }
                }
                0x15 => {
                    let v = self.pi();
                    self.push(V::I64(-v));
                }

                // Arithmetic f64 (0x18-0x1F)
                0x18 => {
                    let l = self.pf();
                    let r = self.pf();
                    self.push(V::F64(l + r));
                }
                0x19 => {
                    let l = self.pf();
                    let r = self.pf();
                    self.push(V::F64(l - r));
                }
                0x1A => {
                    let l = self.pf();
                    let r = self.pf();
                    self.push(V::F64(l * r));
                }
                0x1B => {
                    let l = self.pf();
                    let r = self.pf();
                    self.push(V::F64(l / r));
                }
                0x1C => {
                    let v = self.pf();
                    self.push(V::F64(-v));
                }

                // Inc/Dec (0x20-0x21)
                0x20 => {
                    let v = self.pi();
                    self.push(V::I64(v + 1));
                }
                0x21 => {
                    let v = self.pi();
                    self.push(V::I64(v - 1));
                }

                // Bitwise (0x30-0x36)
                0x30 => {
                    let l = self.pi();
                    let r = self.pi();
                    self.push(V::I64(l & r));
                }
                0x31 => {
                    let l = self.pi();
                    let r = self.pi();
                    self.push(V::I64(l | r));
                }
                0x32 => {
                    let l = self.pi();
                    let r = self.pi();
                    self.push(V::I64(l ^ r));
                }
                0x33 => {
                    // O-069: Boolean NOT (not bitwise complement)
                    let v = self.pop();
                    self.push(V::Bool(!self.truthy(&v)));
                }
                0x34 => {
                    let l = self.pi();
                    let r = self.pi();
                    self.push(V::I64(l << r));
                }
                0x35 => {
                    let l = self.pi();
                    let r = self.pi();
                    self.push(V::I64(l >> r));
                }

                // Comparison (0x40-0x47)
                0x40 => {
                    let l = self.pop();
                    let r = self.pop();
                    self.push(V::Bool(self.eq(&l, &r)));
                }
                0x41 => {
                    let l = self.pop();
                    let r = self.pop();
                    self.push(V::Bool(!self.eq(&l, &r)));
                }
                0x42 => {
                    let l = self.pop();
                    let r = self.pop();
                    self.push(V::Bool(self.vcmp_lt(&l, &r)));
                }
                0x43 => {
                    let l = self.pop();
                    let r = self.pop();
                    self.push(V::Bool(self.vcmp_le(&l, &r)));
                }
                0x44 => {
                    let l = self.pop();
                    let r = self.pop();
                    self.push(V::Bool(self.vcmp_gt(&l, &r)));
                }
                0x45 => {
                    let l = self.pop();
                    let r = self.pop();
                    self.push(V::Bool(self.vcmp_ge(&l, &r)));
                }
                0x46 => {
                    // Cmp: three-way comparison, returns -1, 0, or 1
                    let l = self.pop();
                    let r = self.pop();
                    let result = match (&l, &r) {
                        (V::I64(a), V::I64(b)) => {
                            if a < b {
                                -1
                            } else if a > b {
                                1
                            } else {
                                0
                            }
                        }
                        (V::F64(a), V::F64(b)) => {
                            if a < b {
                                -1
                            } else if a > b {
                                1
                            } else {
                                0
                            }
                        }
                        _ => {
                            let a = self.vf(&l);
                            let b = self.vf(&r);
                            if a < b {
                                -1
                            } else if a > b {
                                1
                            } else {
                                0
                            }
                        }
                    };
                    self.push(V::I64(result));
                }
                0x47 => {
                    let v = self.pop();
                    self.push(V::Bool(matches!(v, V::Null)));
                }

                // Control (0x50-0x5C)
                0x50 => {
                    let o = self.ri32();
                    self.pc = (self.pc as i64 + o as i64) as usize;
                }
                0x52 => {
                    let o = self.ri32();
                    if !self.tr() {
                        self.pc = (self.pc as i64 + o as i64) as usize;
                    }
                }
                0x53 => {
                    let o = self.ri32();
                    if self.tr() {
                        self.pc = (self.pc as i64 + o as i64) as usize;
                    }
                }
                0x58 => {
                    let idx = self.ru32() as usize;
                    // O-012: Stack overflow protection
                    if self.frames.len() >= 10000 {
                        return Err("stack overflow: max call depth (10000) exceeded".into());
                    }
                    if idx < self.funcs.len() {
                        let fname = self.funcs[idx].0.clone();
                        match fname.as_str() {
                            "print" | "println" => {
                                let v = self.pop();
                                self.pv(&v);
                                if fname == "println" {
                                    println!();
                                }
                                self.push(V::Null);
                            }
                            "format" | "to_string" | "str" => {
                                let v = self.pop();
                                self.push(V::Str(self.vs(&v)));
                            }
                            "int" => {
                                let v = self.pop();
                                self.push(V::I64(self.vi(&v)));
                            }
                            "float" => {
                                let v = self.pop();
                                self.push(V::F64(self.vf(&v)));
                            }
                            "type_of" => {
                                let v = self.pop();
                                self.push(V::Str(self.vt(&v).into()));
                            }
                            "len" => {
                                let v = self.pop();
                                let l = match &v {
                                    V::Str(s) => s.len() as i64,
                                    V::Array(a) => a.borrow().len() as i64,
                                    _ => 0,
                                };
                                self.push(V::I64(l));
                            }
                            "assert" => {
                                let v = self.pop();
                                if !self.truthy(&v) {
                                    return Err("assertion failed".into());
                                }
                                self.push(V::Null);
                            }
                            "sqrt" => {
                                let v = self.pop();
                                self.push(V::F64(self.vf(&v).sqrt()));
                            }
                            "abs" => {
                                let v = self.pop();
                                let i = self.vi(&v);
                                self.push(V::I64(i.abs()));
                            }
                            // File I/O stubs (O-105) — not yet implemented
                            "open" => {
                                let _path = self.pop();
                                return Err("syscall 'open' is not implemented (O-105: file I/O not yet available)".into());
                            }
                            "read_file" => {
                                let _path = self.pop();
                                return Err("syscall 'read_file' is not implemented (O-105: file I/O not yet available)".into());
                            }
                            "write_file" => {
                                let _content = self.pop();
                                let _path = self.pop();
                                return Err("syscall 'write_file' is not implemented (O-105: file I/O not yet available)".into());
                            }
                            "close" => {
                                let _fd = self.pop();
                                return Err("syscall 'close' is not implemented (O-105: file I/O not yet available)".into());
                            }
                            // Command-line args (O-106)
                            "args" | "argv" => {
                                let arr: Vec<V> =
                                    self.args.iter().map(|a| V::Str(a.clone())).collect();
                                self.push(V::Array(std::rc::Rc::new(std::cell::RefCell::new(arr))));
                            }
                            "arg_count" | "argc" => {
                                self.push(V::I64(self.args.len() as i64));
                            }
                            "arg_at" => {
                                let idx = self.pop();
                                let i = self.vi(&idx) as usize;
                                let val = if i < self.args.len() {
                                    V::Str(self.args[i].clone())
                                } else {
                                    V::Null
                                };
                                self.push(val);
                            }
                            _ => {
                                let faddr = self.funcs[idx].1;
                                let param_count = self.funcs[idx].2 as usize;
                                // Pop args from stack (last arg is on top)
                                let mut args = Vec::with_capacity(param_count);
                                for _ in 0..param_count {
                                    args.push(self.pop());
                                }
                                args.reverse(); // first arg first
                                                // Push call frame
                                self.frames.push(Frame {
                                    ret_pc: self.pc,
                                    base: self.stack.len(),
                                    locals: std::array::from_fn(|_| V::Null),
                                });
                                // Store args in callee's locals
                                if let Some(frame) = self.frames.last_mut() {
                                    for (i, arg) in args.into_iter().enumerate() {
                                        if i < 128 {
                                            frame.locals[i] = arg;
                                        }
                                    }
                                }
                                self.pc = faddr;
                                continue;
                            }
                        }
                    } else {
                        // O-072: Invalid function index error
                        return Err(format!("invalid function index {} (have {} functions)", idx, self.funcs.len()));
                    }
                }
                0x5A => {
                    let r = self.pop();
                    if let Some(f) = self.frames.pop() {
                        self.stack.truncate(f.base);
                        self.pc = f.ret_pc;
                        self.push(r);
                    } else {
                        return Ok(());
                    }
                }
                0x5B => {
                    if let Some(f) = self.frames.pop() {
                        self.stack.truncate(f.base);
                        self.pc = f.ret_pc;
                        self.push(V::Null);
                    } else {
                        return Ok(());
                    }
                }
                0x59 => {
                    // CallInd: indirect call — function index on stack
                    let func_val = self.pop();
                    let idx = self.vi(&func_val) as usize;
                    if idx < self.funcs.len() {
                        let faddr = self.funcs[idx].1;
                        let param_count = self.funcs[idx].2 as usize;
                        let mut args = Vec::with_capacity(param_count);
                        for _ in 0..param_count {
                            args.push(self.pop());
                        }
                        args.reverse();
                        let mut frame = Frame {
                            ret_pc: self.pc,
                            base: self.stack.len(),
                            locals: std::array::from_fn(|_| V::Null),
                        };
                        for (i, arg) in args.into_iter().enumerate() {
                            if i < 128 {
                                frame.locals[i] = arg;
                            }
                        }
                        self.frames.push(frame);
                        self.pc = faddr;
                        continue;
                    }
                }

                // Locals (0x60-0x65)
                0x60 => {
                    let i = self.ru16() as usize;
                    let v = if let Some(f) = self.frames.last() {
                        if i < f.locals.len() {
                            f.locals[i].clone()
                        } else {
                            V::Null
                        }
                    } else if i < 256 {
                        self.global_locals[i].clone()
                    } else {
                        V::Null
                    };
                    self.push(v);
                }
                0x61 => {
                    let i = self.ru16() as usize;
                    let v = self.pop();
                    if let Some(f) = self.frames.last_mut() {
                        if i < f.locals.len() {
                            f.locals[i] = v;
                        }
                    } else if i < 256 {
                        self.global_locals[i] = v;
                    }
                }
                0x64 => {
                    /* alloc_loc — true no-op, don't push anything */
                    let _ = self.ru16(); // skip operand
                }

                // Globals (0x70-0x72)
                0x70 => {
                    self.ru32();
                    self.push(V::Null);
                } // LoadGlb
                0x71 => {
                    self.ru32();
                    self.pop();
                } // StoreGlb
                0x72 => {
                    let i = self.ru32() as usize;
                    if i < self.consts.len() {
                        self.push(self.consts[i].clone());
                    } else {
                        self.push(V::Null);
                    }
                }

                // Objects/Structs (0x90-0x95)
                0x90 => {
                    // New: create object — type name index u32, field count u16
                    let _type_idx = self.ru32();
                    let field_count = self.ru16() as usize;
                    let mut obj = std::collections::HashMap::new();
                    // Fields are pushed as key-value pairs (name, value)
                    let mut vals = Vec::with_capacity(field_count * 2);
                    for _ in 0..field_count * 2 {
                        vals.push(self.pop());
                    }
                    vals.reverse();
                    let mut i = 0;
                    while i < vals.len() {
                        let key = self.vs(&vals[i]);
                        let val = vals[i + 1].clone();
                        obj.insert(key, val);
                        i += 2;
                    }
                    self.push(V::Object(std::rc::Rc::new(std::cell::RefCell::new(obj))));
                }
                0x91 => {
                    // GetField: object on stack, field name index u32
                    let field_idx = self.ru32() as usize;
                    let obj = self.pop();
                    let field_name = if field_idx < self.consts.len() {
                        self.vs(&self.consts[field_idx].clone())
                    } else {
                        String::new()
                    };
                    match &obj {
                        V::Object(o) => {
                            let val = o.borrow().get(&field_name).cloned().unwrap_or(V::Null);
                            self.push(val);
                        }
                        V::Null => {
                            return Err(format!(
                                "null dereference: cannot access field '{}' on null at pc={}",
                                field_name,
                                self.pc - 1
                            ));
                        }
                        _ => {
                            return Err(format!(
                                "type error: cannot access field '{}' on {} at pc={}",
                                field_name,
                                self.vt(&obj),
                                self.pc - 1
                            ));
                        }
                    }
                }
                0x92 => {
                    // SetField: value on stack top, object below, field name u32
                    let field_idx = self.ru32() as usize;
                    let val = self.pop();
                    let field_name = if field_idx < self.consts.len() {
                        self.vs(&self.consts[field_idx].clone())
                    } else {
                        String::new()
                    };
                    if let Some(obj) = self.stack.last() {
                        if let V::Object(o) = obj {
                            o.borrow_mut().insert(field_name, val);
                        }
                    }
                    self.pop(); // pop the object we peeked at
                }

                // Arrays (0xA0-0xA4)
                0xA0 => {
                    // NewArray: count elements from stack, create array
                    let count = self.ru32() as usize;
                    let mut arr = Vec::with_capacity(count);
                    for _ in 0..count {
                        arr.push(self.pop());
                    }
                    arr.reverse();
                    self.push(V::Array(std::rc::Rc::new(std::cell::RefCell::new(arr))));
                }
                0xA1 => {
                    // ArrayLen: push length of array on stack
                    let v = self.pop();
                    let len = match &v {
                        V::Array(a) => a.borrow().len() as i64,
                        V::Str(s) => s.len() as i64,
                        _ => 0,
                    };
                    self.push(V::I64(len));
                }
                0xA2 => {
                    // ArrayGet: array, index on stack → push element
                    let idx_val = self.pop();
                    let arr_val = self.pop();
                    let idx = self.vi(&idx_val) as usize;
                    match &arr_val {
                        V::Array(a) => {
                            let borrowed = a.borrow();
                            let val = borrowed.get(idx).cloned().unwrap_or(V::Null);
                            self.push(val);
                        }
                        V::Str(s) => {
                            let ch = s.as_bytes().get(idx).copied().unwrap_or(0);
                            self.push(V::I64(ch as i64));
                        }
                        _ => self.push(V::Null),
                    }
                }
                0xA3 => {
                    // ArraySet: stack has [..., array, index, value] (value on top)
                    // Peek (don't pop) the array so we modify the original RefCell
                    let val = self.pop();
                    let idx_val = self.pop();
                    let idx = self.vi(&idx_val) as usize;
                    if let Some(arr_val) = self.stack.last() {
                        if let V::Array(a) = arr_val {
                            let mut borrowed = a.borrow_mut();
                            if idx < borrowed.len() {
                                borrowed[idx] = val;
                            }
                        }
                    }
                    self.pop(); // pop the array we peeked at
                }
                0xA4 => {
                    // ArraySlice: array, start, end on stack
                    let end_val = self.pop();
                    let start_val = self.pop();
                    let arr_val = self.pop();
                    let start = self.vi(&start_val) as usize;
                    let end = self.vi(&end_val) as usize;
                    if let V::Array(a) = &arr_val {
                        let borrowed = a.borrow();
                        let slice: Vec<V> = borrowed[start..end.min(borrowed.len())].to_vec();
                        self.push(V::Array(std::rc::Rc::new(std::cell::RefCell::new(slice))));
                    } else {
                        self.push(V::Array(std::rc::Rc::new(std::cell::RefCell::new(
                            Vec::new(),
                        ))));
                    }
                }

                // Conversion (0xB0-0xB3)
                0xB0 => {
                    let v = self.pi();
                    self.push(V::F64(v as f64));
                }
                0xB1 => {
                    let v = self.pf();
                    self.push(V::I64(v as i64));
                }
                0xB2 => {
                    let p = self.pop();
                    self.push(V::Bool(self.truthy(&p)));
                }
                0xB3 => {
                    let v = self.pop();
                    self.push(V::I64(if self.truthy(&v) { 1 } else { 0 }));
                }

                // Heap (0x80-0x8C)
                0x88 => {
                    // Alloc: size from operand (u32), push heap pointer
                    let size = self.ru32() as usize;
                    const MAX_HEAP_ALLOC: usize = 64 * 1024 * 1024; // 64MB max per alloc
                    if size > MAX_HEAP_ALLOC {
                        return Err(format!(
                            "heap allocation too large: {} bytes (max {})",
                            size, MAX_HEAP_ALLOC
                        ));
                    }
                    let ptr = if let Some(idx) = self.heap_free.pop() {
                        self.heap[idx] = Some(vec![0u8; size]);
                        idx
                    } else {
                        let idx = self.heap.len();
                        self.heap.push(Some(vec![0u8; size]));
                        idx
                    };
                    self.push(V::I64(ptr as i64));
                }
                0x89 => {
                    // Realloc: ptr on stack, new size from operand (u32)
                    let new_size = self.ru32() as usize;
                    let v = self.pop();
                    if let V::I64(ptr) = v {
                        let idx = ptr as usize;
                        if idx < self.heap.len() {
                            if let Some(ref mut data) = self.heap[idx] {
                                const MAX_HEAP_ALLOC: usize = 64 * 1024 * 1024;
                                if new_size > MAX_HEAP_ALLOC {
                                    return Err(format!(
                                        "heap realloc too large: {} bytes",
                                        new_size
                                    ));
                                }
                                data.resize(new_size, 0);
                            }
                        }
                    }
                    self.push(v);
                }
                0x8A => {
                    // Free: ptr on stack
                    let v = self.pop();
                    if let V::I64(ptr) = v {
                        let idx = ptr as usize;
                        if idx < self.heap.len() {
                            self.heap[idx] = None;
                            self.heap_free.push(idx);
                        }
                    }
                }
                0x8B => {
                    // Memcpy: dst_ptr, src_ptr, size on stack
                    let size_val = self.pop();
                    let src_val = self.pop();
                    let dst_val = self.pop();
                    if let (V::I64(dst), V::I64(src), V::I64(sz)) = (&dst_val, &src_val, &size_val)
                    {
                        let (di, si, n) = (*dst as usize, *src as usize, *sz as usize);
                        if si < self.heap.len() && di < self.heap.len() {
                            let src_data =
                                self.heap[si].as_ref().map(|d| d[..n.min(d.len())].to_vec());
                            if let (Some(ref s), Some(ref mut d)) =
                                (src_data, self.heap[di].as_mut())
                            {
                                let copy_len = n.min(s.len()).min(d.len());
                                d[..copy_len].copy_from_slice(&s[..copy_len]);
                            }
                        }
                    }
                }
                0x8C => {
                    // Memset: ptr, value, size on stack
                    let size_val = self.pop();
                    let fill_val = self.pop();
                    let ptr_val = self.pop();
                    if let (V::I64(ptr), V::I64(val), V::I64(sz)) = (&ptr_val, &fill_val, &size_val)
                    {
                        let idx = *ptr as usize;
                        if idx < self.heap.len() {
                            if let Some(ref mut data) = self.heap[idx] {
                                let n = (*sz as usize).min(data.len());
                                data[..n].fill(*val as u8);
                            }
                        }
                    }
                }

                // Control (0xF0-0xFF)
                0xF0 => {
                    // Syscall: u16 constant index = native function name (e.g., "core::println")
                    let name_idx = self.ru16();
                    let fname = match self.consts.get(name_idx as usize) {
                        Some(V::Str(s)) => s.clone(),
                        _ => return Err(format!("Invalid syscall index {}", name_idx)),
                    };
                    // Extract function name (strip module prefix)
                    let func_name = fname.split("::").last().unwrap_or(&fname);
                    match func_name {
                        "print" => {
                            let v = self.pop();
                            self.pv(&v);
                            self.push(V::Null);
                        }
                        "println" => {
                            let v = self.pop();
                            self.pv(&v);
                            println!();
                            self.push(V::Null);
                        }
                        "format" | "to_string" | "stringify" | "str" => {
                            let v = self.pop();
                            self.push(V::Str(self.vs(&v)));
                        }
                        "type_of" => {
                            let v = self.pop();
                            self.push(V::Str(self.vt(&v).into()));
                        }
                        "len" => {
                            let v = self.pop();
                            let l = match &v {
                                V::Str(s) => s.len() as i64,
                                _ => 0,
                            };
                            self.push(V::I64(l));
                        }
                        "assert" => {
                            let v = self.pop();
                            if !self.truthy(&v) {
                                return Err("assertion failed".into());
                            }
                            self.push(V::Null);
                        }
                        "int" | "to_int" => {
                            let v = self.pop();
                            self.push(V::I64(self.vi(&v)));
                        }
                        "float" | "to_float" => {
                            let v = self.pop();
                            self.push(V::F64(self.vf(&v)));
                        }
                        "sqrt" => {
                            let v = self.pop();
                            self.push(V::F64(self.vf(&v).sqrt()));
                        }
                        "abs" => {
                            let v = self.pop();
                            self.push(V::I64(self.vi(&v).abs()));
                        }
                        // File I/O stubs (O-105) — not yet implemented
                        "open" => {
                            let _path = self.pop(); // pop path argument
                            return Err("syscall 'open' is not implemented (O-105: file I/O not yet available)".into());
                        }
                        "read_file" => {
                            let _path = self.pop();
                            return Err("syscall 'read_file' is not implemented (O-105: file I/O not yet available)".into());
                        }
                        "write_file" => {
                            let _content = self.pop();
                            let _path = self.pop();
                            return Err("syscall 'write_file' is not implemented (O-105: file I/O not yet available)".into());
                        }
                        "close" => {
                            let _fd = self.pop();
                            return Err("syscall 'close' is not implemented (O-105: file I/O not yet available)".into());
                        }
                        // Command-line args (O-106)
                        "args" | "argv" => {
                            let arr: Vec<V> = self.args.iter().map(|a| V::Str(a.clone())).collect();
                            self.push(V::Array(std::rc::Rc::new(std::cell::RefCell::new(arr))));
                        }
                        "arg_count" | "argc" => {
                            self.push(V::I64(self.args.len() as i64));
                        }
                        "arg_at" => {
                            let idx = self.pop();
                            let i = self.vi(&idx) as usize;
                            let val = if i < self.args.len() {
                                V::Str(self.args[i].clone())
                            } else {
                                V::Null
                            };
                            self.push(val);
                        }
                        _ => {
                            // Unknown native — just leave args on stack and push null
                            self.push(V::Null);
                        }
                    }
                }
                0xFE => return Ok(()),
                0xFF => {
                    let m = self.pop();
                    return Err(format!("panic: {}", self.vs(&m)));
                }

                0xF1..=0xF3 => {} // Debug, Trace, Assert — skip
                _ => { /* unknown — skip */ }
            }
        }
    }

    fn push(&mut self, v: V) {
        self.stack.push(v);
    }
    fn pop(&mut self) -> V {
        self.stack.pop().unwrap_or(V::Null)
    }
    fn pk(&self, o: usize) -> V {
        self.stack
            .get(self.stack.len() - 1 - o)
            .cloned()
            .unwrap_or(V::Null)
    }
    fn pi(&mut self) -> i64 {
        let v = self.pop();
        self.vi(&v)
    }
    fn pf(&mut self) -> f64 {
        let v = self.pop();
        self.vf(&v)
    }
    fn tr(&mut self) -> bool {
        let v = self.pop();
        self.truthy(&v)
    }
    fn vi(&self, v: &V) -> i64 {
        match v {
            V::I64(i) => *i,
            V::F64(f) => *f as i64,
            V::Bool(b) => *b as i64,
            _ => 0,
        }
    }
    fn vf(&self, v: &V) -> f64 {
        match v {
            V::F64(f) => *f,
            V::I64(i) => *i as f64,
            _ => 0.0,
        }
    }
    fn truthy(&self, v: &V) -> bool {
        match v {
            V::Null => false,
            V::Bool(b) => *b,
            V::I64(i) => *i != 0,
            V::F64(f) => *f != 0.0,
            V::Str(s) => !s.is_empty(),
            V::Array(arr) => !arr.borrow().is_empty(),
            V::Object(_) => true,
        }
    }
    fn vs(&self, v: &V) -> String {
        match v {
            V::Null => "null".into(),
            V::Bool(b) => b.to_string(),
            V::I64(i) => i.to_string(),
            V::F64(f) => f.to_string(),
            V::Str(s) => s.clone(),
            V::Array(arr) => {
                let items: Vec<String> = arr.borrow().iter().map(|v| self.vs(v)).collect();
                format!("[{}]", items.join(", "))
            }
            V::Object(obj) => {
                let entries: Vec<String> = obj
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.vs(v)))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
        }
    }
    fn vt(&self, v: &V) -> &'static str {
        match v {
            V::Null => "null",
            V::Bool(_) => "bool",
            V::I64(_) => "int",
            V::F64(_) => "float",
            V::Str(_) => "string",
            V::Array(_) => "array",
            V::Object(_) => "object",
        }
    }
    fn pv(&self, v: &V) {
        print!("{}", self.vs(v));
    }
    fn eq(&self, a: &V, b: &V) -> bool {
        match (a, b) {
            (V::I64(x), V::I64(y)) => x == y,
            (V::F64(x), V::F64(y)) => (x - y).abs() < f64::EPSILON,
            (V::Str(x), V::Str(y)) => x == y,
            (V::Bool(x), V::Bool(y)) => x == y,
            (V::Null, V::Null) => true,
            _ => false,
        }
    }
    fn to_f64(&self, v: &V) -> f64 {
        match v {
            V::I64(i) => *i as f64,
            V::F64(f) => *f,
            V::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
    fn vcmp_lt(&self, a: &V, b: &V) -> bool {
        match (a, b) {
            (V::I64(x), V::I64(y)) => x < y,
            _ => self.to_f64(a) < self.to_f64(b),
        }
    }
    fn vcmp_le(&self, a: &V, b: &V) -> bool {
        match (a, b) {
            (V::I64(x), V::I64(y)) => x <= y,
            _ => self.to_f64(a) <= self.to_f64(b),
        }
    }
    fn vcmp_gt(&self, a: &V, b: &V) -> bool {
        match (a, b) {
            (V::I64(x), V::I64(y)) => x > y,
            _ => self.to_f64(a) > self.to_f64(b),
        }
    }
    fn vcmp_ge(&self, a: &V, b: &V) -> bool {
        match (a, b) {
            (V::I64(x), V::I64(y)) => x >= y,
            _ => self.to_f64(a) >= self.to_f64(b),
        }
    }
    fn ri64(&mut self) -> i64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(&self.code[self.pc..self.pc + 8]);
        self.pc += 8;
        i64::from_le_bytes(b)
    }
    fn rf64(&mut self) -> f64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(&self.code[self.pc..self.pc + 8]);
        self.pc += 8;
        f64::from_le_bytes(b)
    }
    fn ru32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        b.copy_from_slice(&self.code[self.pc..self.pc + 4]);
        self.pc += 4;
        u32::from_le_bytes(b)
    }
    fn ru16(&mut self) -> u16 {
        let mut b = [0u8; 2];
        b.copy_from_slice(&self.code[self.pc..self.pc + 2]);
        self.pc += 2;
        u16::from_le_bytes(b)
    }
    fn ri32(&mut self) -> i32 {
        self.ru32() as i32
    }
}
