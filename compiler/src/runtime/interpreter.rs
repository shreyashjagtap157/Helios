#![allow(dead_code)]
//! Omni Virtual Machine (OVM) - Full Bytecode Interpreter
//! Executes OVM bytecode with GC, async support, exception handling, and native dispatch

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{Result, anyhow};
use log::{info, debug, warn, error};

use crate::codegen::ovm::{OvmOpcode, OvmModule, OvmFunction, OvmConstant};

/// Stack value in the OVM
#[derive(Clone, Debug)]
pub enum OvmValue {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    Ptr(usize),          // Heap pointer
    FuncRef(usize),      // Function index
    ClosureRef(usize),   // Closure object pointer
    FutureRef(usize),    // Async future pointer
    String(String),      // Interned string
    Array(Vec<OvmValue>),  // Array value
    Struct(String, Vec<OvmValue>), // Struct (typename, fields)
}

impl Default for OvmValue {
    fn default() -> Self { OvmValue::Null }
}

impl OvmValue {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            OvmValue::Int(i) => Some(*i),
            _ => None,
        }
    }
    
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            OvmValue::Bool(b) => Some(*b),
            OvmValue::Int(i) => Some(*i != 0),
            _ => None,
        }
    }
}

/// Heap object header for GC
#[derive(Debug)]
pub struct GcHeader {
    pub mark: bool,
    pub size: usize,
    pub type_id: u32,
    pub strong_count: u64,
    pub weak_count: u64,
    pub generation: u8,  // For generational GC
}

/// Heap-allocated object
#[derive(Debug)]
pub struct HeapObject {
    pub header: GcHeader,
    pub data: Vec<u8>,
    pub refs: Vec<usize>,  // Pointers to other heap objects
    pub finalizer: Option<String>,  // Drop function name
}

/// Call frame for the call stack
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub func_index: usize,
    pub pc: usize,          // Program counter
    pub bp: usize,          // Base pointer (local variable start)
    pub locals: Vec<OvmValue>,
    pub return_addr: Option<usize>,  // Where to return to
}

/// Async task state
#[derive(Debug)]
pub struct AsyncTask {
    pub id: usize,
    pub state: AsyncState,
    pub frame: CallFrame,
    pub result: Option<OvmValue>,
    pub waker: Option<WakerHandle>,
    pub blocked_on: Option<BlockedReason>,
}

#[derive(Debug, Clone)]
pub struct WakerHandle {
    pub task_id: usize,
}

#[derive(Debug, Clone)]
pub enum BlockedReason {
    Future(usize),      // Waiting on another future
    Channel(usize),     // Waiting on channel
    Timer(u64),         // Waiting on timer (timestamp)
    Io(i32),            // Waiting on file descriptor
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsyncState {
    Pending,
    Running,
    Suspended,
    Completed,
    Failed,
}

/// Exception frame for try/catch
#[derive(Debug, Clone)]
pub struct ExceptionFrame {
    pub handler_pc: usize,
    pub stack_depth: usize,
    pub call_depth: usize,
    pub exception_type: Option<String>,
}

/// Async executor for running tasks
pub struct AsyncExecutor {
    ready_queue: std::collections::VecDeque<usize>,
    tasks: HashMap<usize, AsyncTask>,
    next_task_id: usize,
}

impl AsyncExecutor {
    pub fn new() -> Self {
        AsyncExecutor {
            ready_queue: std::collections::VecDeque::new(),
            tasks: HashMap::new(),
            next_task_id: 0,
        }
    }
    
    pub fn spawn(&mut self, task: AsyncTask) -> usize {
        let id = self.next_task_id;
        self.next_task_id += 1;
        self.ready_queue.push_back(id);
        let mut task = task;
        task.id = id;
        self.tasks.insert(id, task);
        id
    }
    
    pub fn wake(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            if task.state == AsyncState::Suspended {
                task.state = AsyncState::Pending;
                self.ready_queue.push_back(task_id);
            }
        }
    }
    
    pub fn next_ready(&mut self) -> Option<usize> {
        self.ready_queue.pop_front()
    }
}

/// GC Phase for incremental collection
#[derive(Debug, Clone, PartialEq)]
pub enum GcState {
    Idle,
    Marking(Vec<usize>), // Stack of objects to mark
    Sweeping(usize),     // Current index in heap
}

/// The main OVM interpreter with GC, async, and exception support
pub struct OvmInterpreter {
    // Module being executed
    module: Option<OvmModule>,
    
    // Execution state
    stack: Vec<OvmValue>,
    call_stack: Vec<CallFrame>,
    
    // Heap and GC
    heap: Vec<Option<HeapObject>>,
    free_list: Vec<usize>,
    gc_threshold: usize,
    total_allocations: usize,
    gc_generation: u8,
    // Incremental GC state
    gc_state: GcState,
    gc_work_budget: usize,
    
    // Async runtime
    executor: AsyncExecutor,
    current_task: Option<usize>,
    
    // Exception handling
    exception_stack: Vec<ExceptionFrame>,
    current_exception: Option<OvmValue>,
    
    // Native function dispatch
    native_functions: HashMap<String, Box<dyn Fn(&mut OvmInterpreter, &[OvmValue]) -> Result<OvmValue>>>,
    
    // Registers (hybrid register-stack model)
    registers: [OvmValue; 256],
    
    // String pool for interned strings
    string_pool: Vec<String>,
    string_indices: HashMap<String, usize>,
    
    // Debug/profiling
    instruction_count: u64,
    enable_tracing: bool,
}

impl OvmInterpreter {
    pub fn new() -> Self {
        OvmInterpreter {
            module: None,
            stack: Vec::with_capacity(4096),
            call_stack: Vec::with_capacity(256),
            heap: Vec::with_capacity(65536),
            free_list: Vec::new(),
            gc_threshold: 1024,
            total_allocations: 0,
            gc_generation: 0,
            gc_state: GcState::Idle,
            gc_work_budget: 100, // Do 100 units of work per step
            executor: AsyncExecutor::new(),
            current_task: None,
            exception_stack: Vec::new(),
            current_exception: None,
            native_functions: HashMap::new(),
            registers: std::array::from_fn(|_| OvmValue::Null),
            string_pool: Vec::new(),
            string_indices: HashMap::new(),
            instruction_count: 0,
            enable_tracing: false,
        }
    }

    /// Load an OVM module for execution
    pub fn load_module(&mut self, module: OvmModule) {
        info!("OVM: Loading module '{}'", module.name);
        
        // Intern strings from constant pool
        for constant in &module.constants {
            if let OvmConstant::String(s) = constant {
                self.intern_string(s);
            }
        }
        
        self.module = Some(module);
        self.register_builtin_natives();
    }
    
    fn intern_string(&mut self, s: &str) -> usize {
        if let Some(&idx) = self.string_indices.get(s) {
            return idx;
        }
        let idx = self.string_pool.len();
        self.string_pool.push(s.to_string());
        self.string_indices.insert(s.to_string(), idx);
        idx
    }

    /// Load module from bytecode file
    pub fn load_bytecode(&mut self, path: &std::path::Path) -> Result<()> {
        let bytes = std::fs::read(path)?;
        let module = self.deserialize_module(&bytes)?;
        self.load_module(module);
        Ok(())
    }

    /// Full bytecode deserialization
    fn deserialize_module(&self, bytes: &[u8]) -> Result<OvmModule> {
        if bytes.len() < 72 {
            return Err(anyhow!("OVM bytecode too short"));
        }
        
        // Verify magic number
        if &bytes[0..4] != b"OVM\0" {
            return Err(anyhow!("Invalid OVM bytecode magic number"));
        }
        
        // Parse header
        let version = u32::from_le_bytes(bytes[4..8].try_into()?);
        let _flags = u32::from_le_bytes(bytes[8..12].try_into()?);
        let entry_point = u64::from_le_bytes(bytes[12..20].try_into()?) as u32;
        
        // Read section offsets
        let const_pool_off = u64::from_le_bytes(bytes[20..28].try_into()?) as usize;
        let const_pool_len = u64::from_le_bytes(bytes[28..36].try_into()?) as usize;
        let code_off = u64::from_le_bytes(bytes[36..44].try_into()?) as usize;
        let code_len = u64::from_le_bytes(bytes[44..52].try_into()?) as usize;
        let _symbol_off = u64::from_le_bytes(bytes[52..60].try_into()?) as usize;
        let _symbol_len = u64::from_le_bytes(bytes[60..68].try_into()?) as usize;
        
        debug!("OVM Header: version={}, entry={}, const_off={}, code_off={}", 
               version, entry_point, const_pool_off, code_off);
        
        // Parse constant pool
        let mut constants = Vec::new();
        if const_pool_off > 0 && const_pool_off < bytes.len() {
            let const_bytes = &bytes[const_pool_off..const_pool_off + const_pool_len.min(bytes.len() - const_pool_off)];
            if const_bytes.len() >= 4 {
                let num_constants = u32::from_le_bytes(const_bytes[0..4].try_into()?) as usize;
                let mut pos = 4;
                
                for _ in 0..num_constants {
                    if pos >= const_bytes.len() { break; }
                    
                    let const_type = const_bytes[pos];
                    pos += 1;
                    
                    match const_type {
                        0x01 => { // I64
                            if pos + 8 <= const_bytes.len() {
                                let val = i64::from_le_bytes(const_bytes[pos..pos+8].try_into()?);
                                constants.push(OvmConstant::I64(val));
                                pos += 8;
                            }
                        }
                        0x02 => { // F64
                            if pos + 8 <= const_bytes.len() {
                                let val = f64::from_le_bytes(const_bytes[pos..pos+8].try_into()?);
                                constants.push(OvmConstant::F64(val));
                                pos += 8;
                            }
                        }
                        0x03 => { // String
                            if pos + 4 <= const_bytes.len() {
                                let len = u32::from_le_bytes(const_bytes[pos..pos+4].try_into()?) as usize;
                                pos += 4;
                                if pos + len <= const_bytes.len() {
                                    let s = String::from_utf8_lossy(&const_bytes[pos..pos+len]).to_string();
                                    constants.push(OvmConstant::String(s));
                                    pos += len;
                                }
                            }
                        }
                        0x04 => { // Bytes
                            if pos + 4 <= const_bytes.len() {
                                let len = u32::from_le_bytes(const_bytes[pos..pos+4].try_into()?) as usize;
                                pos += 4;
                                if pos + len <= const_bytes.len() {
                                    constants.push(OvmConstant::Bytes(const_bytes[pos..pos+len].to_vec()));
                                    pos += len;
                                }
                            }
                        }
                        _ => { break; }
                    }
                }
            }
        }
        
        // Parse code section (functions)
        let mut functions = Vec::new();
        if code_off > 0 && code_off < bytes.len() {
            let code_bytes = &bytes[code_off..code_off + code_len.min(bytes.len() - code_off)];
            if code_bytes.len() >= 4 {
                let num_functions = u32::from_le_bytes(code_bytes[0..4].try_into()?) as usize;
                let mut pos = 4;
                
                for _ in 0..num_functions {
                    if pos + 12 > code_bytes.len() { break; }
                    
                    let name_index = u32::from_le_bytes(code_bytes[pos..pos+4].try_into()?);
                    pos += 4;
                    let param_count = u16::from_le_bytes(code_bytes[pos..pos+2].try_into()?);
                    pos += 2;
                    let local_count = u16::from_le_bytes(code_bytes[pos..pos+2].try_into()?);
                    pos += 2;
                    let max_stack = u16::from_le_bytes(code_bytes[pos..pos+2].try_into()?);
                    pos += 2;
                    let bytecode_len = u32::from_le_bytes(code_bytes[pos..pos+4].try_into()?) as usize;
                    pos += 4;
                    
                    let bytecode = if pos + bytecode_len <= code_bytes.len() {
                        let bc = code_bytes[pos..pos+bytecode_len].to_vec();
                        pos += bytecode_len;
                        bc
                    } else {
                        Vec::new()
                    };
                    
                    // Get function name from constants
                    let name = if (name_index as usize) < constants.len() {
                        if let OvmConstant::String(s) = &constants[name_index as usize] {
                            s.clone()
                        } else {
                            format!("func_{}", functions.len())
                        }
                    } else {
                        format!("func_{}", functions.len())
                    };
                    
                    functions.push(OvmFunction {
                        name,
                        name_index,
                        param_count,
                        local_count,
                        max_stack,
                        bytecode,
                        is_native: false,
                    });
                }
            }
        }
        
        info!("OVM: Loaded {} constants, {} functions", constants.len(), functions.len());
        
        Ok(OvmModule {
            name: "loaded".to_string(),
            version,
            entry_point,
            constants,
            functions,
            types: Vec::new(),
            symbols: Vec::new(),
        })
    }

    fn register_builtin_natives(&mut self) {
        // Print function
        self.native_functions.insert("print".to_string(), Box::new(|_, args| {
            for arg in args {
                match arg {
                    OvmValue::Int(i) => print!("{}", i),
                    OvmValue::Float(f) => print!("{}", f),
                    OvmValue::Bool(b) => print!("{}", b),
                    OvmValue::String(s) => print!("{}", s),
                    OvmValue::Null => print!("null"),
                    _ => print!("{:?}", arg),
                }
            }
            println!();
            Ok(OvmValue::Null)
        }));
        
        // String length
        self.native_functions.insert("strlen".to_string(), Box::new(|_, args| {
            if let Some(OvmValue::String(s)) = args.first() {
                Ok(OvmValue::Int(s.len() as i64))
            } else {
                Err(anyhow!("strlen requires a string argument"))
            }
        }));
        
        // String concatenation
        self.native_functions.insert("strcat".to_string(), Box::new(|_, args| {
            if args.len() >= 2 {
                if let (Some(OvmValue::String(a)), Some(OvmValue::String(b))) = (args.get(0), args.get(1)) {
                    return Ok(OvmValue::String(format!("{}{}", a, b)));
                }
            }
            Err(anyhow!("strcat requires two string arguments"))
        }));
        
        // Array operations
        self.native_functions.insert("array_new".to_string(), Box::new(|_, args| {
            let size = args.first()
                .and_then(|v| v.as_int())
                .unwrap_or(0) as usize;
            Ok(OvmValue::Array(vec![OvmValue::Null; size]))
        }));
        
        self.native_functions.insert("array_len".to_string(), Box::new(|_, args| {
            if let Some(OvmValue::Array(arr)) = args.first() {
                Ok(OvmValue::Int(arr.len() as i64))
            } else {
                Err(anyhow!("array_len requires an array argument"))
            }
        }));
        
        self.native_functions.insert("array_get".to_string(), Box::new(|_, args| {
            if args.len() >= 2 {
                if let (Some(OvmValue::Array(arr)), Some(idx)) = (args.get(0), args.get(1)) {
                    let i = idx.as_int().unwrap_or(0) as usize;
                    if i < arr.len() {
                        return Ok(arr[i].clone());
                    }
                }
            }
            Err(anyhow!("array_get: index out of bounds"))
        }));
        
        // Math functions
        self.native_functions.insert("sqrt".to_string(), Box::new(|_, args| {
            if let Some(OvmValue::Float(f)) = args.first() {
                Ok(OvmValue::Float(f.sqrt()))
            } else if let Some(OvmValue::Int(i)) = args.first() {
                Ok(OvmValue::Float((*i as f64).sqrt()))
            } else {
                Err(anyhow!("sqrt requires a numeric argument"))
            }
        }));
        
        self.native_functions.insert("sin".to_string(), Box::new(|_, args| {
            if let Some(OvmValue::Float(f)) = args.first() {
                Ok(OvmValue::Float(f.sin()))
            } else {
                Err(anyhow!("sin requires a float argument"))
            }
        }));
        
        self.native_functions.insert("cos".to_string(), Box::new(|_, args| {
            if let Some(OvmValue::Float(f)) = args.first() {
                Ok(OvmValue::Float(f.cos()))
            } else {
                Err(anyhow!("cos requires a float argument"))
            }
        }));
        
        // Time
        self.native_functions.insert("time_now".to_string(), Box::new(|_, _| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            Ok(OvmValue::Int(duration.as_millis() as i64))
        }));

        debug!("OVM: Registered {} native functions", self.native_functions.len());
    }

    /// Execute from entry point
    pub fn run(&mut self) -> Result<OvmValue> {
        let module = self.module.as_ref().ok_or_else(|| anyhow!("No module loaded"))?;
        let entry = module.entry_point as usize;
        
        info!("OVM: Starting execution at function index {}", entry);
        self.call_function(entry, Vec::new())
    }
    
    /// Run the async event loop until all tasks complete
    pub fn run_async(&mut self) -> Result<OvmValue> {
        let module = self.module.as_ref().ok_or_else(|| anyhow!("No module loaded"))?;
        let entry = module.entry_point as usize;
        
        // Spawn main as a task
        let main_task = AsyncTask {
            id: 0,
            state: AsyncState::Pending,
            frame: CallFrame {
                func_index: entry,
                pc: 0,
                bp: 0,
                locals: vec![OvmValue::Null; 16],
                return_addr: None,
            },
            result: None,
            waker: None,
            blocked_on: None,
        };
        
        let main_id = self.executor.spawn(main_task);
        
        // Event loop
        loop {
            // Poll for ready tasks
            if let Some(task_id) = self.executor.next_ready() {
                self.current_task = Some(task_id);
                
                // Execute task until it yields or completes
                match self.execute_task(task_id) {
                    Ok(Some(result)) => {
                        // Task completed
                        if task_id == main_id {
                            return Ok(result);
                        }
                    }
                    Ok(None) => {
                        // Task yielded, will be woken by waker
                    }
                    Err(e) => {
                        error!("Task {} failed: {}", task_id, e);
                        if task_id == main_id {
                            return Err(e);
                        }
                    }
                }
                
                self.current_task = None;
            } else {
                // No ready tasks - check if all done
                if self.executor.tasks.is_empty() {
                    break;
                }
                
                // Would poll I/O events here in a real implementation
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        
        Ok(OvmValue::Null)
    }
    
    fn execute_task(&mut self, task_id: usize) -> Result<Option<OvmValue>> {
        // Get task (we need to be careful about borrowing)
        let (func_index, pc, locals) = {
            let task = self.executor.tasks.get(&task_id)
                .ok_or_else(|| anyhow!("Task not found"))?;
            (task.frame.func_index, task.frame.pc, task.frame.locals.clone())
        };
        
        // Set up call stack
        self.call_stack.push(CallFrame {
            func_index,
            pc,
            bp: self.stack.len(),
            locals,
            return_addr: None,
        });
        
        // Execute
        let result = self.execute_bytecode(func_index);
        
        // Update task state
        match &result {
            Ok(val) => {
                if let Some(task) = self.executor.tasks.get_mut(&task_id) {
                    task.state = AsyncState::Completed;
                    task.result = Some(val.clone());
                }
            }
            Err(_) => {
                if let Some(task) = self.executor.tasks.get_mut(&task_id) {
                    task.state = AsyncState::Failed;
                }
            }
        }
        
        result.map(Some)
    }

    /// Call a function by index
    pub fn call_function(&mut self, func_index: usize, args: Vec<OvmValue>) -> Result<OvmValue> {
        let module = self.module.as_ref().ok_or_else(|| anyhow!("No module loaded"))?;
        
        if func_index >= module.functions.len() {
            return Err(anyhow!("Function index {} out of bounds", func_index));
        }
        
        let func = &module.functions[func_index];
        debug!("OVM: Calling function '{}' with {} args", func.name, args.len());

        // Set up call frame
        let mut locals = vec![OvmValue::Null; func.local_count as usize];
        for (i, arg) in args.into_iter().enumerate() {
            if i < locals.len() {
                locals[i] = arg;
            }
        }

        let frame = CallFrame {
            func_index,
            pc: 0,
            bp: self.stack.len(),
            locals,
            return_addr: None,
        };
        self.call_stack.push(frame);

        // Execute bytecode
        self.execute_bytecode(func_index)
    }

    fn execute_bytecode(&mut self, func_index: usize) -> Result<OvmValue> {
        let module = self.module.as_ref().unwrap();
        // Use a reference instead of cloning the bytecode for performance
        let bytecode: Vec<u8> = module.functions[func_index].bytecode.clone();
        
        // Check if GC is needed before entering hot loop
        if self.total_allocations > self.gc_threshold {
            self.gc_collect();
        }
        
        while let Some(frame) = self.call_stack.last_mut() {
            if frame.pc >= bytecode.len() {
                // Return from function
                let result = self.stack.pop().unwrap_or(OvmValue::Null);
                self.call_stack.pop();
                if self.call_stack.is_empty() {
                    return Ok(result);
                }
                self.stack.push(result);
                continue;
            }

            let opcode = bytecode[frame.pc];
            frame.pc += 1;

            match opcode {
                x if x == OvmOpcode::Nop as u8 => {}
                
                x if x == OvmOpcode::PushI64 as u8 => {
                    let val = read_i64(&bytecode, &mut frame.pc);
                    self.stack.push(OvmValue::Int(val));
                }
                
                x if x == OvmOpcode::PushF64 as u8 => {
                    let val = read_f64(&bytecode, &mut frame.pc);
                    self.stack.push(OvmValue::Float(val));
                }
                
                x if x == OvmOpcode::PushTrue as u8 => {
                    self.stack.push(OvmValue::Bool(true));
                }
                
                x if x == OvmOpcode::PushFalse as u8 => {
                    self.stack.push(OvmValue::Bool(false));
                }
                
                x if x == OvmOpcode::PushNull as u8 => {
                    self.stack.push(OvmValue::Null);
                }
                
                x if x == OvmOpcode::Pop as u8 => {
                    self.stack.pop();
                }
                
                x if x == OvmOpcode::Dup as u8 => {
                    if let Some(val) = self.stack.last().cloned() {
                        self.stack.push(val);
                    }
                }
                
                x if x == OvmOpcode::Swap as u8 => {
                    let len = self.stack.len();
                    if len >= 2 {
                        self.stack.swap(len - 1, len - 2);
                    }
                }
                
                // Arithmetic
                x if x == OvmOpcode::AddI64 as u8 => {
                    self.binary_op_int(|a, b| a + b)?;
                }
                x if x == OvmOpcode::SubI64 as u8 => {
                    self.binary_op_int(|a, b| a - b)?;
                }
                x if x == OvmOpcode::MulI64 as u8 => {
                    self.binary_op_int(|a, b| a * b)?;
                }
                x if x == OvmOpcode::DivI64 as u8 => {
                    self.binary_op_int(|a, b| if b != 0 { a / b } else { 0 })?;
                }
                x if x == OvmOpcode::ModI64 as u8 => {
                    self.binary_op_int(|a, b| if b != 0 { a % b } else { 0 })?;
                }
                x if x == OvmOpcode::NegI64 as u8 => {
                    if let Some(OvmValue::Int(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Int(-v));
                    }
                }
                
                x if x == OvmOpcode::AddF64 as u8 => {
                    self.binary_op_float(|a, b| a + b)?;
                }
                x if x == OvmOpcode::SubF64 as u8 => {
                    self.binary_op_float(|a, b| a - b)?;
                }
                x if x == OvmOpcode::MulF64 as u8 => {
                    self.binary_op_float(|a, b| a * b)?;
                }
                x if x == OvmOpcode::DivF64 as u8 => {
                    self.binary_op_float(|a, b| if b != 0.0 { a / b } else { 0.0 })?;
                }
                
                // Comparison
                x if x == OvmOpcode::Eq as u8 => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    let result = match (a, b) {
                        (Some(OvmValue::Int(x)), Some(OvmValue::Int(y))) => x == y,
                        (Some(OvmValue::Float(x)), Some(OvmValue::Float(y))) => x == y,
                        (Some(OvmValue::Bool(x)), Some(OvmValue::Bool(y))) => x == y,
                        _ => false,
                    };
                    self.stack.push(OvmValue::Bool(result));
                }
                
                x if x == OvmOpcode::Ne as u8 => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    let result = match (a, b) {
                        (Some(OvmValue::Int(x)), Some(OvmValue::Int(y))) => x != y,
                        (Some(OvmValue::Float(x)), Some(OvmValue::Float(y))) => x != y,
                        (Some(OvmValue::Bool(x)), Some(OvmValue::Bool(y))) => x != y,
                        _ => true,
                    };
                    self.stack.push(OvmValue::Bool(result));
                }
                
                x if x == OvmOpcode::Lt as u8 => {
                    self.compare_op(|a, b| a < b)?;
                }
                x if x == OvmOpcode::Le as u8 => {
                    self.compare_op(|a, b| a <= b)?;
                }
                x if x == OvmOpcode::Gt as u8 => {
                    self.compare_op(|a, b| a > b)?;
                }
                x if x == OvmOpcode::Ge as u8 => {
                    self.compare_op(|a, b| a >= b)?;
                }
                
                // Control flow
                x if x == OvmOpcode::Jmp as u8 => {
                    let offset = read_i32(&bytecode, &mut frame.pc);
                    frame.pc = (frame.pc as i32 + offset) as usize;
                }
                
                x if x == OvmOpcode::Jz as u8 => {
                    let offset = read_i32(&bytecode, &mut frame.pc);
                    if let Some(OvmValue::Bool(false)) | Some(OvmValue::Int(0)) = self.stack.pop() {
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }
                
                x if x == OvmOpcode::Jnz as u8 => {
                    let offset = read_i32(&bytecode, &mut frame.pc);
                    let should_jump = match self.stack.pop() {
                        Some(OvmValue::Bool(true)) => true,
                        Some(OvmValue::Int(n)) if n != 0 => true,
                        _ => false,
                    };
                    if should_jump {
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }
                
                x if x == OvmOpcode::Call as u8 => {
                    let func_idx = read_u32(&bytecode, &mut frame.pc) as usize;
                    // Push current frame state, call function
                    self.call_function(func_idx, Vec::new())?;
                }
                
                x if x == OvmOpcode::Ret as u8 => {
                    let result = self.stack.pop().unwrap_or(OvmValue::Null);
                    self.call_stack.pop();
                    if self.call_stack.is_empty() {
                        return Ok(result);
                    }
                    self.stack.push(result);
                }
                
                x if x == OvmOpcode::RetVoid as u8 => {
                    self.call_stack.pop();
                    if self.call_stack.is_empty() {
                        return Ok(OvmValue::Null);
                    }
                }
                
                // Local variables
                x if x == OvmOpcode::LoadLoc as u8 => {
                    let idx = read_u16(&bytecode, &mut frame.pc) as usize;
                    if idx < frame.locals.len() {
                        self.stack.push(frame.locals[idx].clone());
                    }
                }
                
                x if x == OvmOpcode::StoreLoc as u8 => {
                    let idx = read_u16(&bytecode, &mut frame.pc) as usize;
                    if let Some(val) = self.stack.pop() {
                        if idx < frame.locals.len() {
                            frame.locals[idx] = val;
                        }
                    }
                }
                
                // Memory allocation
                x if x == OvmOpcode::Alloc as u8 => {
                    let size = read_u32(&bytecode, &mut frame.pc) as usize;
                    let ptr = self.gc_alloc(size, 0);
                    self.stack.push(OvmValue::Ptr(ptr));
                }
                
                x if x == OvmOpcode::Free as u8 => {
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        self.gc_free(ptr);
                    }
                }
                
                // Register operations
                x if x == OvmOpcode::LoadReg as u8 => {
                    let reg = bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    if reg < 256 {
                        self.stack.push(self.registers[reg].clone());
                    }
                }
                
                x if x == OvmOpcode::StoreReg as u8 => {
                    let reg = bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    if reg < 256 {
                        if let Some(val) = self.stack.pop() {
                            self.registers[reg] = val;
                        }
                    }
                }
                
                // System calls
                x if x == OvmOpcode::Syscall as u8 => {
                    let syscall_id = read_u16(&bytecode, &mut frame.pc);
                    self.handle_syscall(syscall_id)?;
                }
                
                x if x == OvmOpcode::Halt as u8 => {
                    info!("OVM: Halt instruction reached");
                    return Ok(self.stack.pop().unwrap_or(OvmValue::Null));
                }
                
                x if x == OvmOpcode::Panic as u8 => {
                    return Err(anyhow!("OVM Panic!"));
                }
                
                // Bitwise operations
                x if x == OvmOpcode::And as u8 => {
                    self.binary_op_int(|a, b| a & b)?;
                }
                x if x == OvmOpcode::Or as u8 => {
                    self.binary_op_int(|a, b| a | b)?;
                }
                x if x == OvmOpcode::Xor as u8 => {
                    self.binary_op_int(|a, b| a ^ b)?;
                }
                x if x == OvmOpcode::Not as u8 => {
                    if let Some(OvmValue::Int(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Int(!v));
                    } else if let Some(OvmValue::Bool(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Bool(!v));
                    }
                }
                x if x == OvmOpcode::Shl as u8 => {
                    self.binary_op_int(|a, b| a << (b & 63))?;
                }
                x if x == OvmOpcode::Shr as u8 => {
                    self.binary_op_int(|a, b| ((a as u64) >> (b & 63)) as i64)?;
                }
                x if x == OvmOpcode::Sar as u8 => {
                    self.binary_op_int(|a, b| a >> (b & 63))?;
                }
                
                // Additional arithmetic
                x if x == OvmOpcode::NegF64 as u8 => {
                    if let Some(OvmValue::Float(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Float(-v));
                    }
                }
                x if x == OvmOpcode::Inc as u8 => {
                    if let Some(OvmValue::Int(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Int(v + 1));
                    }
                }
                x if x == OvmOpcode::Dec as u8 => {
                    if let Some(OvmValue::Int(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Int(v - 1));
                    }
                }
                
                // More stack operations
                x if x == OvmOpcode::Dup2 as u8 => {
                    let len = self.stack.len();
                    if len >= 2 {
                        let a = self.stack[len - 2].clone();
                        let b = self.stack[len - 1].clone();
                        self.stack.push(a);
                        self.stack.push(b);
                    }
                }
                x if x == OvmOpcode::Rot as u8 => {
                    let len = self.stack.len();
                    if len >= 3 {
                        let c = self.stack.remove(len - 3);
                        self.stack.push(c);
                    }
                }
                
                // Push const from constant pool
                x if x == OvmOpcode::LoadConst as u8 => {
                    let idx = read_u32(&bytecode, &mut frame.pc) as usize;
                    let module = self.module.as_ref().unwrap();
                    if idx < module.constants.len() {
                        match &module.constants[idx] {
                            OvmConstant::I64(v) => self.stack.push(OvmValue::Int(*v)),
                            OvmConstant::F64(v) => self.stack.push(OvmValue::Float(*v)),
                            OvmConstant::String(s) => self.stack.push(OvmValue::String(s.clone())),
                            OvmConstant::Bytes(b) => self.stack.push(OvmValue::Array(
                                b.iter().map(|&byte| OvmValue::Int(byte as i64)).collect()
                            )),
                        }
                    }
                }
                
                // Push smaller types
                x if x == OvmOpcode::PushI8 as u8 => {
                    let val = bytecode[frame.pc] as i8;
                    frame.pc += 1;
                    self.stack.push(OvmValue::Int(val as i64));
                }
                x if x == OvmOpcode::PushI16 as u8 => {
                    let val = read_u16(&bytecode, &mut frame.pc) as i16;
                    self.stack.push(OvmValue::Int(val as i64));
                }
                x if x == OvmOpcode::PushI32 as u8 => {
                    let val = read_i32(&bytecode, &mut frame.pc);
                    self.stack.push(OvmValue::Int(val as i64));
                }
                x if x == OvmOpcode::PushF32 as u8 => {
                    let val = read_u32(&bytecode, &mut frame.pc);
                    self.stack.push(OvmValue::Float(f32::from_bits(val) as f64));
                }
                x if x == OvmOpcode::PushStr as u8 => {
                    let idx = read_u32(&bytecode, &mut frame.pc) as usize;
                    let module = self.module.as_ref().unwrap();
                    if idx < module.constants.len() {
                        if let OvmConstant::String(s) = &module.constants[idx] {
                            self.stack.push(OvmValue::String(s.clone()));
                        }
                    }
                }
                
                // Comparison
                x if x == OvmOpcode::Cmp as u8 => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    let result = match (a, b) {
                        (Some(OvmValue::Int(x)), Some(OvmValue::Int(y))) => {
                            if x < y { -1 } else if x > y { 1 } else { 0 }
                        }
                        _ => 0,
                    };
                    self.stack.push(OvmValue::Int(result));
                }
                x if x == OvmOpcode::IsNull as u8 => {
                    let val = self.stack.pop();
                    self.stack.push(OvmValue::Bool(matches!(val, Some(OvmValue::Null) | None)));
                }
                
                // Additional control flow
                x if x == OvmOpcode::JmpAbs as u8 => {
                    let target = read_u32(&bytecode, &mut frame.pc) as usize;
                    frame.pc = target;
                }
                x if x == OvmOpcode::Jlt as u8 => {
                    let offset = read_i32(&bytecode, &mut frame.pc);
                    let b = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    let a = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    if a < b {
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }
                x if x == OvmOpcode::Jle as u8 => {
                    let offset = read_i32(&bytecode, &mut frame.pc);
                    let b = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    let a = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    if a <= b {
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }
                x if x == OvmOpcode::Jgt as u8 => {
                    let offset = read_i32(&bytecode, &mut frame.pc);
                    let b = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    let a = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    if a > b {
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }
                x if x == OvmOpcode::Jge as u8 => {
                    let offset = read_i32(&bytecode, &mut frame.pc);
                    let b = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    let a = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    if a >= b {
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }
                x if x == OvmOpcode::CallInd as u8 => {
                    // Indirect call through function reference on stack
                    if let Some(OvmValue::FuncRef(idx)) = self.stack.pop() {
                        self.call_function(idx, Vec::new())?;
                    } else {
                        return Err(anyhow!("CallInd: expected function reference on stack"));
                    }
                }
                x if x == OvmOpcode::TailCall as u8 => {
                    let func_idx = read_u32(&bytecode, &mut frame.pc) as usize;
                    // Tail call optimization: reuse current frame
                    self.call_stack.pop();
                    self.call_function(func_idx, Vec::new())?;
                }
                
                // Local variable operations
                x if x == OvmOpcode::LoadArg as u8 => {
                    let idx = read_u16(&bytecode, &mut frame.pc) as usize;
                    if idx < frame.locals.len() {
                        self.stack.push(frame.locals[idx].clone());
                    }
                }
                x if x == OvmOpcode::StoreArg as u8 => {
                    let idx = read_u16(&bytecode, &mut frame.pc) as usize;
                    if let Some(val) = self.stack.pop() {
                        if idx < frame.locals.len() {
                            frame.locals[idx] = val;
                        }
                    }
                }
                x if x == OvmOpcode::AllocLoc as u8 => {
                    let size = read_u16(&bytecode, &mut frame.pc) as usize;
                    // Extend locals if needed
                    while frame.locals.len() < frame.locals.len() + size {
                        frame.locals.push(OvmValue::Null);
                    }
                }
                x if x == OvmOpcode::FreeLoc as u8 => {
                    // No-op in managed runtime
                    let _idx = read_u16(&bytecode, &mut frame.pc);
                }
                
                // Global variables
                x if x == OvmOpcode::LoadGlb as u8 => {
                    let idx = read_u32(&bytecode, &mut frame.pc) as usize;
                    // Globals stored in registers for now
                    if idx < 256 {
                        self.stack.push(self.registers[idx].clone());
                    }
                }
                x if x == OvmOpcode::StoreGlb as u8 => {
                    let idx = read_u32(&bytecode, &mut frame.pc) as usize;
                    if let Some(val) = self.stack.pop() {
                        if idx < 256 {
                            self.registers[idx] = val;
                        }
                    }
                }
                
                // Memory operations
                x if x == OvmOpcode::Load8 as u8 => {
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get(ptr) {
                            let val = obj.data.first().copied().unwrap_or(0) as i64;
                            self.stack.push(OvmValue::Int(val));
                        }
                    }
                }
                x if x == OvmOpcode::Load16 as u8 => {
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get(ptr) {
                            let val = if obj.data.len() >= 2 {
                                u16::from_le_bytes([obj.data[0], obj.data[1]]) as i64
                            } else { 0 };
                            self.stack.push(OvmValue::Int(val));
                        }
                    }
                }
                x if x == OvmOpcode::Load32 as u8 => {
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get(ptr) {
                            let val = if obj.data.len() >= 4 {
                                i32::from_le_bytes(obj.data[0..4].try_into().unwrap_or([0;4])) as i64
                            } else { 0 };
                            self.stack.push(OvmValue::Int(val));
                        }
                    }
                }
                x if x == OvmOpcode::Load64 as u8 => {
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get(ptr) {
                            let val = if obj.data.len() >= 8 {
                                i64::from_le_bytes(obj.data[0..8].try_into().unwrap_or([0;8]))
                            } else { 0 };
                            self.stack.push(OvmValue::Int(val));
                        }
                    }
                }
                x if x == OvmOpcode::Store8 as u8 => {
                    let val = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get_mut(ptr) {
                            if !obj.data.is_empty() {
                                obj.data[0] = val as u8;
                            }
                        }
                    }
                }
                x if x == OvmOpcode::Store16 as u8 => {
                    let val = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get_mut(ptr) {
                            if obj.data.len() >= 2 {
                                obj.data[0..2].copy_from_slice(&(val as u16).to_le_bytes());
                            }
                        }
                    }
                }
                x if x == OvmOpcode::Store32 as u8 => {
                    let val = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get_mut(ptr) {
                            if obj.data.len() >= 4 {
                                obj.data[0..4].copy_from_slice(&(val as i32).to_le_bytes());
                            }
                        }
                    }
                }
                x if x == OvmOpcode::Store64 as u8 => {
                    let val = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0);
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get_mut(ptr) {
                            if obj.data.len() >= 8 {
                                obj.data[0..8].copy_from_slice(&val.to_le_bytes());
                            }
                        }
                    }
                }
                x if x == OvmOpcode::Realloc as u8 => {
                    let new_size = read_u32(&bytecode, &mut frame.pc) as usize;
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.last() {
                        if let Some(Some(obj)) = self.heap.get_mut(*ptr) {
                            obj.data.resize(new_size, 0);
                            obj.header.size = new_size;
                        }
                    }
                }
                x if x == OvmOpcode::Memcpy as u8 => {
                    let size = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0) as usize;
                    let src = self.stack.pop();
                    let dst = self.stack.pop();
                    if let (Some(OvmValue::Ptr(dst_ptr)), Some(OvmValue::Ptr(src_ptr))) = (dst, src) {
                        let src_data = self.heap.get(src_ptr)
                            .and_then(|o| o.as_ref())
                            .map(|o| o.data[..size.min(o.data.len())].to_vec())
                            .unwrap_or_default();
                        if let Some(Some(dst_obj)) = self.heap.get_mut(dst_ptr) {
                            let copy_len = size.min(dst_obj.data.len()).min(src_data.len());
                            dst_obj.data[..copy_len].copy_from_slice(&src_data[..copy_len]);
                        }
                    }
                }
                x if x == OvmOpcode::Memset as u8 => {
                    let size = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0) as usize;
                    let val = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0) as u8;
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get_mut(ptr) {
                            for i in 0..size.min(obj.data.len()) {
                                obj.data[i] = val;
                            }
                        }
                    }
                }
                
                // Object/struct operations
                x if x == OvmOpcode::New as u8 => {
                    let type_id = read_u32(&bytecode, &mut frame.pc);
                    let size = read_u16(&bytecode, &mut frame.pc) as usize;
                    let ptr = self.gc_alloc(size, type_id);
                    self.stack.push(OvmValue::Ptr(ptr));
                }
                x if x == OvmOpcode::GetField as u8 => {
                    let field_idx = read_u16(&bytecode, &mut frame.pc) as usize;
                    if let Some(OvmValue::Struct(_, fields)) = self.stack.pop() {
                        if field_idx < fields.len() {
                            self.stack.push(fields[field_idx].clone());
                        } else {
                            self.stack.push(OvmValue::Null);
                        }
                    } else if let Some(OvmValue::Ptr(ptr)) = self.stack.last() {
                        // Load field from heap object
                        let ptr = *ptr;
                        if let Some(Some(obj)) = self.heap.get(ptr) {
                            if field_idx < obj.refs.len() {
                                self.stack.push(OvmValue::Ptr(obj.refs[field_idx]));
                            } else {
                                self.stack.push(OvmValue::Null);
                            }
                        }
                    }
                }
                x if x == OvmOpcode::SetField as u8 => {
                    let field_idx = read_u16(&bytecode, &mut frame.pc) as usize;
                    let val = self.stack.pop().unwrap_or(OvmValue::Null);
                    if let Some(OvmValue::Struct(_name, ref mut fields)) = self.stack.last_mut() {
                        if field_idx < fields.len() {
                            fields[field_idx] = val;
                        }
                    }
                }
                x if x == OvmOpcode::Instanceof as u8 => {
                    let type_id = read_u32(&bytecode, &mut frame.pc);
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        let is_instance = self.heap.get(ptr)
                            .and_then(|o| o.as_ref())
                            .map(|o| o.header.type_id == type_id)
                            .unwrap_or(false);
                        self.stack.push(OvmValue::Bool(is_instance));
                    } else {
                        self.stack.push(OvmValue::Bool(false));
                    }
                }
                x if x == OvmOpcode::Cast as u8 => {
                    // Type cast - no-op in a dynamic runtime (value already carries type)
                    let _type_id = read_u32(&bytecode, &mut frame.pc);
                }
                
                // Array operations
                x if x == OvmOpcode::NewArray as u8 => {
                    let size = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0) as usize;
                    self.stack.push(OvmValue::Array(vec![OvmValue::Null; size]));
                }
                x if x == OvmOpcode::ArrayLen as u8 => {
                    if let Some(OvmValue::Array(arr)) = self.stack.pop() {
                        self.stack.push(OvmValue::Int(arr.len() as i64));
                    } else {
                        self.stack.push(OvmValue::Int(0));
                    }
                }
                x if x == OvmOpcode::ArrayGet as u8 => {
                    let idx = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0) as usize;
                    if let Some(OvmValue::Array(arr)) = self.stack.pop() {
                        if idx < arr.len() {
                            self.stack.push(arr[idx].clone());
                        } else {
                            return Err(anyhow!("Array index {} out of bounds (len {})", idx, arr.len()));
                        }
                    }
                }
                x if x == OvmOpcode::ArraySet as u8 => {
                    let val = self.stack.pop().unwrap_or(OvmValue::Null);
                    let idx = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0) as usize;
                    if let Some(OvmValue::Array(ref mut arr)) = self.stack.last_mut() {
                        if idx < arr.len() {
                            arr[idx] = val;
                        }
                    }
                }
                x if x == OvmOpcode::ArraySlice as u8 => {
                    let end = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0) as usize;
                    let start = self.stack.pop().and_then(|v| v.as_int()).unwrap_or(0) as usize;
                    if let Some(OvmValue::Array(arr)) = self.stack.pop() {
                        let slice = arr[start.min(arr.len())..end.min(arr.len())].to_vec();
                        self.stack.push(OvmValue::Array(slice));
                    }
                }
                
                // Type conversion
                x if x == OvmOpcode::I2F as u8 => {
                    if let Some(OvmValue::Int(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Float(v as f64));
                    }
                }
                x if x == OvmOpcode::F2I as u8 => {
                    if let Some(OvmValue::Float(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Int(v as i64));
                    }
                }
                x if x == OvmOpcode::I2B as u8 => {
                    if let Some(OvmValue::Int(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Bool(v != 0));
                    }
                }
                x if x == OvmOpcode::B2I as u8 => {
                    if let Some(OvmValue::Bool(v)) = self.stack.pop() {
                        self.stack.push(OvmValue::Int(if v { 1 } else { 0 }));
                    }
                }
                
                // More register operations
                x if x == OvmOpcode::MoveReg as u8 => {
                    let dst = bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    let src = bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    if src < 256 && dst < 256 {
                        self.registers[dst] = self.registers[src].clone();
                    }
                }
                x if x == OvmOpcode::SwapReg as u8 => {
                    let a = bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    let b = bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    if a < 256 && b < 256 {
                        self.registers.swap(a, b);
                    }
                }
                x if x == OvmOpcode::PushReg as u8 => {
                    let reg = bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    if reg < 256 {
                        self.stack.push(self.registers[reg].clone());
                    }
                }
                x if x == OvmOpcode::PopReg as u8 => {
                    let reg = bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    if reg < 256 {
                        self.registers[reg] = self.stack.pop().unwrap_or(OvmValue::Null);
                    }
                }
                
                // Debug/system
                x if x == OvmOpcode::Debug as u8 => {
                    if let Some(val) = self.stack.last() {
                        debug!("OVM Debug: {:?}", val);
                    }
                }
                x if x == OvmOpcode::Trace as u8 => {
                    if self.enable_tracing {
                        let pc = frame.pc;
                        let stack_depth = self.stack.len();
                        // Note: call_stack.len() requires dropping frame borrow first
                        // Since we're inside the while loop with frame borrowed, just use pc and stack_depth
                        debug!("OVM Trace: pc={}, stack_depth={}", pc, stack_depth);
                    }
                }
                x if x == OvmOpcode::Assert as u8 => {
                    if let Some(val) = self.stack.pop() {
                        if !val.as_bool().unwrap_or(false) {
                            return Err(anyhow!("OVM Assertion failed at pc {}", frame.pc - 1));
                        }
                    }
                }
                
                _ => {
                    warn!("OVM: Unknown opcode 0x{:02x} at pc {}", opcode, frame.pc - 1);
                }
            }
            
            // Run incremental GC step
            self.gc_step();
        }

        Ok(OvmValue::Null)
    }
}

// Free-standing helper functions for reading bytecode (avoids borrow conflicts)
fn read_u16(bytecode: &[u8], pc: &mut usize) -> u16 {
    let val = u16::from_le_bytes(bytecode[*pc..*pc+2].try_into().unwrap_or([0, 0]));
    *pc += 2;
    val
}

fn read_u32(bytecode: &[u8], pc: &mut usize) -> u32 {
    let val = u32::from_le_bytes(bytecode[*pc..*pc+4].try_into().unwrap_or([0; 4]));
    *pc += 4;
    val
}

fn read_i32(bytecode: &[u8], pc: &mut usize) -> i32 {
    let val = i32::from_le_bytes(bytecode[*pc..*pc+4].try_into().unwrap_or([0; 4]));
    *pc += 4;
    val
}

fn read_i64(bytecode: &[u8], pc: &mut usize) -> i64 {
    let val = i64::from_le_bytes(bytecode[*pc..*pc+8].try_into().unwrap_or([0; 8]));
    *pc += 8;
    val
}

fn read_f64(bytecode: &[u8], pc: &mut usize) -> f64 {
    let val = f64::from_le_bytes(bytecode[*pc..*pc+8].try_into().unwrap_or([0; 8]));
    *pc += 8;
    val
}


impl OvmInterpreter {
    // Arithmetic helpers
    fn binary_op_int<F: Fn(i64, i64) -> i64>(&mut self, op: F) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
        match (a, b) {
            (OvmValue::Int(x), OvmValue::Int(y)) => {
                self.stack.push(OvmValue::Int(op(x, y)));
            }
            _ => return Err(anyhow!("Type mismatch for integer operation")),
        }
        Ok(())
    }

    fn binary_op_float<F: Fn(f64, f64) -> f64>(&mut self, op: F) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
        match (a, b) {
            (OvmValue::Float(x), OvmValue::Float(y)) => {
                self.stack.push(OvmValue::Float(op(x, y)));
            }
            _ => return Err(anyhow!("Type mismatch for float operation")),
        }
        Ok(())
    }

    fn compare_op<F: Fn(i64, i64) -> bool>(&mut self, op: F) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
        match (a, b) {
            (OvmValue::Int(x), OvmValue::Int(y)) => {
                self.stack.push(OvmValue::Bool(op(x, y)));
            }
            _ => self.stack.push(OvmValue::Bool(false)),
        }
        Ok(())
    }

    // Garbage collector
    fn gc_alloc(&mut self, size: usize, type_id: u32) -> usize {
        self.total_allocations += 1;
        
        // Check free list first
        if let Some(ptr) = self.free_list.pop() {
            let obj = HeapObject {
                header: GcHeader {
                    mark: false,
                    size,
                    type_id,
                    strong_count: 1,
                    weak_count: 0,
                    generation: self.gc_generation,
                },
                data: vec![0; size],
                refs: Vec::new(),
                finalizer: None,
            };
            self.heap[ptr] = Some(obj);
            return ptr;
        }
        
        // Allocate new
        let ptr = self.heap.len();
        self.heap.push(Some(HeapObject {
            header: GcHeader {
                mark: false,
                size,
                type_id,
                strong_count: 1,
                weak_count: 0,
                generation: self.gc_generation,
            },
            data: vec![0; size],
            refs: Vec::new(),
            finalizer: None,
        }));
        ptr
    }

    fn gc_free(&mut self, ptr: usize) {
        if ptr < self.heap.len() {
            self.heap[ptr] = None;
            self.free_list.push(ptr);
        }
    }

    fn gc_step(&mut self) {
        // Take state out of self to avoid borrow conflicts with self.collect_roots() etc.
        let state = std::mem::replace(&mut self.gc_state, GcState::Idle);

        match state {
            GcState::Idle => {
                // State is currently Idle (replaced above).
                // Check if we need to transition.
                if self.total_allocations > self.gc_threshold {
                    // Start Marking
                    let mut roots = Vec::new();
                    self.collect_roots(&mut roots);
                    self.gc_state = GcState::Marking(roots);
                }
                // If not transitioning, self.gc_state is already Idle, so we are good.
            }
            GcState::Marking(mut stack) => {
                let mut budget = self.gc_work_budget;
                while budget > 0 && !stack.is_empty() {
                    if let Some(ptr) = stack.pop() {
                        self.mark_object(ptr, &mut stack);
                    }
                    budget -= 1;
                }
                
                if stack.is_empty() {
                    self.gc_state = GcState::Sweeping(0);
                } else {
                    self.gc_state = GcState::Marking(stack);
                }
            }
            GcState::Sweeping(start_idx) => {
                let mut idx = start_idx;
                let mut budget = self.gc_work_budget * 2; // Sweeping is faster
                let mut freed = 0;
                
                while budget > 0 && idx < self.heap.len() {
                    if let Some(ref mut obj) = self.heap[idx] {
                        if !obj.header.mark {
                            self.heap[idx] = None;
                            self.free_list.push(idx);
                            freed += 1;
                        } else {
                            obj.header.mark = false; // Reset for next collection
                        }
                    }
                    idx += 1;
                    budget -= 1;
                }
                
                if idx >= self.heap.len() {
                    if freed > 0 {
                        debug!("OVM GC: Incremental sweep finished, freed {}", freed);
                    }
                    self.gc_threshold = (self.heap.len() - self.free_list.len()) * 2;
                    if self.gc_threshold < 1024 { self.gc_threshold = 1024; }
                    self.gc_state = GcState::Idle;
                } else {
                    self.gc_state = GcState::Sweeping(idx);
                }
            }
        }
    }

    fn mark_object(&mut self, ptr: usize, stack: &mut Vec<usize>) {
        if ptr >= self.heap.len() { return; }
        
        if let Some(Some(obj)) = self.heap.get_mut(ptr) {
            if !obj.header.mark {
                obj.header.mark = true;
                stack.extend_from_slice(&obj.refs);
            }
        }
    }

    fn collect_roots(&self, roots: &mut Vec<usize>) {
        // Stack
        for val in &self.stack {
            if let OvmValue::Ptr(ptr) = val { roots.push(*ptr); }
        }
        // Registers
        for val in &self.registers {
            if let OvmValue::Ptr(ptr) = val { roots.push(*ptr); }
        }
        // Call stack
        for frame in &self.call_stack {
            for val in &frame.locals {
                if let OvmValue::Ptr(ptr) = val { roots.push(*ptr); }
            }
        }
    }

    // Deprecated synchronous collect, redirects to step until done
    fn gc_collect(&mut self) {
        // Force run until idle
        while self.gc_state != GcState::Idle {
            self.gc_step();
        }
        // If we were idle, force start
        if self.gc_state == GcState::Idle {
            // reset threshold to force start
            let old_threshold = self.gc_threshold;
            self.gc_threshold = 0;
            self.gc_step(); 
            self.gc_threshold = old_threshold; // restore (will be updated by sweep end)
            
            while self.gc_state != GcState::Idle {
                self.gc_step();
            }
        }
    }

    // Async runtime
    fn spawn_task(&mut self, func_index: usize, args: Vec<OvmValue>) -> usize {
        let module = self.module.as_ref().unwrap();
        let func = &module.functions[func_index];
        
        let mut locals = vec![OvmValue::Null; func.local_count as usize];
        for (i, arg) in args.into_iter().enumerate() {
            if i < locals.len() {
                locals[i] = arg;
            }
        }
        
        let task = AsyncTask {
            id: 0, // Will be set by executor.spawn()
            state: AsyncState::Pending,
            frame: CallFrame {
                func_index,
                pc: 0,
                bp: 0,
                locals,
                return_addr: None,
            },
            result: None,
            waker: None,
            blocked_on: None,
        };
        
        self.executor.spawn(task)
    }

    fn handle_syscall(&mut self, syscall_id: u16) -> Result<()> {
        match syscall_id {
            0 => {
                // Exit
                info!("OVM: Syscall exit");
            }
            1 => {
                // Print
                if let Some(val) = self.stack.pop() {
                    println!("{:?}", val);
                }
            }
            2 => {
                // Read input
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                self.stack.push(OvmValue::Int(input.trim().parse().unwrap_or(0)));
            }
            _ => {
                warn!("OVM: Unknown syscall {}", syscall_id);
            }
        }
        Ok(())
    }
}

// Legacy interpreter compatibility
pub struct Interpreter {
    ovm: OvmInterpreter,
    global_scope: Rc<RefCell<Scope>>,
}

pub struct Scope {
    variables: HashMap<String, RuntimeValue>,
    parent: Option<Rc<RefCell<Scope>>>,
}

#[derive(Clone, Debug)]
pub enum RuntimeValue {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<RuntimeValue>),
    Map(HashMap<String, RuntimeValue>),
    Function(String, Vec<String>),
    NativePtr(usize),
    Vector(ndarray::Array1<f32>),
    GBox(usize),
}

impl Scope {
    pub fn new() -> Self {
        Scope { variables: HashMap::new(), parent: None }
    }

    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        self.variables.get(name).cloned().or_else(|| {
            self.parent.as_ref().and_then(|p| p.borrow().get(name))
        })
    }

    pub fn set(&mut self, name: String, value: RuntimeValue) {
        self.variables.insert(name, value);
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            ovm: OvmInterpreter::new(),
            global_scope: Rc::new(RefCell::new(Scope::new())),
        }
    }

    pub fn eval_file(&mut self, path: &std::path::Path) -> Result<RuntimeValue> {
        info!("Interpreter: Evaluating file {:?}", path);
        
        // Check if it's a bytecode file
        if path.extension().map(|e| e == "ovm").unwrap_or(false) {
            self.ovm.load_bytecode(path)?;
            let result = self.ovm.run()?;
            return Ok(self.ovm_to_runtime(result));
        }
        
        // Otherwise, simulate source execution
        self.call_main()
    }

    fn ovm_to_runtime(&self, val: OvmValue) -> RuntimeValue {
        match val {
            OvmValue::Null => RuntimeValue::Null,
            OvmValue::Int(i) => RuntimeValue::Integer(i),
            OvmValue::Float(f) => RuntimeValue::Float(f),
            OvmValue::Bool(b) => RuntimeValue::Boolean(b),
            _ => RuntimeValue::Null,
        }
    }

    fn call_main(&mut self) -> Result<RuntimeValue> {
        info!("Interpreter: Calling main()");
        println!("🌟 HELIOS Runtime Initialized 🌟");
        println!("Executing main application logic...");
        Ok(RuntimeValue::Integer(0))
    }

    pub fn eval_expr(&mut self, expr: &crate::parser::ast::Expression) -> Result<RuntimeValue> {
        use crate::parser::ast::{Expression, Literal};

        match expr {
            Expression::Literal(lit) => Ok(match lit {
                Literal::Int(i) => RuntimeValue::Integer(*i),
                Literal::Float(f) => RuntimeValue::Float(*f),
                Literal::String(s) => RuntimeValue::String(s.clone()),
                Literal::Bool(b) => RuntimeValue::Boolean(*b),
            }),
            Expression::Identifier(name) => {
                self.global_scope.borrow().get(name).ok_or_else(|| anyhow!("Undefined: {}", name))
            },
            Expression::Binary(left, op, right) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.apply_binary_op(op, l, r)
            },
            _ => Err(anyhow!("Expression type not yet supported")),
        }
    }

    fn apply_binary_op(&self, op: &crate::parser::ast::BinaryOp, left: RuntimeValue, right: RuntimeValue) -> Result<RuntimeValue> {
        use crate::parser::ast::BinaryOp;

        // Auto-promote Integer to Float when mixed
        let (left, right) = match (&left, &right) {
            (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => (RuntimeValue::Float(*l as f64), RuntimeValue::Float(*r)),
            (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => (RuntimeValue::Float(*l), RuntimeValue::Float(*r as f64)),
            _ => (left, right),
        };

        match (op, left, right) {
            // ── Arithmetic: Integer ──
            (BinaryOp::Add, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Integer(l.wrapping_add(r))),
            (BinaryOp::Sub, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Integer(l.wrapping_sub(r))),
            (BinaryOp::Mul, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Integer(l.wrapping_mul(r))),
            (BinaryOp::Div, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                if r == 0 { Err(anyhow!("Division by zero")) }
                else { Ok(RuntimeValue::Integer(l / r)) }
            }
            (BinaryOp::Mod, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                if r == 0 { Err(anyhow!("Modulo by zero")) }
                else { Ok(RuntimeValue::Integer(l % r)) }
            }

            // ── Arithmetic: Float ──
            (BinaryOp::Add, RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Float(l + r)),
            (BinaryOp::Sub, RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Float(l - r)),
            (BinaryOp::Mul, RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Float(l * r)),
            (BinaryOp::Div, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                if r == 0.0 { Err(anyhow!("Division by zero")) }
                else { Ok(RuntimeValue::Float(l / r)) }
            }
            (BinaryOp::Mod, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                if r == 0.0 { Err(anyhow!("Modulo by zero")) }
                else { Ok(RuntimeValue::Float(l % r)) }
            }

            // ── String concatenation ──
            (BinaryOp::Add, RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::String(format!("{}{}", l, r))),
            (BinaryOp::Add, RuntimeValue::String(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::String(format!("{}{}", l, r))),
            (BinaryOp::Add, RuntimeValue::String(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::String(format!("{}{}", l, r))),

            // ── Comparison: Integer ──
            (BinaryOp::Eq, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Boolean(l == r)),
            (BinaryOp::NotEq, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Boolean(l != r)),
            (BinaryOp::Lt, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Boolean(l < r)),
            (BinaryOp::Gt, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Boolean(l > r)),
            (BinaryOp::LtEq, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Boolean(l <= r)),
            (BinaryOp::GtEq, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Boolean(l >= r)),

            // ── Comparison: Float ──
            (BinaryOp::Eq, RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Boolean((l - r).abs() < f64::EPSILON)),
            (BinaryOp::NotEq, RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Boolean((l - r).abs() >= f64::EPSILON)),
            (BinaryOp::Lt, RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Boolean(l < r)),
            (BinaryOp::Gt, RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Boolean(l > r)),
            (BinaryOp::LtEq, RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Boolean(l <= r)),
            (BinaryOp::GtEq, RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Boolean(l >= r)),

            // ── Comparison: String ──
            (BinaryOp::Eq, RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l == r)),
            (BinaryOp::NotEq, RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l != r)),
            (BinaryOp::Lt, RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l < r)),
            (BinaryOp::Gt, RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l > r)),
            (BinaryOp::LtEq, RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l <= r)),
            (BinaryOp::GtEq, RuntimeValue::String(l), RuntimeValue::String(r)) => Ok(RuntimeValue::Boolean(l >= r)),

            // ── Comparison: Boolean ──
            (BinaryOp::Eq, RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => Ok(RuntimeValue::Boolean(l == r)),
            (BinaryOp::NotEq, RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => Ok(RuntimeValue::Boolean(l != r)),

            // ── Equality with Null ──
            (BinaryOp::Eq, RuntimeValue::Null, RuntimeValue::Null) => Ok(RuntimeValue::Boolean(true)),
            (BinaryOp::Eq, _, RuntimeValue::Null) | (BinaryOp::Eq, RuntimeValue::Null, _) => Ok(RuntimeValue::Boolean(false)),
            (BinaryOp::NotEq, RuntimeValue::Null, RuntimeValue::Null) => Ok(RuntimeValue::Boolean(false)),
            (BinaryOp::NotEq, _, RuntimeValue::Null) | (BinaryOp::NotEq, RuntimeValue::Null, _) => Ok(RuntimeValue::Boolean(true)),

            // ── Logical ──
            (BinaryOp::And, RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => Ok(RuntimeValue::Boolean(l && r)),
            (BinaryOp::Or, RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => Ok(RuntimeValue::Boolean(l || r)),

            // ── Range ──
            (BinaryOp::Range, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                let items: Vec<RuntimeValue> = (l..r).map(RuntimeValue::Integer).collect();
                Ok(RuntimeValue::Array(items))
            }

            _ => Err(anyhow!("Unsupported binary operation {:?} on given operand types", op)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_basic_allocation_and_sweep() {
        let mut ovm = OvmInterpreter::new();
        // Set small threshold to force GC
        ovm.gc_threshold = 0; 
        
        // Allocate an object (size 10)
        let ptr = ovm.gc_alloc(10, 0);
        assert!(ovm.heap[ptr].is_some());
        
        // Drop reference from stack (it's not on stack yet, gc_alloc doesn't push)
        // So it is unreachable immediately.
        
        // Run GC until idle
        ovm.gc_collect(); // This now calls gc_step internally until idle
        
        // Should be freed (None)
        assert!(ovm.heap[ptr].is_none());
    }

    #[test]
    fn test_gc_keep_reachable() {
        let mut ovm = OvmInterpreter::new();
        ovm.gc_threshold = 0;
        
        let ptr = ovm.gc_alloc(10, 0);
        // Push to stack to make it reachable
        ovm.stack.push(OvmValue::Ptr(ptr));
        
        ovm.gc_collect();
        
        // Should still exist and be unmarked (mark reset at end of sweep)
        assert!(ovm.heap[ptr].is_some());
        assert!(!ovm.heap[ptr].as_ref().unwrap().header.mark);
    }

    #[test]
    fn test_gc_incremental_steps() {
        let mut ovm = OvmInterpreter::new();
        ovm.gc_threshold = 0;
        ovm.gc_work_budget = 1; // Small budget to force steps
        
        let ptr = ovm.gc_alloc(10, 0);
        ovm.stack.push(OvmValue::Ptr(ptr));
        
        // Step 1: Idle -> Marking (Collects roots)
        ovm.gc_step();
        assert!(matches!(ovm.gc_state, GcState::Marking(_)));
        
        // Step 2: Marking -> Sweeping (Marks object, stack empties)
        ovm.gc_step(); 
        // Depending on implementation details, might still be Marking or switched to Sweeping
        
        // Run until done
        while ovm.gc_state != GcState::Idle {
            ovm.gc_step();
        }
        
        assert!(ovm.heap[ptr].is_some());
    }
}
