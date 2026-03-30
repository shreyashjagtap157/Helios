name: "Issue: Bug Report"

about: Report a bug in the Omni compiler or Helios framework
title: "[BUG] "
labels: ["bug", "needs-triage"]
assignees: []

body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to report a bug!
        Please fill out the sections below so we can reproduce and fix it.

  - type: textarea
    id: description
    attributes:
      label: Bug Description
      description: A clear and concise description of the bug.
      placeholder: Tell us what happened.
    validations:
      required: true

  - type: textarea
    id: steps
    attributes:
      label: Steps to Reproduce
      description: How can we reproduce this behavior?
      placeholder: |
        1. Create a file `test.omni` with the following content:
        ```omni
        module test
        fn main():
            println("hello")
        ```
        2. Run `omnc --run test.omni`
        3. See error
    validations:
      required: true

  - type: textarea
    id: expected
    attributes:
      label: Expected Behavior
      description: What did you expect to happen?
    validations:
      required: true

  - type: textarea
    id: actual
    attributes:
      label: Actual Behavior
      description: What actually happened? Include any error messages or output.
    validations:
      required: true

  - type: dropdown
    id: component
    attributes:
      label: Component
      description: Which part of the project is affected?
      options:
        - Compiler (omnc)
        - Standard Library
        - Self-Hosted Compiler
        - Helios Framework
        - omni-fmt
        - omni-lsp
        - omni-dap
        - opm (Package Manager)
        - VS Code Extension
        - Documentation
    validations:
      required: true

  - type: input
    id: os
    attributes:
      label: Operating System
      description: What OS are you running?
      placeholder: "e.g., Windows 11, macOS 14, Ubuntu 24.04"
    validations:
      required: true

  - type: input
    id: rust-version
    attributes:
      label: Rust Version
      description: Output of `rustc --version`
      placeholder: "e.g., rustc 1.75.0"
    validations:
      required: true

  - type: textarea
    id: additional
    attributes:
      label: Additional Context
      description: Any other context, screenshots, or log output.
