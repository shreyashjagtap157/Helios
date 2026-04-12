---
name: "Issue: Good First Issue (Docs)"
about: Starter issue template for first-time contributors improving docs/examples
title: "[GOOD FIRST ISSUE][DOCS] "
labels: ["good first issue", "documentation", "needs-triage"]
assignees: []

body:
  - type: markdown
    attributes:
      value: |
        This template is for beginner-friendly documentation contributions.
        Keep scope focused and measurable.

  - type: input
    id: doc_target
    attributes:
      label: Documentation Target
      description: Which file or section should be improved?
      placeholder: "OMNI_README.md - Add language examples for match expressions"
    validations:
      required: true

  - type: textarea
    id: problem
    attributes:
      label: What is missing or unclear?
      placeholder: "Current quick-start omits parser-only command usage and expected output."
    validations:
      required: true

  - type: textarea
    id: deliverable
    attributes:
      label: Expected Deliverable
      description: Define exactly what should be added/updated.
      placeholder: |
        - Add one runnable example
        - Add expected output block
        - Cross-link related section
    validations:
      required: true

  - type: textarea
    id: acceptance
    attributes:
      label: Acceptance Criteria
      placeholder: |
        - [ ] Text is technically accurate
        - [ ] Examples compile/run if applicable
        - [ ] Links are valid
        - [ ] Style matches existing docs
    validations:
      required: true

  - type: dropdown
    id: effort
    attributes:
      label: Estimated Effort
      options:
        - < 1 hour
        - 1-3 hours
        - 3-6 hours
    validations:
      required: true

  - type: input
    id: reviewer_hint
    attributes:
      label: Suggested Reviewer Focus
      placeholder: "Check command accuracy and consistency with ROADMAP.md"
