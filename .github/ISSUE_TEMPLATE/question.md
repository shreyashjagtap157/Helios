name: "Issue: Question"
about: Ask a question about Omni, Helios, or the project
title: "[QUESTION] "
labels: ["question"]
assignees: []

body:
  - type: textarea
    id: question
    attributes:
      label: Your Question
      description: What would you like to know?
    validations:
      required: true

  - type: textarea
    id: context
    attributes:
      label: Context
      description: Any relevant context, code snippets, or what you've already tried.

  - type: dropdown
    id: component
    attributes:
      label: Related Component
      description: Which part of the project is this about?
      options:
        - Language Syntax
        - Compiler Usage
        - Standard Library
        - Helios Framework
        - Tooling
        - Building from Source
        - General
    validations:
      required: true
