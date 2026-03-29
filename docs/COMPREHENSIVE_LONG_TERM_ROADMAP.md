# HELIOS & Omni Language — Comprehensive 10-Year Implementation Roadmap

**Version:** 1.0 — Updated 2026-03-14  
**Status:** PRE-MVP (15-20% complete)  
**Current Critical Blockers:** 5 blocking issues preventing Phase 1.0 completion  
**Timeline:** 10+ years (no deadline pressure)  
**Scope:** Complete production-ready system with full feature suite, advanced reasoning, distributed execution, plugin ecosystem, and cross-platform deployment  
**Audience:** Engineering teams, stakeholders, long-term planning

---

## 🚨 CRITICAL BLOCKERS — FIX IMMEDIATELY

These 5 issues block MVP completion. All target **Week 1** resolution:

1. **Bytecode VM has NO native function support**
   - Problem: `OpCode::CallNative` doesn't exist; VM can't call I/O, network, crypto functions
   - Impact: Framework can't run ANY real code (all external operations fail)
   - Fix: Add CallNative opcode + integrate NativeManager into OmniVM ([see 1.1.1](./roadmap/Phase%201.1.1))
   - Effort: 3-4 days
   - Blocker for: 1.2, 1.3, entire service deployment

2. **Framework compilation never tested**
   - Problem: `omnc compile helios-framework/main.omni` never run; unknown if code parses
   - Impact: Parser/codegen bugs discovered late (cascading delays)
   - Fix: Run compilation test Day 1; fix bugs immediately ([see 1.1.3](./roadmap/Phase%201.1.3))
   - Effort: 1 day + bug fixes
   - Blocker for: Service testing, cognitive layer testing

3. **Knowledge store NOT crash-safe**
   - Problem: Direct file write with no atomic pattern; crash during write = **DATA LOSS**
   - Impact: Unacceptable for deployment; loses reasoning history
   - Fix: Implement temp+rename pattern in `knowledge.omni::flush()` ([see 1.1.2](./roadmap/Phase%201.1.2))
   - Effort: 2-3 days including tests
   - Blocker for: Data persistence guarantee, production deployment

4. **IPC native functions NOT implemented**
   - Problem: No `pipe_create`, `pipe_accept`, `pipe_read`, `pipe_write` in native.rs
   - Impact: Service can't communicate with GUI or clients (HTTP-only, unreliable)
   - Fix: Add 6 Windows pipe natives to native.rs; wire into VM ([see 1.1.4](./roadmap/Phase%201.1.4))
   - Effort: 3-4 days + testing
   - Blocker for: IPC wrapper (1.2.1), multi-client support, GUI

5. **Framework has ZERO test files**
   - Problem: `helios-framework/tests/` directory doesn't exist; 0 integration tests
   - Impact: Bugs discovered in production; no CI gate
   - Fix: Create test directory + write integration tests ([see 1.6.1](./roadmap/Phase%201.6.1))
   - Effort: Ongoing (2-4 weeks)
   - Blocker for: Release quality gate, Phase 1 completion

**Together, these blockers explain why MVP is only 15% complete despite many files existing.**

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Assessment](#current-state-assessment)
3. [Phasing Overview](#phasing-overview)
4. [Phase 1: MVP (Year 1-1.5)](#phase-1-mvp-year-1--15)
5. [Phase 2: Hardening & Expansion (Year 1.5-3)](#phase-2-hardening--expansion-year-15-3)
6. [Phase 3: Advanced Reasoning (Year 3-5)](#phase-3-advanced-reasoning-year-3-5)
7. [Phase 4: Distributed Deployment (Year 5-7)](#phase-4-distributed-deployment-year-5-7)
8. [Phase 5: Ecosystem & AI Integration (Year 7-10)](#phase-5-ecosystem--ai-integration-year-7-10)
9. [Detailed Component Specifications](#detailed-component-specifications)
10. [Quality & Testing Strategy](#quality--testing-strategy)
11. [Documentation & Knowledge Base](#documentation--knowledge-base)

---

## Executive Summary

### Project Vision

The HELIOS Cognitive Framework is a **next-generation AI cognition system** paired with the **Omni programming language** — a modern, safe, feature-rich systems language designed for:

- **Local-first cognitive computing** (no cloud dependency)
- **Evidence-driven reasoning** (audit trails for every inference)
- **Hardware-accelerated execution** (CPU, GPU, TPU, custom ASICs)
- **Distributed edge intelligence** (federation across devices)
- **Transparent, explainable AI** (reasoning traces for every output)

### Core Design Philosophy

#### Omni Language: Comprehensive Feature Coverage

**Omni is designed as a comprehensive, detail-oriented, feature-complete systems programming language** with the following principles:

- **Extreme Balance Across All Dimensions**: Every feature is implemented with balanced consideration of:
  - **Performance** (optimization techniques, CPU cache awareness, memory layout)
  - **Efficiency** (minimal overhead, predictable resource usage)
  - **Execution Speed** (JIT compilation, interpreter optimization, bytecode efficiency)
  - **Processing Power** (parallel algorithms, SIMD support, GPU integration)
  - **Safety** (type checking, borrow semantics, exception handling)

- **Comprehensive Algorithm & Technique Coverage**: Omni includes **nearly all major algorithms and programming language techniques**:
  - Data structures (arrays, linked lists, B-trees, hash tables, skip lists, etc.)
  - Sorting algorithms (quicksort, mergesort, heapsort, radix sort, etc.)
  - Graph algorithms (DFS, BFS, A*, Dijkstra, Floyd-Warshall, topological sort, etc.)
  - Pattern matching and regular expressions (NFA/DFA with optimization)
  - Functional programming (closures, higher-order functions, lambda calculus support)
  - Metaprogramming (macros, reflection, code generation)
  - Concurrency models (async/await, coroutines, lightweight threads, channels)
  - Memory management (garbage collection, reference counting, arena allocation)
  - DSL support (parser combinators, macro systems)

#### Development Order: Language First, Then Framework

**Critical sequencing constraint:** The Omni language must be **stabilized before HELIOS framework development accelerates**.

- **Year 1**: Omni compiler reaches feature parity + standard library complete. Language specification locked.
- **Year 1+**: HELIOS framework built ON TOP of stable Omni, not in parallel.
- **Rationale**: Language changes cascade into framework; framework deployment must trust language stability. Early commitment to framework on unstable language = architectural rework later.

#### HELIOS: Cognitive Framework Without ML Training

**HELIOS is a proper cognitive framework, not a machine learning system.**

- **No Training Phase Required**: Unlike neural networks (TensorFlow, PyTorch), HELIOS derives cognition from:
  - Logical rules (L1 RETE forward chaining)
  - Backward inference (L2 Prolog-like resolution)
  - Causal models (L3 structural causal models)
  - Distributed consensus (L4 multi-agent reasoning)
  
- **Symbolic + Hybrid**: HELIOS can integrate LLMs (text understanding, generation) but cores cognition on symbolic reasoning, making it:
  - Interpretable (reasoning traces auditable)
  - Deterministic (same query = same answer, always)
  - Debuggable (breakpoints on rule firing, inference steps)
  - Transparent (no black-box neural weights)

#### HELIOS SDK: Library for Custom Cognitive Frameworks

**HELIOS is designed as a foundational framework library**, enabling third-party developers to build **custom cognitive frameworks**.

- **Role**: HELIOS to cognitive frameworks ≈ TensorFlow to machine learning frameworks
  - TensorFlow provides: tensor operations, gradient computation, GPU scheduling
  - Users build: Keras, PyTorch, JAX, etc. on top
  - Users create: Custom architectures, training loops, loss functions

- **HELIOS Provides**:
  - Core: Knowledge store, experience log, capability registry, IPC transport
  - Reasoning primitives: L0-L4 execution engines (reflex, RETE, backchain, causal, distributed)
  - Integration points: Custom capability plugins, custom storage backends, custom rule integrations
  - Observability: Telemetry hooks, debugging interfaces, tracing subsystem

- **Users Build**:
  - Custom cognitive frameworks with domain-specific reasoning
  - Industry-specific implementations (legal reasoning, medical diagnosis, financial analysis)
  - Proprietary rule sets and knowledge models
  - Extended capabilities via plugin SDK
  - Custom frontends (mobile apps, web UIs, voice interfaces)

- **Library + Marketplace Model**:
  - Phase 5: HELIOS SDK released as importable library (like `import helios`)
  - Community plugins available in plugin marketplace
  - Licensing: Core is open-source; commercial support available
  - Users can fork, extend, and deploy privately

### Success Definition for Each Phase

| Phase | Duration | Main Goals | Shipped Artifact | Team Size |
|-------|----------|-----------|-----------------|-----------|
| **MVP** | Y1-1.5 | L0 reflex + L1 rules + service + GUI | Standalone Windows bot service + GUI app | 6-10 |
| **Hardening** | Y1.5-3 | L2 backward chain + plugins + distributed | Multi-platform CLI tools + plugin SDK | 10-15 |
| **Advanced Reasoning** | Y3-5 | L3 causal + L4 distributed + synthesis | Homogeneous cluster deployment framework | 15-20 |
| **Enterprise** | Y5-7 | Multi-tenant, federation, UI dashboard | Commercial licensing + support infrastructure | 20-30 |
| **Ecosystem** | Y7-10 | LLM integration + WASM plugins + mobile | Cross-platform SDKs + marketplace | 30+ |

### High-Level Phases

```
Y1.0  ┌─ MVP: Service + GUI on Windows
      │  └─ L0+L1 reasoning, JSON storage, single-threaded
Y1.5  │
      │
Y2.0  ├─ Hardening: Multi-platform, plugins, distributed comms
      │  └─ L2 backward chain, plugin sandbox, IPC federation
Y3.0  │
      │
Y3.5  ├─ Advanced Reasoning: Causal inference, synthesis
      │  └─ L3 causal + L4 distribution, cluster orchestration
Y4.5  │
      │
Y5.0  ├─ Enterprise: Multi-tenant, licensing, commercial dashboards
      │  └─ SaaS infrastructure, audit compliance, HA/DR
Y6.5  │
      │
Y7.0  ├─ Ecosystem: LLM integration, WASM, mobile, marketplace
      │  └─ Cross-platform SDKs, app store, community plugins
Y10   ▼
```

---

## Current State Assessment

### Codebase Inventory

**What's Implemented (Working or Near-Working):**

```
✅ Omni Language Compiler
   ├─ Lexer + Parser (1619 LOC, 37 test cases)
   ├─ Semantic Analysis (type inference HM, borrow checking)
   ├─ Bytecode Codegen (36 opcodes, OVM bytecode IR)
   ├─ Runtime (two execution engines: tree-walk interpreter + stack-based VM)
   │  ├─ Located in: compiler/src/runtime/ (NOT a separate ovm/ crate)
   │  ├─ interpreter.rs (135KB, uses RuntimeValue)
   │  ├─ vm.rs (67KB, uses VmValue, has tri-color garbage collector)
   │  └─ NOTE: Value types are incompatible between engines
   └─ CLI toolchain (omnc compiler, opm package manager)

✅ Omni Standard Library
   ├─ Core modules (11): math, io, fs, net, threading, async, etc.
   ├─ Stdlib modules (33+): collections, crypto, compression, JSON, TOML
   └─ Test coverage: ~70% (inline unit tests)

✅ HELIOS Framework (Omni source)
   ├─ Runtime core (338 LOC)
   ├─ Knowledge store (798 LOC, JSON serialization)
   ├─ Experience log (event recording)
   ├─ Capability registry (15+ built-in capabilities)
   ├─ HTTP API (basic REST server)
   └─ Service skeleton (entry point present)

🟡 Partial / Needs Work:
   ├─ OVM Runtime (two incompatible engines, need consolidation)
   ├─ Named Pipe IPC (HTTP only, Windows named pipes not implemented)
   ├─ Cognitive layers (L0 ~50%, L1 ~30%, L2-L4 not started)
   ├─ GUI (helios-framework/app/gui.omni is design document only, NOT C# code)
   ├─ RETE engine (forward rules framework exists, not integrated end-to-end)
   ├─ Distributed execution (stub code in compiler/src/runtime/network.rs)
   └─ Plugin system (manifest framework exists, sandboxing incomplete)

❌ Not Yet Started:
   ├─ Backward chaining (L2 reasoning)
   ├─ Causal inference (L3 reasoning)
   ├─ Distributed consensus (L4 reasoning)
   ├─ Windows service wrapper (tools/service_bridge.rs)
   ├─ WinUI3 C# GUI application (gui/ directory is empty)
   ├─ Multi-platform support (Linux, macOS)
   ├─ Multi-tenant SaaS infrastructure
   ├─ LLM integration layer
   └─ Mobile apps (iOS, Android)
```

### Critical Issues to Resolve (Pre-MVP)

1. **Two-runtime unification** — Consolidate interpreter + VM in `compiler/src/runtime/` (value types incompatible)
2. **Knowledge store atomicity** — Fix crash-safety bug in `helios-framework/helios/knowledge.omni::flush()`
3. **Framework compilation** — Verify all `.omni` files in `helios-framework/` compile end-to-end
4. **IPC implementation** — Add Windows named pipe natives to `compiler/src/runtime/native.rs`
5. **Test infrastructure** — Create integration test suite (currently zero tests in helios-framework/)

### Implementation Status Dashboard

This roadmap serves as a LIVING DOCUMENT tracking actual implementation progress. Here's a quick overview of Phase 1 MVP readiness:

| Component | Status | Blocker | ETA |
|-----------|--------|---------|-----|
| **1.1.1 Runtime Consolidation** | 🟡 PARTIAL | CallNative opcode missing | Week 2-3 |
| **1.1.2 Knowledge Store Atomicity** | 🟡 PARTIAL | Atomic write pattern needed | Week 1 |
| **1.1.3 Framework Compilation** | ⚠️ UNTESTED | Never run compilation test | Week 1 Day 1 |
| **1.1.4 IPC Native Functions** | ❌ NOT STARTED | Needs windows-sys feature | Week 1-2 |
| **1.2.1 IPC Wrapper (Omni)** | ❌ BLOCKED | Waiting for 1.1.4 | Week 3 |
| **1.2.2 Service State Machine** | 🟡 SKETCHED | Implementation incomplete | Week 3-4 |
| **1.2.3 Dual Listen** | ❌ BLOCKED | Waiting for 1.2.1 | Week 4 |
| **1.3.1 L0 Reflex Layer** | ❌ BLOCKED | Knowledge/service needed | Week 5-6 |
| **1.3.2 L1 RETE Engine** | 🟡 PARTIAL | Brain modules skeletal | Week 6-8 |
| **1.3.3 Experience Log** | 🟡 PARTIAL | File exists but incomplete | Week 7 |
| **1.4 GUI (WinUI3)** | ❌ NOT STARTED | Complete new C# project needed | Week 10-12 |
| **1.5 Service Wrapper** | ❌ NOT STARTED | Depends on working service | Week 12 |
| **1.6 Testing** | ❌ MISSING | Tests framework has ZERO test files | Week 13-16 |

**Critical Path:** 1.1.1 (Runtime) → 1.1.4 (IPC natives) → 1.2.1 (IPC wrapper) → 1.2.3 (Dual listen) → 1.3 (Cognitive) → Full integration

**MVP Readiness Estimate:** Currently 15-20% complete. On track for Week 18 release if blockers resolved immediately.

### Important Structural Notes

- **Runtime Location:** All bytecode VM code lives in `omni-lang/compiler/src/runtime/`, NOT a separate `ovm/` crate
  - `vm.rs` — Stack-based virtual machine (67KB)
  - `bytecode_compiler.rs` — AST→bytecode compilation (49KB)
  - `interpreter.rs` — Tree-walk interpreter, for REPL only (135KB)
  - `native.rs` — Native function bindings (8KB)
  - **TODO:** Add native functions for Windows named pipes (currently HTTP-only)

- **GUI Status:** `helios-framework/app/gui.omni` is a **design document**, NOT C# code
  - File size: 13KB of Omni syntax describing UX/structure/concepts
  - Actual WinUI3 C# project must be created from scratch in Phase 1.4
  - See Plan.md §7 for WinUI3 implementation tasks

-  **Test Status:** `helios-framework/` has **zero test files**
  - Tests in `compiler/` are inline `#[cfg(test)]` blocks, not separate test files
  - Must create comprehensive integration tests in Phase 1.6

---

## Phasing Overview

### Phase 1: MVP (Year 1 — 1.5 Years)
**Goal:** Single-user Windows desktop service with basic reasoning and GUI.

**Deliverables:**
- [ ] Windows service running HELIOS framework
- [ ] Working GUI (WinUI3 C#)
- [ ] L0 (reflex) + L1 (basic RETE) reasoning
- [ ] JSON knowledge store with atomic writes
- [ ] HTTP + named pipe IPC
- [ ] Deployment installer
- [ ] Basic documentation

**Exit Criteria:**
- Service stays up for 24+ hours without crashes
- GUI connects successfully
- End-to-end conversation works
- <100ms response time for simple queries
- Zero data corruption on kill -9

---

### Phase 2: Hardening & Expansion (Year 1.5 — 3 Years)
**Goal:** Production-ready multi-platform system with plugin ecosystem and federation basics.

**Deliverables:**
- [ ] Multi-platform support (Windows, macOS, Linux)
- [ ] L2 (backward chaining) reasoning layer
- [ ] Plugin SDK + security sandbox
- [ ] Distributed communication (IPC federation)
- [ ] Better knowledge store (B+ tree, compression)
- [ ] Commercial licensing
- [ ] Support infrastructure

**Exit Criteria:**
- All 3 platforms pass acceptance tests
- Plugin system enables 3rd-party code execution safely
- Cluster of 2+ services can coordinate reasoning
- 80%+ test coverage
- <1 hour to deploy n nodes

---

### Phase 3: Advanced Reasoning (Year 3 — 5 Years)
**Goal:** Sophisticated reasoning with causal inference and multi-agent coordination.

**Deliverables:**
- [ ] L3 (causal inference) reasoning
- [ ] L4 (distributed consensus) coordination
- [ ] Knowledge synthesis engine
- [ ] Causal DAG learning
- [ ] Cluster orchestration (Kubernetes support)
- [ ] High-availability (primary/backup, quorum-based recovery)
- [ ] Advanced monitoring + observability

**Exit Criteria:**
- Causal reasoning passes 100-scenario benchmark
- Distributed reasoning maintains consistency across 5+ nodes
- Sub-second coordination for common patterns
- Zero data loss on node failure
- Enterprise support SLA achievable

---

### Phase 4: Enterprise & Distribution (Year 5 — 7 Years)
**Goal:** Multi-tenant cloud-ready system with governance and audit compliance.

**Deliverables:**
- [ ] Multi-tenant architecture
- [ ] Role-based access control (RBAC)
- [ ] Audit logging + compliance (SOC2, HIPAA-ready)
- [ ] SaaS infrastructure (API, web dashboard)
- [ ] Commercial licensing + billing
- [ ] High-availability & disaster recovery
- [ ] Advanced analytics + business intelligence

**Exit Criteria:**
- Support 1000+ concurrent users
- Deploy, manage 100+ clusters via unified API
- Full audit trail for all inferencing
- 99.99% uptime SLA achievable
- Compliance pre-audit ready

---

### Phase 5: Ecosystem & LLM Integration (Year 7 — 10 Years)
**Goal:** Open ecosystem with LLM bridges, WASM plugins, mobile support.

**Deliverables:**
- [ ] LLM integration layer (GPT-4, Claude, Llama bridges)
- [ ] WASM plugin runtime
- [ ] Mobile SDKs (iOS, Android)
- [ ] Web app framework
- [ ] Plugin marketplace
- [ ] Community tooling + open-source SDK
- [ ] Research partnerships + academic grants

**Exit Criteria:**
- 10,000+ developers using for reasoning tasks
- 1000+ published plugins in marketplace
- HELIOS + LLM + human feedback loop working
- Sub-second fine-tuning on domain knowledge
- Cross-platform deployment to 10+ OS platforms

---

## Phase 1: MVP (Year 1 — 1.5 Years)

### Phase 1.1: Foundation & Critical Fixes (Months 1-3)

#### 1.1.1 Runtime Consolidation
**Goal:** Single unified execution engine.

**Current Status:** ✅ STARTED / 🟡 INCOMPLETE
- ✅ Two engines exist (`interpreter.rs` 135KB, `vm.rs` 67KB) in `compiler/src/runtime/`
- ✅ `RuntimeValue` enum defined with 13 variants (includes NativePtr, Vector, Module support)
- ✅ `VmValue` enum defined with 8 variants (includes HeapRef for GC)
- 🟡 **BLOCKER**: Value types remain incompatible (no unified representation yet)
- 🟡 **BLOCKER**: `NativeManager` works with interpreter only; VM has NO native function support
- 🟡 Native functions file (`native.rs`, 199 LOC) implements only 20 of 50+ required functions (missing: IPC, crypto, JSON, msgpack)
- 🟡 No converter bridge implemented yet
- ❌ Runtime tests exist but incomplete (inline `#[cfg(test)]` blocks, no dedicated integration tests)

**Tasks:**
- [ ] **CRITICAL**: Add `OpCode::CallNative(module, func, n_args)` to bytecode VM
- [ ] **CRITICAL**: Implement value conversion (`VmValue <-> RuntimeValue`) at FFI boundary
- [ ] **HIGH**: Complete missing native functions (IPC pipes, Windows events, crypto, JSON)
- [ ] **HIGH**: Add native function support layer to `vm.rs` (currently has no `NativeManager` field)
- [ ] **MEDIUM**: Merge `RuntimeValue` and `VmValue` into single `Value` enum with union of all variants
- [ ] **MEDIUM**: Write comprehensive runtime tests (100+ test cases in `compiler/tests/runtime_integration.rs`)
- [ ] **LOW**: Deprecate tree-walk interpreter for deployment paths (keep for REPL: `omnc repl`)

**Implementation Roadmap:**
1. **Week 1**: Add `CallNative` opcode to VM, wire NativeManager into OmniVM struct
2. **Week 1-2**: Implement 15 missing IPC/native functions (pipe_create, pipe_accept, etc.)
3. **Week 2**: Test VM + native function roundtrip (send message via pipe, receive correctly)
4. **Week 3**: Build value converter library; add 100+ integration tests
5. **Week 3-4**: Merge value types; deprecate interpreter in production

**Deliverables:**
- ✅ `compiler/src/runtime/value.rs` — Unified value representation (IN PROGRESS: needs merge)
- 🟡 `compiler/src/runtime/native.rs` — INCOMPLETE: 20/50+ functions implemented (missing IPC, crypto, serialization)
- ❌ `compiler/src/runtime/error.rs` — Not created yet (currently uses String error type everywhere)
- ❌ Test suite: `compiler/tests/runtime_integration.rs` — Not created yet (tests inline in source)

**Success Criteria:**
- ✅ `omnc compile` works for all stdlib functions
- ✅ All 360+ compiler tests pass (baseline)
- 🟡 **NEW**: `ovm bytecode_execute` works with native function calls (currently fails: VM has no CallNative support)
- 🟡 **NEW**: All 50+ native functions implemented and tested
- ❌ No panics/unwraps in production code paths (currently: 10+ in vm.rs line 244-295 GC collect)
- ❌ Proper error type propagation (currently: all errors are `String`)
- ❌ <50KB code growth (currently: value consolidation will require ~15KB refactoring)

---

#### 1.1.2 Knowledge Store Atomicity Fix
**Goal:** Crash-safe persistent storage.

**Current Status:** 🟡 PARTIALLY IMPLEMENTED
- ✅ Knowledge store exists (`helios-framework/helios/knowledge.omni`, 798 LOC, JSON serialization)
- ✅ Basic store/query/flush methods defined
- 🟡 **BLOCKER**: `flush()` method uses direct write (NOT atomic!) — code shows: `fs::write_string(&data_path, &data)` with NO error recovery
- 🟡 No backup snapshots implemented
- ❌ No recovery logic on startup (corrupt file = data loss)
- ❌ No crash-recovery tests
- ❌ No auto-flush timer
- ❌ No performance tests

**Tasks:**
- [ ] **CRITICAL**: Implement atomic write pattern in `knowledge.omni::flush()`:
  ```omni
  fn flush(&mut self):
      if !self.dirty: return
      let tmp = format("{}.tmp", data_path)
      fs::write_string(&tmp, serialize::to_json(self))  // Write to temp
      if fs::exists(data_path):
          fs::rename(data_path, format("{}.bak", data_path))  // Backup old
      fs::rename(tmp, data_path)  // Atomic rename
      self.dirty = false
  ```
- [ ] **HIGH**: Add startup recovery (verify JSON valid, restore from .bak if corrupted)
- [ ] **HIGH**: Add backup snapshots every 1000 writes
- [ ] **MEDIUM**: Write crash-recovery tests (simulate kill -9, verify data integrity)
- [ ] **MEDIUM**: Add auto-flush timer (every 60s or on 100 new facts)
- [ ] **LOW**: Performance test (10K facts, measure query latency)

**Deliverables:**
- 🟡 `helios-framework/helios/knowledge.omni` — INCOMPLETE: atomic flush needed (currently unsafe)
- ❌ `helios-framework/helios/checkpoint.omni` — Not created (recovery logic missing)
- ❌ Test suite: `helios-framework/tests/knowledge_safety.omni` — Tests directory doesn't exist (framework has ZERO tests)

**Success Criteria:**
- ❌ Zero data corruption on crash (currently: **CAN LOSE DATA** if process crashes during write)
- ❌ Recovery restores all knowledge correctly (currently: no recovery logic)
- ✅ <10ms write latency for single fact (appears achievable, needs measurement)
- ✅ <100ms query latency for 10K facts (appears achievable, needs measurement)

---

#### 1.1.3 Framework Compilation Verification
**Goal:** All framework code compiles end-to-end.

**Current Status:** 🟡 UNTESTED
- ✅ Framework files exist and have Omni syntax
- ✅ `helios-framework/main.omni` entry point defined (11.5 KB)
- ✅ All `helios/*.omni` modules exist (10 files, ~135 KB total)
- ✅ All `brain/*.omni` modules exist (14 files, ~170 KB total)
- ✅ `app/` modules exist (4 files)
- 🟡 **UNKNOWN**: Whether compilation actually works (no verify test run yet)
- ❌ No integration test `compiler/tests/framework_compile.rs` exists
- ❌ Any parser/codegen bugs NOT YET DISCOVERED (because compilation hasn't been tested)

**Tasks:**
- [ ] **CRITICAL Day 1**: Run `omnc compile helios-framework/main.omni` and document all errors
- [ ] **HIGH**: Fix parser gaps discovered during compilation test
- [ ] **HIGH**: Fix codegen bugs for Omni features used by framework (structs, imports, patterns)
- [ ] **MEDIUM**: Create integration test file `compiler/tests/framework_compile.rs` with CI gate
- [ ] **LOW**: Add compilation step to build_and_deploy.ps1

**Deliverables:**
- ❌ All `.omni` files in `helios-framework/` compile without errors (NOT YET VERIFIED)
- ❌ `compiler/tests/framework_compile.rs` — Not created yet

**Success Criteria:**
- ❌ `omnc compile helios-framework/main.omni` produces valid bytecode (UNVERIFIED)
- ❌ All framework modules can be loaded as dependencies (UNVERIFIED)
- ❌ No parser warnings on any framework file (UNVERIFIED)

---

#### 1.1.4 IPC Native Functions
**Goal:** Foundation for service-client communication.

**Current Status:** ❌ NOT IMPLEMENTED
- ❌ No `pipe_create`, `pipe_accept`, `pipe_connect` functions in `native.rs`
- ❌ No `socket_create`, `socket_bind`, `socket_listen` functions
- ❌ No IPC roundtrip tests
- ❌ **BLOCKER**: Bytecode VM has no native function calling support (see 1.1.1)
- ❌ Dependency `windows-sys` not in Cargo.toml features (required for Windows pipe API)

**Tasks:**
- [ ] **CRITICAL**: Add `windows-sys = { version = "0.52", features = ["Win32_Storage_FileSystem", "Win32_System_Pipes"] }` to Cargo.toml
- [ ] **CRITICAL**: Add 6 Windows named pipe natives to `native.rs`:
  ```rust
  ("ipc", "pipe_create") → CreateNamedPipeW
  ("ipc", "pipe_accept") → ConnectNamedPipe
  ("ipc", "pipe_connect") → CreateFileW
  ("ipc", "pipe_write") → WriteFile
  ("ipc", "pipe_read") → ReadFile with max size
  ("ipc", "pipe_close") → CloseHandle
  ```
- [ ] **HIGH**: Add Unix socket natives (for Phase 2.1 multi-platform support)
- [ ] **MEDIUM**: Wire natives into bytecode VM (currently impossible: CallNative opcode doesn't exist)
- [ ] **LOW**: Write comprehensive IPC round-trip tests

**Deliverables:**
- ❌ `compiler/src/runtime/native.rs` — 12 new IPC functions (NOT IMPLEMENTED)
- ❌ Updated `compiler/Cargo.toml` with `windows-sys` dependency (PENDING)
- ❌ Test: `compiler/tests/ipc_native_test.rs` (NOT CREATED)

**Success Criteria:**
- ❌ Pipe creation succeeds on Windows (NOT TESTED)
- ❌ Message roundtrip works (NOT TESTED)
- ❌ No resource leaks (NOT TESTED)

---

### Phase 1.2: Service Layer (Months 4-6)

#### 1.2.1 Named Pipe IPC Wrapper (Omni)
**Goal:** High-level IPC protocol for service-client communication.

**Tasks:**
- [ ] Create `helios-framework/helios/ipc.omni` (new file, ~500 LOC)
  - `IpcServer`: accepts connections on named pipe
  - `Message`: JSON wrapper for RPC
  - `IpcClient`: connects to named pipe
  - Request routing (map `{method: "...", args: {...}}` to handlers)
- [ ] Create request/response types
- [ ] Add JSON serialization for messages
- [ ] Implement async message handling (tokio underneath)

**Deliverables:**
- `helios-framework/helios/ipc.omni` — Full IPC server/client
- Test: `helios-framework/tests/ipc_server.omni`

**Success Criteria:**
- Server listens on `\\.\pipe\helios-{pid}`
- Client connects successfully
- Messages roundtrip without corruption
- 100+ concurrent connections supported
- <50ms message latency

---

#### 1.2.2 Service State Machine
**Goal:** Proper lifecycle management.

**Current Status:** 🟡 PARTIALLY SKETCHED
- ✅ `service.omni` file exists (4.2KB)
- 🟡 Basic structure outlines state idea
- ❌ No proper state enum implementation
- ❌ No state transitions implemented
- ❌ No health check endpoint
- ❌ No graceful shutdown logic
- ❌ No error recovery with backoff

**Tasks:**
- [ ] **HIGH**: Implement state machine in `service.omni`:
  - Enum: `ServiceState { Starting, Ready, Processing, ShuttingDown, Stopped }`
  - Transitions: startup → Ready, input received → Processing, shutdown signal → ShuttingDown
- [ ] **HIGH**: Add health check endpoint (HTTP + IPC)
- [ ] **MEDIUM**: Implement graceful shutdown (flush knowledge, close pipes, cleanup)
- [ ] **MEDIUM**: Add error recovery (auto-restart with exponential backoff)
- [ ] **MEDIUM**: Add logging (startup, transitions, errors)
- [ ] **LOW**: Add telemetry (uptime, request count, error count)

**Deliverables:**
- 🟡 `service.omni` — INCOMPLETE: state machine skeleton exists but not fully implemented

**Success Criteria:**
- ❌ Service transitions through all states correctly (NOT IMPLEMENTED)
- ❌ Graceful shutdown persists knowledge (NOT TESTED)
- ❌ Health check returns <10ms (NOT IMPLEMENTED)
- ❌ Auto-restart with exponential backoff works (NOT IMPLEMENTED)

---

#### 1.2.3 Dual-Listen Service (HTTP + IPC)
**Goal:** Service listens on both HTTP:8765 and named pipe simultaneously.

**Tasks:**
- [ ] Update `helios-framework/main.omni` to:
  - Start HTTP listener (existing `api.omni`)
  - Start IPC listener (new `ipc.omni`)
  - Both running concurrently
  - Shared request handling (delegate both to same `process_input()`)
  - Signal handling (on SIGTERM, gracefully shut down both)
- [ ] Add request multiplexing (route both HTTP + IPC to same handlers)
- [ ] Test concurrent requests on both ports

**Deliverables:**
- Updated `main.omni` with parallel listeners

**Success Criteria:**
- HTTP and IPC both receive requests simultaneously
- Requests processed correctly on either channel
- Shutdown cleanly closes both listeners
- 100+ concurrent requests handled

---

### Phase 1.3: Cognitive L0 + L1 (Months 7-9)

#### 1.3.1 Cognitive L0 (Reflex) Layer
**Goal:** Instant knowledge lookup.

**Tasks:**
- [ ] Implement in `helios-framework/helios/cognitive.omni`:
  - `Reflex::new()` — create reflex layer
  - `Reflex::query(pattern)` → immediate fact match or null
  - `Reflex::record(fact)` → add to knowledge store
  - Pattern matching: `{subject: "France", predicate: ?}` → matches any value
- [ ] Integrate with `KnowledgeStore` (direct index lookup)
- [ ] Add confidence scoring (only return if confidence > threshold)
- [ ] Add response formatting (convert fact to natural language)
- [ ] Performance: <10ms for any query (99th percentile)

**Deliverables:**
- `cognitive.omni` L0 implementation
- Test: `helios-framework/tests/cognitive_reflex.omni`

**Success Criteria:**
- Query "What is capital of France" → returns "Paris" from knowledge
- Response latency <10ms (99%ile)
- Accuracy 100% (returns correct facts)
- Can handle 1000+ queries/sec

---

#### 1.3.2 RETE Engine (L1 Forward Chaining)
**Goal:** Basic multi-fact inference via rules.

**Tasks:**
- [ ] Create `helios-framework/brain/rete.omni` (~800 LOC):
  - `ReteNetwork`: stores rules, facts, partial matches
  - `Rule`: `Pattern1 AND Pattern2 → Action`
  - `Pattern`: `{field1: value1, field2: ?var}` with variable binding
  - `execute_rule()`: fire rule when conditions met
  - `insert_fact()`: trigger rule evaluation
  - `query_facts()`: retrieve all known facts
- [ ] Implement 2-condition rules initially (extend later)
- [ ] Forward chaining: new facts trigger rule re-evaluation
- [ ] Performance: fire rules in <100ms

**Deliverables:**
- `helios-framework/brain/rete.omni` — RETE network implementation
- Test: `helios-framework/tests/rete_rules.omni`

**Success Criteria:**
- Rule firing adds derived facts to knowledge store
- Chaining works (rule A fires → adds fact → rule B fires)
- 100+ rules run without slowdown
- Query-derived facts correctly
- <100ms per rule execution

---

#### 1.3.3 Experience Log Integration
**Goal:** Record every inference for learning + debugging.

**Tasks:**
- [ ] Enhance `helios-framework/helios/experience.omni`:
  - `log_input()` — record user query
  - `log_inference()` — record internal reasoning (which facts matched, which rules fired)
  - `log_response()` — record response generated
  - `log_confidence()` — record confidence score
- [ ] Serialize to JSON (separate from knowledge store)
- [ ] Append-only WAL (never rewrite, only append)
- [ ] Query experience (what did we infer about X?)
- [ ] Export experience (for offline analysis / training)

**Deliverables:**
- Updated `experience.omni` with full logging
- Test: `helios-framework/tests/experience_log.omni`

**Success Criteria:**
- Every inference logged with timestamps
- Experience persists across service restarts
- Memory efficient (no unbounded growth)
- Query experience: "What rules fired on user input X?"

---

### Phase 1.4: GUI Foundation (Months 7-9)

#### 1.4.1 WinUI3 Project Scaffold
**Goal:** Minimal GUI that connects to service.

**Tasks:**
- [ ] Create C# WinUI3 project:
  ```
  gui/WinUI3/HeliosGui.sln
  ├── HeliosGui/
  │   ├── MainWindow.xaml + xaml.cs
  │   ├── Views/ConversationPage.xaml (main chat)
  │   ├── ViewModels/ConversationViewModel.cs
  │   ├── Services/HeliosClient.cs (IPC client)
  │   ├── App.xaml + xaml.cs
  │   └── Resources/ (icons, themes)
  └── HeliosGui.Tests/ (unit tests)
  ```
- [ ] Add NuGet dependencies:
  - `Microsoft.WindowsAppSDK`
  - `MessagePack` (for serialization)
  - `System.IO.Pipelines` (async pipes)
  - `CommunityToolkit.Mvvm` (MVVM)
- [ ] Create basic UI: text box (input), text block (output)
- [ ] Implement IPC client connection

**Deliverables:**
- ❌ `gui/WinUI3/HeliosGui.sln` — NOT CREATED
- ❌ `gui/WinUI3/HeliosGui/` — NOT CREATED
- ❌ NuGet dependencies — NOT RESOLVED

**Success Criteria:**
- ❌ `dotnet build` succeeds (CANNOT TEST: project doesn't exist)
- ❌ UI compiles without errors (NOT TESTED)
- ❌ Can instantiate HeliosClient (NOT TESTED)

---

#### 1.4.2 IPC Client Implementation
**Goal:** GUI talks to service via named pipe.

**Current Status:** ❌ NOT IMPLEMENTED
- ❌ No `HeliosClient.cs` implementation
- ❌ **BLOCKER**: WinUI3 project doesn't exist yet
- ❌ **BLOCKER**: Service IPC endpoint not implemented (Phase 1.2.1)

**Tasks:**
- [ ] **HIGH**: Implement `HeliosClient.cs` (~300 LOC):
  - `ConnectAsync(pipeName)` → connect to service
  - `SendQueryAsync(query: string)` → `response: string`
  - `OnResponseReceived` event for UI binding
  - Error handling (retry, timeout, disconnect detection)
  - JSON serialization/deserialization
- [ ] **MEDIUM**: Message format implementation
- [ ] **MEDIUM**: Connection pooling + reconnect logic
- [ ] **LOW**: Write IPC client tests

**Deliverables:**
- ❌ `gui/WinUI3/HeliosGui/Services/HeliosClient.cs` — NOT CREATED
- ❌ Test: `gui/WinUI3/HeliosGui.Tests/IpcClientTest.cs` — NOT CREATED

**Success Criteria:**
- ❌ Connects to running service (CANNOT TEST: project doesn't exist)
- ❌ Sends query, receives response (CANNOT TEST)
- ❌ Handles disconnect gracefully (CANNOT TEST)
- ❌ <100ms roundtrip latency (CANNOT TEST)

---

#### 1.4.3 Chat Page Implementation
**Goal:** MVP UI flow.

**Tasks:**
- [ ] Implement `ConversationPage.xaml`:
  - Input TextBox + Send button
  - Output TextBlock (scrollable)
  - Status indicator (connected/waiting/error)
  - Chat history list (scroll, copy, etc.)
- [ ] Connect to `ConversationViewModel`:
  - `OnSendClicked` → send query via IPC
  - Bind response to UI text block
  - Add to chat history
  - Show confidence score
  - Disable send button while awaiting response
- [ ] Add error handling (show error dialog on failure)
- [ ] Add loading indicator while waiting

**Deliverables:**
- `/ConversationPage.xaml`
- `/ViewModels/ConversationViewModel.cs`
- Full UI flow

**Success Criteria:**
- Type message → see response
- Chat history retained
- Status indicator reflects connection state
- All UI interactions work without exceptions

---

### Phase 1.5: Packaging & Deployment (Months 10-13)

#### 1.5.1 Service Wrapper (Windows Service)
**Goal:** Register HELIOS as Windows service.

**Tasks:**
- [ ] Create `tools/service_bridge.rs` (Rust, ~400 LOC):
  - Implements `windows_service::ServiceMainFunction`
  - Launches shell command to run `helios-framework` binary
  - Handles start/stop/pause/continue service events
  - Logs to Windows Event Log
  - Handles service shutdown signal (gracefully shutdown Helios)
- [ ] Add `windows-service` crate to Cargo
- [ ] Build release binary: `helios-service.exe`

**Deliverables:**
- `tools/service_bridge.rs` complete implementation
- Compiled `helios-service.exe` binary

**Success Criteria:**
- Service registers without errors: `sc create HeliosService`
- Service starts: `Start-Service HeliosService`
- Service stops gracefully: `Stop-Service HeliosService`
- Events logged to Application Event Viewer

---

#### 1.5.2 Installation Scripts
**Goal:** One-click install/uninstall.

**Tasks:**
- [ ] Create `scripts/install.ps1`:
  - Check admin privileges
  - Copy binaries to `C:\Program Files\Helios\bin\`
  - Copy configs to `C:\Program Files\Helios\config\`
  - Create `C:\Users\{user}\AppData\Local\Helios\data\` for knowledge
  - Register Windows service
  - Start service
  - Verify service started (wait 5s, check Status = Running)
- [ ] Create `scripts/uninstall.ps1`:
  - Stop service
  - Unregister service
  - Delete files
  - Remove shortcuts
  - Cleanup

**Deliverables:**
- `scripts/install.ps1` (150 LOC)
- `scripts/uninstall.ps1` (100 LOC)

**Success Criteria:**
- Fresh Windows install: run `install.ps1` → all files copied, service running
- Verify service accessible via IPC
- Uninstall completely removes all files

---

#### 1.5.3 Distribution Package
**Goal:** Deployable ZIP containing all components.

**Tasks:**
- [ ] Create directory structure:
  ```
  dist/helios-v1.0/
  ├── bin/
  │   ├── omnc.exe              ← Compiler
  │   ├── opm.exe               ← Package manager
  │   ├── helios-service.exe    ← Service wrapper
  │   └── HeliosGui.exe         ← GUI app
  ├── lib/
  │   ├── helios-framework.ovc  ← Compiled framework bytecode
  │   └── stdlib.ovc            ← Stdlib bytecode
  ├── config/
  │   └── default.toml          ← Default configuration
  ├── scripts/
  │   ├── install.ps1
  │   └── uninstall.ps1
  ├── data/                     ← Empty initially (knowledge store location)
  ├── docs/
  │   ├── README.md
  │   ├── USER_GUIDE.md
  │   └── CHANGELOG.md
  └── LICENSE
  ```
- [ ] Create installer script (`build_and_deploy.ps1`) that:
  - Builds all components (omnc, opm, service, GUI)
  - Copies to `dist/`
  - Creates ZIP: `dist/helios-v1.0.zip`
  - Calculates checksums
  - Signs (optional)
- [ ] Test install on clean Windows VM

**Deliverables:**
- `dist/helios-v1.0.zip` (full distribution)
- `build_and_deploy.ps1` (build orchestration script)
- `CHECKSUMS.txt` (for integrity verification)

**Success Criteria:**
- ZIP under 200MB
- Unzip + run install.ps1 works on fresh Windows 10/11
- Service starts and accepts IPC queries
- GUI connects and works

---

### Phase 1.6: Testing & Documentation (Months 11-18)

#### 1.6.1 Comprehensive Test Suite
**Goal:** 80%+ code coverage, all scenarios passing.

**Tasks:**
- [ ] Integration tests (Rust):
  - `compiler/tests/framework_compile.rs` — compile all framework files
  - `compiler/tests/runtime_integration.rs` — all native functions work
  - `compiler/tests/bytecode_execute.rs` — compile + execute bytecode
- [ ] Framework tests (Omni):
  - `helios-framework/tests/knowledge_test.omni` — store/query/persist
  - `helios-framework/tests/service_test.omni` — startup/shutdown/state
  - `helios-framework/tests/cognitive_test.omni` — L0+L1 reasoning
- [ ] GUI tests (C#):
  - `gui/HeliosGui.Tests/IpcClientTest.cs` — IPC communication
  - WinAppDriver tests for UI interactions
- [ ] Acceptance tests (PowerShell):
  - `tests/acceptance_test.ps1` — End-to-end: remember → ask → verify
  - `tests/stress_test.ps1` — 1000 concurrent queries
  - `tests/crash_recovery_test.ps1` — Kill service, restart, verify data

**Deliverables:**
- Test suite in all languages
- CI/CD pipeline ready (GitHub Actions)

**Success Criteria:**
- All tests pass
- >80% code coverage
- <5 false failures per day
- Each test takes <10s

---

#### 1.6.2 Documentation
**Goal:** Complete guides for users and developers.

**Tasks:**
- [ ] **User Documentation**:
  - `docs/USER_GUIDE.md` — How to use GUI
  - `docs/GETTING_STARTED.md` — Installation and first run
  - `docs/KNOWN_ISSUES.md` — Known bugs, workarounds
- [ ] **Developer Documentation**:
  - `docs/BUILDING.md` — Build from source
  - `docs/ARCHITECTURE.md` — System design overview
  - `docs/API.md` — HTTP + IPC API reference with examples
  - `docs/PLUGIN_SDK.md` — How to write plugins (preview)
- [ ] **Command-line Help**:
  - `omnc --help` prints usage
  - `opm --help` lists subcommands
  - Every command has `--help` flag

**Deliverables:**
- 5+ markdown docs
- In-code documentation (docstrings, comments)
- Runnable code examples

**Success Criteria:**
- Docs are complete and accurate
- Examples run without modification
- <5 minute setup for new user

---

### Phase 1.7: MVP Release (Month 18)

**Final Release Checklist:**

- [ ] All Phase 1 tasks complete
- [ ] All tests passing
- [ ] Documentation reviewed
- [ ] Installer tested on 2+ fresh Windows VMs
- [ ] GUI UX reviewed (usability)
- [ ] Performance benchmarks met (<100ms query latency)
- [ ] Security review (code audit for injection, overflow, etc.)
- [ ] Release notes written
- [ ] GitHub Release created with v1.0.0 tag
- [ ] Announcement post written

**Deliverables:**
- `v1.0.0` GitHub Release
- Installer `helios-v1.0.zip`
- Release notes + changelog
- Security advisory (if any bugs found)

---

## Phase 2: Hardening & Expansion (Year 1.5 — 3 Years)

### Phase 2.1: Multi-Platform Support (Months 19-27)

#### 2.1.1 Linux Port
**Goal:** Full Linux support (Ubuntu/CentOS primary).

**Tasks:**
- [ ] Update IPC natives for Linux (Unix sockets instead of named pipes)
- [ ] Update build scripts (`build_and_deploy.ps1` → cross-platform scripts)
- [ ] Test on Ubuntu 20.04 LTS + Ubuntu 22.04 LTS
- [ ] Package as `.deb` (Debian package)
- [ ] Package as `.rpm` (Red Hat package)
- [ ] Create systemd service (Linux equivalent of Windows service)
- [ ] Database: add SQLite backend option (Linux commonly doesn't have NTFS)

**Deliverables:**
- Linux binaries (omnc, opm, helios-service, GUI)
- Install/uninstall scripts for Linux
- `.deb` and `.rpm` packages
- systemd service unit file

**Success Criteria:**
- `apt install helios-1.0.deb` works
- systemctl start helios → service running
- GUI connections work
- Performance equivalent to Windows

---

#### 2.1.2 macOS Port
**Goal:** macOS 12+ support (Intel + Apple Silicon M1/M2).

**Tasks:**
- [ ] Compile for both x86_64 and arm64 architectures
- [ ] Create `.dmg` installer (macOS standard)
- [ ] Create `.pkg` installer (alternative)
- [ ] Add to Homebrew (community package manager)
- [ ] Code signing + notarization (Apple requirements)
- [ ] GUI: ensure SwiftUI compat (not just WinUI3)
  - Consider: create macOS-native GUI using Swift or keep WinUI3 cross-platform

**Deliverables:**
- macOS binaries (both architectures)
- Homebrew formula
- `.dmg` installer
- Code-signed and notarized

**Success Criteria:**
- `brew install helios` works
- Runs on Apple Silicon without Rosetta
- Performance comparable to Windows/Linux

---

### Phase 2.2: Backward Chaining (L2) — Months 21-30

#### 2.2.1 L2 Implementation
**Goal:** Query-driven reasoning (prove goals from facts and rules).

**Tasks:**
- [ ] Create `helios-framework/brain/backward_chaining.omni` (~1000 LOC):
  - `Goal`: `{subject: ?, predicate: ?}` to prove
  - `prove(goal)` → true/false + proof tree
  - Apply rules in reverse: goal matches rule's consequent → add rule's antecedents as sub-goals
  - Handle negation: `NOT factX` in rule conditions
  - Handle cut: `!` stop exploring alternatives
  - Memoization: cache proved goals (avoid re-proving)
- [ ] Integrate with L0 + L1 (combined reasoning stack)
- [ ] Performance: prove goal in <1s (99%ile)

**Deliverables:**
- `backward_chaining.omni` (backward chaining engine)
- Test: `helios-framework/tests/backward_chain.omni`

**Success Criteria:**
- Prove complex goals (5+ levels of indirection)
- Negation handling correct
- Memoization improves performance 10x
- All tests pass

---

### Phase 2.3: Plugin System (Months 22-32)

#### 2.3.1 Plugin Manifest & Security
**Goal:** Plugins with fine-grained permissions + code signature verification.

**Tasks:**
- [ ] Define plugin manifest format:
  ```toml
  [plugin]
  name = "MyPlugin"
  version = "1.0.0"
  author = "acme"
  
  [manifest]
  entry_point = "my_plugin.main"
  capabilities = ["read_files", "write_knowledge", "network"]
  
  [signature]
  public_key = "ed25519:abc123"
  signature = "hex:..."
  ```
- [ ] Create capability system:
  - `read_files` — plugin can call `io::readfile`
  - `write_knowledge` — plugin can add facts to knowledge
  - `network` — plugin can make HTTP requests
  - `execute_code` — plugin can spawn processes
- [ ] Implement capability checking in OVM (permission checks at native call sites)
- [ ] Implement code signing/verification (BLAKE3 + Ed25519 signatures)
- [ ] Create plugin sandbox (limit memory, CPU, network bandwidth)

**Deliverables:**
- Plugin manifest spec document
- Capability enforcement in `native.rs`
- Code signature verification in `ovm`
- Permission checker tests

**Success Criteria:**
- Plugin loads only if signature valid
- Plugin cannot access denied capabilities
- Sandbox prevents resource exhaustion
- Plugin can be revoked immediately

---

#### 2.3.2 Plugin SDK + Tooling
**Goal:** Easy plugin development.

**Tasks:**
- [ ] Create plugin template:
  ```bash
  opm init-plugin my-plugin
  # Creates:
  # - plugin.toml (manifest template)
  # - src/lib.omni (entry point)
  # - tests/tests.omni
  ```
- [ ] Create `opm plugin build` command (compile plugin)
- [ ] Create `opm plugin sign` command (sign with key)
- [ ] Create `opm plugin install` command (install locally)
- [ ] Create plugin examples:
  - File ingester plugin (read data files, ingest to knowledge)
  - Web crawler plugin (fetch URLs, extract facts)
  - Calculator plugin (math query handler)

**Deliverables:**
- Plugin templates + examples
- SDK documentation
- `opm plugin` subcommands

**Success Criteria:**
- `opm init-plugin` creates working template
- Plugin compiles + signs + installs without errors
- Examples run successfully

---

### Phase 2.4: Distributed Communication (Months 25-36)

#### 2.4.1 Service Federation
**Goal:** Multiple Helios instances coordinate reasoning.

**Tasks:**
- [ ] Create `helios-framework/brain/federation.omni` (~800 LOC):
  - `FederationNode`: represents peer HELIOS instance
  - `discover_peers()` — find other nodes (DNS, broadcast, config)
  - `sync_facts()` — replicate facts to peers
  - `query_remote()` — ask peer for reasoning
  - `consensus()` — vote on derived facts (if confidence < threshold)
- [ ] Implement gossip protocol (periodic sync of new facts)
- [ ] Implement quorum-based consistency (n/2 + 1 nodes must agree)
- [ ] Handle network failures (retry, fallback to local reasoning)

**Deliverables:**
- `federation.omni`
- Test: `helios-framework/tests/federation.omni`

**Success Criteria:**
- 3 nodes discover each other
- Fact syncs to all peers
- Quorum consensus works
- Handles 1 node failure gracefully

---

#### 2.4.2 Inter-Service IPC (Omni-to-Omni)
**Goal:** Efficient message passing between Helios instances.

**Tasks:**
- [ ] Create `helios-framework/helios/rpc.omni`:
  - Define RPC message format (method, args, id)
  - Async request-response pattern
  - Multiplexing (many requests in-flight simultaneously)
  - Error handling (timeout, network error, service error)
- [ ] Test 10+ concurrent federation messages
- [ ] Performance: <50ms latency for federation query

**Deliverables:**
- `rpc.omni` RPC framework
- Test: integration tests

**Success Criteria:**
- Remote method calls work over IPC
- Multiplexing maintains order
- <50ms latency

---

### Phase 2.5: Knowledge Store V2 (Months 28-36)

#### 2.5.1 B+ Tree Indexing
**Goal:** Faster queries on large knowledge bases.

**Tasks:**
- [ ] Design B+ tree structure (order 32, suitable for in-memory)
- [ ] Implement in `helios-framework/helios/knowledge_v2.omni`:
  - `BTreeIndex` data structure
  - Insert, delete, search operations
  - Range queries (all facts matching predicate)
- [ ] Compare performance: HashMap vs B+ tree (10K, 100K, 1M facts)
- [ ] Add optional binary serialization (MessagePack)

**Deliverables:**
- Knowledge store V2 with B+ tree
- Migration script (JSON → binary format)

**Success Criteria:**
- Query latency <5ms (99%ile) on 100K facts (vs >50ms with HashMap)
- Serialization reduces storage 30%+
- No data loss during migration

---

#### 2.5.2 Compression
**Goal:** Reduce knowledge store size.

**Tasks:**
- [ ] Add compression options:
  - Zstd (default, balanced)
  - Brotli (higher ratio, slower)
  - Gzip (compatibility)
- [ ]Pages: compress each 1MB page independently
- [ ] Benchmark (compression ratio, write latency, read latency)

**Deliverables:**
- Compression integration
- Performance benchmarks

**Success Criteria:**
- 50%+ compression on typical knowledge
- <50ms decompress on read
- <500ms recompress on flush

---

### Phase 2.6: Commercial Licensing (Months 24-36)

#### 2.6.1 Licensing Engine
**Goal:** Support multiple license types.

**Tasks:**
- [ ] Define license types:
  - **Community** — Free, single-user, non-commercial
  - **Professional** — $$ per year, 1 developer, 5 nodes max
  - **Enterprise** — $$$ per year, 100+ developers, unlimited nodes
  - **Trial** — Free for 30 days, all features
- [ ] Implement license check:
  - On startup, read license file
  - Validate signature (must be signed by Helios key)
  - Check expiration date
  - Check feature limits (node count, user count, etc.)
  - Store check result in memory (don't re-verify every call)
- [ ] Add command: `omnc license status` (show current license)
- [ ] Add command: `omnc license activate <key>` (apply new license)

**Deliverables:**
- License checking code in service startup
- License management commands
- License format specification

**Success Criteria:**
- License validation takes <10ms
- Enterprises can manage 100+ nodes via licensing
- Community tier fully functional for personal use

---

### Phase 2.7: Support Infrastructure (Months 30-36)

#### 2.7.1 Telemetry & Monitoring
**Goal:** Understand service health + usage.

**Tasks:**
- [ ] Add telemetry collection (opt-in):
  - Service uptime
  - Queries per hour
  - Average query latency
  - Error rate
  - Plugin activations
- [ ] Create `/telemetry` HTTP endpoint
- [ ] Export to monitoring tools (Prometheus format)
- [ ] Create monitoring dashboard (Grafana template)

**Deliverables:**
- Telemetry infrastructure
- Prometheus exporter format
- Grafana dashboard

**Success Criteria:**
- Telemetry adds <1% overhead
- Can monitor 1000+ service instances via central dashboard

---

#### 2.7.2 Support Portal & Documentation
**Goal:** Handle customer requests + provide FAQ.

**Tasks:**
- [ ] Create support website:
  - FAQ (50+ common issues)
  - Knowledge base articles
  - Contact form (routes to support team)
  - Issue tracker (public-facing for Known Issues)
- [ ] Create internal support tools:
  - License lookup database
  - Customer account management
  - Telemetry dashboard
  - Support ticket system
- [ ] Document support procedures (SLAs, escalation)

**Deliverables:**
- Support portal
- FAQ / knowledge base
- Support ticket system

**Success Criteria:**
- 90% of questions answered in FAQ
- <8 hour response time for support tickets
- <24 hour resolution for tier-1 issues

---

## Phase 3: Advanced Reasoning (Year 3 — 5 Years)

### Phase 3.1: Causal Inference (L3) — Months 37-54

#### 3.1.1 Causal DAG Learning
**Goal:** Automatically learn causal relationships from observations.

**Tasks:**
- [ ] Create `helios-framework/brain/causal.omni` (~2000 LOC):
  - `CausalDAG`: directed acyclic graph of causal relationships
  - `learn_causal_graph(observations)` → DAG
  - Algorithms: constraint-based (PC algorithm), score-based (GES)
  - Handle hidden confounders (instrumental variables)
- [ ] Integrate with knowledge store (facts have causal relationships)
- [ ] Query: "What causes X?" → trace DAG

**Deliverables:**
- `causal.omni` with causal inference engine
- Test: learn causal graph from synthetic data

**Success Criteria:**
- Learns causal graph correctly on benchmark datasets
- Identifies hidden confounders (if data allows)
- >80% accuracy on causality queries

---

#### 3.1.2 Counterfactual Reasoning
**Goal:** Answer "what if" questions.

**Tasks:**
- [ ] Implement counterfactual inference:
  - `if_then_what(cause_changed, effect_query)` → predicted effect
  - Use causal DAG + propensity scoring
  - Monte Carlo sampling for uncertainty quantification
- [ ] Test on benchmark (Pearl's Causal Model theory)

**Deliverables:**
- Counterfactual reasoning implementation
- Test suite

**Success Criteria:**
- Counterfactual predictions match ground truth (on synthetic data)
- Uncertainty intervals contain true value (95% coverage)

---

### Phase 3.2: Distributed Consensus (L4) — Months 48-60

#### 3.2.1 Consensus Algorithm
**Goal:** Multiple Helios nodes agree on facts despite failures.

**Tasks:**
- [ ] Choose consensus algorithm:
  - **PBFT (Practical Byzantine Fault Tolerance)** — robust, n ≥ 3f+1
  - **Raft** — simpler, n ≥ f+1
  - **Recommendation:** Raft for v1, PBFT for v2
- [ ] Implement in `helios-framework/brain/consensus.omni`:
  - Leader election
  - Log replication
  - Snapshotting (periodic)
  - Fault tolerance (tolerate f node failures with n ≥ 2f+1)
- [ ] Test:
  - Add fact on node A → replicated to B, C, D
  - Kill node A → still consistent
  - Network partition → resolve after healing

**Deliverables:**
- Consensus implementation (Raft)
- Test suite

**Success Criteria:**
- Cluster of 5 nodes handles 1 failure
- All non-faulty nodes reach consistency within 1s
- Data loss is zero

---

#### 3.2.2 Distributed Query Processing
**Goal:** Query can be processed across multiple nodes.

**Tasks:**
- [ ] Implement distributed query planner:
  - Decompose query into subqueries
  - Assign subqueries to nodes (based on data locality)
  - Execute in parallel
  - Combine results
- [ ] Handle fault tolerance (retry on node failure)
- [ ] Performance: 10-node cluster processes queries faster than single node

**Deliverables:**
- Distributed query planner
- Test: 10-node cluster handles queries

**Success Criteria:**
- Queries return correct results with any node failures
- 10-node cluster 8x faster than single for complex queries

---

### Phase 3.3: Knowledge Synthesis — Months 48-60

#### 3.3.1 Knowledge Integration from Multiple Sources
**Goal:** Merge knowledge from multiple data sources without conflicts.

**Tasks:**
- [ ] Implement source tracking (each fact has origin — user input, plugin, inference, external)
- [ ] Implement conflict resolution:
  - Human-provided facts >plugin facts >inference >external
  - Confidence-based (higher confidence wins ties)
  - Temporal (more recent wins)
- [ ] Implement data fusion (combine multiple measurements of same thing)
- [ ] Test on knowledge from 3+ sources

**Deliverables:**
- Source tracking + conflict resolution

**Success Criteria:**
- Conflicts resolved consistently
- No data loss
- User can see provenance of each fact

---

### Phase 3.4: Cluster Orchestration — Months 54-66

#### 3.4.1 Kubernetes Support
**Goal:** Deploy HELIOS cluster on Kubernetes.

**Tasks:**
- [ ] Create Kubernetes manifests:
  - Deployment (pod replicas)
  - Service (load balancer)
  - ConfigMap (configuration)
  - PersistentVolume (knowledge store)
  - StatefulSet (ordered, stable pod identity)
- [ ] Add health checks (liveness, readiness probes)
- [ ] Add metrics (Prometheus export)
- [ ] Test: `kubectl apply -f helios.yaml` → 3-node cluster running

**Deliverables:**
- K8s manifests + documentation

**Success Criteria:**
- Cluster scales up/down smoothly
- Zero downtime during rolling updates
- Persistent knowledge across pod recreation

---

#### 3.4.2 High Availability & Disaster Recovery
**Goal:** Service survives component failures.

**Tasks:**
- [ ] Implement:
  - Primary + standby configuration (automatic failover)
  - Backup to separate datacenter (async replication)
  - Recovery point objective (RPO): ≤ 1 minute of data loss
  - Recovery time objective (RTO): ≤ 5 minutes to failover
- [ ] Test failure scenarios:
  - One node crashes → failover to replica
  - Primary datacenter floods → activate remote backup
- [ ] Document runbooks (disaster recovery procedures)

**Deliverables:**
- HA configuration + runbooks
- Failover tests

**Success Criteria:**
- Automatic failover in <5 minutes
- Zero data loss (RPO met)
- Recovery achievable without manual intervention

---

## Phase 4: Enterprise & Multi-Tenant (Year 5 — 7 Years)

### Phase 4.1: Multi-Tenant Architecture — Months 61-84

#### 4.1.1 Tenant Isolation
**Goal:** Single HELIOS deployment serves many customers, fully isolated.

**Tasks:**
- [ ] Implement isolation at multiple levels:
  - **API isolation**: each API call includes tenant ID
  - **Database isolation**: facts stored in `tenant/{tenant_id}/` prefix
  - **Resource isolation**: each tenant has quotas (knowledge size, QPS, storage)
  - **Compute isolation**: (optional for v1) VMs or containers per tenant
- [ ] Add tenant management API:
  - Create tenant
  - Set quotas
  - Monitor usage
  - Suspend/delete tenant
- [ ] Add billing integration (track usage, generate invoices)

**Deliverables:**
- Tenant isolation layer
- Multi-tenant API
- Billing integration

**Success Criteria:**
- Tenant A cannot access tenant B's data
- 1000 tenants on single cluster
- Quotas enforced strictly

---

#### 4.1.2 Role-Based Access Control (RBAC)
**Goal:** Fine-grained permissions for users.

**Tasks:**
- [ ] Define roles:
  - **Admin**: full control
  - **Developer**: can create/modify rules, query knowledge
  - **Analyst**: read-only access to reports
  - **Service**: machine account (API keys), limited scope
- [ ] Implement permission checks:
  - On every operation (query, update, delete)
  - Deny by default, explicit allow
  - Audit log every permission check (for compliance)
- [ ] Add RBAC management API + UI

**Deliverables:**
- RBAC engine
- Role definitions
- Permission checking

**Success Criteria:**
- Enforce permissions correctly
- Audit trail of all permission-based operations
- Can revoke access immediately

---

### Phase 4.2: Audit & Compliance — Months 70-84

#### 4.2.1 Audit Logging
**Goal:** Immutable record of all system activities.

**Tasks:**
- [ ] Create audit log (append-only):
  - User/API key making request
  - Timestamp
  - Operation (create fact, run rule, delete knowledge)
  - Resource affected
  - Result (success/failure)
  - Client IP address
- [ ] Store audit log separately from knowledge (can't be modified retroactively)
- [ ] Implement search/query on audit logs
- [ ] Retention policy (keep audit logs for 7 years, archive older)

**Deliverables:**
- Audit logging system
- Audit log search/export

**Success Criteria:**
- No gaps in audit trail
- Tamper-evident (cryptographically signed pages)
- SOC2 auditors satisfy with logs
- Export for compliance reporting

---

#### 4.2.2 Compliance Certifications
**Goal:** Certify for regulated industries.

**Tasks:**
- [ ] Prepare for:
  - **SOC2 Type II** (security, availability, processing integrity)
  - **HIPAA** (healthcare — if supporting PHI)
  - **GDPR** (EU privacy — data deletion, etc.)
  - **FedRAMP** (US government — if pursuing)
- [ ] Implement controls:
  - Encryption at rest (AES-256)
  - Encryption in transit (TLS 1.3)
  - Access controls (RBAC + audit logging)
  - Data retention policies
  - Incident response procedures
- [ ] Third-party audit (hire auditor firm)

**Deliverables:**
- Compliance documentation
- Third-party audit report
- Compliance certificates

**Success Criteria:**
- SOC2 Type II certified
- Can support regulated workloads (healthcare, finance)

---

### Phase 4.3: SaaS Infrastructure — Months 84-96

#### 4.3.1 Web Dashboard
**Goal:** Customers manage service via web UI.

**Tasks:**
- [ ] Build dashboard (React or Vue):
  - Authentication (OAuth 2.0 + MFA)
  - Tenant management (view/edit settings)
  - Knowledge browser (search + visualize facts)
  - Query builder (construct queries with UI)
  - Rule editor (write and test rules in browser)
  - User management (invite team members, manage roles)
  - Usage analytics (charts: queries/day, latency, errors)
  - Support tickets (create, track)
-[ ] Deploy dashboard with high availability
  - CDN for static assets (CloudFront or Cloudflare)
  - RDS for user accounts + settings
  - Auth service (AWS Cognito or Auth0)

**Deliverables:**
- Web dashboard
- Backend APIs for dashboard

**Success Criteria:**
- <2 second page load (optimized)
- No single point of failure
- Can manage 10,000+ active users simultaneously

---

#### 4.3.2 Managed API
**Goal:** Customer applications call HELIOS via public API.

**Tasks:**
- [ ] Design REST API (or gRPC):
  - `POST /api/v1/query` — submit query
  - `GET /api/v1/query/{id}` — poll for result
  - `POST /api/v1/fact` — add fact
  - `GET /api/v1/facts` — search facts
  - `POST /api/v1/rule` — define rule
  - Rate limiting (per API key, per tenant)
  - Versioning (v1, v2, etc.)
- [ ] API Gateway (rate limiting, routing, auth)
- [ ] SDK for customers (Python, Node.js, Java, Go)

**Deliverables:**
- Public API specification (OpenAPI/Swagger)
- API implementation
- SDKs

**Success Criteria:**
- API handles 10K QPS
- <200ms latency p99
- Backwards compatible versioning

---

### Phase 4.4: Billing & Monetization — Months 90-96

#### 4.4.1 Usage Metering
**Goal:** Track customer usage for billing.

**Tasks:**
- [ ] Meter:
  - API requests (per operation type: query, fact write, rule definition)
  - Data storage (knowledge GB-months)
  - Compute hours (CPU cores * hours)
  - Premium features (backward chaining, distributed, etc.)
- [ ] Track in real-time (or as close as possible)
- [ ] Expose via API (customers can check current month usage)

**Deliverables:**
- Metering system
- Usage reporting API

**Success Criteria:**
- Accurate to within 1%
- Customers can predict final bill
- Handle billing edge cases (free tier transitions, trial expiry)

---

#### 4.4.2 Billing & Invoicing
**Goal:** Generate invoices, process payment.

**Tasks:**
- [ ] Integrate with payment provider (Stripe or Zuora):
  - Monthly invoicing (usage-based)
  - Support subscriptions (fixed-cost tiers)
  - Handle free trial (charge card after x days)
  - Failed payment collection (retry logic)
- [ ] Generate PDF invoices
- [ ] Support multiple currencies
- [ ] Dunning (notify customers of upcoming payment failures)

**Deliverables:**
- Billing system + payment processor integration

**Success Criteria:**
- 99%+ successful payment collection
- Invoices accurate & clear
- Dispute resolution process documented

---

## Phase 5: Ecosystem & LLM Integration (Year 7 — 10 Years)

### Phase 5.1: LLM Integration — Months 97-120

#### 5.1.1 LLM Backends
**Goal:** Use LLMs for knowledge synthesis + response generation.

**Tasks:**
- [ ] Create `helios-framework/brain/llm.omni` (`~2000 LOC):
  - Adapter pattern: support multiple LLM providers
    - OpenAI GPT-4
    - Anthropic Claude
    - Open source Llama 2 (can self-host)
  - Each adapter implements: `generate(prompt) → response`
  - Caching (same prompt → cached response, no new API call)
  - Cost tracking (log $ spent on API calls per tenant)
  - Fallback chain (if GPT-4 fails, try Claude, then Llama)
- [ ] Tie LLMs into reasoning:
  - L0 lookup + present to LLM → natural language response
  - L1 rules + present to LLM → summarized findings
  - L3 causal → LLM explains causality in narrative

**Deliverables:**
- LLM integration layer
- Adapters for 3+ providers

**Success Criteria:**
- LLM responses rank-ordered by quality (user feedback)
- Cost per query <$0.01 (for GPT-4 cheap tier)
- Seamless fallback if primary LLM unavailable

---

#### 5.1.2 Fine-Tuning Support
**Goal:** Allow customers to fine-tune LLMs on their data.

**Tasks:**
- [ ] Enable fine-tuning pipeline:
  - Export knowledge + queries as training data
  - Submit to provider (OpenAI fine-tuning API)
  - Deploy fine-tuned model as preferred backend
  - Track model versions (can rollback)
- [ ] Measure fine-tuning ROI (compare quality before/after)
- [ ] Implement A/B testing (50% users test new model)

**Deliverables:**
- Fine-tuning integration
- Model versioning + rollback

**Success Criteria:**
- Fine-tuning improves response quality by 20%+
- A/B tests show statistically significant improvement
- Customers can revert to base model if needed

---

### Phase 5.2: WASM Plugin Runtime — Months 110-132

#### 5.2.1 WebAssembly Support
**Goal:** Plugins written in any language compiled to WASM.

**Tasks:**
- [ ] Create WASM runtime (embed Wasmer or Wasmtime in Omni):
  - Load `.wasm` module
  - Execute function calls
  - Enforce capability restrictions (plugins can't access disk without `read_files` cap)
  - Memory isolation (each plugin has 256MB max heap)
- [ ] Define plugin interface (WIT — WebAssembly Interface Types):
  - Function signature for `process_query(query_str) → response_str`
  - Capability declarations
- [ ] Support languages (show example plugins in):
  - Rust (most common)
  - C/C++
  - Python (via PyO3)
  - JavaScript (via wasm-pack)

**Deliverables:**
- WASM runtime integration
- Plugin interface definition (WIT)
- Example plugins in 4+ languages

**Success Criteria:**
- WASM plugins can call curated APIs (read_knowledge, write_knowledge, call_llm)
- Memory/CPU limits enforced
- Sandbox prevents capability escalation
- Performance within 2x of native

---

#### 5.2.2 Plugin Marketplace (AppStore for HELIOS)
**Goal:** Discover + install community plugins.

**Tasks:**
- [ ] Create marketplace website:
  - Browse plugins by category
  - Search
  - View ratings + reviews
  - Install with one click
  - Revenue split (70% developer, 30% Helios)
- [ ] Implement signing + verification (only signed plugins accepted)
- [ ] Automated testing of plugins (sandbox, run tests, check safety)
- [ ] Version management (install specific version, auto-update)

**Deliverables:**
- Plugin marketplace website
- Plugin submission + review process
- Automated testing harness

**Success Criteria:**
- 1000+ plugins published
- 10,000+ installs total
- <1% of plugins cause security issues
- Developer revenue model sustainable

---

### Phase 5.3: Mobile Support — Months 120-144

#### 5.3.1 iOS App
**Goal:** iOS app for HELIOS queries on the go.

**Tasks:**
- [ ] Create iOS app (Swift, SwiftUI):
  - Login (OAuth)
  - Submit query
  - View response (with reasoning trace)
  - Browse knowledge (my facts, etc.)
  - Voice input (speech-to-text)
  - Voice output (TTS for response)
- [ ] Offline support (cache recent queries/responses)
- [ ] Push notifications (when interesting facts added)
- [ ] Publish to App Store

**Deliverables:**
- iOS app (source + published on App Store)

**Success Criteria:**
- 50,000+ downloads
- 4.5+ star rating
- <2 second response time over 4G

---

#### 5.3.2 Android App
**Goal:** Android app (parallel to iOS).

**Tasks:**
- [ ] Create Android app (Kotlin, Jetpack Compose):
  - Same feature set as iOS
  - Material Design
  - Publish to Google Play

**Deliverables:**
- Android app

**Success Criteria:**
- Parallel with iOS in functionality and performance

---

### Phase 5.4: Cross-Platform Web Framework — Months 132-144

#### 5.4.1 Web App Framework
**Goal:** HELIOS apps run in browser.

**Tasks:**
- [ ] Choose framework:
  - Vue 3 or React (both good)
  - TypeScript (for type safety)
- [ ] Implement:
  - Routing (pages: home, query, knowledge browser, plugins, settings)
  - Real-time chat (WebSocket to service)
  - Dark mode
  - Multi-language support (i18n)
  - Mobile-responsive
- [ ] Component library (reusable UI controls)
- [ ] Deploy on CDN (CloudFront)

**Deliverables:**
- Web framework + starter template
- Component library
- Example apps

**Success Criteria:**
- Template-based app development takes <2 days
- Supports 1000+ concurrent users
- <1 second page load

---

### Phase 5.5: Community & Partnerships — Months 132-144

#### 5.5.1 Research Partnerships
**Goal:** Academic research using HELIOS.

**Tasks:**
- [ ] Approach universities:
  - MIT, Stanford, CMU (Carnegie Mellon), UC Berkeley
  - Offer free access to framework
  - Joint publications (research credit)
  - Internship program (students contribute to project)
- [ ] Fund grants (NSF, DARPA, EU Horizon)
- [ ] Conference talks + workshops

**Deliverables:**
- Research collaboration agreements
- Open source research extensions

**Success Criteria:**
- 5+ papers published using HELIOS
- 10+ university collaborations
- 50+ interns trained

---

#### 5.5.2 Enterprise Partnerships
**Goal:** Integration into enterprise products.

**Tasks:**
- [ ] Partner with:
  - Cloud providers (AWS, Azure, GCP) — HELIOS managed service
  - BI tools (Tableau, Looker) — HELIOS as data source
  - CRM systems (Salesforce) — HELIOS for customer insights
  - ERP systems (SAP) — HELIOS for supply chain reasoning
- [ ] Co-marketing
- [ ] Revenue sharing

**Deliverables:**
- Partnership agreements + integrations

**Success Criteria:**
- 10+ enterprise partnerships
- $50M+ annual revenue from partnerships

---

---

## Detailed Component Specifications

This section provides ultra-detailed specifications for each subsystem (expanded on from phases above).

### Omni Language Specification

**File:** `omni-lang/LANGUAGE_SPEC.md` (to create)

**Subsections:**
1. Type System (hindley-milner inference, subtyping, GADTs)
2. Effect System (pure, io, network, learning)
3. Linear Types (ownership tracking)
4. Pattern Matching (exhaustiveness, guards)
5. Module System (import, export, visibility)
6. Generics & Trait System (parametric polymorphism, trait bounds)
7. Macro System (hygiene, AST manipulation)
8. Async/Await (coroutines, structured concurrency)
9. Error Handling (Result types, error propagation)
10. Optimization (inlining, specialization, devirtualization)
11. Runtime (memory model, GC, FFI)
12. Standard Library (complete API docs for each module)

...

*[Note: Full specification would be 5000+ lines — see docs/HELIOS & Omni Language — Comprehensive.md for complete details]*

---

## Quality & Testing Strategy

### Test Coverage Goals by Phase

| Phase | Unit Tests | Integration | E2E | Coverage Target |
|-------|-----------|-------------|-----|-----------------|
| MVP   | All files | Core flows  | Happy path | 70% |
| Hard  | All files | Federation, plugins | All scenarios | 85% |
| Adv   | All files | Distributed reasoning | Failover scenarios | 90% |
| Ent   | All files | Multi-tenant, compliance | Audit trail, RBAC | 95% |
| Eco   | All files | LLM, WASM, mobile | Cross-platform | 95%+ |

### Testing Frameworks

- **Rust tests**: `cargo test` (built-in)
- **Omni tests**: `#[test]` attributes, `assert` macros
- **C# GUI tests**: WinAppDriver, xUnit
- **Acceptance tests**: PowerShell Pester
- **Performance tests**: Criterion (Rust), BenchmarkDotNet (C#)
- **Fuzz testing**: libFuzzer (via cargo-fuzz)
- **Property testing**: proptest (Rust)
- **Load testing**: Apache JMeter, custom Rust harness

### Each Release Must Pass

1. **Unit test suite** (all code)
2. **Integration tests** (full workflows)
3. **Acceptance tests** (end-to-end scenarios)
4. **Security audit** (code review + static analysis)
5. **Performance benchmarks** (latency, memory, throughput)
6. **Compatibility tests** (3 OSes, 2 architectures)
7. **Compliance audit** (for Ent/Eco phases)

---

## Documentation & Knowledge Base

### Documentation Artifacts (Per Phase)

**MVP:**
- User Guide (30 pages)
- API Reference (50 pages)
- Architecture Guide (20 pages)
- Deployment Guide (15 pages)
- Troubleshooting (20 pages)

**Total: ~150 pages**

**Final (Eco):**
- Full developer guides (500+ pages)
- API references (all languages) (300+ pages)
- Research papers (in collaboration)
- Video tutorials (100+ hours)
- Community forum (active)

---

## Success Metrics & Revenue

### By Phase (Projected)

| Phase | Users | Revenue | Team Size | Infrastructure Cost |
|-------|-------|---------|-----------|------------------|
| MVP   | 100   | $0      | 10        | $2K/mo           |
| Hard  | 1K    | $100K   | 15        | $10K/mo          |
| Adv   | 10K   | $1M     | 20        | $50K/mo          |
| Ent   | 100K  | $10M    | 30        | $500K/mo         |
| Eco   | 1M    | $100M   | 50+       | $5M/mo           |

---

## Implementation Audit Summary (as of 2026-03-14)

This section captures the **actual state** of the codebase versus planned features, based on comprehensive code review.

### Phase 1 MVP Completion Status

| Category | Component | Files | LOC | Status | Gap |
|----------|-----------|-------|-----|--------|-----|
| **Compiler** | Lexer | parser/lexer.rs | 850 | ✅ Complete | None |
|  | Parser | parser/parser.rs | 1619 | ✅ Complete | None |
|  | Semantic | semantic/* | ~2500 | ✅ ~80% | Generic types incomplete |
|  | Bytecode Codegen | codegen/bytecode.rs | 1200 | ✅ ~70% | Opcodes incomplete |
| **Runtime** | Tree-walk Interp | runtime/interpreter.rs | 135KB | ✅ Partial | NativePtr issues |
|  | Stack-based VM | runtime/vm.rs | 67KB | ✅ Partial | **NO native calls** |
|  | Native Functions | runtime/native.rs | 8KB | 🟡 20/50 | IPC, crypto, JSON |
|  | Error Types | runtime/ | — | ❌ Missing | All errors are String |
| **Framework** | Knowledge Store | helios/knowledge.omni | 25KB | 🟡 Partial | **NOT atomiccrash-safe** |
|  | Service Core | helios/service.omni | 4.2KB | 🟡 Skeleton | State machine incomplete |
|  | Cognitive L0 | helios/cognitive.omni | 10KB | 🟡 Types only | Logic not implemented |
|  | Cognitive L1 | brain/reasoning_engine.omni | 10KB | 🟡 Types only | RETE not integrated |
|  | Experience Log | helios/experience.omni | 13.1KB | 🟡 Partial | Missing query logic |
|  | HTTP API | helios/api.omni | 6KB | 🟡 Skeleton | Not integrated with cognitive |
|  | IPC Wrapper | helios/ipc.omni | — | ❌ Missing | **BLOCKED on 1.1.4** |
| **Tools** | Package Manager | tools/opm | — | ❌ Missing | Not started |
|  | CLI | omnc | ~500 LOC | ✅ Basic | Needs polish |
|  | LSP Server | tools/omni-lsp | — | ❌ Stub | Deferred to v1.1 |
| **GUI** | WinUI3 Project | gui/WinUI3 | — | ❌ Missing | **Complete rewrite needed** |
|  | GUI Design | app/gui.omni | 13KB | ℹ️ Design Doc | NOT runnable code |
| **Tests** | Framework Tests | helios-framework/tests | — | ❌ Directory missing | **ZERO tests** |
|  | Compiler Tests | compiler/tests | ~3KB | ✅ ~70% | Coverage gaps |
|  | Integration Tests | — | — | ❌ Missing | None for framework |

### Critical Implementation Gaps

1. **VM-Native Integration** (🔴 BLOCKER)
   - Issue: `OpCode::CallNative` doesn't exist; native functions unreachable from bytecode
   - Current: Interpreter has NativeManager; VM doesn't
   - Required: Add CallNative opcode, integrate NativeManager into OmniVM, converter bridge
   - Impact: Framework can't run real code; service deployment impossible
   - Fix Effort: 3-4 days
   - Fix Complexity: MEDIUM (clear architecture, standard FFI pattern)

2. **Atomicity & Crash Safety** (🔴 BLOCKER)
   - Issue: Knowledge store uses direct write; crash during write = data loss
   - Current: `knowledge.omni::flush()` -> `fs::write_string(data_path, data)` [NO recovery]
   - Required: Temp file + atomic rename pattern + recovery on startup
   - Impact: Unacceptable for production; loses user data
   - Fix Effort: 2-3 days
   - Fix Complexity: EASY (standard OS pattern)

3. **Framework Compilation Untested** (🟠 UNKNOWN)
   - Issue: `omnc compile helios-framework/main.omni` never run; unknown if it works
   - Current: No verification test; parser/codegen bugs could be lurking
   - Required: Run compilation immediately; fix any parser/codegen issues
   - Impact: Bugs discovered late; cascading delays, rework
   - Fix Effort: 1 day + fix time (unknown)
   - Fix Complexity: UNKNOWN

4. **IPC Natives Not Implemented** (🔴 BLOCKER)
   - Issue: No `pipe_create`, `pipe_accept`, `pipe_read`, `pipe_write` functions
   - Current: native.rs has 20 functions; IPC functions missing
   - Required: 6 Windows named pipe natives + wire into VM via CallNative
   - Impact: No service-client communication; GUI can't connect
   - Fix Effort: 3-4 days
   - Fix Complexity: MEDIUM (Windows API learning curve)

5. **Zero Framework Tests** (🟠 BLOCKER)
   - Issue: `helios-framework/tests/` directory doesn't exist; 0 integration tests
   - Current: Tests only in compiler/ (inline #[cfg(test)])
   - Required: Create test directory + comprehensive integration tests
   - Impact: Bugs reach production; no CI gate; hard to maintain
   - Fix Effort: 2-4 weeks (ongoing)
   - Fix Complexity: MEDIUM (test infrastructure + many scenarios)

6. **GUI Completely Missing** (🟠 NOT BLOCKING FOR MVP)
   - Issue: `gui/` directory doesn't exist; gui.omni is design doc only
   - Current: NO WinUI3 project structure, NO XAML files, NO C# code
   - Required: Complete new C# project from scratch (sln, project files, XAML, code-behind, MVVM)
   - Impact: No user interface; testing via PowerShell only
   - Fix Effort: 4-6 weeks  
   - Fix Complexity: MEDIUM (standard WinUI3 patterns)
   - Status: CAN DEFER GUI to v1.1; MVP can ship with HTTP + PowerShell testing

### Codebase Health Metrics

| Metric | Status | Notes |
|--------|--------|-------|
| **Compiler tests** | ✅ 360+ passing | Good coverage |
| **Runtime panic/unwrap** | 🟡 ~10 in vm.rs GC | Need error type |
| **Error handling** | ❌ All String errors | Should use proper Error enum |
| **Code duplication** | 🟡 RuntimeValue vs VmValue | 15% duplicate logic |
| **Documented code** | 🟡 ~50% | Good module docs, sparse function docs |
| **Dead code** | 🟡 ~15% | Legacy interpreter-only code |
| **Framework structure** | ✅ Well-organized | Clear module hierachy |
| **Framework logic** | 🟡 Skeletal | Types defined, logic sparse |

### Dependency Resolution Required

```
Phase 1.1.1 (Runtime Consolidation)
├─ Merge RuntimeValue + VmValue → new Value enum
├─ Add CallNative opcode to bytecode
├─ Integrate NativeManager into OmniVM
├─ Write value converter (VmValue ↔ RuntimeValue)
└─ 100+ runtime tests
   ↓ (enables 1.1.4, 1.2.1, 1.2.2, 1.3, 1.4)

Phase 1.1.2 (Knowledge Store Atomicity)
├─ Implement atomic write pattern
├─ Add recovery logic
├─ Crash-safety tests
└─ Done independently (no deps)

Phase 1.1.3 (Framework Compilation)
├─ Run omnc compile main.omni
├─ Fix parser/codegen bugs discovered
└─ Done independently (depends on compiler)

Phase 1.1.4 (IPC Natives)
├─ Add 6 Windows pipe natives
├─ Depends on: 1.1.1 (CallNative opcode)
├─ Blocks: 1.2.1, 1.2.3, 1.4
└─ windows-sys dependency needed

Phase 1.2.1 (IPC Wrapper)
├─ Depends on: 1.1.4 (native functions)
├─ Blocks: 1.2.3, 1.4
└─ ~500 LOC of Omni

Phase 1.2.3 (Dual Listen)
├─ Depends on: 1.2.1 (IPC wrapper) + 1.1.1 (runtime)
├─ Blocks: 1.3 (cognitive), 1.4 (GUI)
└─ Integrates HTTP + IPC listeners

Phase 1.3 (Cognitive L0+L1)
├─ Depends on: 1.2.3 (service ready)
├─ Knowledge store, RETE engine, experience log
└─ Core reasoning paths

Phase 1.4 (GUI)
├─ Depends on: 1.2.3 (service ready + IPC working)
├─ OPTIONAL for v1.0 (can ship CLI-only MVP)
└─ Complete new C# project needed
```

### Recommended Action Plan

**Week 1 (Days 1-5):** Fix 4 Critical Blockers
- Day 1: Framework compilation test + fix bugs
- Days 1-3: Add CallNative opcode (1.1.1)
- Days 1-2: Atomic write for knowledge store (1.1.2)
- Days 3-5: IPC natives (1.1.4) + wire into VM

**Week 2-3 (Days 6-14):** Build IPC + Service Layer
- Days 6-10: IPC wrapper Omni code (1.2.1)
- Days 10-14: Dual-listen service (1.2.3) + testing

**Week 4-6 (Days 15-28):** Cognitive Layers
- Days 15-21: L0 reflex + L1 RETE integration (1.3)
- Days 22-28: Testing + bug fixes

**Week 7-10 (Days 29-50):** GUI + Polish
- Days 29-40: WinUI3 GUI project (1.4) — OPTIONAL, can skip for CLI-only MVP
- Days 41-50: Testing, packaging, release prep

**Week 11-14 (Days 51-70):** Documentation + Release
- Comprehensive test suite (1.6.1)
- Documentation (1.6.2)
- Release package + installer (1.5.3)

---

## Risk Mitigation — Implementation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Parser has bugs preventing compilation | HIGH | CRITICAL | **Run Day 1**, fix immediately |
| Atomicity bug causes data loss in prod | MEDIUM | CRITICAL | Implement temp+rename, add crash tests |
| VM native integration takes 2+ weeks | LOW | HIGH | Clear architecture, similar patterns elsewhere |
| IPC integration 2+ weeks | LOW | HIGH | Windows API is mature, standard patterns |
| Framework tests uncover major issues | MEDIUM | MEDIUM | **Write tests as you go**, CI gate |
| GUI project overruns timeline | MEDIUM | LOW | GUI can be deferred to v1.1 |

---

## Risk Mitigation

**Key Risks:**

1. **LLM Dependency** — If LLM APIs change/cost explodes
   - Mitigation: Support local LLama models as fallback
2. **Security Issues** — If major vulnerability discovered
   - Mitigation: Rapid patch cycle, security team on retainer
3. **Regulatory** — New AI regulations complicate compliance
   - Mitigation: Legal counsel tracking regulations, pre-emptive compliance
4. **Competition** — Other cognitive frameworks emerge
   - Mitigation: Focus on transparency + explainability (unique value prop)

---

## Conclusion

This 10-year roadmap provides a **comprehensive vision** for HELIOS from MVP to production ecosystem. Each phase is structured with:

- **Clear deliverables** (what ships)
- **Detailed specifications** (how to build)
- **Success criteria** (how to verify)
- **Timeline estimates** (capacity planning)
- **Resource requirements** (people, infrastructure)

The roadmap is **flexible** — phases can be reordered, extended, or skipped based on market feedback and resource availability.

**Start with Phase 1 (MVP) — 18 months to get to market.** Then adapt based on customer demand.

---

*Document Version: 1.0 — 2026-03-14*  
*Last Updated: [Auto-update on phase completion]*  
*Next Review: After Phase 1 completion (Month 18)*
