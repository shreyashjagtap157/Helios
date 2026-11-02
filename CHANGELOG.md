# Changelog

## 2026-11-02 — Documentation CI gate

- Added a dependency-free Ruby repository validator for required files, stale project references, issue-form YAML, workflow safety, local Markdown links and non-training requirement IDs.
- Added a GitHub Actions workflow with read-only token permissions, SHA-pinned checkout and a stable `docs` job check.
- Added local validation instructions; the workflow's first hosted run and branch-protection update remain pending verification.


## 2026-11-02 — Non-training assurance refinement

- Explicitly prohibited local and remote model inference, including optional or indirect model-service paths.
- Added dependency-admission criteria covering transitive dependencies, optional features, SDKs, plugins, OS services, network calls and build artifacts.
- Added requirements for SBOM generation, model-asset checks and auditable dependency review when implementation begins.

## 2026-11-02 — Repository governance cleanup

- Removed the remaining obsolete project references from GitHub issue templates, CODEOWNERS, pull-request guidance, NOTICE and contributor documentation.
- Converted the issue templates to GitHub's YAML issue-form format with .yml filenames and required top-level name, description and body keys.
- Replaced toolchain-specific contribution checkboxes with evidence-based Helios review criteria.
- Recorded the mismatch between protected-branch required checks and the absence of repository-owned CI workflows.

## 2026-11-02 — Helios design baseline reset

- Re-established Helios as a personal cognitive computing framework.
- Corrected project phase to concept/specification only; no implementation has started.
- Adopted strict exclusion of all model training, fine-tuning, pretrained-model inference and learned embedding dependencies.
- Added hardware interaction as a first-class target capability with independent action authorization and safety enforcement.
- Added the long-term operating-system ecosystem vision.
- Replaced the mixed project documentation with a canonical Helios master specification and supporting project-control records.
- Removed obsolete implementation artifacts and unreviewed mixed archives from the current project tree while preserving Git history.
