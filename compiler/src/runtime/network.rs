#![allow(dead_code)]
//! Native Network Integration
//! Provides TCP/UDP socket operations and HTTP client functionality

use crate::runtime::interpreter::RuntimeValue;
use log::{info, debug};
use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr, ToSocketAddrs};
use std::io::{Read, Write};
use std::collections::HashMap;
use std::time::Duration;

/// Manages open network connections
pub struct NetworkManager {
    tcp_streams: HashMap<usize, TcpStream>,
    tcp_listeners: HashMap<usize, TcpListener>,
    udp_sockets: HashMap<usize, UdpSocket>,
    next_handle: usize,
}

impl NetworkManager {
    pub fn new() -> Self {
        NetworkManager {
            tcp_streams: HashMap::new(),
            tcp_listeners: HashMap::new(),
            udp_sockets: HashMap::new(),
            next_handle: 1,
        }
    }

    /// Connect to a TCP endpoint
    pub fn tcp_connect(&mut self, addr: &str, timeout_ms: u64) -> Result<usize, String> {
        let socket_addr: SocketAddr = addr.parse()
            .or_else(|_| addr.to_socket_addrs()
                .map_err(|e| format!("DNS resolve failed: {}", e))?
                .next()
                .ok_or("No address found".to_string()))
            .map_err(|e| format!("Invalid address '{}': {}", addr, e))?;
        
        let stream = if timeout_ms > 0 {
            TcpStream::connect_timeout(&socket_addr, Duration::from_millis(timeout_ms))
        } else {
            TcpStream::connect(socket_addr)
        }.map_err(|e| format!("TCP connect failed: {}", e))?;
        
        stream.set_nodelay(true).ok();
        
        let handle = self.next_handle;
        self.next_handle += 1;
        self.tcp_streams.insert(handle, stream);
        info!("Network: TCP connected to {} (handle={})", addr, handle);
        Ok(handle)
    }

    /// Send data over a TCP connection
    pub fn tcp_send(&mut self, handle: usize, data: &[u8]) -> Result<usize, String> {
        let stream = self.tcp_streams.get_mut(&handle)
            .ok_or_else(|| format!("Invalid TCP handle: {}", handle))?;
        stream.write(data)
            .map_err(|e| format!("TCP send failed: {}", e))
    }

    /// Receive data from a TCP connection with timeout support
    pub fn tcp_recv(&mut self, handle: usize, max_bytes: usize) -> Result<Vec<u8>, String> {
        let stream = self.tcp_streams.get_mut(&handle)
            .ok_or_else(|| format!("Invalid TCP handle: {}", handle))?;
        // Set a reasonable read timeout to prevent indefinite blocking
        stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
        let mut buf = vec![0u8; max_bytes.min(65536)]; // Cap buffer to 64KB to prevent OOM
        let n = stream.read(&mut buf)
            .map_err(|e| format!("TCP recv failed: {}", e))?;
        buf.truncate(n);
        Ok(buf)
    }

    /// Listen on a TCP port
    pub fn tcp_listen(&mut self, addr: &str) -> Result<usize, String> {
        let listener = TcpListener::bind(addr)
            .map_err(|e| format!("TCP bind failed: {}", e))?;
        let handle = self.next_handle;
        self.next_handle += 1;
        self.tcp_listeners.insert(handle, listener);
        info!("Network: TCP listening on {} (handle={})", addr, handle);
        Ok(handle)
    }

    /// Accept an incoming TCP connection
    pub fn tcp_accept(&mut self, listener_handle: usize) -> Result<usize, String> {
        let listener = self.tcp_listeners.get(&listener_handle)
            .ok_or_else(|| format!("Invalid listener handle: {}", listener_handle))?;
        let (stream, addr) = listener.accept()
            .map_err(|e| format!("TCP accept failed: {}", e))?;
        stream.set_nodelay(true).ok(); // Reduce latency on accepted connections
        let handle = self.next_handle;
        self.next_handle += 1;
        debug!("Network: Accepted connection from {} (handle={})", addr, handle);
        self.tcp_streams.insert(handle, stream);
        Ok(handle)
    }

    /// Create a UDP socket
    pub fn udp_bind(&mut self, addr: &str) -> Result<usize, String> {
        let socket = UdpSocket::bind(addr)
            .map_err(|e| format!("UDP bind failed: {}", e))?;
        let handle = self.next_handle;
        self.next_handle += 1;
        self.udp_sockets.insert(handle, socket);
        info!("Network: UDP bound to {} (handle={})", addr, handle);
        Ok(handle)
    }

    /// Send UDP datagram
    pub fn udp_send_to(&mut self, handle: usize, data: &[u8], addr: &str) -> Result<usize, String> {
        let socket = self.udp_sockets.get(&handle)
            .ok_or_else(|| format!("Invalid UDP handle: {}", handle))?;
        socket.send_to(data, addr)
            .map_err(|e| format!("UDP send failed: {}", e))
    }

    /// Receive UDP datagram
    pub fn udp_recv_from(&mut self, handle: usize, max_bytes: usize) -> Result<(Vec<u8>, String), String> {
        let socket = self.udp_sockets.get(&handle)
            .ok_or_else(|| format!("Invalid UDP handle: {}", handle))?;
        let mut buf = vec![0u8; max_bytes];
        let (n, addr) = socket.recv_from(&mut buf)
            .map_err(|e| format!("UDP recv failed: {}", e))?;
        buf.truncate(n);
        Ok((buf, addr.to_string()))
    }

    /// Close a network handle (TCP stream, listener, or UDP socket)
    pub fn close(&mut self, handle: usize) -> Result<(), String> {
        if self.tcp_streams.remove(&handle).is_some() {
            debug!("Network: Closed TCP stream {}", handle);
            Ok(())
        } else if self.tcp_listeners.remove(&handle).is_some() {
            debug!("Network: Closed TCP listener {}", handle);
            Ok(())
        } else if self.udp_sockets.remove(&handle).is_some() {
            debug!("Network: Closed UDP socket {}", handle);
            Ok(())
        } else {
            Err(format!("Unknown handle: {}", handle))
        }
    }

    /// Perform a simple HTTP GET request
    pub fn http_get(&mut self, url: &str) -> Result<String, String> {
        // Parse URL into host:port and path
        let url = url.trim_start_matches("http://");
        let (host_port, path) = if let Some(idx) = url.find('/') {
            (&url[..idx], &url[idx..])
        } else {
            (url, "/")
        };
        
        let addr = if host_port.contains(':') {
            host_port.to_string()
        } else {
            format!("{}:80", host_port)
        };
        
        let mut stream = TcpStream::connect(&addr)
            .map_err(|e| format!("HTTP connect failed: {}", e))?;
        stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
        
        let host = host_port.split(':').next().unwrap_or(host_port);
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: Omni/1.0\r\n\r\n",
            path, host
        );
        
        stream.write_all(request.as_bytes())
            .map_err(|e| format!("HTTP send failed: {}", e))?;
        
        let mut response = String::new();
        stream.read_to_string(&mut response)
            .map_err(|e| format!("HTTP read failed: {}", e))?;
        
        // Extract body from HTTP response
        if let Some(body_start) = response.find("\r\n\r\n") {
            Ok(response[body_start + 4..].to_string())
        } else {
            Ok(response)
        }
    }
}

/// Legacy handle_call interface for compatibility
pub fn handle_call(func: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, String> {
    // This is a simplified shim; the full NetworkManager is used by the OVM interpreter
    match func {
        "connect" => {
            let addr = match args.first() {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err("connect requires a string address".to_string()),
            };
            info!("Network: Connecting TCP to {}", addr);
            match TcpStream::connect(&addr) {
                Ok(_) => Ok(RuntimeValue::Integer(1)),
                Err(e) => Err(format!("TCP connect failed: {}", e)),
            }
        }
        "send" => {
            info!("Network: Sending data");
            let data_len = match args.get(1) {
                Some(RuntimeValue::String(s)) => s.len() as i64,
                _ => 0,
            };
            Ok(RuntimeValue::Integer(data_len))
        }
        "http_get" => {
            let url = match args.first() {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err("http_get requires a URL string".to_string()),
            };
            let mut mgr = NetworkManager::new();
            match mgr.http_get(&url) {
                Ok(body) => Ok(RuntimeValue::String(body)),
                Err(e) => Err(e),
            }
        }
        _ => Err(format!("Unknown Network function: {}", func)),
    }
}
