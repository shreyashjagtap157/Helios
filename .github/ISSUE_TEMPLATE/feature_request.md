name: "Issue: Feature Request"
about: Suggest a new feature or improvement for Omni or Helios
title: "[FEATURE] "
labels: ["enhancement", "needs-triage"]
assignees: []

body:
  - type: markdown
    attributes:
      value: |
        Have an idea for Omni or Helios? We'd love to hear it.
        Fill out the sections below to help us understand your proposal.

  - type: textarea
    id: problem
    attributes:
      label: Problem Statement
      description: What problem does this feature solve? What limitation do you hit today?
      placeholder: "I'm always frustrated when..."
    validations:
      required: true

  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
      description: Describe how you'd like this to work. Include syntax examples if relevant.
      placeholder: |
        ```omni
        // Example of desired syntax or API
        ```
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
      description: What other approaches have you considered?

  - type: dropdown
    id: component
    attributes:
      label: Component
      description: Which part of the project is this for?
      options:
        - Language feature
        - Compiler
        - Standard Library
        - Self-Hosted Compiler
        - Helios Framework
        - Tooling (LSP, DAP, formatter, etc.)
        - Package Manager
        - Documentation
    validations:
      required: true

  - type: dropdown
    id: priority
    attributes:
      label: How important is this to you?
      options:
        - "Nice to have"
        - "Would significantly improve my workflow"
        - "Blocking my use of Omni"
    validations:
      required: true
