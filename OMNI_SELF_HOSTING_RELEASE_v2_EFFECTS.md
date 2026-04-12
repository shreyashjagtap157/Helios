Omni v2 Self-Hosting Release — Effects Feature

Summary
-------
This release completes parser, semantic, and IR support for function "effects" (v2 effect-clauses) and includes minimal stdlib stubs to enable Stage0 bootstrap.

Key Deliverables
----------------
- Parser: structured parsing and validation of effect clauses (rejects malformed forms like trailing `+`).
- Semantic: `validate_function_effects` enforces `async`/`throw<T>` constraints and performs capability checks for effect propagation.
- AST: `Function` includes `effects: Vec<String>` and call sites were updated.
- Typed AST: `TypedFunction` includes `effects: Vec<String>`.
- IR: `IrFunction` includes `effects: Vec<String>` and IR generation preserves effects.
- Tests: Parser and semantic unit tests for effect parsing and validation; an integration test verifies effects propagate into generated IR.
- Stdlib: Minimal `core.omni` and `collections.omni` stubs added to allow Stage0 bootstrap; originals preserved as `*.orig.omni` backups.

Build & Verify
--------------
Run the compiler crate tests locally:

```powershell
cd omni-lang/compiler
cargo test -- --nocapture
```

All compiler-crate tests passed during validation.

Next Steps
----------
1. Wire `effects` into lowering and codegen semantics for runtime enforcement where applicable.
2. Add semantic/e2e tests exercising effect propagation in larger integration scenarios.
3. Create release artifact and packaging (OVM bundle + stage0 bootstrap instructions).

Notes
-----
- All edits preserve original sources; backups are available in the stdlib directory with `.backup.*.orig.omni` suffixes.
- If you want, I can proceed to wire effects into lowering/IR semantics and prepare the release artifact next.
