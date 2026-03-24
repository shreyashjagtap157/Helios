#![allow(dead_code)]
//! Omni Virtual Machine (OVM) - Full Bytecode Interpreter
//! Executes OVM bytecode with GC, async support, exception handling, and native dispatch

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::codegen::ovm::{OvmConstant, OvmFunction, OvmModule, OvmOpcode};

/// Stack value in the OVM
#[derive(Clone, Debug)]
pub enum OvmValue {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    Ptr(usize),                    // Heap pointer
    FuncRef(usize),                // Function index
    ClosureRef(usize),             // Closure object pointer
    FutureRef(usize),              // Async future pointer
    String(String),                // Interned string
    Array(Vec<OvmValue>),          // Array value
    Struct(String, Vec<OvmValue>), // Struct (typename, fields)
}

impl Default for OvmValue {
    fn default() -> Self {
        OvmValue::Null
    }
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
    pub generation: u8, // For generational GC
}

/// Heap-allocated object
#[derive(Debug)]
pub struct HeapObject {
    pub header: GcHeader,
    pub data: Vec<u8>,
    pub refs: Vec<usize>,          // Pointers to other heap objects
    pub finalizer: Option<String>, // Drop function name
}

/// Call frame for the call stack
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub func_index: usize,
    pub pc: usize, // Program counter
    pub bp: usize, // Base pointer (local variable start)
    pub locals: Vec<OvmValue>,
    pub return_addr: Option<usize>, // Where to return to
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
    Future(usize),  // Waiting on another future
    Channel(usize), // Waiting on channel
    Timer(u64),     // Waiting on timer (timestamp)
    Io(i32),        // Waiting on file descriptor
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
    native_functions:
        HashMap<String, Box<dyn Fn(&mut OvmInterpreter, &[OvmValue]) -> Result<OvmValue>>>,

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

    /// Load OVM bytecode from raw bytes (for standalone runner)
    pub fn load_from_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let module = self.deserialize_module(bytes)?;
        self.load_module(module);
        Ok(())
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

        debug!(
            "OVM Header: version={}, entry={}, const_off={}, code_off={}",
            version, entry_point, const_pool_off, code_off
        );

        // Parse constant pool
        let mut constants = Vec::new();
        if const_pool_off > 0 && const_pool_off < bytes.len() {
            let const_bytes = &bytes
                [const_pool_off..const_pool_off + const_pool_len.min(bytes.len() - const_pool_off)];
            if const_bytes.len() >= 4 {
                let num_constants = u32::from_le_bytes(const_bytes[0..4].try_into()?) as usize;
                let mut pos = 4;

                for _ in 0..num_constants {
                    if pos >= const_bytes.len() {
                        break;
                    }

                    let const_type = const_bytes[pos];
                    pos += 1;

                    match const_type {
                        0x01 => {
                            // I64
                            if pos + 8 <= const_bytes.len() {
                                let val = i64::from_le_bytes(const_bytes[pos..pos + 8].try_into()?);
                                constants.push(OvmConstant::I64(val));
                                pos += 8;
                            }
                        }
                        0x02 => {
                            // F64
                            if pos + 8 <= const_bytes.len() {
                                let val = f64::from_le_bytes(const_bytes[pos..pos + 8].try_into()?);
                                constants.push(OvmConstant::F64(val));
                                pos += 8;
                            }
                        }
                        0x03 => {
                            // String
                            if pos + 4 <= const_bytes.len() {
                                let len = u32::from_le_bytes(const_bytes[pos..pos + 4].try_into()?)
                                    as usize;
                                pos += 4;
                                if pos + len <= const_bytes.len() {
                                    let s = String::from_utf8_lossy(&const_bytes[pos..pos + len])
                                        .to_string();
                                    constants.push(OvmConstant::String(s));
                                    pos += len;
                                }
                            }
                        }
                        0x04 => {
                            // Bytes
                            if pos + 4 <= const_bytes.len() {
                                let len = u32::from_le_bytes(const_bytes[pos..pos + 4].try_into()?)
                                    as usize;
                                pos += 4;
                                if pos + len <= const_bytes.len() {
                                    constants.push(OvmConstant::Bytes(
                                        const_bytes[pos..pos + len].to_vec(),
                                    ));
                                    pos += len;
                                }
                            }
                        }
                        _ => {
                            break;
                        }
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
                    if pos + 12 > code_bytes.len() {
                        break;
                    }

                    let name_index = u32::from_le_bytes(code_bytes[pos..pos + 4].try_into()?);
                    pos += 4;
                    let param_count = u16::from_le_bytes(code_bytes[pos..pos + 2].try_into()?);
                    pos += 2;
                    let local_count = u16::from_le_bytes(code_bytes[pos..pos + 2].try_into()?);
                    pos += 2;
                    let max_stack = u16::from_le_bytes(code_bytes[pos..pos + 2].try_into()?);
                    pos += 2;
                    let bytecode_len =
                        u32::from_le_bytes(code_bytes[pos..pos + 4].try_into()?) as usize;
                    pos += 4;

                    let bytecode = if pos + bytecode_len <= code_bytes.len() {
                        let bc = code_bytes[pos..pos + bytecode_len].to_vec();
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

        info!(
            "OVM: Loaded {} constants, {} functions",
            constants.len(),
            functions.len()
        );

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
        self.native_functions.insert(
            "print".to_string(),
            Box::new(|_, args| {
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
            }),
        );

        // String length
        self.native_functions.insert(
            "strlen".to_string(),
            Box::new(|_, args| {
                if let Some(OvmValue::String(s)) = args.first() {
                    Ok(OvmValue::Int(s.len() as i64))
                } else {
                    Err(anyhow!("strlen requires a string argument"))
                }
            }),
        );

        // String concatenation
        self.native_functions.insert(
            "strcat".to_string(),
            Box::new(|_, args| {
                if args.len() >= 2 {
                    if let (Some(OvmValue::String(a)), Some(OvmValue::String(b))) =
                        (args.get(0), args.get(1))
                    {
                        return Ok(OvmValue::String(format!("{}{}", a, b)));
                    }
                }
                Err(anyhow!("strcat requires two string arguments"))
            }),
        );

        // Array operations
        self.native_functions.insert(
            "array_new".to_string(),
            Box::new(|_, args| {
                let size = args.first().and_then(|v| v.as_int()).unwrap_or(0) as usize;
                Ok(OvmValue::Array(vec![OvmValue::Null; size]))
            }),
        );

        self.native_functions.insert(
            "array_len".to_string(),
            Box::new(|_, args| {
                if let Some(OvmValue::Array(arr)) = args.first() {
                    Ok(OvmValue::Int(arr.len() as i64))
                } else {
                    Err(anyhow!("array_len requires an array argument"))
                }
            }),
        );

        self.native_functions.insert(
            "array_get".to_string(),
            Box::new(|_, args| {
                if args.len() >= 2 {
                    if let (Some(OvmValue::Array(arr)), Some(idx)) = (args.get(0), args.get(1)) {
                        let i = idx.as_int().unwrap_or(0) as usize;
                        if i < arr.len() {
                            return Ok(arr[i].clone());
                        }
                    }
                }
                Err(anyhow!("array_get: index out of bounds"))
            }),
        );

        // Math functions
        self.native_functions.insert(
            "sqrt".to_string(),
            Box::new(|_, args| {
                if let Some(OvmValue::Float(f)) = args.first() {
                    Ok(OvmValue::Float(f.sqrt()))
                } else if let Some(OvmValue::Int(i)) = args.first() {
                    Ok(OvmValue::Float((*i as f64).sqrt()))
                } else {
                    Err(anyhow!("sqrt requires a numeric argument"))
                }
            }),
        );

        self.native_functions.insert(
            "sin".to_string(),
            Box::new(|_, args| {
                if let Some(OvmValue::Float(f)) = args.first() {
                    Ok(OvmValue::Float(f.sin()))
                } else {
                    Err(anyhow!("sin requires a float argument"))
                }
            }),
        );

        self.native_functions.insert(
            "cos".to_string(),
            Box::new(|_, args| {
                if let Some(OvmValue::Float(f)) = args.first() {
                    Ok(OvmValue::Float(f.cos()))
                } else {
                    Err(anyhow!("cos requires a float argument"))
                }
            }),
        );

        // Time
        self.native_functions.insert(
            "time_now".to_string(),
            Box::new(|_, _| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                Ok(OvmValue::Int(duration.as_millis() as i64))
            }),
        );

        debug!(
            "OVM: Registered {} native functions",
            self.native_functions.len()
        );
    }

    /// Execute from entry point
    pub fn run(&mut self) -> Result<OvmValue> {
        let module = self
            .module
            .as_ref()
            .ok_or_else(|| anyhow!("No module loaded"))?;
        let entry = module.entry_point as usize;

        info!("OVM: Starting execution at function index {}", entry);
        self.call_function(entry, Vec::new())
    }

    /// Run the async event loop until all tasks complete
    pub fn run_async(&mut self) -> Result<OvmValue> {
        let module = self
            .module
            .as_ref()
            .ok_or_else(|| anyhow!("No module loaded"))?;
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
            let task = self
                .executor
                .tasks
                .get(&task_id)
                .ok_or_else(|| anyhow!("Task not found"))?;
            (
                task.frame.func_index,
                task.frame.pc,
                task.frame.locals.clone(),
            )
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
        let module = self
            .module
            .as_ref()
            .ok_or_else(|| anyhow!("No module loaded"))?;

        if func_index >= module.functions.len() {
            return Err(anyhow!("Function index {} out of bounds", func_index));
        }

        let func = &module.functions[func_index];
        debug!(
            "OVM: Calling function '{}' with {} args",
            func.name,
            args.len()
        );

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
                                b.iter().map(|&byte| OvmValue::Int(byte as i64)).collect(),
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
                            if x < y {
                                -1
                            } else if x > y {
                                1
                            } else {
                                0
                            }
                        }
                        _ => 0,
                    };
                    self.stack.push(OvmValue::Int(result));
                }
                x if x == OvmOpcode::IsNull as u8 => {
                    let val = self.stack.pop();
                    self.stack
                        .push(OvmValue::Bool(matches!(val, Some(OvmValue::Null) | None)));
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
                            } else {
                                0
                            };
                            self.stack.push(OvmValue::Int(val));
                        }
                    }
                }
                x if x == OvmOpcode::Load32 as u8 => {
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get(ptr) {
                            let val = if obj.data.len() >= 4 {
                                i32::from_le_bytes(obj.data[0..4].try_into().unwrap_or([0; 4]))
                                    as i64
                            } else {
                                0
                            };
                            self.stack.push(OvmValue::Int(val));
                        }
                    }
                }
                x if x == OvmOpcode::Load64 as u8 => {
                    if let Some(OvmValue::Ptr(ptr)) = self.stack.pop() {
                        if let Some(Some(obj)) = self.heap.get(ptr) {
                            let val = if obj.data.len() >= 8 {
                                i64::from_le_bytes(obj.data[0..8].try_into().unwrap_or([0; 8]))
                            } else {
                                0
                            };
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
                    if let (Some(OvmValue::Ptr(dst_ptr)), Some(OvmValue::Ptr(src_ptr))) = (dst, src)
                    {
                        let src_data = self
                            .heap
                            .get(src_ptr)
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
                        let is_instance = self
                            .heap
                            .get(ptr)
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
                            return Err(anyhow!(
                                "Array index {} out of bounds (len {})",
                                idx,
                                arr.len()
                            ));
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
                    warn!(
                        "OVM: Unknown opcode 0x{:02x} at pc {}",
                        opcode,
                        frame.pc - 1
                    );
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
    let val = u16::from_le_bytes(bytecode[*pc..*pc + 2].try_into().unwrap_or([0, 0]));
    *pc += 2;
    val
}

fn read_u32(bytecode: &[u8], pc: &mut usize) -> u32 {
    let val = u32::from_le_bytes(bytecode[*pc..*pc + 4].try_into().unwrap_or([0; 4]));
    *pc += 4;
    val
}

fn read_i32(bytecode: &[u8], pc: &mut usize) -> i32 {
    let val = i32::from_le_bytes(bytecode[*pc..*pc + 4].try_into().unwrap_or([0; 4]));
    *pc += 4;
    val
}

fn read_i64(bytecode: &[u8], pc: &mut usize) -> i64 {
    let val = i64::from_le_bytes(bytecode[*pc..*pc + 8].try_into().unwrap_or([0; 8]));
    *pc += 8;
    val
}

fn read_f64(bytecode: &[u8], pc: &mut usize) -> f64 {
    let val = f64::from_le_bytes(bytecode[*pc..*pc + 8].try_into().unwrap_or([0; 8]));
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
                    if self.gc_threshold < 1024 {
                        self.gc_threshold = 1024;
                    }
                    self.gc_state = GcState::Idle;
                } else {
                    self.gc_state = GcState::Sweeping(idx);
                }
            }
        }
    }

    fn mark_object(&mut self, ptr: usize, stack: &mut Vec<usize>) {
        if ptr >= self.heap.len() {
            return;
        }

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
            if let OvmValue::Ptr(ptr) = val {
                roots.push(*ptr);
            }
        }
        // Registers
        for val in &self.registers {
            if let OvmValue::Ptr(ptr) = val {
                roots.push(*ptr);
            }
        }
        // Call stack
        for frame in &self.call_stack {
            for val in &frame.locals {
                if let OvmValue::Ptr(ptr) = val {
                    roots.push(*ptr);
                }
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
                self.stack
                    .push(OvmValue::Int(input.trim().parse().unwrap_or(0)));
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
    /// Stores the last return value for unwinding
    return_value: Option<RuntimeValue>,
    /// Stores the mutated `self` after a &mut self method call
    last_mutated_self: Option<RuntimeValue>,
    /// Tracks variables declared with `let mut` for auto-mutation writeback
    mut_variables: std::collections::HashSet<String>,
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
    List(Vec<RuntimeValue>),
    Map(HashMap<String, RuntimeValue>),
    Function(Box<crate::parser::ast::Function>),
    NativeFunction(String),
    NativePtr(usize),
    Vector(ndarray::Array1<f32>),
    GBox(usize),
    StructDef {
        name: String,
        fields: Vec<crate::parser::ast::Field>,
        methods: Vec<crate::parser::ast::Function>,
    },
    StructInstance {
        type_name: String,
        fields: HashMap<String, RuntimeValue>,
    },
    Module {
        name: String,
        members: HashMap<String, RuntimeValue>,
    },
}

impl RuntimeValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Null => false,
            RuntimeValue::Boolean(b) => *b,
            RuntimeValue::Integer(i) => *i != 0,
            RuntimeValue::Float(f) => *f != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::Array(a) => !a.is_empty(),
            RuntimeValue::List(l) => !l.is_empty(),
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Null => "null",
            RuntimeValue::Integer(_) => "int",
            RuntimeValue::Float(_) => "float",
            RuntimeValue::Boolean(_) => "bool",
            RuntimeValue::String(_) => "string",
            RuntimeValue::Array(_) | RuntimeValue::List(_) => "list",
            RuntimeValue::Map(_) => "map",
            RuntimeValue::Function(_) | RuntimeValue::NativeFunction(_) => "function",
            RuntimeValue::NativePtr(_) => "ptr",
            RuntimeValue::Vector(_) => "vector",
            RuntimeValue::GBox(_) => "gbox",
            RuntimeValue::StructDef { .. } => "struct_def",
            RuntimeValue::StructInstance { .. } => "struct",
            RuntimeValue::Module { .. } => "module",
        }
    }

    pub fn display_string(&self) -> String {
        match self {
            RuntimeValue::Null => "null".to_string(),
            RuntimeValue::Integer(i) => i.to_string(),
            RuntimeValue::Float(f) => format!("{}", f),
            RuntimeValue::Boolean(b) => b.to_string(),
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Array(a) | RuntimeValue::List(a) => {
                let items: Vec<String> = a.iter().map(|v| v.display_string()).collect();
                format!("[{}]", items.join(", "))
            }
            RuntimeValue::Map(m) => {
                let items: Vec<String> = m
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.display_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            RuntimeValue::Function(f) => format!("<function {}>", f.name),
            RuntimeValue::NativeFunction(name) => format!("<builtin {}>", name),
            RuntimeValue::StructDef { name, .. } => format!("<struct {}>", name),
            RuntimeValue::Module { name, .. } => format!("<module {}>", name),
            RuntimeValue::StructInstance { type_name, fields } => {
                let items: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.display_string()))
                    .collect();
                format!("{} {{{}}}", type_name, items.join(", "))
            }
            _ => format!("{:?}", self),
        }
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Scope>>) -> Self {
        Scope {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        self.variables
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.borrow().get(name)))
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
            return_value: None,
            last_mutated_self: None,
            mut_variables: std::collections::HashSet::new(),
        }
    }

    /// Delegates to the canonical `is_hard_type_error` in `semantic::type_inference`.
    /// The duplicate definition was removed (O-097).
    fn is_hard_type_error(err: &crate::semantic::type_inference::TypeError) -> bool {
        crate::semantic::type_inference::is_hard_type_error(err)
    }

    pub fn eval_file(&mut self, path: &std::path::Path) -> Result<RuntimeValue> {
        info!("Interpreter: Evaluating file {:?}", path);

        // Check if file exists
        if !path.exists() {
            return Err(anyhow!(
                "Error[E001]: source file '{}' does not exist",
                path.display()
            ));
        }

        // Check if it's a bytecode file
        if path.extension().map(|e| e == "ovm").unwrap_or(false) {
            self.ovm.load_bytecode(path)?;
            let result = self.ovm.run()?;
            return Ok(self.ovm_to_runtime(result));
        }

        // Real source execution: read, tokenize, parse, evaluate
        let source = std::fs::read_to_string(path).map_err(|e| {
            anyhow!(
                "Error[E002]: failed to read source file '{}': {}",
                path.display(),
                e
            )
        })?;

        self.eval_source(&source, &path.display().to_string())
    }

    /// Evaluate Omni source code string
    pub fn eval_source(&mut self, source: &str, filename: &str) -> Result<RuntimeValue> {
        use crate::lexer::tokenize;
        use crate::parser::Parser;

        // Tokenize
        let tokens = tokenize(source).map_err(|e| anyhow!("Error[E010] in {}: {}", filename, e))?;

        // Parse
        let mut parser = Parser::new(tokens);
        let module = parser
            .parse_module()
            .map_err(|e| anyhow!("Error[E020] in {}: {}", filename, e))?;

        // ── TYPE CHECKING ── fatal for concrete type mismatches ──
        let type_result = crate::semantic::type_inference::check_types(&module);
        match type_result {
            Ok(result) => {
                // Print any non-fatal warnings from inference
                for w in &result.warnings {
                    eprintln!("warning: type inference: {}", w);
                }
            }
            Err(errors) => {
                let hard_errors: Vec<_> = errors
                    .iter()
                    .filter(|e| Self::is_hard_type_error(e))
                    .collect();
                let warnings: Vec<_> = errors
                    .iter()
                    .filter(|e| !Self::is_hard_type_error(e))
                    .collect();
                for w in &warnings {
                    eprintln!("warning: type inference: {}", w);
                }
                if !hard_errors.is_empty() {
                    for e in &hard_errors {
                        eprintln!("error[E005]: type error: {}", e);
                    }
                    return Err(anyhow::anyhow!(
                        "Type checking failed with {} error(s)",
                        hard_errors.len()
                    ));
                }
            }
        }

        // ── BORROW CHECKING ── warnings for ownership violations ──
        let borrow_errors = crate::semantic::borrow_check::BorrowChecker::check_module(&module);
        if !borrow_errors.is_empty() {
            for e in &borrow_errors {
                eprintln!("warning[E006]: borrow check: {}", e);
            }
        }

        // Execute: walk the AST
        let mut last_result = RuntimeValue::Null;
        let mut found_main = false;

        for item in &module.items {
            match item {
                crate::parser::ast::Item::Function(func) => {
                    // Register function in global scope
                    let func_clone = func.clone();
                    self.global_scope.borrow_mut().set(
                        func.name.clone(),
                        RuntimeValue::Function(Box::new(func_clone)),
                    );
                    if func.name == "main" {
                        found_main = true;
                    }
                }
                crate::parser::ast::Item::Module(_) => {
                    // Module declarations are handled at parse time
                }
                crate::parser::ast::Item::Struct(struct_def) => {
                    // Register struct definition in global scope
                    self.global_scope.borrow_mut().set(
                        struct_def.name.clone(),
                        RuntimeValue::StructDef {
                            name: struct_def.name.clone(),
                            fields: struct_def.fields.clone(),
                            methods: struct_def.methods.clone(),
                        },
                    );
                }
                crate::parser::ast::Item::Import(import_decl) => {
                    // Determine the alias or last segment as the binding name
                    let bind_name = import_decl
                        .alias
                        .clone()
                        .unwrap_or_else(|| import_decl.path.last().cloned().unwrap_or_default());
                    let full_path = import_decl.path.join("::");

                    // Create a module stub with native functions based on the module path
                    let mut members = HashMap::new();
                    match full_path.as_str() {
                        "core::logging" => {
                            members.insert(
                                "debug".to_string(),
                                RuntimeValue::NativeFunction("__log_debug".to_string()),
                            );
                            members.insert(
                                "info".to_string(),
                                RuntimeValue::NativeFunction("__log_info".to_string()),
                            );
                            members.insert(
                                "warn".to_string(),
                                RuntimeValue::NativeFunction("__log_warn".to_string()),
                            );
                            members.insert(
                                "error".to_string(),
                                RuntimeValue::NativeFunction("__log_error".to_string()),
                            );
                            members.insert(
                                "trace".to_string(),
                                RuntimeValue::NativeFunction("__log_trace".to_string()),
                            );
                        }
                        "core::math" => {
                            members.insert(
                                "sqrt".to_string(),
                                RuntimeValue::NativeFunction("__math_sqrt".to_string()),
                            );
                            members.insert(
                                "abs".to_string(),
                                RuntimeValue::NativeFunction("__math_abs".to_string()),
                            );
                            members.insert(
                                "pow".to_string(),
                                RuntimeValue::NativeFunction("__math_pow".to_string()),
                            );
                            members.insert(
                                "min".to_string(),
                                RuntimeValue::NativeFunction("__math_min".to_string()),
                            );
                            members.insert(
                                "max".to_string(),
                                RuntimeValue::NativeFunction("__math_max".to_string()),
                            );
                            members.insert(
                                "floor".to_string(),
                                RuntimeValue::NativeFunction("__math_floor".to_string()),
                            );
                            members.insert(
                                "ceil".to_string(),
                                RuntimeValue::NativeFunction("__math_ceil".to_string()),
                            );
                            members.insert(
                                "pi".to_string(),
                                RuntimeValue::NativeFunction("__math_pi".to_string()),
                            );
                        }
                        "core::json" => {
                            members.insert(
                                "parse".to_string(),
                                RuntimeValue::NativeFunction("__json_parse".to_string()),
                            );
                            members.insert(
                                "stringify".to_string(),
                                RuntimeValue::NativeFunction("__json_stringify".to_string()),
                            );
                        }
                        _ => {
                            // Generic module stub — functions will be no-ops
                            eprintln!(
                                "warning: unresolved import '{}', using stub module",
                                full_path
                            );
                        }
                    }
                    self.global_scope.borrow_mut().set(
                        bind_name,
                        RuntimeValue::Module {
                            name: full_path,
                            members,
                        },
                    );
                }
                crate::parser::ast::Item::Impl(impl_block) => {
                    // Add trait/impl methods to the struct definition
                    let type_name = impl_block.type_name.clone();
                    let scope = self.global_scope.borrow();
                    if let Some(existing) = scope.get(&type_name) {
                        if let RuntimeValue::StructDef {
                            name,
                            fields,
                            methods,
                        } = existing
                        {
                            let mut updated_methods = methods.clone();
                            for method in &impl_block.methods {
                                updated_methods.retain(|m| m.name != method.name);
                                updated_methods.push(method.clone());
                            }
                            drop(scope);
                            self.global_scope.borrow_mut().set(
                                type_name,
                                RuntimeValue::StructDef {
                                    name,
                                    fields,
                                    methods: updated_methods,
                                },
                            );
                        } else {
                            drop(scope);
                        }
                    } else {
                        drop(scope);
                    }
                }
                _ => {}
            }
        }

        // If there's a main function, call it
        if found_main {
            last_result = self.call_function("main", &[])?;
        }

        Ok(last_result)
    }

    /// Call a registered function by name
    pub fn call_function(&mut self, name: &str, args: &[RuntimeValue]) -> Result<RuntimeValue> {
        let func = {
            let scope = self.global_scope.borrow();
            match scope.get(name) {
                Some(RuntimeValue::Function(f)) => (*f).clone(),
                Some(_) => return Err(anyhow!("'{}' is not a function", name)),
                None => return Err(anyhow!("Undefined function: {}", name)),
            }
        };

        self.call_function_ast(&func, args)
    }

    /// Evaluate a block of statements
    pub fn eval_block(&mut self, block: &crate::parser::ast::Block) -> Result<RuntimeValue> {
        use crate::parser::ast::Statement;

        let mut last = RuntimeValue::Null;

        for stmt in &block.statements {
            last = self.eval_statement(stmt)?;
        }

        Ok(last)
    }

    /// Evaluate a single statement
    pub fn eval_statement(&mut self, stmt: &crate::parser::ast::Statement) -> Result<RuntimeValue> {
        use crate::parser::ast::Statement;

        match stmt {
            Statement::Let {
                name,
                mutable,
                ty: _,
                value,
            } => {
                let val = if let Some(v) = value {
                    self.eval_expr(v)?
                } else {
                    RuntimeValue::Null
                };
                if *mutable {
                    self.mut_variables.insert(name.clone());
                }
                self.global_scope.borrow_mut().set(name.clone(), val);
                Ok(RuntimeValue::Null)
            }
            Statement::Var { name, ty: _, value } => {
                let val = if let Some(v) = value {
                    self.eval_expr(v)?
                } else {
                    RuntimeValue::Null
                };
                self.global_scope.borrow_mut().set(name.clone(), val);
                Ok(RuntimeValue::Null)
            }
            Statement::Return(Some(expr)) => {
                let val = self.eval_expr(expr)?;
                self.return_value = Some(val);
                Err(anyhow!("__return__"))
            }
            Statement::Return(None) => {
                self.return_value = Some(RuntimeValue::Null);
                Err(anyhow!("__return__"))
            }
            Statement::Expression(expr) => self.eval_expr(expr),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond = self.eval_expr(condition)?;
                if cond.is_truthy() {
                    self.eval_block(then_block)
                } else if let Some(else_b) = else_block {
                    self.eval_block(else_b)
                } else {
                    Ok(RuntimeValue::Null)
                }
            }
            Statement::While { condition, body } => {
                loop {
                    let cond = self.eval_expr(condition)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    let result = self.eval_block(body);
                    match &result {
                        Err(_e) if format!("{}", _e) == "__break__" => break,
                        Err(_e) if format!("{}", _e) == "__continue__" => continue,
                        Err(_e) => return result,
                        _ => {}
                    }
                }
                Ok(RuntimeValue::Null)
            }
            Statement::For { var, iter, body } => {
                let iterable = self.eval_expr(iter)?;
                match iterable {
                    RuntimeValue::List(items) | RuntimeValue::Array(items) => {
                        for item in items {
                            self.global_scope.borrow_mut().set(var.clone(), item);
                            let result = self.eval_block(body);
                            match &result {
                                Err(_e) if format!("{}", _e) == "__break__" => break,
                                Err(_e) if format!("{}", _e) == "__continue__" => continue,
                                Err(_e) => return result,
                                _ => {}
                            }
                        }
                    }
                    RuntimeValue::Integer(end) => {
                        // Range 0..end
                        for i in 0..end {
                            self.global_scope
                                .borrow_mut()
                                .set(var.clone(), RuntimeValue::Integer(i));
                            let result = self.eval_block(body);
                            match &result {
                                Err(_e) if format!("{}", _e) == "__break__" => break,
                                Err(_e) if format!("{}", _e) == "__continue__" => continue,
                                Err(_e) => return result,
                                _ => {}
                            }
                        }
                    }
                    _ => return Err(anyhow!("Cannot iterate over {:?}", iterable)),
                }
                Ok(RuntimeValue::Null)
            }
            Statement::Assignment { target, op, value } => {
                use crate::parser::ast::{BinaryOp, Expression};
                let new_val = self.eval_expr(value)?;
                match target {
                    Expression::Identifier(name) => {
                        let final_val = if let Some(binop) = op {
                            let current = self
                                .global_scope
                                .borrow()
                                .get(name)
                                .ok_or_else(|| anyhow!("Undefined variable: {}", name))?;
                            self.apply_binary_op(binop, current, new_val)?
                        } else {
                            new_val
                        };
                        self.global_scope.borrow_mut().set(name.clone(), final_val);
                    }
                    Expression::Field(obj_expr, field_name) => {
                        // Field assignment: obj.field = value
                        if let Expression::Identifier(var_name) = obj_expr.as_ref() {
                            let current_obj = self
                                .global_scope
                                .borrow()
                                .get(var_name)
                                .ok_or_else(|| anyhow!("Undefined variable: {}", var_name))?;
                            match current_obj {
                                RuntimeValue::StructInstance {
                                    type_name,
                                    mut fields,
                                } => {
                                    let final_val = if let Some(binop) = op {
                                        let current = fields
                                            .get(field_name)
                                            .ok_or_else(|| {
                                                anyhow!("No field '{}' on struct", field_name)
                                            })?
                                            .clone();
                                        self.apply_binary_op(binop, current, new_val)?
                                    } else {
                                        new_val
                                    };
                                    fields.insert(field_name.clone(), final_val);
                                    self.global_scope.borrow_mut().set(
                                        var_name.clone(),
                                        RuntimeValue::StructInstance { type_name, fields },
                                    );
                                }
                                RuntimeValue::Map(mut m) => {
                                    let final_val = if let Some(binop) = op {
                                        let current = m
                                            .get(field_name)
                                            .ok_or_else(|| {
                                                anyhow!("No field '{}' on map", field_name)
                                            })?
                                            .clone();
                                        self.apply_binary_op(binop, current, new_val)?
                                    } else {
                                        new_val
                                    };
                                    m.insert(field_name.clone(), final_val);
                                    self.global_scope
                                        .borrow_mut()
                                        .set(var_name.clone(), RuntimeValue::Map(m));
                                }
                                _ => {
                                    return Err(anyhow!(
                                        "Cannot set field '{}' on {:?}",
                                        field_name,
                                        current_obj
                                    ))
                                }
                            }
                        }
                    }
                    _ => {} // Index assignment - simplified
                }
                Ok(RuntimeValue::Null)
            }
            Statement::Pass => Ok(RuntimeValue::Null),
            Statement::Break(_) => Err(anyhow!("__break__")),
            Statement::Continue => Err(anyhow!("__continue__")),
            Statement::Match { expr, arms } => {
                let subject = self.eval_expr(expr)?;
                for arm in arms {
                    if let Some(bindings) = self.match_pattern(&arm.pattern, &subject) {
                        // Apply bindings to scope
                        for (name, value) in &bindings {
                            self.global_scope
                                .borrow_mut()
                                .set(name.clone(), value.clone());
                        }
                        // Evaluate body
                        return match &arm.body {
                            crate::parser::ast::MatchBody::Block(block) => self.eval_block(block),
                            crate::parser::ast::MatchBody::Expr(e) => self.eval_expr(e),
                        };
                    }
                }
                Err(anyhow!(
                    "Match exhausted: no arm matched value {:?}",
                    subject
                ))
            }
            _ => Ok(RuntimeValue::Null), // Other statements handled minimally
        }
    }

    fn match_pattern(
        &self,
        pattern: &crate::parser::ast::Pattern,
        value: &RuntimeValue,
    ) -> Option<std::collections::HashMap<String, RuntimeValue>> {
        use crate::parser::ast::{Literal, Pattern};
        let mut bindings = std::collections::HashMap::new();

        match pattern {
            Pattern::Wildcard => Some(bindings), // _ matches everything

            Pattern::Binding(name) => {
                // Variable binding: captures the value
                if name == "_" {
                    Some(bindings) // underscore binding, don't capture
                } else if name == "None" || name == "Option::None" {
                    // Match Null as None
                    if matches!(value, RuntimeValue::Null) {
                        Some(bindings)
                    } else {
                        None
                    }
                } else if name == "Some" {
                    // Match any non-null value as Some
                    if matches!(value, RuntimeValue::Null) {
                        None
                    } else {
                        Some(bindings)
                    }
                } else {
                    bindings.insert(name.clone(), value.clone());
                    Some(bindings)
                }
            }

            Pattern::Literal(lit) => {
                // Compare literal to value
                let matches = match (lit, value) {
                    (Literal::Int(n), RuntimeValue::Integer(v)) => *n == *v,
                    (Literal::Float(f), RuntimeValue::Float(v)) => (*f - *v).abs() < f64::EPSILON,
                    (Literal::String(s), RuntimeValue::String(v)) => s == v,
                    (Literal::Bool(b), RuntimeValue::Boolean(v)) => *b == *v,
                    (Literal::Null, RuntimeValue::Null) => true,
                    _ => false,
                };
                if matches {
                    Some(bindings)
                } else {
                    None
                }
            }

            Pattern::Constructor(name, sub_patterns) => {
                // Special handling for Option::Some(x) — match any non-null value
                if name == "Option::Some" || name == "Some" {
                    if matches!(value, RuntimeValue::Null) {
                        return None;
                    }
                    // Bind the value to the sub-pattern binding
                    if let Some(pat) = sub_patterns.first() {
                        if let Pattern::Binding(var_name) = pat {
                            bindings.insert(var_name.clone(), value.clone());
                        }
                    }
                    return Some(bindings);
                }
                // Check if value is a struct instance with matching type name
                match value {
                    RuntimeValue::StructInstance {
                        type_name,
                        fields: _,
                    } => {
                        if type_name != name {
                            return None;
                        }
                        Some(bindings)
                    }
                    _ => None,
                }
            }

            Pattern::Or(patterns) => {
                // OR patterns: try each sub-pattern, return first match
                for sub_pat in patterns {
                    if let Some(bindings) = self.match_pattern(sub_pat, value) {
                        return Some(bindings);
                    }
                }
                None
            }
        }
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

    pub fn eval_expr(&mut self, expr: &crate::parser::ast::Expression) -> Result<RuntimeValue> {
        use crate::parser::ast::{Expression, Literal, UnaryOp};

        match expr {
            Expression::Literal(lit) => Ok(match lit {
                Literal::Int(i) => RuntimeValue::Integer(*i),
                Literal::Float(f) => RuntimeValue::Float(*f),
                Literal::String(s) => RuntimeValue::String(s.clone()),
                Literal::Bool(b) => RuntimeValue::Boolean(*b),
                Literal::Null => RuntimeValue::Null,
            }),
            Expression::Identifier(name) => {
                // Check builtins first
                match name.as_str() {
                    "print" | "println" | "len" | "assert" | "type_of" | "format" | "str"
                    | "int" | "float" | "range" | "input" | "format!" | "println!"
                    | "write_file" | "read_file" | "file_exists" | "shell_exec" => {
                        Ok(RuntimeValue::NativeFunction(name.clone()))
                    }
                    // Built-in type names — return as type descriptors
                    "String" | "Int" | "Float" | "Bool" | "Void" | "str" | "int" | "float"
                    | "bool" | "usize" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32"
                    | "i64" | "f32" | "f64" => Ok(RuntimeValue::String(name.clone())),
                    // Collection type names
                    "Vector" | "HashMap" | "HashSet" | "Vec" | "Map" | "Array" | "Option"
                    | "Result" | "Some" | "None" | "Ok" | "Err" => {
                        Ok(RuntimeValue::String(name.clone()))
                    }
                    // Ownership keywords — treat as identity (pass-through for the value)
                    "own" | "shared" => {
                        // These are ownership annotations; the next expression is the value
                        // Return a marker that the parser handles the actual semantics
                        Ok(RuntimeValue::String(name.clone()))
                    }
                    _ => self
                        .global_scope
                        .borrow()
                        .get(name)
                        .ok_or_else(|| anyhow!("Undefined variable: {}", name)),
                }
            }
            Expression::Binary(left, op, right) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.apply_binary_op(op, l, r)
            }
            Expression::Unary(op, operand) => {
                let val = self.eval_expr(operand)?;
                match op {
                    UnaryOp::Neg => match val {
                        RuntimeValue::Integer(i) => Ok(RuntimeValue::Integer(-i)),
                        RuntimeValue::Float(f) => Ok(RuntimeValue::Float(-f)),
                        _ => Err(anyhow!("Cannot negate {:?}", val)),
                    },
                    UnaryOp::Not => Ok(RuntimeValue::Boolean(!val.is_truthy())),
                }
            }
            Expression::Call(callee, args) => {
                let func_val = self.eval_expr(callee)?;
                // Ownership annotations: shared/own pass-through
                match &func_val {
                    RuntimeValue::String(s) if s == "shared" || s == "own" => {
                        if let Some(first) = args.first() {
                            return self.eval_expr(first);
                        }
                        return Ok(RuntimeValue::Null);
                    }
                    _ => {}
                }
                let mut eval_args = Vec::new();
                for arg in args {
                    eval_args.push(self.eval_expr(arg)?);
                }
                match func_val {
                    RuntimeValue::Function(func) => self.call_function_ast(&*func, &eval_args),
                    RuntimeValue::NativeFunction(name) => self.call_builtin(&name, &eval_args),
                    _ => Err(anyhow!("Cannot call {:?} as a function", func_val)),
                }
            }
            Expression::MethodCall {
                receiver,
                method,
                args,
            } => {
                let obj = self.eval_expr(receiver)?;
                let mut eval_args = Vec::new();
                for arg in args {
                    eval_args.push(self.eval_expr(arg)?);
                }
                self.last_mutated_self = None;
                let result = self.call_method(&obj, method, &eval_args)?;
                // Write back mutated self to the receiver variable
                if let Some(mutated) = self.last_mutated_self.take() {
                    if let Expression::Identifier(var_name) = receiver.as_ref() {
                        self.global_scope
                            .borrow_mut()
                            .set(var_name.clone(), mutated);
                    }
                }
                Ok(result)
            }
            Expression::Field(obj, field) => {
                let val = self.eval_expr(obj)?;
                match val {
                    RuntimeValue::Map(m) => m
                        .get(field)
                        .cloned()
                        .ok_or_else(|| anyhow!("No field '{}' on map", field)),
                    RuntimeValue::StructInstance { ref fields, .. } => fields
                        .get(field)
                        .cloned()
                        .ok_or_else(|| anyhow!("No field '{}' on struct", field)),
                    _ => Err(anyhow!("Cannot access field '{}' on {:?}", field, val)),
                }
            }
            Expression::Index(obj, idx) => {
                let val = self.eval_expr(obj)?;
                let index = self.eval_expr(idx)?;
                match (&val, &index) {
                    (RuntimeValue::Array(a), RuntimeValue::Integer(i))
                    | (RuntimeValue::List(a), RuntimeValue::Integer(i)) => {
                        let idx = if *i < 0 {
                            (a.len() as i64 + *i) as usize
                        } else {
                            *i as usize
                        };
                        a.get(idx)
                            .cloned()
                            .ok_or_else(|| anyhow!("Index {} out of bounds (len {})", i, a.len()))
                    }
                    (RuntimeValue::Map(m), RuntimeValue::String(k)) => m
                        .get(k)
                        .cloned()
                        .ok_or_else(|| anyhow!("Key '{}' not found", k)),
                    (RuntimeValue::String(s), RuntimeValue::Integer(i)) => {
                        let idx = if *i < 0 {
                            (s.len() as i64 + *i) as usize
                        } else {
                            *i as usize
                        };
                        s.chars()
                            .nth(idx)
                            .map(|c| RuntimeValue::String(c.to_string()))
                            .ok_or_else(|| anyhow!("String index {} out of bounds", i))
                    }
                    _ => Err(anyhow!("Cannot index {:?} with {:?}", val, index)),
                }
            }
            Expression::Array(items) => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.eval_expr(item)?);
                }
                Ok(RuntimeValue::Array(vals))
            }

            Expression::Tuple(items) => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.eval_expr(item)?);
                }
                Ok(RuntimeValue::Array(vals)) // Tuples as arrays for now
            }
            Expression::Lambda { params, body } => {
                let func = crate::parser::ast::Function {
                    name: "<lambda>".to_string(),
                    is_async: false,
                    attributes: vec![],
                    params: params.clone(),
                    return_type: None,
                    body: crate::parser::ast::Block {
                        statements: vec![crate::parser::ast::Statement::Return(Some(
                            *body.clone(),
                        ))],
                    },
                };
                Ok(RuntimeValue::Function(Box::new(func)))
            }
            Expression::StructLiteral { name, fields } => {
                // Create a struct instance with evaluated field values
                let type_name = name.clone();
                let mut field_map = HashMap::new();
                for (field_name, field_expr) in fields {
                    let val = self.eval_expr(field_expr)?;
                    field_map.insert(field_name.clone(), val);
                }
                Ok(RuntimeValue::StructInstance {
                    type_name,
                    fields: field_map,
                })
            }
            Expression::Path(expr, member_name) => {
                // Handle Struct::method() and Module::function() lookups
                let base = self.eval_expr(expr)?;
                match base {
                    RuntimeValue::StructDef {
                        ref name,
                        ref methods,
                        ..
                    } => {
                        // Look for a method (typically a static/associated function like new())
                        if let Some(method) = methods.iter().find(|m| m.name == *member_name) {
                            Ok(RuntimeValue::Function(Box::new(method.clone())))
                        } else {
                            Err(anyhow!(
                                "No static method '{}' on struct '{}'",
                                member_name,
                                name
                            ))
                        }
                    }
                    RuntimeValue::Module {
                        ref members,
                        ref name,
                    } => {
                        if let Some(member) = members.get(member_name) {
                            Ok(member.clone())
                        } else {
                            // Return a no-op native function for unknown module members
                            eprintln!(
                                "warning: unknown member '{}' on module '{}'",
                                member_name, name
                            );
                            Ok(RuntimeValue::NativeFunction(format!(
                                "__noop_{}",
                                member_name
                            )))
                        }
                    }
                    RuntimeValue::String(ref type_name) => {
                        // Handle type constructors: String::from(), Int::from(), etc.
                        match member_name.as_str() {
                            "from" | "new" | "from_utf8" | "from_bytes" => {
                                Ok(RuntimeValue::NativeFunction(format!(
                                    "{}::{}",
                                    type_name, member_name
                                )))
                            }
                            _ => Ok(RuntimeValue::NativeFunction(format!(
                                "{}::{}",
                                type_name, member_name
                            ))),
                        }
                    }
                    _ => Err(anyhow!(
                        "Cannot resolve path '::{}' on {:?}",
                        member_name,
                        base
                    )),
                }
            }
            Expression::Borrow { mutable: _, expr } => {
                // In interpreter mode, borrows are just identity (no actual pointer semantics)
                self.eval_expr(expr)
            }
            Expression::Deref(expr) => {
                // In interpreter mode, derefs are just identity
                self.eval_expr(expr)
            }
            Expression::Range {
                start,
                end,
                inclusive,
            } => {
                let s = if let Some(ref e) = start {
                    match self.eval_expr(e)? {
                        RuntimeValue::Integer(i) => i,
                        _ => 0,
                    }
                } else {
                    0
                };
                let e = if let Some(ref e) = end {
                    match self.eval_expr(e)? {
                        RuntimeValue::Integer(i) => i,
                        _ => 0,
                    }
                } else {
                    0
                };
                let items: Vec<RuntimeValue> = if *inclusive {
                    (s..=e).map(RuntimeValue::Integer).collect()
                } else {
                    (s..e).map(RuntimeValue::Integer).collect()
                };
                Ok(RuntimeValue::Array(items))
            }
            Expression::Await(expr) => {
                // In interpreter mode, await is just identity (no actual async)
                self.eval_expr(expr)
            }
            Expression::Shared(expr) => {
                // shared is ownership annotation; pass through the value
                self.eval_expr(expr)
            }
            Expression::Own(expr) => {
                // own is ownership annotation; pass through the value
                self.eval_expr(expr)
            }
            Expression::StructLiteral { name: _, fields } => {
                let mut field_map = HashMap::new();
                for (field_name, field_expr) in fields {
                    let val = self.eval_expr(field_expr)?;
                    field_map.insert(field_name.clone(), val);
                }
                Ok(RuntimeValue::Map(field_map))
            }
            _ => Err(anyhow!(
                "Expression type not yet supported: {:?}",
                std::mem::discriminant(expr)
            )),
        }
    }

    /// Call an AST function with given arguments
    fn call_function_ast(
        &mut self,
        func: &crate::parser::ast::Function,
        args: &[RuntimeValue],
    ) -> Result<RuntimeValue> {
        let func_scope = Rc::new(RefCell::new(Scope::with_parent(self.global_scope.clone())));

        // Bind arguments to parameters
        for (i, param) in func.params.iter().enumerate() {
            if param.name != "self" {
                let val = args.get(i).cloned().unwrap_or(RuntimeValue::Null);
                func_scope.borrow_mut().set(param.name.clone(), val);
            }
        }

        let saved_scope = self.global_scope.clone();
        self.global_scope = func_scope;

        let result = self.eval_block(&func.body);

        self.global_scope = saved_scope;

        match result {
            Ok(val) => Ok(val),
            Err(e) => {
                let msg = format!("{}", e);
                if msg == "__return__" {
                    Ok(self.return_value.take().unwrap_or(RuntimeValue::Null))
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Call a method function with `self` bound to the struct instance
    fn call_function_with_self(
        &mut self,
        func: &crate::parser::ast::Function,
        self_val: RuntimeValue,
        args: &[RuntimeValue],
    ) -> Result<RuntimeValue> {
        let func_scope = Rc::new(RefCell::new(Scope::with_parent(self.global_scope.clone())));

        // Bind self to the struct instance
        func_scope.borrow_mut().set("self".to_string(), self_val);

        // Bind arguments to non-self parameters
        let mut arg_idx = 0;
        for param in &func.params {
            if param.name != "self" {
                let val = args.get(arg_idx).cloned().unwrap_or(RuntimeValue::Null);
                func_scope.borrow_mut().set(param.name.clone(), val);
                arg_idx += 1;
            }
        }

        let saved_scope = self.global_scope.clone();
        self.global_scope = func_scope.clone();

        let result = self.eval_block(&func.body);

        // Capture the mutated self before restoring scope
        self.last_mutated_self = func_scope.borrow().get("self");

        self.global_scope = saved_scope;

        match result {
            Ok(val) => Ok(val),
            Err(e) => {
                let msg = format!("{}", e);
                if msg == "__return__" {
                    Ok(self.return_value.take().unwrap_or(RuntimeValue::Null))
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Call a builtin function
    fn call_builtin(&mut self, name: &str, args: &[RuntimeValue]) -> Result<RuntimeValue> {
        match name {
            "print" => {
                // Support format strings: print("Hello {}", name)
                if let Some(RuntimeValue::String(template)) = args.first() {
                    if template.contains("{}") && args.len() > 1 {
                        let mut result = template.clone();
                        for arg in &args[1..] {
                            if let Some(pos) = result.find("{}") {
                                result.replace_range(pos..pos + 2, &arg.display_string());
                            }
                        }
                        print!("{}", result);
                    } else {
                        let parts: Vec<String> = args.iter().map(|a| a.display_string()).collect();
                        print!("{}", parts.join(" "));
                    }
                } else {
                    let parts: Vec<String> = args.iter().map(|a| a.display_string()).collect();
                    print!("{}", parts.join(" "));
                }
                Ok(RuntimeValue::Null)
            }
            "println" | "println!" => {
                // Support format strings: println("Hello {}", name)
                if let Some(RuntimeValue::String(template)) = args.first() {
                    if template.contains("{}") && args.len() > 1 {
                        let mut result = template.clone();
                        for arg in &args[1..] {
                            if let Some(pos) = result.find("{}") {
                                result.replace_range(pos..pos + 2, &arg.display_string());
                            }
                        }
                        println!("{}", result);
                    } else {
                        let parts: Vec<String> = args.iter().map(|a| a.display_string()).collect();
                        println!("{}", parts.join(" "));
                    }
                } else {
                    let parts: Vec<String> = args.iter().map(|a| a.display_string()).collect();
                    println!("{}", parts.join(" "));
                }
                Ok(RuntimeValue::Null)
            }
            "len" => match args.first() {
                Some(RuntimeValue::String(s)) => Ok(RuntimeValue::Integer(s.len() as i64)),
                Some(RuntimeValue::Array(a)) | Some(RuntimeValue::List(a)) => {
                    Ok(RuntimeValue::Integer(a.len() as i64))
                }
                Some(RuntimeValue::Map(m)) => Ok(RuntimeValue::Integer(m.len() as i64)),
                _ => Err(anyhow!("len() requires a string, list, or map")),
            },
            "assert" => match args.first() {
                Some(val) if val.is_truthy() => Ok(RuntimeValue::Null),
                Some(val) => {
                    let msg = args
                        .get(1)
                        .map(|m| m.display_string())
                        .unwrap_or_else(|| format!("Assertion failed: {:?}", val));
                    Err(anyhow!("AssertionError: {}", msg))
                }
                None => Err(anyhow!("assert() requires at least one argument")),
            },
            "type_of" => match args.first() {
                Some(val) => Ok(RuntimeValue::String(val.type_name().to_string())),
                None => Err(anyhow!("type_of() requires one argument")),
            },
            "str" => match args.first() {
                Some(val) => Ok(RuntimeValue::String(val.display_string())),
                None => Ok(RuntimeValue::String(String::new())),
            },
            "int" => match args.first() {
                Some(RuntimeValue::Integer(i)) => Ok(RuntimeValue::Integer(*i)),
                Some(RuntimeValue::Float(f)) => Ok(RuntimeValue::Integer(*f as i64)),
                Some(RuntimeValue::String(s)) => s
                    .parse::<i64>()
                    .map(RuntimeValue::Integer)
                    .map_err(|_| anyhow!("Cannot convert '{}' to int", s)),
                Some(RuntimeValue::Boolean(b)) => Ok(RuntimeValue::Integer(if *b { 1 } else { 0 })),
                _ => Err(anyhow!("int() cannot convert given type")),
            },
            "float" => match args.first() {
                Some(RuntimeValue::Float(f)) => Ok(RuntimeValue::Float(*f)),
                Some(RuntimeValue::Integer(i)) => Ok(RuntimeValue::Float(*i as f64)),
                Some(RuntimeValue::String(s)) => s
                    .parse::<f64>()
                    .map(RuntimeValue::Float)
                    .map_err(|_| anyhow!("Cannot convert '{}' to float", s)),
                _ => Err(anyhow!("float() cannot convert given type")),
            },
            "range" => match (args.get(0), args.get(1), args.get(2)) {
                (Some(RuntimeValue::Integer(end)), None, None) => {
                    let items: Vec<RuntimeValue> = (0..*end).map(RuntimeValue::Integer).collect();
                    Ok(RuntimeValue::Array(items))
                }
                (Some(RuntimeValue::Integer(start)), Some(RuntimeValue::Integer(end)), None) => {
                    let items: Vec<RuntimeValue> =
                        (*start..*end).map(RuntimeValue::Integer).collect();
                    Ok(RuntimeValue::Array(items))
                }
                (
                    Some(RuntimeValue::Integer(start)),
                    Some(RuntimeValue::Integer(end)),
                    Some(RuntimeValue::Integer(step)),
                ) => {
                    let mut items = Vec::new();
                    let mut i = *start;
                    if *step > 0 {
                        while i < *end {
                            items.push(RuntimeValue::Integer(i));
                            i += step;
                        }
                    } else if *step < 0 {
                        while i > *end {
                            items.push(RuntimeValue::Integer(i));
                            i += step;
                        }
                    }
                    Ok(RuntimeValue::Array(items))
                }
                _ => Err(anyhow!("range() requires integer arguments")),
            },
            "format" | "format!" => {
                // format("template {} with {}", arg1, arg2) — replaces {} placeholders
                if let Some(RuntimeValue::String(template)) = args.first() {
                    let mut result = template.clone();
                    for arg in &args[1..] {
                        if let Some(pos) = result.find("{}") {
                            result.replace_range(pos..pos + 2, &arg.display_string());
                        }
                    }
                    Ok(RuntimeValue::String(result))
                } else {
                    let parts: Vec<String> = args.iter().map(|a| a.display_string()).collect();
                    Ok(RuntimeValue::String(parts.join("")))
                }
            }
            // Type constructors: String::from(), Int::from(), etc.
            name if name.starts_with("String::") => match name {
                "String::from" | "String::new" => {
                    if let Some(val) = args.first() {
                        Ok(RuntimeValue::String(val.display_string()))
                    } else {
                        Ok(RuntimeValue::String(String::new()))
                    }
                }
                _ => {
                    if let Some(val) = args.first() {
                        Ok(RuntimeValue::String(val.display_string()))
                    } else {
                        Ok(RuntimeValue::String(String::new()))
                    }
                }
            },
            // Collection constructors: Vector::new(), HashMap::new(), etc.
            name if name.starts_with("Vector::")
                || name.starts_with("Vec::")
                || name.starts_with("HashMap::")
                || name.starts_with("HashSet::")
                || name.starts_with("Array::")
                || name.starts_with("Map::") =>
            {
                match name {
                    "Vector::new" | "Vec::new" | "Array::new" => {
                        Ok(RuntimeValue::Array(Vec::new()))
                    }
                    "Vector::from" | "Vec::from" => {
                        if let Some(RuntimeValue::Array(items)) = args.first() {
                            Ok(RuntimeValue::Array(items.clone()))
                        } else {
                            Ok(RuntimeValue::Array(Vec::new()))
                        }
                    }
                    "HashMap::new" | "Map::new" => Ok(RuntimeValue::Map(HashMap::new())),
                    _ => Ok(RuntimeValue::Array(Vec::new())),
                }
            }
            "input" => {
                if let Some(prompt) = args.first() {
                    print!("{}", prompt.display_string());
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
                let mut line = String::new();
                std::io::stdin()
                    .read_line(&mut line)
                    .map_err(|e| anyhow!("Failed to read input: {}", e))?;
                Ok(RuntimeValue::String(line.trim_end().to_string()))
            }
            // ── Logging module functions ──
            "__log_debug" => {
                let msg = Self::format_log_args(args);
                eprintln!("[DEBUG] {}", msg);
                Ok(RuntimeValue::Null)
            }
            "__log_info" => {
                let msg = Self::format_log_args(args);
                eprintln!("[INFO] {}", msg);
                Ok(RuntimeValue::Null)
            }
            "__log_warn" => {
                let msg = Self::format_log_args(args);
                eprintln!("[WARN] {}", msg);
                Ok(RuntimeValue::Null)
            }
            "__log_error" => {
                let msg = Self::format_log_args(args);
                eprintln!("[ERROR] {}", msg);
                Ok(RuntimeValue::Null)
            }
            "__log_trace" => {
                let msg = Self::format_log_args(args);
                eprintln!("[TRACE] {}", msg);
                Ok(RuntimeValue::Null)
            }
            // ── Math module functions ──
            "__math_sqrt" => match args.first() {
                Some(RuntimeValue::Float(f)) => Ok(RuntimeValue::Float(f.sqrt())),
                Some(RuntimeValue::Integer(i)) => Ok(RuntimeValue::Float((*i as f64).sqrt())),
                _ => Err(anyhow!("sqrt() requires a numeric argument")),
            },
            "__math_abs" => match args.first() {
                Some(RuntimeValue::Float(f)) => Ok(RuntimeValue::Float(f.abs())),
                Some(RuntimeValue::Integer(i)) => Ok(RuntimeValue::Integer(i.abs())),
                _ => Err(anyhow!("abs() requires a numeric argument")),
            },
            "__math_pow" => match (args.get(0), args.get(1)) {
                (Some(RuntimeValue::Float(b)), Some(RuntimeValue::Float(e))) => {
                    Ok(RuntimeValue::Float(b.powf(*e)))
                }
                (Some(RuntimeValue::Integer(b)), Some(RuntimeValue::Integer(e))) => {
                    Ok(RuntimeValue::Integer(b.pow(*e as u32)))
                }
                (Some(RuntimeValue::Float(b)), Some(RuntimeValue::Integer(e))) => {
                    Ok(RuntimeValue::Float(b.powf(*e as f64)))
                }
                (Some(RuntimeValue::Integer(b)), Some(RuntimeValue::Float(e))) => {
                    Ok(RuntimeValue::Float((*b as f64).powf(*e)))
                }
                _ => Err(anyhow!("pow() requires two numeric arguments")),
            },
            "__math_min" => match (args.get(0), args.get(1)) {
                (Some(RuntimeValue::Integer(a)), Some(RuntimeValue::Integer(b))) => {
                    Ok(RuntimeValue::Integer(*a.min(b)))
                }
                (Some(RuntimeValue::Float(a)), Some(RuntimeValue::Float(b))) => {
                    Ok(RuntimeValue::Float(a.min(*b)))
                }
                _ => Err(anyhow!("min() requires two numeric arguments")),
            },
            "__math_max" => match (args.get(0), args.get(1)) {
                (Some(RuntimeValue::Integer(a)), Some(RuntimeValue::Integer(b))) => {
                    Ok(RuntimeValue::Integer(*a.max(b)))
                }
                (Some(RuntimeValue::Float(a)), Some(RuntimeValue::Float(b))) => {
                    Ok(RuntimeValue::Float(a.max(*b)))
                }
                _ => Err(anyhow!("max() requires two numeric arguments")),
            },
            "__math_floor" => match args.first() {
                Some(RuntimeValue::Float(f)) => Ok(RuntimeValue::Integer(f.floor() as i64)),
                Some(RuntimeValue::Integer(i)) => Ok(RuntimeValue::Integer(*i)),
                _ => Err(anyhow!("floor() requires a numeric argument")),
            },
            "__math_ceil" => match args.first() {
                Some(RuntimeValue::Float(f)) => Ok(RuntimeValue::Integer(f.ceil() as i64)),
                Some(RuntimeValue::Integer(i)) => Ok(RuntimeValue::Integer(*i)),
                _ => Err(anyhow!("ceil() requires a numeric argument")),
            },
            "__math_pi" => Ok(RuntimeValue::Float(std::f64::consts::PI)),
            // ── JSON module functions ──
            "__json_parse" => {
                match args.first() {
                    Some(RuntimeValue::String(s)) => {
                        // Simplified JSON parsing — return the string as-is for now
                        Ok(RuntimeValue::String(s.clone()))
                    }
                    _ => Err(anyhow!("json::parse() requires a string argument")),
                }
            }
            "__json_stringify" => match args.first() {
                Some(val) => Ok(RuntimeValue::String(val.display_string())),
                None => Err(anyhow!("json::stringify() requires an argument")),
            },
            _ if name.starts_with("__noop_") => {
                // No-op stub for unknown module functions
                Ok(RuntimeValue::Null)
            }
            // ── File I/O builtins ──
            "write_file" => {
                let path = args
                    .get(0)
                    .ok_or_else(|| anyhow!("write_file(path, content) needs path"))?
                    .display_string();
                let content = args
                    .get(1)
                    .ok_or_else(|| anyhow!("write_file(path, content) needs content"))?
                    .display_string();
                std::fs::write(&path, &content)
                    .map_err(|e| anyhow!("Cannot write '{}': {}", path, e))?;
                Ok(RuntimeValue::Null)
            }
            "read_file" => {
                let path = args
                    .get(0)
                    .ok_or_else(|| anyhow!("read_file(path) needs path"))?
                    .display_string();
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| anyhow!("Cannot read '{}': {}", path, e))?;
                Ok(RuntimeValue::String(content))
            }
            "file_exists" => {
                let path = args
                    .get(0)
                    .ok_or_else(|| anyhow!("file_exists(path) needs path"))?
                    .display_string();
                Ok(RuntimeValue::Boolean(std::path::Path::new(&path).exists()))
            }
            "shell_exec" => {
                let cmd = args
                    .get(0)
                    .ok_or_else(|| anyhow!("shell_exec(cmd) needs cmd"))?
                    .display_string();
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output()
                    .map_err(|e| anyhow!("Cannot exec '{}': {}", cmd, e))?;
                Ok(RuntimeValue::String(
                    String::from_utf8_lossy(&output.stdout).to_string(),
                ))
            }
            _ => Err(anyhow!("Unknown builtin function: {}", name)),
        }
    }

    /// Format log arguments with {} placeholder substitution
    fn format_log_args(args: &[RuntimeValue]) -> String {
        if args.is_empty() {
            return String::new();
        }
        if let RuntimeValue::String(template) = &args[0] {
            let mut result = template.clone();
            for arg in &args[1..] {
                if let Some(pos) = result.find("{}") {
                    result.replace_range(pos..pos + 2, &arg.display_string());
                }
            }
            result
        } else {
            args.iter()
                .map(|a| a.display_string())
                .collect::<Vec<_>>()
                .join(" ")
        }
    }

    /// Call a method on a runtime value
    fn call_method(
        &mut self,
        obj: &RuntimeValue,
        method: &str,
        args: &[RuntimeValue],
    ) -> Result<RuntimeValue> {
        match (obj, method) {
            (RuntimeValue::String(s), "len") => Ok(RuntimeValue::Integer(s.len() as i64)),
            (RuntimeValue::String(s), "upper") => Ok(RuntimeValue::String(s.to_uppercase())),
            (RuntimeValue::String(s), "lower") => Ok(RuntimeValue::String(s.to_lowercase())),
            (RuntimeValue::String(s), "trim") => Ok(RuntimeValue::String(s.trim().to_string())),
            (RuntimeValue::String(s), "contains") => match args.first() {
                Some(RuntimeValue::String(sub)) => {
                    Ok(RuntimeValue::Boolean(s.contains(sub.as_str())))
                }
                _ => Err(anyhow!("contains() requires a string argument")),
            },
            (RuntimeValue::String(s), "split") => match args.first() {
                Some(RuntimeValue::String(sep)) => {
                    let items: Vec<RuntimeValue> = s
                        .split(sep.as_str())
                        .map(|p| RuntimeValue::String(p.to_string()))
                        .collect();
                    Ok(RuntimeValue::Array(items))
                }
                _ => Err(anyhow!("split() requires a string separator")),
            },
            (RuntimeValue::String(s), "replace") => match (args.get(0), args.get(1)) {
                (Some(RuntimeValue::String(old)), Some(RuntimeValue::String(new))) => {
                    Ok(RuntimeValue::String(s.replace(old.as_str(), new.as_str())))
                }
                _ => Err(anyhow!("replace() requires two string arguments")),
            },
            (RuntimeValue::String(s), "starts_with") => match args.first() {
                Some(RuntimeValue::String(prefix)) => {
                    Ok(RuntimeValue::Boolean(s.starts_with(prefix.as_str())))
                }
                _ => Err(anyhow!("starts_with() requires a string argument")),
            },
            (RuntimeValue::String(s), "ends_with") => match args.first() {
                Some(RuntimeValue::String(suffix)) => {
                    Ok(RuntimeValue::Boolean(s.ends_with(suffix.as_str())))
                }
                _ => Err(anyhow!("ends_with() requires a string argument")),
            },
            (RuntimeValue::String(s), "push_str") => match args.first() {
                Some(RuntimeValue::String(extra)) => {
                    let mut new_s = s.clone();
                    new_s.push_str(extra);
                    self.last_mutated_self = Some(RuntimeValue::String(new_s));
                    Ok(RuntimeValue::Null)
                }
                _ => Err(anyhow!("push_str() requires a string argument")),
            },
            (RuntimeValue::String(s), "push") => match args.first() {
                Some(RuntimeValue::String(c)) => {
                    let mut new_s = s.clone();
                    new_s.push_str(c);
                    self.last_mutated_self = Some(RuntimeValue::String(new_s));
                    Ok(RuntimeValue::Null)
                }
                _ => Err(anyhow!("push() requires an argument")),
            },
            (RuntimeValue::String(s), "to_uppercase") => Ok(RuntimeValue::String(s.to_uppercase())),
            (RuntimeValue::String(s), "to_lowercase") => Ok(RuntimeValue::String(s.to_lowercase())),
            (RuntimeValue::String(s), "as_str") => Ok(RuntimeValue::String(s.clone())),
            (RuntimeValue::String(s), "is_empty") => Ok(RuntimeValue::Boolean(s.is_empty())),
            (RuntimeValue::Array(a), "len") | (RuntimeValue::List(a), "len") => {
                Ok(RuntimeValue::Integer(a.len() as i64))
            }
            (RuntimeValue::Array(a), "push") | (RuntimeValue::List(a), "push") => {
                let mut new_arr = a.clone();
                for arg in args {
                    new_arr.push(arg.clone());
                }
                self.last_mutated_self = Some(RuntimeValue::Array(new_arr.clone()));
                Ok(RuntimeValue::Array(new_arr))
            }
            (RuntimeValue::Array(a), "pop") | (RuntimeValue::List(a), "pop") => {
                let mut new_arr = a.clone();
                let val = new_arr.pop().unwrap_or(RuntimeValue::Null);
                Ok(val)
            }
            (RuntimeValue::Array(a), "get") | (RuntimeValue::List(a), "get") => {
                match args.first() {
                    Some(RuntimeValue::Integer(idx)) => {
                        let i = *idx as usize;
                        if i < a.len() {
                            Ok(a[i].clone())
                        } else {
                            Err(anyhow!("Index {} out of bounds (len={})", idx, a.len()))
                        }
                    }
                    _ => Err(anyhow!("get() requires an integer index")),
                }
            }
            (RuntimeValue::Array(a), "sort") | (RuntimeValue::List(a), "sort") => {
                let mut sorted = a.clone();
                sorted.sort_by(|x, y| match (x, y) {
                    (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => a.cmp(b),
                    (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (RuntimeValue::String(a), RuntimeValue::String(b)) => a.cmp(b),
                    _ => std::cmp::Ordering::Equal,
                });
                Ok(RuntimeValue::Array(sorted))
            }
            (RuntimeValue::Array(a), "is_empty") | (RuntimeValue::List(a), "is_empty") => {
                Ok(RuntimeValue::Boolean(a.is_empty()))
            }
            (RuntimeValue::Array(a), "iter") | (RuntimeValue::List(a), "iter") => {
                Ok(RuntimeValue::Array(a.clone()))
            }
            // Option/Result unwrap - just return the value or Null
            (_, "unwrap") => Ok(obj.clone()),
            (_, "ok") => Ok(obj.clone()),
            (_, "is_empty") => Ok(RuntimeValue::Boolean(false)),
            (_, "is_some") => Ok(RuntimeValue::Boolean(!matches!(obj, RuntimeValue::Null))),
            (_, "is_none") => Ok(RuntimeValue::Boolean(matches!(obj, RuntimeValue::Null))),
            (_, "clone") => Ok(obj.clone()),
            (RuntimeValue::Array(a), "map") | (RuntimeValue::List(a), "map") => {
                match args.first() {
                    Some(RuntimeValue::Function(func)) => {
                        let mut results = Vec::new();
                        for item in a {
                            results.push(self.call_function_ast(func, &[item.clone()])?);
                        }
                        Ok(RuntimeValue::Array(results))
                    }
                    _ => Err(anyhow!("map() requires a function argument")),
                }
            }
            (RuntimeValue::Array(a), "filter") | (RuntimeValue::List(a), "filter") => {
                match args.first() {
                    Some(RuntimeValue::Function(func)) => {
                        let mut results = Vec::new();
                        for item in a {
                            let result = self.call_function_ast(func, &[item.clone()])?;
                            if result.is_truthy() {
                                results.push(item.clone());
                            }
                        }
                        Ok(RuntimeValue::Array(results))
                    }
                    _ => Err(anyhow!("filter() requires a function argument")),
                }
            }
            (RuntimeValue::Map(m), "keys") => {
                let keys: Vec<RuntimeValue> =
                    m.keys().map(|k| RuntimeValue::String(k.clone())).collect();
                Ok(RuntimeValue::Array(keys))
            }
            (RuntimeValue::Map(m), "values") => {
                let vals: Vec<RuntimeValue> = m.values().cloned().collect();
                Ok(RuntimeValue::Array(vals))
            }
            (RuntimeValue::Map(m), "contains_key") => match args.first() {
                Some(RuntimeValue::String(k)) => Ok(RuntimeValue::Boolean(m.contains_key(k))),
                _ => Err(anyhow!("contains_key() requires a string argument")),
            },
            (RuntimeValue::Map(m), "insert") => {
                if let (Some(RuntimeValue::String(k)), Some(v)) = (args.get(0), args.get(1)) {
                    let mut new_map = m.clone();
                    new_map.insert(k.clone(), v.clone());
                    self.last_mutated_self = Some(RuntimeValue::Map(new_map));
                    Ok(RuntimeValue::Null)
                } else {
                    Err(anyhow!("insert() requires (key: string, value) arguments"))
                }
            }
            (RuntimeValue::Map(m), "get") => match args.first() {
                Some(RuntimeValue::String(k)) => {
                    Ok(m.get(k).cloned().unwrap_or(RuntimeValue::Null))
                }
                _ => Err(anyhow!("get() requires a string key")),
            },
            (RuntimeValue::Map(m), "len") => Ok(RuntimeValue::Integer(m.len() as i64)),
            (RuntimeValue::StructInstance { type_name, .. }, _) => {
                // Look up the struct definition to find the method
                let struct_def = self
                    .global_scope
                    .borrow()
                    .get(type_name)
                    .ok_or_else(|| anyhow!("Unknown struct type: {}", type_name))?;
                match struct_def {
                    RuntimeValue::StructDef { methods, name, .. } => {
                        if let Some(func) = methods.iter().find(|m| m.name == method) {
                            self.call_function_with_self(&func.clone(), obj.clone(), args)
                        } else {
                            Err(anyhow!("No method '{}' on struct '{}'", method, name))
                        }
                    }
                    _ => Err(anyhow!("'{}' is not a struct definition", type_name)),
                }
            }
            _ => Err(anyhow!(
                "No method '{}' on type {}",
                method,
                obj.type_name()
            )),
        }
    }

    fn apply_binary_op(
        &self,
        op: &crate::parser::ast::BinaryOp,
        left: RuntimeValue,
        right: RuntimeValue,
    ) -> Result<RuntimeValue> {
        use crate::parser::ast::BinaryOp;

        // Auto-promote Integer to Float when mixed
        let (left, right) = match (&left, &right) {
            (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => {
                (RuntimeValue::Float(*l as f64), RuntimeValue::Float(*r))
            }
            (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => {
                (RuntimeValue::Float(*l), RuntimeValue::Float(*r as f64))
            }
            _ => (left, right),
        };

        match (op, left, right) {
            // ── Arithmetic: Integer ──
            (BinaryOp::Add, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Integer(l.wrapping_add(r)))
            }
            (BinaryOp::Sub, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Integer(l.wrapping_sub(r)))
            }
            (BinaryOp::Mul, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Integer(l.wrapping_mul(r)))
            }
            (BinaryOp::Div, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                if r == 0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(RuntimeValue::Integer(l / r))
                }
            }
            (BinaryOp::Mod, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                if r == 0 {
                    Err(anyhow!("Modulo by zero"))
                } else {
                    Ok(RuntimeValue::Integer(l % r))
                }
            }

            // ── Arithmetic: Float ──
            (BinaryOp::Add, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Float(l + r))
            }
            (BinaryOp::Sub, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Float(l - r))
            }
            (BinaryOp::Mul, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Float(l * r))
            }
            (BinaryOp::Div, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                if r == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(RuntimeValue::Float(l / r))
                }
            }
            (BinaryOp::Mod, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                if r == 0.0 {
                    Err(anyhow!("Modulo by zero"))
                } else {
                    Ok(RuntimeValue::Float(l % r))
                }
            }

            // ── String concatenation ──
            (BinaryOp::Add, RuntimeValue::String(l), RuntimeValue::String(r)) => {
                Ok(RuntimeValue::String(format!("{}{}", l, r)))
            }
            (BinaryOp::Add, RuntimeValue::String(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::String(format!("{}{}", l, r)))
            }
            (BinaryOp::Add, RuntimeValue::String(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::String(format!("{}{}", l, r)))
            }

            // ── Comparison: Integer ──
            (BinaryOp::Eq, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Boolean(l == r))
            }
            (BinaryOp::NotEq, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Boolean(l != r))
            }
            (BinaryOp::Lt, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Boolean(l < r))
            }
            (BinaryOp::Gt, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Boolean(l > r))
            }
            (BinaryOp::LtEq, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Boolean(l <= r))
            }
            (BinaryOp::GtEq, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Boolean(l >= r))
            }

            // ── Comparison: Float ──
            (BinaryOp::Eq, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Boolean((l - r).abs() < f64::EPSILON))
            }
            (BinaryOp::NotEq, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Boolean((l - r).abs() >= f64::EPSILON))
            }
            (BinaryOp::Lt, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Boolean(l < r))
            }
            (BinaryOp::Gt, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Boolean(l > r))
            }
            (BinaryOp::LtEq, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Boolean(l <= r))
            }
            (BinaryOp::GtEq, RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Boolean(l >= r))
            }

            // ── Comparison: String ──
            (BinaryOp::Eq, RuntimeValue::String(l), RuntimeValue::String(r)) => {
                Ok(RuntimeValue::Boolean(l == r))
            }
            (BinaryOp::NotEq, RuntimeValue::String(l), RuntimeValue::String(r)) => {
                Ok(RuntimeValue::Boolean(l != r))
            }
            (BinaryOp::Lt, RuntimeValue::String(l), RuntimeValue::String(r)) => {
                Ok(RuntimeValue::Boolean(l < r))
            }
            (BinaryOp::Gt, RuntimeValue::String(l), RuntimeValue::String(r)) => {
                Ok(RuntimeValue::Boolean(l > r))
            }
            (BinaryOp::LtEq, RuntimeValue::String(l), RuntimeValue::String(r)) => {
                Ok(RuntimeValue::Boolean(l <= r))
            }
            (BinaryOp::GtEq, RuntimeValue::String(l), RuntimeValue::String(r)) => {
                Ok(RuntimeValue::Boolean(l >= r))
            }

            // ── Comparison: Boolean ──
            (BinaryOp::Eq, RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => {
                Ok(RuntimeValue::Boolean(l == r))
            }
            (BinaryOp::NotEq, RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => {
                Ok(RuntimeValue::Boolean(l != r))
            }

            // ── Equality with Null ──
            (BinaryOp::Eq, RuntimeValue::Null, RuntimeValue::Null) => {
                Ok(RuntimeValue::Boolean(true))
            }
            (BinaryOp::Eq, _, RuntimeValue::Null) | (BinaryOp::Eq, RuntimeValue::Null, _) => {
                Ok(RuntimeValue::Boolean(false))
            }
            (BinaryOp::NotEq, RuntimeValue::Null, RuntimeValue::Null) => {
                Ok(RuntimeValue::Boolean(false))
            }
            (BinaryOp::NotEq, _, RuntimeValue::Null) | (BinaryOp::NotEq, RuntimeValue::Null, _) => {
                Ok(RuntimeValue::Boolean(true))
            }

            // ── Logical ──
            (BinaryOp::And, RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => {
                Ok(RuntimeValue::Boolean(l && r))
            }
            (BinaryOp::Or, RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => {
                Ok(RuntimeValue::Boolean(l || r))
            }

            // ── Range ──
            (BinaryOp::Range, RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                let items: Vec<RuntimeValue> = (l..r).map(RuntimeValue::Integer).collect();
                Ok(RuntimeValue::Array(items))
            }

            _ => Err(anyhow!(
                "Unsupported binary operation {:?} on given operand types",
                op
            )),
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
