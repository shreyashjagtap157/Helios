# HELIOS Framework

HELIOS is the broader cognitive and knowledge framework for the Omni project. It is not a narrow deterministic-only rules engine, and it is not a machine-learning or neural-model system that trains weights from datasets and accepts accuracy loss as normal. HELIOS is intended to be a broader assistant framework built around exact knowledge storage, evidence intake, verification, confidence accounting, web-learning pipelines, plugin and capability growth, governed self-modification proposals, service and app deployment, and persistent recall.

## Product Position

- HELIOS is evidence-driven, not model-weight-driven.
- “Training” in this repository means knowledge ingestion, verification, consolidation, checkpointing, pruning, and recall maintenance.
- External and web-acquired information is staged as evidence and must not be promoted to trusted knowledge without explicit policy satisfaction.
- Every substantive answer is expected to carry provenance, confidence posture, contradiction awareness, and freshness awareness.

## Canonical Scope

The canonical framework tree is `helios-framework/`. It already contains the broader source areas that were previously split between root-level folders and narrower framework scaffolding:

- `helios/`
  Core assistant surfaces such as the canonical runtime, capability management, knowledge storage, experience logging, input/output, service, API, and self-modeling.
- `brain/`
  Reasoning, knowledge graph management, learning and ingestion pipelines, web-learning, checkpointing, storage, and cognitive orchestration modules.
- `training/`
  Corpus ingestion, pruning, optimization, checkpointing, and compression-style maintenance utilities for knowledge and runtime state.
- `app/`
  User-facing application, GUI hooks, extension/plugin loading, and operating-system integration.
- `safety/`
  Governance and action-control logic.
- `biometrics/`
  Identity-verification surfaces for sensitive approvals.
- `config/`
  Configuration loaders and deployment defaults.

## Current Canonical Direction

The framework is being normalized toward these contracts:

- Exact knowledge store:
  Facts are stored exactly as provided, with source, accuracy status, confidence, history, and conflict tracking.
- Exact experience log:
  Interactions and system events are recorded as exact history, not as training samples for probabilistic model fitting.
- Typed capability registry:
  HELIOS capabilities are explicit, auditable, approval-aware operations rather than opaque prompts.
- Self model:
  HELIOS carries an internal description of its current scope, confidence policy, governance requirements, and capability surface.
- Multi-mode operation:
  REPL, service, API, and app surfaces are all part of the intended architecture, even though some are more mature than others.

## Implemented Canonical Bootstrap

The canonical `main.omni` has been reworked into a usable bootstrap around the exact-knowledge core:

- `helios/runtime.omni`
  Shared canonical `Helios` runtime used by both the CLI bootstrap and the API layer.

- `--status`
  Prints the self-model posture, knowledge counts, and capability counts.
- `--capabilities`
  Lists the currently registered typed capabilities.
- `--repl`
  Starts an interactive shell for exact knowledge entry and capability execution.
- `--api` / `--service`
  Starts the canonical runtime behind the API service wrapper on the default local port.

The current REPL supports:

- `remember <subject>: <content>`
- `ask <topic>`
- `read <path>`
- `ls [path]`
- `grep <path> <query>`
- `find-files <directory> <pattern>`
- `calc <expression>`

This is not the end-state product. It is the canonical bootstrap surface that aligns the framework with the master self-hosting and HELIOS delivery plan instead of the older narrow deterministic-only entrypoint.

## Non-ML Knowledge Contract

HELIOS must continue to follow these constraints:

- Do not replace exact facts with weight-trained approximations.
- Do not describe ingestion pipelines as neural training unless the architecture actually changes and the project explicitly re-specifies that direction.
- Do not claim certainty without evidence, verification state, and confidence policy support.
- Do not auto-promote web or external knowledge to trusted truth merely because it was fetched successfully.

## Next Integration Targets

The remaining large work is still real and not complete:

- Merge the richer root-level HELIOS behavior into the canonical framework without preserving duplicate live trees.
- Align service, API, GUI, plugin, and web-learning flows on the same typed knowledge and capability contracts.
- Replace placeholder or divergent modules that still describe HELIOS as only deterministic or only service-oriented.
- Continue normalizing the Omni self-hosting toolchain so HELIOS eventually builds, packages, and deploys through Omni-native flows only.
