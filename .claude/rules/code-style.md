# Code Style Rules

## General

- Prefer clear, descriptive names.
- Keep functions focused and side-effect boundaries explicit.
- Favor deterministic behavior in core logic.
- Avoid unnecessary abstraction layers.

## Patch Discipline

- Make minimal, surgical changes.
- Preserve existing style and conventions in each subsystem.
- Do not reformat unrelated code.

## Documentation

- Update docs when behavior, interfaces, or workflows change.
- For vault notes, include explicit workspace links for navigation.

## Safety

- Do not include secrets or tokens in code or docs.
- Avoid hidden behavior and implicit fallbacks in critical paths.
