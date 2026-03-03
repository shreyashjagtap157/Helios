# HELIOS Cognitive Framework

<!-- Badges -->
![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Language](https://img.shields.io/badge/language-Omni-orange)
![Status](https://img.shields.io/badge/status-active-brightgreen)
![Modules](https://img.shields.io/badge/modules-10-informational)

---

## Overview

**HELIOS** (Heuristic Engine for Logical Inference and Operational Synthesis) is a deterministic, logic-driven cognitive assistant framework built entirely in the [Omni programming language](../omni-lang/README.md). Unlike neural-network-based AI systems, HELIOS uses forward/backward chaining inference, pattern matching, and structured knowledge graphs to reason about problems transparently and predictably.

### Design Philosophy

- **Deterministic reasoning** — no neural networks; every conclusion has an explicit logical chain
- **Safety-first** — Three Laws of Robotics implemented as immutable, priority-ordered constraints
- **Multi-level memory** — four-layer architecture mimicking cognitive science models
- **Adaptive learning** — learns from interactions through pattern extraction, not gradient descent
- **Transparent decisions** — every response includes a traceable thought process

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      HELIOS Service                          │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                  Cognitive Loop                         │  │
│  │        perceive → think → respond → reflect             │  │
│  └───────────┬──────────┬──────────────┬──────────────────┘  │
│              │          │              │                      │
│  ┌───────────▼──┐ ┌─────▼──────┐ ┌────▼───────────┐         │
│  │  Reasoning   │ │  Memory    │ │   Adaptive     │         │
│  │  Engine      │ │  System    │ │   Learning     │         │
│  │ (forward/    │ │ (4-layer)  │ │  (patterns,    │         │
│  │  backward    │ │            │ │   concepts)    │         │
│  │  chaining)   │ │            │ │                │         │
│  └──────────────┘ └────────────┘ └────────────────┘         │
│              │                                               │
│  ┌───────────▼──────────┐  ┌──────────────────────┐         │
│  │  Knowledge Graph      │  │  Safety Framework    │         │
│  │  (property graph,     │  │  (Three Laws,        │         │
│  │   BFS/DFS, PageRank)  │  │   PII, bias check)  │         │
│  └───────────────────────┘  └──────────────────────┘         │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                Capability Registry                      │  │
│  │  question_answering │ summarization │ analysis │ ...    │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Core Components

| Component | Module | Lines | Description |
|-----------|--------|-------|-------------|
| **Cognitive Core** | `helios/cognitive.omni` | 284 | Main cognitive loop — perceive, think, respond, reflect |
| **Service Layer** | `helios/service.omni` | 162 | Service lifecycle, request handling, health checks |
| **Capability Registry** | `helios/capability.omni` | 200 | Extensible capability system with permissions |
| **Reasoning Engine** | `brain/reasoning_engine.omni` | 279 | Forward/backward chaining, hypothesis evaluation |
| **Knowledge Graph** | `brain/knowledge_graph.omni` | 303 | Property graph with CRUD, BFS, DFS, PageRank |
| **Memory Architecture** | `brain/memory_architecture.omni` | 251 | 4-layer memory: working, short-term, long-term, episodic |
| **Adaptive Learning** | `brain/adaptive_learning.omni` | 228 | Pattern recognition, concept extraction, feedback loop |
| **Safety Framework** | `safety/asimov.omni` | 255 | Three Laws, PII detection, bias checking, content filtering |
| **Configuration** | `config/default.omni` | 73 | TOML-based configuration with defaults |
| **Entry Point** | `main.omni` | 47 | Initializes and starts the HELIOS service |

---

## Features

### Multi-Level Memory System

HELIOS employs a four-layer memory architecture inspired by cognitive science:

| Layer | Behavior |
|-------|----------|
| **Working Memory** | Active context, capacity-limited (evicts oldest items) |
| **Short-Term Memory** | Time-decaying storage, automatic cleanup of expired items |
| **Long-Term Memory** | Indexed persistent storage, keyword-based recall |
| **Episodic Memory** | Temporal record of past interactions, ordered by timestamp |

### Reasoning Engine

- **Forward chaining** — derives new facts from known facts and inference rules
- **Backward chaining** — works backward from a goal to find supporting evidence
- **Hypothesis evaluation** — scores hypotheses by counting supporting vs. contradicting facts
- **Confidence propagation** — each conclusion carries a confidence score through the inference chain

### Knowledge Graph

- **Property graph model** — nodes with typed labels and key-value properties, directed weighted edges
- **Graph algorithms** — BFS traversal, DFS traversal, shortest path, PageRank
- **Persistence** — serialization to/from TOML format
- **Querying** — find neighbors, filter by node type, path search

### Safety Framework (Asimov Protocol)

Three immutable, priority-ordered laws govern all actions:

1. **Law 1 (Harm Prevention):** A system shall not cause harm to a human or allow harm through inaction.
2. **Law 2 (Obedience):** A system shall obey commands unless they conflict with Law 1.
3. **Law 3 (Self-Preservation):** A system shall preserve itself unless it conflicts with Laws 1 or 2.

Additional safety checks: PII detection, bias term filtering, content classification, deception detection, and full audit logging.

### Capability System

Five built-in capabilities with a permission model:

- `question_answering` — Answer factual questions using knowledge and reasoning
- `summarization` — Condense information into summaries
- `analysis` — Analyze data and provide structured insights
- `knowledge_management` — CRUD operations on the knowledge graph
- `self_reflection` — Introspect on performance and learning metrics

---

## Getting Started

### Prerequisites

- Omni compiler (`omnc`) — see [omni-lang/README.md](../omni-lang/README.md) for build instructions

### Configuration

Create or edit `config/default.toml`:

```toml
[helios]
name = "HELIOS"
version = "0.1.0"
api_port = 8080
log_level = "info"

[memory]
working_capacity = 10
short_term_ttl_ms = 300000
consolidation_interval = 10

[safety]
enabled = true
audit_logging = true
```

### Running

```bash
# From the helios-framework directory
omnc run main.omni
```

HELIOS will initialize all subsystems and begin listening for requests.

---

## Module Dependency Graph

```
main
  ├─→ helios::service (HeliosService, ServiceState)
  ├─→ helios::cognitive (HeliosCognitive)
  ├─→ helios::capability (CapabilityRegistry)
  ├─→ brain::reasoning_engine (ReasoningEngine)
  ├─→ brain::knowledge_graph (KnowledgeGraph)
  ├─→ brain::memory_architecture (MemorySystem)
  ├─→ brain::adaptive_learning (LearningEngine)
  ├─→ safety::asimov (SafetyFramework)
  └─→ config::default (HeliosConfig, load_config)

helios::cognitive
  ├─→ brain::reasoning_engine
  ├─→ brain::memory_architecture
  ├─→ brain::adaptive_learning
  └─→ safety::asimov

helios::service
  ├─→ helios::cognitive
  └─→ config::default
```

---

## API Overview

### Processing a Request

```omni
import helios::service::HeliosService
import config::default::{load_config}

fn handle_user_query():
    let config = load_config("config/default.toml".to_string())
    let mut service = HeliosService::new(config)
    service.start()

    let response = service.handle_request(Request {
        id: 1,
        input: "What is the capital of France?".to_string(),
        session_id: 42,
        timestamp: time::now_ms(),
        metadata: HashMap::new()
    })

    println("Response: {}", response.output)
```

### Cognitive Pipeline

Each request flows through the full cognitive pipeline:

1. **Perceive** — classify intent (question, command, greeting, etc.)
2. **Think** — recall relevant memories, apply logical reasoning, build a thought chain
3. **Respond** — generate a natural-language response scaled by confidence
4. **Reflect** — store as episodic memory, feed to learning engine, periodically consolidate

### Direct Knowledge Graph Access

```omni
import brain::knowledge_graph::KnowledgeGraph

let mut kg = KnowledgeGraph::new()
let node_id = kg.add_node("Paris".to_string(), NodeType::Entity)
kg.set_property(node_id, "country".to_string(), "France".to_string())
let neighbors = kg.get_neighbors(node_id)
```

---

## Project Structure

```
helios-framework/
├── main.omni                # Entry point
├── helios/
│   ├── cognitive.omni       # Cognitive loop
│   ├── service.omni         # Service layer
│   └── capability.omni      # Capability registry
├── brain/
│   ├── reasoning_engine.omni   # Inference engine
│   ├── knowledge_graph.omni    # Property graph
│   ├── memory_architecture.omni # Memory system
│   └── adaptive_learning.omni  # Learning engine
├── safety/
│   └── asimov.omni          # Safety framework
└── config/
    └── default.omni         # Configuration
```

---

## License

Proprietary — HELIOS Project. All rights reserved.
