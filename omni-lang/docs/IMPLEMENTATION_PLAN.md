# Omni Language Implementation Plan

**Version:** 1.0  
**Created:** 2026-03-26  
**Target:** Complete Self-Hosting & Standalone Deployment

---

## 📋 File Extensions

| Extension | File Type | Status |
|-----------|-----------|--------|
| `.omni` | Omni source code | ✅ Established |
| `.omh` | Omni header files | ✅ Proposed |
| `.om` | Omni object/bytecode | ✅ Proposed |
| `.ovm` | OVM bytecode files | ✅ Established |
| `.ove` | Bundled executable | ✅ Proposed |
| `.oml` | Omni library/package | ✅ Proposed |
| `.omt` | Token dump | ✅ Proposed |
| `.oir` | IR dump | ✅ Proposed |
| `.omnirc` | Config file | ✅ Proposed |

---

## Phase 1: OVM VM with Full GC

| Task | Status | Files | Notes |
|------|--------|-------|-------|
| 1.1 Create OVM VM project structure | 🔴 PENDING | `ovm/vm.c`, `ovm/gc.c`, `ovm/main.c` | |
| 1.2 Memory allocator (bump pointer) | 🔴 PENDING | `gc.c:alloc()` | |
| 1.3 GC header structures | 🔴 PENDING | `gc.h:OvmObjectHeader` | |
| 1.4 Mark phase | 🔴 PENDING | `gc.c:mark_from_roots()` | |
| 1.5 Sweep phase | 🔴 PENDING | `gc.c:sweep()` | |
| 1.6 Compact phase (LISP2) | 🔴 PENDING | `gc.c:compact()` | |
| 1.7 Generational promotion | 🔴 PENDING | `gc.c:promote()` | |
| 1.8 Write barrier | 🔴 PENDING | `vm.c:write_barrier()` | |
| 1.9 Finalizer execution | 🔴 PENDING | `gc.c:run_finalizers()` | |
| 1.10 Bounds checking | 🔴 PENDING | `vm.c:check_bounds()` | |
| 1.11 W^X for JIT | 🔴 PENDING | `jit.c:mmap_rwx()` | |
| 1.12 Stack overflow protection | 🔴 PENDING | `vm.c:check_stack()` | |
| 1.13 Build system | 🔴 PENDING | `ovm/Makefile` | |
| 1.14 Test with sample .ovm | 🔴 PENDING | `tests/` | |

---

## Phase 2: PE Bundler

| Task | Status | Files | Notes |
|------|--------|-------|-------|
| 2.1 PE header writer | 🔴 PENDING | `linker/pe_header.c` | |
| 2.2 Section creation | 🔴 PENDING | `linker/pe_sections.c` | |
| 2.3 VM code embedding | 🔴 PENDING | `linker/pe_embed.c` | |
| 2.4 Entry point resolution | 🔴 PENDING | `linker/pe_entry.c` | |
| 2.5 OVM→PE conversion tool | 🔴 PENDING | `tools/ovm2pe.omni` | |
| 2.6 Test PE bundling | 🔴 PENDING | `tests/pe_test.omni` | |

---

## Phase 3: Self-Hosted Compiler Completion

| Task | Status | Files | Notes |
|------|--------|-------|-------|
| 3.1 Audit codegen/ovm.omni | 🔴 PENDING | Verify all 80+ opcodes | |
| 3.2 Audit linker/mod.omni | 🔴 PENDING | Check PE/ELF support | |
| 3.3 Add --emit exe to main.omni | 🔴 PENDING | `main.omni:emit_target` | |
| 3.4 Implement OVM emission | 🔴 PENDING | `codegen/emit_ovm.omni` | |
| 3.5 Add PE bundler integration | 🔴 PENDING | `linker/pe_bundler.omni` | |
| 3.6 Test self-host compilation | 🔴 PENDING | Compile mini_compiler.omni | |

---

## Phase 4: Bootstrap Pipeline

| Task | Status | Files | Notes |
|------|--------|-------|-------|
| 4.1 Build Stage 0 (Rust omnc) | 🔴 PENDING | `cargo build --release` | |
| 4.2 Compile self-hosted (Stage 1) | 🔴 PENDING | Run Rust omnc on omni/* | |
| 4.3 Compile self-hosted (Stage 2) | 🔴 PENDING | Run Stage 1 on omni/* | |
| 4.4 Compile self-hosted (Stage 3) | 🔴 PENDING | Run Stage 2 on omni/* | |
| 4.5 Verify SHA-256 match | 🔴 PENDING | `bootstrap.omni:verify()` | |

---

## Phase 5: Code Quality & Performance

| Task | Status | Files | Notes |
|------|--------|-------|-------|
| 5.1 Modularize interpreter.rs | 🔴 PENDING | Split into 10+ modules | |
| 5.2 Modularize optimizer.rs | 🔴 PENDING | Split into 5+ modules | |
| 5.3 Add parallel compilation | 🔴 PENDING | Use rayon | |
| 5.4 Implement incremental compilation | 🔴 PENDING | Cache AST per module | |
| 5.5 Add fuzzing for parser | 🔴 PENDING | `fuzz/parser_fuzz.rs` | |
| 5.6 Add fuzzing for OVM | 🔴 PENDING | `fuzz/ovm_fuzz.c` | |
| 5.7 Remove duplicate codegen | 🔴 PENDING | Consolidate backends | |

---

## Phase 6: Security Hardening

| Task | Status | Files | Notes |
|------|--------|-------|-------|
| 6.1 W^X memory policy in JIT | 🔴 PENDING | `jit.c:mmap_wx()` | |
| 6.2 Strict bounds checking OVM | 🔴 PENDING | `vm.c:bounds_check()` | |
| 6.3 Bytecode verification | 🔴 PENDING | `vm.c:verify_bytecode()` | |
| 6.4 Stack canary | 🔴 PENDING | `vm.c:stack_check()` | |
| 6.5 Replace custom crypto | 🔴 PENDING | Use libsodium via FFI | |

---

## Issues to Fix (From Analysis)

| Issue | Status | Priority | Notes |
|-------|--------|----------|-------|
| iter.omni (stub) | 🔴 PENDING | High | Implement iterator protocol |
| Clippy warnings | 🔴 PENDING | Low | Fix 3 warnings |
| Missing docs | 🔴 PENDING | Medium | Add language_guide.md |
| omni-fmt expansion | 🔴 PENDING | Medium | Full formatting |
| omni-dap expansion | 🔴 PENDING | Medium | Breakpoint support |

---

## Verification Checklist

| Milestone | Status | Verification |
|-----------|--------|--------------|
| OVM VM | ⬜ PENDING | Run test suite |
| Bounds Checking | ⬜ PENDING | Fuzzing tests |
| W^X JIT | ⬜ PENDING | Security audit |
| PE Bundler | ⬜ PENDING | Run on clean Windows |
| Self-Host Compile | ⬜ PENDING | Compile test program |
| Stage 1 | ⬜ PENDING | Verify output |
| Stage 2 | ⬜ PENDING | Compare outputs |
| Stage 3 | ⬜ PENDING | SHA-256 match |
| No External Deps | ⬜ PENDING | Audit imports |

---

## Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Compilation Speed | 10K LOC/s | TBD | ⬜ |
| GC Pause Time | <10ms | N/A | ⬜ |
| Binary Size (hello) | <100KB | TBD | ⬜ |
| Bootstrap Time | <30s | N/A | ⬜ |
| Memory Usage | <50MB | N/A | ⬜ |

---

## Implementation Log

| Date | Phase | Task | Notes |
|------|-------|------|-------|
| 2026-03-26 | - | Plan created | Initial implementation plan |
