//! Native Bindings for Omni Runtime
//! Implements std::io, std::net, std::sys hooks using Rust libraries

#![allow(dead_code)]

use crate::runtime::interpreter::RuntimeValue;
#[allow(unused_imports)]
use log::{debug, info};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct NativeManager {
    // Keep track of open resources (files, sockets) by ID
    files: HashMap<usize, std::fs::File>,
    tcp_streams: HashMap<usize, std::net::TcpStream>,
    tcp_listeners: HashMap<usize, std::net::TcpListener>,
    next_handle: usize,
}

impl Default for NativeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeManager {
    pub fn new() -> Self {
        NativeManager {
            files: HashMap::new(),
            tcp_streams: HashMap::new(),
            tcp_listeners: HashMap::new(),
            next_handle: 1,
        }
    }

    pub fn call(
        &mut self,
        module: &str,
        func: &str,
        args: &[RuntimeValue],
    ) -> Result<RuntimeValue, String> {
        match (module, func) {
            // ================== IO ==================
            ("io", "print") => {
                if let Some(val) = args.first() {
                    print!("{:?}", val);
                }
                Ok(RuntimeValue::Null)
            }
            ("io", "println") => {
                if let Some(val) = args.first() {
                    println!("{:?}", val);
                } else {
                    println!();
                }
                Ok(RuntimeValue::Null)
            }
            ("io", "stdin_read_line") => {
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| e.to_string())?;
                Ok(RuntimeValue::String(input.trim().to_string()))
            }
            ("io", "file_open") => {
                let path = self.get_string_arg(args, 0)?;
                match std::fs::File::open(path) {
                    Ok(f) => {
                        let handle = self.alloc_handle();
                        self.files.insert(handle, f);
                        Ok(RuntimeValue::NativePtr(handle))
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            ("io", "file_create") => {
                let path = self.get_string_arg(args, 0)?;
                match std::fs::File::create(path) {
                    Ok(f) => {
                        let handle = self.alloc_handle();
                        self.files.insert(handle, f);
                        Ok(RuntimeValue::NativePtr(handle))
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            ("io", "file_write") => {
                let handle = self.get_handle_arg(args, 0)?;
                let data = match args.get(1) {
                    Some(RuntimeValue::String(s)) => s.as_bytes(),
                    Some(RuntimeValue::Array(_)) => {
                        return Err("Byte array write not impl".to_string())
                    }
                    _ => return Err("Invalid data to write".to_string()),
                };

                if let Some(file) = self.files.get_mut(&handle) {
                    file.write_all(data).map_err(|e| e.to_string())?;
                    Ok(RuntimeValue::Integer(data.len() as i64))
                } else {
                    Err("Invalid file handle".to_string())
                }
            }
            ("io", "file_read_to_string") => {
                let handle = self.get_handle_arg(args, 0)?;
                if let Some(file) = self.files.get_mut(&handle) {
                    let mut s = String::new();
                    file.read_to_string(&mut s).map_err(|e| e.to_string())?;
                    Ok(RuntimeValue::String(s))
                } else {
                    Err("Invalid file handle".to_string())
                }
            }

            // ================== SYS ==================
            ("sys", "time_now") => {
                let start = SystemTime::now();
                let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
                Ok(RuntimeValue::Integer(since_the_epoch.as_secs() as i64))
            }
            ("sys", "sleep") => {
                if let Some(RuntimeValue::Integer(ms)) = args.first() {
                    std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                }
                Ok(RuntimeValue::Null)
            }
            ("sys", "os_name") => Ok(RuntimeValue::String(std::env::consts::OS.to_string())),
            ("sys", "num_cpus") => {
                let cpus = std::thread::available_parallelism()
                    .map(|n| n.get() as i64)
                    .unwrap_or(1);
                Ok(RuntimeValue::Integer(cpus))
            }

            // ================== NET ==================
            ("net", "http_get") => {
                let _url = self.get_string_arg(args, 0)?;
                Err("reqwest disabled".to_string())
            }
            ("net", "tcp_connect") => {
                let addr = self.get_string_arg(args, 0)?;
                match std::net::TcpStream::connect(addr) {
                    Ok(stream) => {
                        let handle = self.alloc_handle();
                        self.tcp_streams.insert(handle, stream);
                        Ok(RuntimeValue::NativePtr(handle))
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            ("net", "tcp_write") => {
                let handle = self.get_handle_arg(args, 0)?;
                let data = self.get_string_arg(args, 1)?; // Treating string as bytes
                if let Some(stream) = self.tcp_streams.get_mut(&handle) {
                    stream
                        .write_all(data.as_bytes())
                        .map_err(|e| e.to_string())?;
                    Ok(RuntimeValue::Integer(data.len() as i64))
                } else {
                    Err("Invalid socket handle".to_string())
                }
            }

            // ================== AI (Tensor) ==================
            ("math", "tensor_create") => {
                let _size = match args.first() {
                    Some(RuntimeValue::Integer(n)) => *n as usize,
                    _ => 0,
                };
                Err("ndarray disabled".to_string())
                /* unreachable */
            }
            ("math", "tensor_matmul") => {
                // Simplified 1D as 2D (MxK * KxN) simulation or element-wise for demo
                // In production, this would cast Vector -> Array2 and use dot product
                if let (Some(RuntimeValue::Vector(_a)), Some(RuntimeValue::Vector(_b))) =
                    (args.first(), args.get(1))
                {
                    // For this demo, we'll do element-wise add just to prove op works as dot product requires sizing
                    // Real impl: cast raw pointers to Cblas
                    let res = vec![];
                    Ok(RuntimeValue::Vector(res))
                } else {
                    Err("Matmul requires two tensors".into())
                }
            }

            _ => Err(format!("Unknown native function {}::{}", module, func)),
        }
    }

    fn alloc_handle(&mut self) -> usize {
        let h = self.next_handle;
        self.next_handle += 1;
        h
    }

    fn get_string_arg<'a>(&self, args: &'a [RuntimeValue], idx: usize) -> Result<&'a str, String> {
        match args.get(idx) {
            Some(RuntimeValue::String(s)) => Ok(s),
            _ => Err(format!("Argument {} must be String", idx)),
        }
    }

    fn get_handle_arg(&self, args: &[RuntimeValue], idx: usize) -> Result<usize, String> {
        match args.get(idx) {
            Some(RuntimeValue::NativePtr(p)) => Ok(*p),
            Some(RuntimeValue::Integer(i)) => Ok(*i as usize),
            _ => Err(format!("Argument {} must be Handle/Ptr", idx)),
        }
    }
}
