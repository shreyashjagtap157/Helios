# DEPLOYMENT CHECKLIST

Date: 2026-04-02
Scope: Omni self-hosting Phase 5 readiness

## Release Readiness Checklist

- [x] Stage1 and Stage2 artifacts exist and are bit-identical
- [x] Stage1 bytecode starts under OVM runtime
- [ ] Stage1 bytecode successfully emits Stage3 artifact
- [ ] Triple fixpoint verified: Stage1 == Stage2 == Stage3
- [x] Documentation updated for current gate state
- [ ] No runtime dependency on Rust beyond documented Stage0 seed path
- [ ] Reproducible builds proven for Stage3 path
- [ ] Full test suite rerun in this closure pass

## Blocking Items

1. OVM runtime fidelity error during Stage1 self-compilation (`null + int` type error).
2. Stage3 not reproducibly generated from successful Stage1 bytecode execution.

## Verification Commands

```powershell
# Hash verification for Stage1/Stage2
(Get-FileHash D:\Project\Helios\build\omnc_stage1.ovm -Algorithm SHA256).Hash
(Get-FileHash D:\Project\Helios\build\omnc_stage2.ovm -Algorithm SHA256).Hash

# Stage1 execution under OVM
D:\Project\Helios\omni-lang\ovm\target\release\ovm-runner.exe \
  D:\Project\Helios\build\omnc_stage1.ovm \
  D:\Project\Helios\omni-lang\omni\compiler_minimal.omni
```

## Go/No-Go

Current decision: NO-GO for self-hosting-complete release tag.

Condition to flip to GO:
- Complete Stage3 gate and produce reproducible triple fixpoint evidence.
