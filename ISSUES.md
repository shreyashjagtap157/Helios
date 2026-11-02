# Helios — Initial Research and Design Backlog

This file tracks design questions, not software defects. No implementation has started.

## P0 — Resolve before the first prototype

- Define the first concrete personal use case.
- Select the first supported reasoning domain.
- Define strict non-training conformance checks for every dependency.
- Select one existing host operating system.
- Define the first hardware target and device classes.
- Specify the knowledge record, provenance and contradiction model.
- Specify the first reasoning engine and acceptance suite.
- Define the action broker and authorization protocol.
- Define what the system must do when intent is ambiguous or device state is unknown.
- Design a minimal documentation-validation workflow and reconcile its real status names with the protected main branch requirements; the current required contexts are remnants of an earlier toolchain and are not produced by any workflow in the current repository tree.

## P1 — Research and architecture

- Compare Datalog, ASP and rule engines.
- Compare SAT/SMT and constraint-solving options.
- Evaluate classical planning and hierarchical task planning.
- Evaluate explicit temporal and causal models.
- Design working, episodic, semantic and procedural memory.
- Research natural-language parsing without neural models.
- Evaluate non-trained image/audio processing only if required by initial use cases.
- Compare local databases and event-sourcing approaches.
- Evaluate plugin isolation and capability security.

## P2 — Hardware and operating-system ecosystem

- Inventory initial PC hardware interfaces.
- Compare host-OS device access APIs.
- Research USB, HID, serial, Bluetooth and network-device adapters.
- Evaluate microcontroller and embedded nodes.
- Design hardware-in-the-loop testing.
- Compare Linux distribution integration, microkernel options and custom-kernel costs.
- Define secure boot, update, recovery and rollback requirements.

## Completion policy

A backlog item is closed only when its question is answered with evidence or explicitly marked as deferred/open. Code existence alone does not close a design question.
