# HELIOS PROJECT MEMORY & RULES

## Current State
- We are building the Helios OS/Compiler from scratch.
- The Single Source of Truth is the `README.md` provided by the user.

## Agent Behavior Guidelines
1.  **Autonomy:** You are authorized to create, edit, and delete files without explicit permission for every step.
2.  **Persistence:** If the output is too long, chunk it, but *never* stop the logic flow.
3.  **Error Handling:**
    - If `cargo check` fails: Analyze Error -> Fix Code -> Retry.
    - If a dependency is missing: Add it to `Cargo.toml`.
4.  **File Structure:**
    - `/compiler` (Rust crate)
    - `/std` (Omni source files)
    - `/brain` (Omni source files)
    - `/tools` (Rust crates)

## Architecture Notes
- The **Compiler** uses `IrInstruction` as the central IR.
- **OVM** is a stack-based virtual machine.
- **Cognitive Engine** is *not* a standard ML model; it is a dynamic graph verifier.