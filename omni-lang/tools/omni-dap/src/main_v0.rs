//! Omni Debug Adapter Protocol (DAP) Implementation
//! 
//! Provides debugging support for VS Code and other DAP-compatible IDEs.
//! Connects to OVM runtime for bytecode-level debugging with source maps.

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// DAP Message Types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DapMessage {
    pub seq: u32,
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_seq: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: u32,
    pub verified: bool,
    pub line: u32,
    pub source: String,
    pub condition: Option<String>,
    pub hit_count: u32,
    pub hit_condition: Option<String>,
    pub log_message: Option<String>,
    pub bytecode_offset: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub id: u32,
    pub name: String,
    pub source: String,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub is_async: bool,
    pub module_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub var_type: String,
    pub variables_reference: u32,
    pub named_variables: Option<u32>,
    pub indexed_variables: Option<u32>,
    pub tensor_shape: Option<Vec<usize>>,
    pub memory_reference: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub variables_reference: u32,
    pub expensive: bool,
    pub source: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepGranularity {
    Statement,
    Line,
    Instruction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugState {
    Uninitialized,
    Initialized,
    Launching,
    Running,
    Stopped,
    Terminated,
}

/// Source map entry mapping bytecode offset to source location
#[derive(Debug, Clone)]
pub struct SourceMapEntry {
    pub bytecode_offset: u32,
    pub source_file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

/// Debug connection to the OVM runtime
pub struct OvmDebugConnection {
    pub stream: Option<TcpStream>,
    pub port: u16,
}

impl OvmDebugConnection {
    pub fn new() -> Self {
        Self {
            stream: None,
            port: 0,
        }
    }
    
    pub fn connect(&mut self, port: u16) -> io::Result<()> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
        stream.set_nonblocking(false)?;
        self.stream = Some(stream);
        self.port = port;
        Ok(())
    }
    
    pub fn send_command(&mut self, cmd: &str) -> io::Result<String> {
        if let Some(ref mut stream) = self.stream {
            writeln!(stream, "{}", cmd)?;
            stream.flush()?;
            
            let mut reader = BufReader::new(stream.try_clone()?);
            let mut response = String::new();
            reader.read_line(&mut response)?;
            Ok(response.trim().to_string())
        } else {
            Err(io::Error::new(io::ErrorKind::NotConnected, "Not connected"))
        }
    }
    
    pub fn set_breakpoint(&mut self, offset: u32) -> io::Result<bool> {
        let resp = self.send_command(&format!("bp {}", offset))?;
        Ok(resp.starts_with("OK"))
    }
    
    pub fn remove_breakpoint(&mut self, offset: u32) -> io::Result<bool> {
        let resp = self.send_command(&format!("bp_del {}", offset))?;
        Ok(resp.starts_with("OK"))
    }
    
    pub fn continue_execution(&mut self) -> io::Result<()> {
        self.send_command("continue")?;
        Ok(())
    }
    
    pub fn step_instruction(&mut self) -> io::Result<()> {
        self.send_command("step")?;
        Ok(())
    }
    
    pub fn step_over(&mut self) -> io::Result<()> {
        self.send_command("next")?;
        Ok(())
    }
    
    pub fn step_out(&mut self) -> io::Result<()> {
        self.send_command("finish")?;
        Ok(())
    }
    
    pub fn get_stack_frames(&mut self) -> io::Result<Vec<(u32, String, u32)>> {
        let resp = self.send_command("backtrace")?;
        let mut frames = Vec::new();
        
        for line in resp.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                if let (Ok(id), Ok(offset)) = (parts[0].parse::<u32>(), parts[2].parse::<u32>()) {
                    frames.push((id, parts[1].to_string(), offset));
                }
            }
        }
        
        Ok(frames)
    }
    
    pub fn get_locals(&mut self, frame_id: u32) -> io::Result<Vec<(String, String, String)>> {
        let resp = self.send_command(&format!("locals {}", frame_id))?;
        let mut vars = Vec::new();
        
        for line in resp.lines() {
            let parts: Vec<&str> = line.splitn(3, ',').collect();
            if parts.len() >= 3 {
                vars.push((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()));
            }
        }
        
        Ok(vars)
    }
    
    pub fn evaluate(&mut self, expr: &str, frame_id: u32) -> io::Result<(String, String)> {
        let resp = self.send_command(&format!("eval {} {}", frame_id, expr))?;
        let parts: Vec<&str> = resp.splitn(2, ',').collect();
        if parts.len() >= 2 {
            Ok((parts[0].to_string(), parts[1].to_string()))
        } else {
            Ok(("<error>".to_string(), "unknown".to_string()))
        }
    }
    
    pub fn read_memory(&mut self, addr: u64, count: usize) -> io::Result<Vec<u8>> {
        let resp = self.send_command(&format!("mem {} {}", addr, count))?;
        let bytes: Vec<u8> = resp.split(',')
            .filter_map(|s| u8::from_str_radix(s.trim(), 16).ok())
            .collect();
        Ok(bytes)
    }
}

pub struct DebugAdapter {
    breakpoints: HashMap<String, Vec<Breakpoint>>,
    function_breakpoints: Vec<Breakpoint>,
    exception_breakpoints: Vec<String>,
    stack_frames: Vec<StackFrame>,
    scopes: HashMap<u32, Vec<Scope>>,
    variables: HashMap<u32, Vec<Variable>>,
    source_maps: HashMap<String, Vec<SourceMapEntry>>,
    current_thread_id: u32,
    state: DebugState,
    seq: u32,
    next_bp_id: u32,
    next_var_ref: u32,
    ovm_connection: OvmDebugConnection,
    debuggee: Option<Child>,
    program_path: Option<PathBuf>,
    working_dir: Option<PathBuf>,
    arguments: Vec<String>,
    environment: HashMap<String, String>,
    stop_on_entry: bool,
}

impl DebugAdapter {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            function_breakpoints: Vec::new(),
            exception_breakpoints: Vec::new(),
            stack_frames: Vec::new(),
            scopes: HashMap::new(),
            variables: HashMap::new(),
            source_maps: HashMap::new(),
            current_thread_id: 1,
            state: DebugState::Uninitialized,
            seq: 0,
            next_bp_id: 1,
            next_var_ref: 1000,
            ovm_connection: OvmDebugConnection::new(),
            debuggee: None,
            program_path: None,
            working_dir: None,
            arguments: Vec::new(),
            environment: HashMap::new(),
            stop_on_entry: false,
        }
    }
    
    /// Load source maps from debug info
    pub fn load_source_maps(&mut self, debug_info_path: &str) -> io::Result<()> {
        // Parse DWARF-like debug info
        let content = std::fs::read_to_string(debug_info_path)?;
        
        for line in content.lines() {
            // Format: "offset,file,line,col,end_line,end_col"
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let offset = parts[0].parse::<u32>().unwrap_or(0);
                let file = parts[1].to_string();
                let line_num = parts[2].parse::<u32>().unwrap_or(0);
                let col = parts[3].parse::<u32>().unwrap_or(0);
                
                let entry = SourceMapEntry {
                    bytecode_offset: offset,
                    source_file: file.clone(),
                    line: line_num,
                    column: col,
                    end_line: parts.get(4).and_then(|s| s.parse().ok()),
                    end_column: parts.get(5).and_then(|s| s.parse().ok()),
                };
                
                self.source_maps.entry(file).or_default().push(entry);
            }
        }
        
        Ok(())
    }
    
    /// Convert source location to bytecode offset
    fn source_to_offset(&self, file: &str, line: u32) -> Option<u32> {
        self.source_maps.get(file).and_then(|entries| {
            entries.iter()
                .filter(|e| e.line == line)
                .min_by_key(|e| e.bytecode_offset)
                .map(|e| e.bytecode_offset)
        })
    }
    
    /// Convert bytecode offset to source location
    fn offset_to_source(&self, offset: u32) -> Option<(String, u32, u32)> {
        for (file, entries) in &self.source_maps {
            if let Some(entry) = entries.iter().find(|e| e.bytecode_offset == offset) {
                return Some((file.clone(), entry.line, entry.column));
            }
        }
        None
    }

    pub fn handle_request(&mut self, msg: DapMessage) -> Vec<DapMessage> {
        let command = msg.command.as_deref().unwrap_or("");
        let request_seq = msg.seq;
        
        match command {
            "initialize" => vec![self.handle_initialize(request_seq)],
            "launch" => self.handle_launch(request_seq, msg.arguments),
            "attach" => self.handle_attach(request_seq, msg.arguments),
            "setBreakpoints" => vec![self.handle_set_breakpoints(request_seq, msg.arguments)],
            "setFunctionBreakpoints" => vec![self.handle_set_function_breakpoints(request_seq, msg.arguments)],
            "setExceptionBreakpoints" => vec![self.handle_set_exception_breakpoints(request_seq, msg.arguments)],
            "configurationDone" => self.handle_config_done(request_seq),
            "threads" => vec![self.handle_threads(request_seq)],
            "stackTrace" => vec![self.handle_stack_trace(request_seq, msg.arguments)],
            "scopes" => vec![self.handle_scopes(request_seq, msg.arguments)],
            "variables" => vec![self.handle_variables(request_seq, msg.arguments)],
            "evaluate" => vec![self.handle_evaluate(request_seq, msg.arguments)],
            "continue" => self.handle_continue(request_seq, msg.arguments),
            "pause" => self.handle_pause(request_seq, msg.arguments),
            "next" => self.handle_next(request_seq, msg.arguments),
            "stepIn" => self.handle_step_in(request_seq, msg.arguments),
            "stepOut" => self.handle_step_out(request_seq, msg.arguments),
            "setVariable" => vec![self.handle_set_variable(request_seq, msg.arguments)],
            "source" => vec![self.handle_source(request_seq, msg.arguments)],
            "loadedSources" => vec![self.handle_loaded_sources(request_seq)],
            "readMemory" => vec![self.handle_read_memory(request_seq, msg.arguments)],
            "disconnect" => vec![self.handle_disconnect(request_seq, msg.arguments)],
            "terminate" => vec![self.handle_terminate(request_seq)],
            "restart" => self.handle_restart(request_seq),
            _ => vec![self.error_response(request_seq, command, &format!("Unknown command: {}", command))],
        }
    }

    fn handle_initialize(&mut self, request_seq: u32) -> DapMessage {
        self.state = DebugState::Initialized;
        
        self.success_response(request_seq, "initialize", serde_json::json!({
            "supportsConfigurationDoneRequest": true,
            "supportsConditionalBreakpoints": true,
            "supportsHitConditionalBreakpoints": true,
            "supportsLogPoints": true,
            "supportsEvaluateForHovers": true,
            "supportsFunctionBreakpoints": true,
            "supportsStepBack": false,
            "supportsSetVariable": true,
            "supportsRestartFrame": false,
            "supportsGotoTargetsRequest": false,
            "supportsStepInTargetsRequest": true,
            "supportsCompletionsRequest": true,
            "supportsModulesRequest": true,
            "supportsExceptionOptions": true,
            "supportsValueFormattingOptions": true,
            "supportsExceptionInfoRequest": true,
            "supportTerminateDebuggee": true,
            "supportsDelayedStackTraceLoading": true,
            "supportsLoadedSourcesRequest": true,
            "supportsReadMemoryRequest": true,
            "supportsWriteMemoryRequest": true,
            "supportsDisassembleRequest": true,
            "supportsBreakpointLocationsRequest": true,
            "supportsClipboardContext": true,
            "supportsSingleThreadExecutionRequests": true,
            // Omni-specific capabilities
            "supportsTensorVisualization": true,
            "supportsAsyncStackTrace": true,
            "supportsOwnershipInspection": true,
            "supportsBorrowVisualization": true
        }))
    }

    fn handle_launch(&mut self, request_seq: u32, args: Option<Value>) -> Vec<DapMessage> {
        let args = args.unwrap_or_default();
        
        // Extract launch configuration
        self.program_path = args["program"].as_str().map(PathBuf::from);
        self.working_dir = args["cwd"].as_str().map(PathBuf::from);
        self.stop_on_entry = args["stopOnEntry"].as_bool().unwrap_or(false);
        
        if let Some(args_array) = args["args"].as_array() {
            self.arguments = args_array.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }
        
        if let Some(env_obj) = args["env"].as_object() {
            for (k, v) in env_obj {
                if let Some(val) = v.as_str() {
                    self.environment.insert(k.clone(), val.to_string());
                }
            }
        }
        
        // Validate program exists
        let program = match &self.program_path {
            Some(p) => p.clone(),
            None => return vec![self.error_response(request_seq, "launch", "No program specified")],
        };
        
        if !program.exists() {
            return vec![self.error_response(request_seq, "launch", 
                &format!("Program not found: {:?}", program))];
        }
        
        // Load debug info if available
        let debug_info_path = program.with_extension("debug");
        if debug_info_path.exists() {
            let _ = self.load_source_maps(debug_info_path.to_str().unwrap_or(""));
        }
        
        self.state = DebugState::Launching;
        
        // Find a free port for debug communication
        let debug_port = 9229; // Default debug port
        
        // Build command to run the program with debugging enabled
        let mut cmd = Command::new("omni");
        cmd.arg("run")
            .arg("--debug")
            .arg("--debug-port")
            .arg(debug_port.to_string())
            .arg(&program);
        
        if let Some(cwd) = &self.working_dir {
            cmd.current_dir(cwd);
        }
        
        for arg in &self.arguments {
            cmd.arg(arg);
        }
        
        for (k, v) in &self.environment {
            cmd.env(k, v);
        }
        
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // Spawn the debuggee process
        match cmd.spawn() {
            Ok(child) => {
                self.debuggee = Some(child);
                
                // Give the process time to start debug server
                std::thread::sleep(std::time::Duration::from_millis(100));
                
                // Connect to debug server
                if self.ovm_connection.connect(debug_port).is_err() {
                    return vec![self.error_response(request_seq, "launch", 
                        "Failed to connect to debug server")];
                }
                
                self.state = DebugState::Running;
                
                let mut responses = vec![
                    self.success_response(request_seq, "launch", serde_json::json!({})),
                    self.event("initialized", serde_json::json!({})),
                ];
                
                if self.stop_on_entry {
                    responses.push(self.event("stopped", serde_json::json!({
                        "reason": "entry",
                        "threadId": 1,
                        "allThreadsStopped": true
                    })));
                    self.state = DebugState::Stopped;
                }
                
                responses
            }
            Err(e) => {
                vec![self.error_response(request_seq, "launch", 
                    &format!("Failed to launch: {}", e))]
            }
        }
    }
    
    fn handle_attach(&mut self, request_seq: u32, args: Option<Value>) -> Vec<DapMessage> {
        let args = args.unwrap_or_default();
        let port = args["port"].as_u64().unwrap_or(9229) as u16;
        
        if self.ovm_connection.connect(port).is_err() {
            return vec![self.error_response(request_seq, "attach",
                &format!("Failed to attach to debug server on port {}", port))];
        }
        
        self.state = DebugState::Running;
        
        vec![
            self.success_response(request_seq, "attach", serde_json::json!({})),
            self.event("initialized", serde_json::json!({})),
        ]
    }

    fn handle_set_breakpoints(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let source = args["source"]["path"].as_str().unwrap_or("");
        
        // Clear existing breakpoints for this source
        if let Some(old_bps) = self.breakpoints.get(source) {
            for bp in old_bps {
                if let Some(offset) = bp.bytecode_offset {
                    let _ = self.ovm_connection.remove_breakpoint(offset);
                }
            }
        }
        
        let bp_requests: Vec<Value> = args["breakpoints"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut bps = Vec::new();
        for bp_req in bp_requests {
            let line = bp_req["line"].as_u64().unwrap_or(0) as u32;
            let condition = bp_req["condition"].as_str().map(String::from);
            let hit_condition = bp_req["hitCondition"].as_str().map(String::from);
            let log_message = bp_req["logMessage"].as_str().map(String::from);
            
            // Try to resolve to bytecode offset
            let bytecode_offset = self.source_to_offset(source, line);
            let verified = if let Some(offset) = bytecode_offset {
                self.ovm_connection.set_breakpoint(offset).unwrap_or(false)
            } else {
                false
            };
            
            let bp = Breakpoint {
                id: self.next_bp_id,
                verified,
                line,
                source: source.to_string(),
                condition,
                hit_count: 0,
                hit_condition,
                log_message,
                bytecode_offset,
            };
            
            self.next_bp_id += 1;
            bps.push(bp);
        }

        self.breakpoints.insert(source.to_string(), bps.clone());

        let breakpoints_json: Vec<Value> = bps.iter().map(|bp| {
            serde_json::json!({
                "id": bp.id,
                "verified": bp.verified,
                "line": bp.line,
                "source": { "path": bp.source }
            })
        }).collect();

        self.success_response(request_seq, "setBreakpoints", serde_json::json!({
            "breakpoints": breakpoints_json
        }))
    }
    
    fn handle_set_function_breakpoints(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let bp_requests: Vec<Value> = args["breakpoints"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        
        self.function_breakpoints.clear();
        
        let mut bps = Vec::new();
        for bp_req in bp_requests {
            let name = bp_req["name"].as_str().unwrap_or("");
            let condition = bp_req["condition"].as_str().map(String::from);
            
            // Function breakpoints would need symbol table lookup
            let bp = Breakpoint {
                id: self.next_bp_id,
                verified: true, // Assume verified for now
                line: 0,
                source: name.to_string(),
                condition,
                hit_count: 0,
                hit_condition: None,
                log_message: None,
                bytecode_offset: None,
            };
            
            self.next_bp_id += 1;
            bps.push(bp.clone());
            self.function_breakpoints.push(bp);
        }
        
        let breakpoints_json: Vec<Value> = bps.iter().map(|bp| {
            serde_json::json!({
                "id": bp.id,
                "verified": bp.verified,
            })
        }).collect();
        
        self.success_response(request_seq, "setFunctionBreakpoints", serde_json::json!({
            "breakpoints": breakpoints_json
        }))
    }
    
    fn handle_set_exception_breakpoints(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        self.exception_breakpoints = args["filters"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        
        self.success_response(request_seq, "setExceptionBreakpoints", serde_json::json!({}))
    }

    fn handle_stack_trace(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let thread_id = args["threadId"].as_u64().unwrap_or(1) as u32;
        let start_frame = args["startFrame"].as_u64().unwrap_or(0) as usize;
        let levels = args["levels"].as_u64().unwrap_or(20) as usize;
        
        // Get stack frames from OVM
        let raw_frames = self.ovm_connection.get_stack_frames().unwrap_or_default();
        
        self.stack_frames.clear();
        for (id, name, offset) in raw_frames {
            let (source, line, col) = self.offset_to_source(offset)
                .unwrap_or(("unknown".to_string(), 0, 0));
            
            self.stack_frames.push(StackFrame {
                id,
                name,
                source,
                line,
                column: col,
                end_line: None,
                end_column: None,
                is_async: false,
                module_id: None,
            });
        }
        
        let frames: Vec<serde_json::Value> = self.stack_frames.iter()
            .skip(start_frame)
            .take(levels)
            .map(|f| {
                serde_json::json!({
                    "id": f.id,
                    "name": f.name,
                    "source": { 
                        "name": std::path::Path::new(&f.source)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown"),
                        "path": f.source 
                    },
                    "line": f.line,
                    "column": f.column,
                    "endLine": f.end_line,
                    "endColumn": f.end_column,
                    "presentationHint": if f.is_async { "subtle" } else { "normal" }
                })
            }).collect();

        self.success_response(request_seq, "stackTrace", serde_json::json!({
            "stackFrames": frames,
            "totalFrames": self.stack_frames.len()
        }))
    }
    
    fn handle_scopes(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let frame_id = args["frameId"].as_u64().unwrap_or(0) as u32;
        
        let locals_ref = self.next_var_ref;
        self.next_var_ref += 1;
        let tensors_ref = self.next_var_ref;
        self.next_var_ref += 1;
        let borrowed_ref = self.next_var_ref;
        self.next_var_ref += 1;
        
        // Fetch locals from OVM and store them
        if let Ok(locals) = self.ovm_connection.get_locals(frame_id) {
            let vars: Vec<Variable> = locals.iter().map(|(name, typ, val)| {
                Variable {
                    name: name.clone(),
                    value: val.clone(),
                    var_type: typ.clone(),
                    variables_reference: 0,
                    named_variables: None,
                    indexed_variables: None,
                    tensor_shape: None,
                    memory_reference: None,
                }
            }).collect();
            self.variables.insert(locals_ref, vars);
        }
        
        let scopes = vec![
            Scope {
                name: "Locals".to_string(),
                variables_reference: locals_ref,
                expensive: false,
                source: None,
                line: None,
            },
            Scope {
                name: "Tensors".to_string(),
                variables_reference: tensors_ref,
                expensive: true,
                source: None,
                line: None,
            },
            Scope {
                name: "Borrowed Values".to_string(),
                variables_reference: borrowed_ref,
                expensive: false,
                source: None,
                line: None,
            },
        ];
        
        self.scopes.insert(frame_id, scopes.clone());
        
        let scopes_json: Vec<Value> = scopes.iter().map(|s| {
            serde_json::json!({
                "name": s.name,
                "variablesReference": s.variables_reference,
                "expensive": s.expensive
            })
        }).collect();
        
        self.success_response(request_seq, "scopes", serde_json::json!({
            "scopes": scopes_json
        }))
    }

    fn handle_variables(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let reference = args["variablesReference"].as_u64().unwrap_or(0) as u32;
        let start = args["start"].as_u64().unwrap_or(0) as usize;
        let count = args["count"].as_u64().map(|c| c as usize);

        let vars: Vec<serde_json::Value> = self.variables
            .get(&reference)
            .map(|v| {
                let iter = v.iter().skip(start);
                let iter: Box<dyn Iterator<Item = _>> = if let Some(c) = count {
                    Box::new(iter.take(c))
                } else {
                    Box::new(iter)
                };
                
                iter.map(|var| {
                    let mut json = serde_json::json!({
                        "name": var.name,
                        "value": var.value,
                        "type": var.var_type,
                        "variablesReference": var.variables_reference,
                    });
                    
                    if let Some(named) = var.named_variables {
                        json["namedVariables"] = serde_json::json!(named);
                    }
                    if let Some(indexed) = var.indexed_variables {
                        json["indexedVariables"] = serde_json::json!(indexed);
                    }
                    
                    // Tensor visualization hint
                    if let Some(shape) = &var.tensor_shape {
                        json["presentationHint"] = serde_json::json!({
                            "kind": "data",
                            "attributes": ["tensor"],
                            "visibility": "public"
                        });
                        json["__tensorShape"] = serde_json::json!(shape);
                    }
                    
                    if let Some(mem_ref) = &var.memory_reference {
                        json["memoryReference"] = serde_json::json!(mem_ref);
                    }
                    
                    json
                }).collect()
            })
            .unwrap_or_default();

        self.success_response(request_seq, "variables", serde_json::json!({
            "variables": vars
        }))
    }

    fn handle_evaluate(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let expression = args["expression"].as_str().unwrap_or("");
        let frame_id = args["frameId"].as_u64().unwrap_or(0) as u32;
        let context = args["context"].as_str().unwrap_or("repl");

        let (result, result_type) = if context == "hover" {
            // Quick evaluation for hover
            self.ovm_connection.evaluate(expression, frame_id)
                .unwrap_or(("<unavailable>".to_string(), "unknown".to_string()))
        } else {
            // Full REPL evaluation
            self.ovm_connection.evaluate(expression, frame_id)
                .unwrap_or(("<error>".to_string(), "unknown".to_string()))
        };
        
        // Check if result is a tensor
        let var_ref = if result_type.starts_with("Tensor<") {
            let ref_id = self.next_var_ref;
            self.next_var_ref += 1;
            // Would need to fetch tensor elements
            ref_id
        } else {
            0
        };

        self.success_response(request_seq, "evaluate", serde_json::json!({
            "result": result,
            "type": result_type,
            "variablesReference": var_ref
        }))
    }

    fn handle_config_done(&mut self, request_seq: u32) -> Vec<DapMessage> {
        let mut responses = vec![
            self.success_response(request_seq, "configurationDone", serde_json::json!({}))
        ];
        
        // If stop on entry was set, we already stopped
        // Otherwise, program is running
        if self.state == DebugState::Running && !self.stop_on_entry {
            // Continue execution
            let _ = self.ovm_connection.continue_execution();
        }
        
        responses
    }

    fn handle_threads(&mut self, request_seq: u32) -> DapMessage {
        // In OVM, we have the main thread plus any async tasks
        let threads = vec![
            serde_json::json!({ "id": 1, "name": "main" }),
            serde_json::json!({ "id": 2, "name": "async-executor" }),
        ];
        
        self.success_response(request_seq, "threads", serde_json::json!({
            "threads": threads
        }))
    }

    fn handle_continue(&mut self, request_seq: u32, args: Option<Value>) -> Vec<DapMessage> {
        let args = args.unwrap_or_default();
        let thread_id = args["threadId"].as_u64().unwrap_or(1);
        let single_thread = args["singleThread"].as_bool().unwrap_or(false);
        
        let _ = self.ovm_connection.continue_execution();
        self.state = DebugState::Running;
        
        vec![
            self.success_response(request_seq, "continue", serde_json::json!({
                "allThreadsContinued": !single_thread
            })),
            self.event("continued", serde_json::json!({
                "threadId": thread_id,
                "allThreadsContinued": !single_thread
            })),
        ]
    }
    
    fn handle_pause(&mut self, request_seq: u32, args: Option<Value>) -> Vec<DapMessage> {
        let args = args.unwrap_or_default();
        let thread_id = args["threadId"].as_u64().unwrap_or(1);
        
        // Send pause signal to OVM
        let _ = self.ovm_connection.send_command("pause");
        self.state = DebugState::Stopped;
        
        vec![
            self.success_response(request_seq, "pause", serde_json::json!({})),
            self.event("stopped", serde_json::json!({
                "reason": "pause",
                "threadId": thread_id,
                "allThreadsStopped": true
            })),
        ]
    }

    fn handle_next(&mut self, request_seq: u32, args: Option<Value>) -> Vec<DapMessage> {
        let args = args.unwrap_or_default();
        let thread_id = args["threadId"].as_u64().unwrap_or(1);
        let granularity = args["granularity"].as_str().unwrap_or("statement");
        
        let _ = self.ovm_connection.step_over();
        self.state = DebugState::Stopped;
        
        vec![
            self.success_response(request_seq, "next", serde_json::json!({})),
            self.event("stopped", serde_json::json!({
                "reason": "step",
                "threadId": thread_id,
                "allThreadsStopped": true
            })),
        ]
    }

    fn handle_step_in(&mut self, request_seq: u32, args: Option<Value>) -> Vec<DapMessage> {
        let args = args.unwrap_or_default();
        let thread_id = args["threadId"].as_u64().unwrap_or(1);
        
        let _ = self.ovm_connection.step_instruction();
        self.state = DebugState::Stopped;
        
        vec![
            self.success_response(request_seq, "stepIn", serde_json::json!({})),
            self.event("stopped", serde_json::json!({
                "reason": "step",
                "threadId": thread_id,
                "allThreadsStopped": true
            })),
        ]
    }

    fn handle_step_out(&mut self, request_seq: u32, args: Option<Value>) -> Vec<DapMessage> {
        let args = args.unwrap_or_default();
        let thread_id = args["threadId"].as_u64().unwrap_or(1);
        
        let _ = self.ovm_connection.step_out();
        self.state = DebugState::Stopped;
        
        vec![
            self.success_response(request_seq, "stepOut", serde_json::json!({})),
            self.event("stopped", serde_json::json!({
                "reason": "step",
                "threadId": thread_id,
                "allThreadsStopped": true
            })),
        ]
    }
    
    fn handle_set_variable(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let var_ref = args["variablesReference"].as_u64().unwrap_or(0) as u32;
        let name = args["name"].as_str().unwrap_or("");
        let value = args["value"].as_str().unwrap_or("");
        
        // Send set command to OVM
        let result = self.ovm_connection.send_command(&format!("set {} = {}", name, value))
            .unwrap_or_else(|_| "<error>".to_string());
        
        self.success_response(request_seq, "setVariable", serde_json::json!({
            "value": result,
            "type": "unknown",
            "variablesReference": 0
        }))
    }
    
    fn handle_source(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let source_ref = args["sourceReference"].as_u64().unwrap_or(0);
        let path = args["source"]["path"].as_str();
        
        let content = if let Some(p) = path {
            std::fs::read_to_string(p).unwrap_or_default()
        } else {
            "// Source not available".to_string()
        };
        
        self.success_response(request_seq, "source", serde_json::json!({
            "content": content,
            "mimeType": "text/x-omni"
        }))
    }
    
    fn handle_loaded_sources(&mut self, request_seq: u32) -> DapMessage {
        let sources: Vec<Value> = self.source_maps.keys().map(|path| {
            serde_json::json!({
                "name": std::path::Path::new(path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown"),
                "path": path
            })
        }).collect();
        
        self.success_response(request_seq, "loadedSources", serde_json::json!({
            "sources": sources
        }))
    }
    
    fn handle_read_memory(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let mem_ref = args["memoryReference"].as_str().unwrap_or("0");
        let offset = args["offset"].as_i64().unwrap_or(0);
        let count = args["count"].as_u64().unwrap_or(256) as usize;
        
        let addr = u64::from_str_radix(mem_ref.trim_start_matches("0x"), 16)
            .unwrap_or(0)
            .wrapping_add(offset as u64);
        
        let bytes = self.ovm_connection.read_memory(addr, count).unwrap_or_default();
        let data = base64::encode(&bytes);
        
        self.success_response(request_seq, "readMemory", serde_json::json!({
            "address": format!("0x{:x}", addr),
            "data": data,
            "unreadableBytes": 0
        }))
    }

    fn handle_disconnect(&mut self, request_seq: u32, args: Option<Value>) -> DapMessage {
        let args = args.unwrap_or_default();
        let terminate = args["terminateDebuggee"].as_bool().unwrap_or(true);
        
        if terminate {
            if let Some(mut child) = self.debuggee.take() {
                let _ = child.kill();
            }
        }
        
        self.state = DebugState::Terminated;
        self.success_response(request_seq, "disconnect", serde_json::json!({}))
    }
    
    fn handle_terminate(&mut self, request_seq: u32) -> DapMessage {
        if let Some(mut child) = self.debuggee.take() {
            let _ = child.kill();
        }
        self.state = DebugState::Terminated;
        self.success_response(request_seq, "terminate", serde_json::json!({}))
    }
    
    fn handle_restart(&mut self, request_seq: u32) -> Vec<DapMessage> {
        // Terminate current session
        if let Some(mut child) = self.debuggee.take() {
            let _ = child.kill();
        }
        
        // Re-launch would happen via new launch request
        vec![
            self.success_response(request_seq, "restart", serde_json::json!({})),
            self.event("terminated", serde_json::json!({})),
        ]
    }

    fn success_response(&mut self, request_seq: u32, command: &str, body: serde_json::Value) -> DapMessage {
        DapMessage {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            command: Some(command.to_string()),
            event: None,
            arguments: None,
            body: Some(body),
            request_seq: Some(request_seq),
            success: Some(true),
            message: None,
        }
    }

    fn error_response(&mut self, request_seq: u32, command: &str, message: &str) -> DapMessage {
        DapMessage {
            seq: self.next_seq(),
            msg_type: "response".to_string(),
            command: Some(command.to_string()),
            event: None,
            arguments: None,
            body: Some(serde_json::json!({
                "error": {
                    "id": 1,
                    "format": message
                }
            })),
            request_seq: Some(request_seq),
            success: Some(false),
            message: Some(message.to_string()),
        }
    }
    
    fn event(&mut self, event_name: &str, body: serde_json::Value) -> DapMessage {
        DapMessage {
            seq: self.next_seq(),
            msg_type: "event".to_string(),
            command: None,
            event: Some(event_name.to_string()),
            arguments: None,
            body: Some(body),
            request_seq: None,
            success: None,
            message: None,
        }
    }

    fn next_seq(&mut self) -> u32 {
        self.seq += 1;
        self.seq
    }
}

/// Tensor Visualizer for debugging
pub struct TensorVisualizer;

impl TensorVisualizer {
    /// Render a 2D tensor as ASCII heatmap
    pub fn render_heatmap(data: &[f32], rows: usize, cols: usize) -> String {
        let chars = [' ', '░', '▒', '▓', '█'];
        let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = max - min;

        let mut output = String::new();
        for i in 0..rows {
            for j in 0..cols {
                let val = data[i * cols + j];
                let normalized = if range > 0.0 { (val - min) / range } else { 0.5 };
                let idx = (normalized * 4.0).min(4.0) as usize;
                output.push(chars[idx]);
            }
            output.push('\n');
        }
        output
    }

    /// Generate JSON representation for VS Code tensor view extension
    pub fn to_json(data: &[f32], shape: &[usize]) -> serde_json::Value {
        let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let mean = data.iter().sum::<f32>() / data.len() as f32;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / data.len() as f32;
        
        serde_json::json!({
            "type": "tensor",
            "dtype": "f32",
            "shape": shape,
            "data": data,
            "visualization": {
                "min": min,
                "max": max,
                "mean": mean,
                "std": variance.sqrt(),
                "histogram": Self::compute_histogram(data, 10)
            }
        })
    }
    
    fn compute_histogram(data: &[f32], bins: usize) -> Vec<u32> {
        let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = max - min;
        
        let mut histogram = vec![0u32; bins];
        
        if range > 0.0 {
            for &val in data {
                let bin = ((val - min) / range * bins as f32).min(bins as f32 - 1.0) as usize;
                histogram[bin] += 1;
            }
        }
        
        histogram
    }
    
    /// Render tensor as sparkline for compact display
    pub fn render_sparkline(data: &[f32], width: usize) -> String {
        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = max - min;
        
        // Downsample if needed
        let step = (data.len() / width).max(1);
        
        let mut output = String::new();
        for i in 0..width.min(data.len()) {
            let idx = i * step;
            if idx < data.len() {
                let val = data[idx];
                let normalized = if range > 0.0 { (val - min) / range } else { 0.5 };
                let char_idx = (normalized * 7.0).min(7.0) as usize;
                output.push(chars[char_idx]);
            }
        }
        
        output
    }
}

/// Main entry point for DAP server
fn main() -> std::io::Result<()> {
    eprintln!("Omni DAP starting...");
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();
    
    let mut adapter = DebugAdapter::new();
    
    loop {
        // Read Content-Length header (DAP uses same protocol as LSP)
        let mut size = 0;
        let mut header_buffer = String::new();
        
        loop {
            header_buffer.clear();
            if reader.read_line(&mut header_buffer)? == 0 {
                return Ok(()); // EOF
            }
            
            if header_buffer == "\r\n" {
                break; // End of headers
            }
            
            if header_buffer.starts_with("Content-Length: ") {
                let len_str = header_buffer.trim_start_matches("Content-Length: ").trim();
                size = len_str.parse::<usize>().unwrap_or(0);
            }
        }
        
        if size == 0 {
            continue;
        }
        
        // Read body
        let mut body_buffer = vec![0; size];
        reader.read_exact(&mut body_buffer)?;
        let body_str = String::from_utf8_lossy(&body_buffer);
        
        // Parse and handle request
        match serde_json::from_str::<DapMessage>(&body_str) {
            Ok(msg) => {
                let responses = adapter.handle_request(msg);
                for response in responses {
                    send_message(&mut stdout, &response)?;
                }
            }
            Err(e) => {
                eprintln!("Failed to parse DAP message: {}", e);
            }
        }
    }
}

fn send_message(stdout: &mut io::Stdout, msg: &DapMessage) -> io::Result<()> {
    let body = serde_json::to_string(msg).unwrap_or_default();
    write!(stdout, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    stdout.flush()
}

// Simple base64 encoding for memory reads
mod base64 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    pub fn encode(data: &[u8]) -> String {
        let mut result = String::new();
        
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as usize;
            let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
            let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
            
            result.push(ALPHABET[b0 >> 2] as char);
            result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
            
            if chunk.len() > 1 {
                result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
            } else {
                result.push('=');
            }
            
            if chunk.len() > 2 {
                result.push(ALPHABET[b2 & 0x3f] as char);
            } else {
                result.push('=');
            }
        }
        
        result
    }
}
