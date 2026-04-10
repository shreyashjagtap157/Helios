# Omni Project Completion TODO

## Phase 1: Core Infrastructure (Priority 1)

### 1.1 Diagnostics with Error Codes
- [x] Add stable E#### error codes to diagnostics system
- [x] Add primary/secondary spans to all errors
- [x] Add help notes to error messages
- [x] Add machine-applicable fix suggestions

### 1.2 Package Manifest
- [x] Implement omni.toml manifest parsing
- [x] Add package metadata (name, version, deps)
- [x] Add build configuration parsing

### 1.3 Standard Library Basics
- [ ] Complete Option<T> type implementation
- [ ] Complete Result<T,E> type implementation
- [ ] Add basic collection types (Vec, HashMap stubs)
- [ ] Add core traits (Copy, Clone, Drop, etc.)

## Phase 2: Type & Safety (Priority 2)

### 2.1 Type System Improvements
- [x] Implement bidirectional type inference (already present)
- [ ] Add trait bounds to generic functions
- [ ] Add associated types support
- [ ] Implement monomorphization fully

### 2.2 Borrow Checker Upgrades
- [x] Add field projection support (already present)
- [ ] Prepare for Polonius migration
- [ ] Add lifetime inference improvements

### 2.3 Effect System Skeleton (v2.0 prep)
- [ ] Define effect type representations
- [ ] Add effect inference framework
- [ ] Add effect annotation parsing

## Phase 3: Tooling (Priority 3)

### 3.1 CLI Enhancements
- [ ] Add `omni check` command
- [ ] Add `omni fmt` command (basic)
- [ ] Add `omni test` command
- [ ] Add `omni run` command

### 3.2 Error Improvements
- [x] Add "Did you mean?" suggestions
- [x] Add Levenshtein distance for typos
- [x] Add JSON output mode

## Phase 4: Self-Hosting (Priority 4)

### 4.1 Mini Compiler Expansion
- [ ] Add more expression types
- [ ] Add struct support
- [ ] Add enum support
- [ ] Add pattern matching

### 4.2 Bootstrap Pipeline
- [ ] Document Stage 0 → Stage 1 → Stage 2
- [ ] Add Stage comparison tests
- [ ] Improve self-hosting coverage