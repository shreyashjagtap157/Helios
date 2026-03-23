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
    let mut vm = VM::new(&bytes);
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
}

impl VM {
    fn new(data: &[u8]) -> Self {
        let mut vm = VM {
            code: Vec::new(),
            consts: Vec::new(),
            funcs: Vec::new(),
            entry: 0,
            stack: Vec::with_capacity(4096),
            frames: Vec::with_capacity(256),
            pc: 0,
            global_locals: std::array::from_fn(|_| V::Null),
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
        self.entry = Self::u64(data, 12) as usize;
        let co = Self::u64(data, 20) as usize;
        let codelen = Self::u64(data, 44) as usize;
        let codeoff = Self::u64(data, 36) as usize;

        // Constants
        if co > 0 && co < data.len() {
            let n = Self::u32(data, co) as usize;
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
        eprintln!(
            "[ovm] Loaded {} constants, {} functions, entry={}",
            self.consts.len(),
            self.funcs.len(),
            self.entry
        );
        for (i, (n, o, p)) in self.funcs.iter().enumerate() {
            eprintln!("[ovm]   func[{}] '{}' at offset {} params={}", i, n, o, p);
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
        eprintln!(
            "[ovm] Entering '{}' at offset {}",
            self.funcs[main_idx].0, self.pc
        );
        let end = (self.pc + 40).min(self.code.len());
        eprint!("[ovm] main bytecode: ");
        for i in self.pc..end {
            eprint!("{:02X} ", self.code[i]);
        }
        eprintln!();
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
            if steps <= 30 {
                eprintln!(
                    "[vm] step {} pc={} op=0x{:02X} sp={} frames={}",
                    steps,
                    self.pc - 1,
                    op,
                    self.stack.len(),
                    self.frames.len()
                );
            }
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
                        _ => {
                            self.push(V::I64(self.vi(&left) + self.vi(&right)));
                        }
                    }
                }
                0x11 => {
                    let l = self.pi();
                    let r = self.pi();
                    self.push(V::I64(l - r));
                }
                0x12 => {
                    let l = self.pi();
                    let r = self.pi();
                    self.push(V::I64(l * r));
                }
                0x13 => {
                    let l = self.pi();
                    let r = self.pi();
                    if r == 0 {
                        return Err("div0".into());
                    }
                    self.push(V::I64(l / r));
                }
                0x14 => {
                    let l = self.pi();
                    let r = self.pi();
                    if r == 0 {
                        return Err("mod0".into());
                    }
                    self.push(V::I64(l % r));
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
                    let v = self.pi();
                    self.push(V::I64(!v));
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
                        _ => self.push(V::Null),
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
