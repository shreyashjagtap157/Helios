---
name: "Issue: Good First Issue (Compiler)"
about: Starter issue template for first-time contributors working on lexer/parser/compiler quality tasks
title: "[GOOD FIRST ISSUE][COMPILER] "
labels: ["good first issue", "component: compiler", "needs-triage"]
assignees: []

body:
  - type: markdown
    attributes:
      value: |
        Thanks for contributing to Omni.
        Use this template to create a small, beginner-friendly compiler task.

  - type: input
    id: scope
    attributes:
      label: Small Task Scope
      description: Describe one narrowly scoped change.
      placeholder: "Add lexer test for escaped tab and carriage return strings"
    validations:
      required: true

  - type: textarea
    id: context
    attributes:
      label: Background and Context
      description: What file(s) and behavior are involved?
      placeholder: |
        Affected files:
        - omni-lang/compiler/src/lexer.rs
        Current behavior:
        Expected behavior:
    validations:
      required: true

  - type: textarea
    id: acceptance
    attributes:
      label: Acceptance Criteria
      description: Checklist used to mark the issue complete.
      placeholder: |
        - [ ] Add or update unit tests
        - [ ] Existing tests still pass
        - [ ] No formatting/lint regressions
    validations:
      required: true

  - type: input
    id: starter_hint
    attributes:
      label: Suggested Starter Files
      description: Entry points for a first-time contributor.
      placeholder: "omni-lang/compiler/src/lexer.rs, omni-lang/compiler/src/parser/mod.rs"

  - type: dropdown
    id: risk
    attributes:
      label: Risk Level
      options:
        - Low (isolated tests/docs)
        - Medium (small code path changes)
    validations:
      required: true

  - type: textarea
    id: validation
    attributes:
      label: Validation Command(s)
      description: Commands a contributor can run before opening a PR.
      placeholder: |
        cargo test --lib lexer::tests
        cargo test --lib parser::tests
