// Copyright 2024 Shreyash Jagtap
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Omni Language Runtime
//! Executes compiled or interpreted Omni code and facilitates native FFI.
#![allow(dead_code)]

pub mod bytecode;
pub mod bytecode_compiler;
pub mod distributed_logic;
mod gui;
pub mod hot_swap;
pub mod interpreter;
pub mod native;
mod network;
mod os;
pub mod profiler;
pub mod tests;
pub mod vm;

use log::{debug, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Runtime environment state
pub struct Runtime {
    interpreter: interpreter::Interpreter,
    native_manager: native::NativeManager,
    gui_context: gui::GuiContext,
    hot_swap_manager: hot_swap::HotSwapManager,
    profiler: profiler::OmniProfiler,
    running: Arc<AtomicBool>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            interpreter: interpreter::Interpreter::new(),
            native_manager: native::NativeManager::new(),
            gui_context: gui::GuiContext::new(),
            hot_swap_manager: hot_swap::HotSwapManager::new(),
            profiler: profiler::OmniProfiler::new(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Execute an Omni program (AST or Bytecode)
    pub fn run(&mut self, source_path: &std::path::Path) -> anyhow::Result<()> {
        info!("Runtime: Booting OVM for {:?}", source_path);
        self.running.store(true, Ordering::SeqCst);

        // 1. Initialize subsystems
        self.gui_context.init()?;
        self.profiler.start();

        info!("Runtime: Native systems initialized. Executing script...");

        // 2. Run interpreter (source or bytecode)
        let result = self.interpreter.eval_file(source_path);

        // 3. Stop profiler and dump report
        self.profiler.stop();
        if let Some(report) = self.profiler.report() {
            debug!("Runtime: Profiler report:\n{}", report);
        }

        // 4. Enter event loop if GUI was active
        if self.gui_context.has_windows() {
            info!("Runtime: Entering GUI event loop");
            let running = self.running.clone();
            while running.load(Ordering::SeqCst) {
                if !self.gui_context.pump_events() {
                    break;
                }

                // Check for hot-swap updates
                if self.hot_swap_manager.check_for_updates() {
                    info!("Runtime: Hot-swap update detected, reloading...");
                    if let Err(e) = self.apply_hot_swap() {
                        log::error!("Runtime: Hot swap failed: {}", e);
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps
            }
        }

        self.running.store(false, Ordering::SeqCst);
        info!("Runtime: Execution finished.");
        result.map(|_| ())
    }

    /// Shutdown the runtime gracefully
    pub fn shutdown(&self) {
        info!("Runtime: Shutting down...");
        self.running.store(false, Ordering::SeqCst);
    }

    /// Apply pending hot swap changes
    fn apply_hot_swap(&mut self) -> anyhow::Result<()> {
        let changes = self.hot_swap_manager.get_pending_changes();
        if changes.is_empty() {
            return Ok(());
        }

        info!("Runtime: Applying hot swap for {} files", changes.len());

        #[cfg(feature = "experimental")]
        {
            // JIT Engine instance (would be persistent in real implementation)
            use crate::codegen::jit::{JitConfig, JitEngine};
            let mut jit_engine = JitEngine::new(JitConfig::default());

            for change in changes {
                info!("Runtime: Processing file {:?}", change.path);

                if let Ok(_source) = std::fs::read_to_string(&change.path) {
                    let dummy_func = crate::ir::IrFunction {
                        name: "hot_swapped_func".to_string(),
                        params: vec![],
                        return_type: crate::ir::IrType::Void,
                        blocks: vec![crate::ir::IrBlock {
                            label: "entry".to_string(),
                            instructions: vec![],
                            terminator: crate::ir::IrTerminator::Return(None),
                        }],
                        locals: vec![],
                    };

                    match jit_engine.recompile_function(&dummy_func) {
                        Ok(compiled) => {
                            info!("Runtime: Successfully recompiled {}", compiled.name);
                            let new_address = 0x12345678;
                            self.hot_swap_manager
                                .update(&compiled.name, new_address)
                                .unwrap_or_else(|e| {
                                    warn!("Failed to update hot swap registry: {}", e)
                                });
                        }
                        Err(e) => {
                            log::error!("Runtime: Failed to recompile {}: {}", dummy_func.name, e);
                        }
                    }
                }
            }
        }

        #[cfg(not(feature = "experimental"))]
        {
            warn!("Runtime: Hot-swap requires the 'experimental' feature (JIT engine). Ignoring {} changes.", changes.len());
        }

        Ok(())
    }
}
