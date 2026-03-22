#![allow(dead_code)]
//! Native OS Integration
//! Provides OS-level functionality: clipboard, notifications, process launch, env vars

use crate::runtime::interpreter::RuntimeValue;
use log::{debug, info, warn};
use std::process::Command;

/// Dispatch OS-level native calls
pub fn handle_call(func: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, String> {
    match func {
        "launch_app" => {
            let program = match args.first() {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err("launch_app requires a program path string".to_string()),
            };
            let cmd_args: Vec<String> = args
                .iter()
                .skip(1)
                .filter_map(|a| match a {
                    RuntimeValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();

            info!(
                "OS: Launching application '{}' with args {:?}",
                program, cmd_args
            );

            match Command::new(&program).args(&cmd_args).spawn() {
                Ok(child) => {
                    debug!("OS: Launched PID {}", child.id());
                    Ok(RuntimeValue::Boolean(true))
                }
                Err(e) => {
                    warn!("OS: Failed to launch '{}': {}", program, e);
                    Ok(RuntimeValue::Boolean(false))
                }
            }
        }
        "exec" => {
            let program = match args.first() {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err("exec requires a command string".to_string()),
            };
            let cmd_args: Vec<String> = args
                .iter()
                .skip(1)
                .filter_map(|a| match a {
                    RuntimeValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();

            match Command::new(&program).args(&cmd_args).output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    Ok(RuntimeValue::String(stdout))
                }
                Err(e) => Err(format!("exec failed: {}", e)),
            }
        }
        "get_clipboard" => {
            // Cross-platform clipboard access
            #[cfg(target_os = "windows")]
            {
                match Command::new("powershell")
                    .args(&["-command", "Get-Clipboard"])
                    .output()
                {
                    Ok(output) => {
                        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        Ok(RuntimeValue::String(text))
                    }
                    Err(_) => Ok(RuntimeValue::String(String::new())),
                }
            }
            #[cfg(target_os = "linux")]
            {
                match Command::new("xclip")
                    .args(&["-selection", "clipboard", "-o"])
                    .output()
                {
                    Ok(output) => {
                        let text = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(RuntimeValue::String(text))
                    }
                    Err(_) => Ok(RuntimeValue::String(String::new())),
                }
            }
            #[cfg(target_os = "macos")]
            {
                match Command::new("pbpaste").output() {
                    Ok(output) => {
                        let text = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(RuntimeValue::String(text))
                    }
                    Err(_) => Ok(RuntimeValue::String(String::new())),
                }
            }
            #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
            {
                Ok(RuntimeValue::String(String::new()))
            }
        }
        "set_clipboard" => {
            let text = match args.first() {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err("set_clipboard requires a string argument".to_string()),
            };

            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("powershell")
                    .args(&[
                        "-command",
                        &format!("Set-Clipboard '{}'", text.replace("'", "''")),
                    ])
                    .output();
            }
            #[cfg(target_os = "linux")]
            {
                let mut child = Command::new("xclip")
                    .args(&["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("xclip failed: {}", e))?;
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(text.as_bytes()).ok();
                }
                child.wait().ok();
            }
            #[cfg(target_os = "macos")]
            {
                let mut child = Command::new("pbcopy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("pbcopy failed: {}", e))?;
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(text.as_bytes()).ok();
                }
                child.wait().ok();
            }

            Ok(RuntimeValue::Null)
        }
        "notification" => {
            let title = match args.first() {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => "Omni".to_string(),
            };
            let body = match args.get(1) {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => String::new(),
            };

            info!("OS: Notification - {}: {}", title, body);

            #[cfg(target_os = "windows")]
            {
                let script = format!(
                    "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
                     $xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent(0); \
                     $text = $xml.GetElementsByTagName('text'); \
                     $text.Item(0).AppendChild($xml.CreateTextNode('{}')) > $null; \
                     $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); \
                     [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Omni').Show($toast)",
                    title.replace("'", "''")
                );
                let _ = Command::new("powershell")
                    .args(&["-command", &script])
                    .output();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("notify-send").args(&[&title, &body]).output();
            }
            #[cfg(target_os = "macos")]
            {
                let script = format!(
                    "display notification \"{}\" with title \"{}\"",
                    body.replace("\"", "\\\""),
                    title.replace("\"", "\\\"")
                );
                let _ = Command::new("osascript").args(&["-e", &script]).output();
            }

            Ok(RuntimeValue::Null)
        }
        "env_get" => {
            let key = match args.first() {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err("env_get requires a key string".to_string()),
            };
            match std::env::var(&key) {
                Ok(val) => Ok(RuntimeValue::String(val)),
                Err(_) => Ok(RuntimeValue::Null),
            }
        }
        "env_set" => {
            let key = match args.first() {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err("env_set requires key and value strings".to_string()),
            };
            let val = match args.get(1) {
                Some(RuntimeValue::String(s)) => s.clone(),
                _ => return Err("env_set requires key and value strings".to_string()),
            };
            std::env::set_var(&key, &val);
            Ok(RuntimeValue::Null)
        }
        "cwd" => match std::env::current_dir() {
            Ok(path) => Ok(RuntimeValue::String(path.to_string_lossy().to_string())),
            Err(e) => Err(format!("cwd failed: {}", e)),
        },
        "hostname" => match hostname::get() {
            Ok(name) => Ok(RuntimeValue::String(name.to_string_lossy().to_string())),
            Err(_) => Ok(RuntimeValue::String("unknown".to_string())),
        },
        _ => Err(format!("Unknown OS function: {}", func)),
    }
}
