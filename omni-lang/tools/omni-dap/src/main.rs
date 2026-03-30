//! Omni Debug Adapter Protocol (DAP) Implementation
//!
//! A Debug Adapter that communicates over stdin/stdout using the DAP JSON
//! protocol with Content-Length framing.  Connects to the omnc OVM runtime
//! to provide breakpoint / stepping support (stubs for now).

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};

// ---------------------------------------------------------------------------
// DAP message types
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
struct DapMessage {
    seq: u32,
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_seq: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

// ---------------------------------------------------------------------------
// Debugger state
// ---------------------------------------------------------------------------
struct DebugSession {
    seq: u32,
    breakpoints: HashMap<String, Vec<u32>>, // file -> sorted line numbers
    stopped: bool,
    current_line: u32,
    current_file: String,
    launched: bool,
}

impl DebugSession {
    fn new() -> Self {
        Self {
            seq: 1,
            breakpoints: HashMap::new(),
            stopped: false,
            current_line: 0,
            current_file: String::new(),
            launched: false,
        }
    }

    fn next_seq(&mut self) -> u32 {
        let s = self.seq;
        self.seq += 1;
        s
    }

    #[allow(dead_code)]
    fn set_breakpoint(&mut self, file: String, line: u32) {
        let breaks = self.breakpoints.entry(file).or_insert_with(Vec::new);
        if !breaks.contains(&line) {
            breaks.push(line);
            breaks.sort();
        }
    }

    #[allow(dead_code)]
    fn clear_breakpoints(&mut self, file: &str) {
        self.breakpoints.remove(file);
    }
}

// ---------------------------------------------------------------------------
// JSON message framing (Content-Length header)
// ---------------------------------------------------------------------------
fn read_message(reader: &mut impl BufRead) -> io::Result<Option<DapMessage>> {
    // Read headers
    let mut content_length: usize = 0;
    loop {
        let mut header = String::new();
        let n = reader.read_line(&mut header)?;
        if n == 0 {
            return Ok(None); // EOF
        }
        let header = header.trim();
        if header.is_empty() {
            break; // end of headers
        }
        if let Some(val) = header.strip_prefix("Content-Length: ") {
            content_length = val
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        }
    }

    if content_length == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing Content-Length header",
        ));
    }

    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    let msg: DapMessage =
        serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(msg))
}

fn write_message(writer: &mut impl Write, msg: &DapMessage) -> io::Result<()> {
    let body = serde_json::to_string(msg)?;
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    writer.flush()
}

// ---------------------------------------------------------------------------
// Response / event helpers
// ---------------------------------------------------------------------------
fn make_response(session: &mut DebugSession, request: &DapMessage, body: Value) -> DapMessage {
    DapMessage {
        seq: session.next_seq(),
        msg_type: "response".into(),
        command: request.command.clone(),
        event: None,
        arguments: None,
        body: Some(body),
        request_seq: Some(request.seq),
        success: Some(true),
        message: None,
    }
}

fn make_event(session: &mut DebugSession, event: &str, body: Value) -> DapMessage {
    DapMessage {
        seq: session.next_seq(),
        msg_type: "event".into(),
        command: None,
        event: Some(event.into()),
        arguments: None,
        body: Some(body),
        request_seq: None,
        success: None,
        message: None,
    }
}

fn make_error_response(session: &mut DebugSession, request: &DapMessage, msg: &str) -> DapMessage {
    DapMessage {
        seq: session.next_seq(),
        msg_type: "response".into(),
        command: request.command.clone(),
        event: None,
        arguments: None,
        body: None,
        request_seq: Some(request.seq),
        success: Some(false),
        message: Some(msg.into()),
    }
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------
fn handle_initialize(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    let resp = make_response(
        session,
        req,
        json!({
            "supportsConfigurationDoneRequest": true,
            "supportsFunctionBreakpoints": false,
            "supportsConditionalBreakpoints": false,
            "supportsEvaluateForHovers": false,
            "supportsStepBack": false,
            "supportsSetVariable": false,
            "supportsRestartFrame": false,
            "supportsStepInTargetsRequest": false,
            "supportsCompletionsRequest": false,
        }),
    );
    let evt = make_event(session, "initialized", json!({}));
    vec![resp, evt]
}

fn handle_launch(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    if let Some(args) = &req.arguments {
        if let Some(program) = args.get("program").and_then(|v| v.as_str()) {
            session.current_file = program.to_string();
            info!("Launching program: {}", program);
        }
    }
    session.launched = true;
    session.current_line = 1;
    let resp = make_response(session, req, json!({}));
    // Send a stopped event so the IDE shows the first line
    let evt = make_event(
        session,
        "stopped",
        json!({ "reason": "entry", "threadId": 1 }),
    );
    session.stopped = true;
    vec![resp, evt]
}

fn handle_set_breakpoints(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    let mut verified = Vec::new();
    if let Some(args) = &req.arguments {
        let source_path = args
            .get("source")
            .and_then(|s| s.get("path"))
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();

        let lines: Vec<u32> = args
            .get("breakpoints")
            .and_then(|b| b.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|bp| bp.get("line").and_then(|l| l.as_u64()).map(|l| l as u32))
                    .collect()
            })
            .unwrap_or_default();

        for &line in &lines {
            verified.push(json!({
                "verified": true,
                "line": line,
            }));
        }
        session.breakpoints.insert(source_path, lines);
    }
    vec![make_response(
        session,
        req,
        json!({ "breakpoints": verified }),
    )]
}

fn handle_configuration_done(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    vec![make_response(session, req, json!({}))]
}

fn handle_threads(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    vec![make_response(
        session,
        req,
        json!({
            "threads": [
                { "id": 1, "name": "main" }
            ]
        }),
    )]
}

fn handle_stack_trace(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    vec![make_response(
        session,
        req,
        json!({
            "stackFrames": [
                {
                    "id": 1,
                    "name": "main",
                    "source": { "name": &session.current_file, "path": &session.current_file },
                    "line": session.current_line,
                    "column": 1,
                }
            ],
            "totalFrames": 1
        }),
    )]
}

fn handle_scopes(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    vec![make_response(
        session,
        req,
        json!({
            "scopes": [
                { "name": "Locals", "variablesReference": 1, "expensive": false },
                { "name": "Globals", "variablesReference": 2, "expensive": false },
            ]
        }),
    )]
}

fn handle_variables(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    // Stub: return empty variable list
    vec![make_response(session, req, json!({ "variables": [] }))]
}

fn handle_continue(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    session.stopped = false;
    // In a real implementation we'd resume the OVM interpreter here.
    // For now, simulate hitting the next breakpoint or terminating.
    let resp = make_response(session, req, json!({ "allThreadsContinued": true }));
    let evt = make_event(session, "terminated", json!({}));
    vec![resp, evt]
}

fn handle_next(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    session.current_line += 1;
    let resp = make_response(session, req, json!({}));
    let evt = make_event(
        session,
        "stopped",
        json!({ "reason": "step", "threadId": 1 }),
    );
    vec![resp, evt]
}

fn handle_step_in(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    session.current_line += 1;
    let resp = make_response(session, req, json!({}));
    let evt = make_event(
        session,
        "stopped",
        json!({ "reason": "step", "threadId": 1 }),
    );
    vec![resp, evt]
}

fn handle_step_out(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    session.current_line += 1;
    let resp = make_response(session, req, json!({}));
    let evt = make_event(
        session,
        "stopped",
        json!({ "reason": "step", "threadId": 1 }),
    );
    vec![resp, evt]
}

fn handle_disconnect(session: &mut DebugSession, req: &DapMessage) -> Vec<DapMessage> {
    vec![make_response(session, req, json!({}))]
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Omni DAP server starting on stdin/stdout");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();
    let mut session = DebugSession::new();

    loop {
        let msg = match read_message(&mut reader) {
            Ok(Some(m)) => m,
            Ok(None) => {
                info!("EOF on stdin, exiting");
                break;
            }
            Err(e) => {
                error!("Failed to read DAP message: {e}");
                break;
            }
        };

        debug!("← {:?}", msg);

        let responses = match msg.command.as_deref() {
            Some("initialize") => handle_initialize(&mut session, &msg),
            Some("launch") => handle_launch(&mut session, &msg),
            Some("attach") => handle_launch(&mut session, &msg), // treat like launch for now
            Some("setBreakpoints") => handle_set_breakpoints(&mut session, &msg),
            Some("configurationDone") => handle_configuration_done(&mut session, &msg),
            Some("threads") => handle_threads(&mut session, &msg),
            Some("stackTrace") => handle_stack_trace(&mut session, &msg),
            Some("scopes") => handle_scopes(&mut session, &msg),
            Some("variables") => handle_variables(&mut session, &msg),
            Some("continue") => handle_continue(&mut session, &msg),
            Some("next") => handle_next(&mut session, &msg),
            Some("stepIn") => handle_step_in(&mut session, &msg),
            Some("stepOut") => handle_step_out(&mut session, &msg),
            Some("disconnect") => {
                let resps = handle_disconnect(&mut session, &msg);
                for r in &resps {
                    let _ = write_message(&mut writer, r);
                }
                info!("Disconnect received, exiting");
                break;
            }
            Some(cmd) => {
                warn!("Unhandled DAP command: {cmd}");
                vec![make_error_response(
                    &mut session,
                    &msg,
                    &format!("Unsupported command: {cmd}"),
                )]
            }
            None => {
                // Likely an event or response from the client, ignore
                continue;
            }
        };

        for r in &responses {
            debug!("→ {:?}", r);
            if let Err(e) = write_message(&mut writer, r) {
                error!("Failed to write DAP message: {e}");
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_session_new() {
        let session = DebugSession::new();
        assert_eq!(session.seq, 1);
        assert!(!session.launched);
        assert!(!session.stopped);
        assert_eq!(session.breakpoints.len(), 0);
    }

    #[test]
    fn test_set_breakpoint() {
        let mut session = DebugSession::new();
        session.set_breakpoint("test.omni".to_string(), 10);
        assert!(session.breakpoints.contains_key("test.omni"));
        assert_eq!(session.breakpoints["test.omni"], vec![10]);
    }

    #[test]
    fn test_multiple_breakpoints() {
        let mut session = DebugSession::new();
        session.set_breakpoint("test.omni".to_string(), 10);
        session.set_breakpoint("test.omni".to_string(), 20);
        session.set_breakpoint("test.omni".to_string(), 5);

        let breaks = &session.breakpoints["test.omni"];
        assert_eq!(breaks.len(), 3);
        assert_eq!(breaks, &vec![5, 10, 20]); // should be sorted
    }

    #[test]
    fn test_clear_breakpoints() {
        let mut session = DebugSession::new();
        session.set_breakpoint("test.omni".to_string(), 10);
        session.clear_breakpoints("test.omni");
        assert!(!session.breakpoints.contains_key("test.omni"));
    }

    #[test]
    fn test_dap_message_serialization() {
        let msg = DapMessage {
            seq: 1,
            msg_type: "request".to_string(),
            command: Some("initialize".to_string()),
            event: None,
            arguments: None,
            body: None,
            request_seq: None,
            success: None,
            message: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"seq\":1"));
        assert!(json.contains("\"type\":\"request\""));
    }
}
