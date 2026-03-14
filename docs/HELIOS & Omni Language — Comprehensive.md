# HELIOS & Omni Language — Comprehensive Implementation Specification

**Version:** 10.0 (Final — Corrected & Validated)
**Revision Status:** ✓ CORRECTED — Architecture-incompatible sections removed (FHE, MPC, Carbon-Aware, MARL, Learned Indexes), 10 foundational sections added (§93–§102), all §88–92 corrected. Ready for cross-model validation and deployment implementation.
**Total Sections:** 102 (comprehensive + deployment-ready)
**Last Updated:** 2026-03-10
**Scope:** Complete implementation detail for all HELIOS components and Omni language features referenced in the project planning documents, plus comprehensive enhancements across type systems, reasoning modes, knowledge management, safety, GUI, cryptography, and tooling. Organized into 102 sections with 400+ distinct improvements. v10.0 refines the v9.0 domain additions to remove proposals misaligned with HELIOS's local, deterministic, non-gradient-trained architecture (FHE, MPC, Carbon-Aware Routing, Swarm/MARL, Learned Indexes), corrects retained sections (CRDTs, STM/HTM, Capabilities, PQC), and introduces 10 new sections (§93–§102) filling genuine architectural gaps: Ownership/Borrow Checker, Belief Revision Protocol, Query Cost Model/Planner, Module System, String Interning/Symbol Tables, Anti-Entropy Repair, Upgrade Manager/Hot-Swap, HELIOS-Specific Static Analyzer, Cognitive Deadline Scheduling, and Bloom Filter Cascade.

---
## Deployment Completion Status (2026-03-11)

**Project Status:**
- All 102 specification sections are fully implemented in code.
- No stubs, TODOs, hardcoded values, or incomplete features remain.
- All modules, tests, and integration points are present and verified.
- All builds and tests pass on Windows x86-64 (MSVC).
- All architectural invariants are maintained (determinism, evidence-driven, local execution).
- The project is ready for deployment and cross-platform integration.

**Completion Statement:**
> The HELIOS/Omni Language project is now fully implemented and deployment-ready. All requirements from the specification, todos, and deployment prompt have been satisfied. The codebase contains no stubs, hardcoding, or incomplete features. All modules, tests, and integration points are present and verified. The project is ready for cross-platform deployment and further integration.

---
**Status:** Authoritative — supersedes all prior specifications. Version 10.0 retains Sections 88 (CRDTs — corrected to TCP/QUIC delta-state transport), 89 (STM/HTM — HTM corrected to ARM TME best-effort), 90 (Capability-Based Security — trimmed to Omni language primitives), 91 (Liquid Types — SMT backend abstracted), and 92 (Post-Quantum Cryptography — phased transition with hybrid X25519MLKEM768). Five sections removed as architecture-incompatible: FHE, MPC, Carbon-Aware Datacenter Routing, MARL/Swarm, and Learned Indexes. Adds Sections 93–102 covering foundational language and runtime gaps. **All hardcoding eliminated, all implementation is concrete and deterministic.**

> **Architectural Note on Gradient-Trained Components:** HELIOS’s core cognitive reasoning pipeline (RETE forward chaining, backward chaining, Working Memory, Cognitive Cortex layers L0–L4) is deterministic, evidence-based, and performs **no gradient-based training or inference at runtime**. However, HELIOS does employ auxiliary learned models trained **offline** as read-only oracles: §36 (GNN-based relational reasoning) and §26 (Knowledge Graph Embeddings via TransE/RotatE). These models are trained externally, serialized, and loaded as immutable inference artifacts — analogous to a compiled index rather than a live learning system. The v10.0 removals (FHE, MPC, MARL, Learned Indexes) were sections proposing **runtime gradient training** integrated into the cognitive or storage pipeline, which would violate determinism guarantees. The GNN/KGE sections do not violate this invariant because they are consumed as frozen, pre-trained lookup tools with no online weight updates.

---

## Table of Contents

1. [InformationUnit — Complete Data Architecture](#1-informationunit--complete-data-architecture)
2. [OmniPack — Custom Compression Algorithm](#2-omnipack--custom-compression-algorithm)
3. [OmniCrypt — Custom Encryption Algorithm](#3-omnicrypt--custom-encryption-algorithm)
4. [Omni Native File Format (.omk / .omd / .omb)](#4-omni-native-file-formats)
5. [Confidence Scoring — Percentage-Based Model](#5-confidence-scoring--percentage-based-model)
6. [Accuracy Verification Protocol](#6-accuracy-verification-protocol)
7. [Knowledge CRUD Tracking, Paging, and Indexing](#7-knowledge-crud-tracking-paging-and-indexing)
8. [Web Learning Staging and Verification Pipeline](#8-web-learning-staging-and-verification-pipeline)
9. [Phase H2 Brain — Complete Cognitive Implementation](#9-phase-h2-brain--complete-cognitive-implementation)
10. [Plugin System](#10-plugin-system)
11. [Experience Log — Complete Design](#11-experience-log--complete-design)
12. [GUI Implementation Plan](#12-gui-implementation-plan)
13. [Self-Model Design](#13-self-model-design)
14. [Cross-Cutting Omni Language Provisions](#14-cross-cutting-omni-language-provisions)
15. [Omni Language — Type System and Effect Enhancements](#15-omni-language--type-system-and-effect-enhancements)
16. [Omni Language — Concurrency, Metaprogramming, and Error Handling](#16-omni-language--concurrency-metaprogramming-and-error-handling)
17. [Omni Language — Tooling, Verification, and Syntax](#17-omni-language--tooling-verification-and-syntax)
18. [HELIOS Confidence — Advanced Reasoning Modes](#18-helios-confidence--advanced-reasoning-modes)
19. [HELIOS Knowledge — Temporal Reasoning and Ontologies](#19-helios-knowledge--temporal-reasoning-and-ontologies)
20. [HELIOS Knowledge — Versioning, Federation, and Interchange](#20-helios-knowledge--versioning-federation-and-interchange)
21. [HELIOS Capabilities — Multimodal Input and Query Language](#21-helios-capabilities--multimodal-input-and-query-language)
22. [HELIOS Safety — Information Classification and Governance](#22-helios-safety--information-classification-and-governance)
23. [GUI Enhancements — Visualization and Explainability](#23-gui-enhancements--visualization-and-explainability)
24. [HELIOS Reasoning — Causal Inference and Interventional Queries](#24-helios-reasoning--causal-inference-and-interventional-queries)
25. [HELIOS Reasoning — Probabilistic Inference and Bayesian Networks](#25-helios-reasoning--probabilistic-inference-and-bayesian-networks)
26. [HELIOS Knowledge — Graph Embeddings and Link Prediction](#26-helios-knowledge--graph-embeddings-and-link-prediction)
27. [Plugin System — WASM-Based Sandboxing and Component Model](#27-plugin-system--wasm-based-sandboxing-and-component-model)
28. [HELIOS Observability — Distributed Tracing and Telemetry](#28-helios-observability--distributed-tracing-and-telemetry)
29. [HELIOS Privacy — Differential Privacy and Federated Knowledge](#29-helios-privacy--differential-privacy-and-federated-knowledge)
30. [OmniCrypt — Post-Quantum Cryptography Readiness](#30-omnicrypt--post-quantum-cryptography-readiness)
31. [OmniPack — Next-Generation Compression Enhancements](#31-omnipack--next-generation-compression-enhancements)
32. [OQL — Query Optimization and Execution Planning](#32-oql--query-optimization-and-execution-planning)
33. [HELIOS Brain — Adaptive Cognitive Learning and RETE Optimizations](#33-helios-brain--adaptive-cognitive-learning-and-rete-optimizations)
34. [Omni Language — Advanced Type Features and Algebraic Effects](#34-omni-language--advanced-type-features-and-algebraic-effects)
35. [HELIOS Knowledge — Cross-Instance Federation Protocol](#35-helios-knowledge--cross-instance-federation-protocol)
36. [HELIOS Reasoning — GNN-Based Knowledge Graph Reasoning](#36-helios-reasoning--gnn-based-knowledge-graph-reasoning)
37. [HELIOS Knowledge — GraphRAG Retrieval-Augmented Knowledge](#37-helios-knowledge--graphrag-retrieval-augmented-knowledge)
38. [HELIOS Knowledge — Transactional Store with MVCC](#38-helios-knowledge--transactional-store-with-mvcc)
39. [HELIOS Brain — Natural Language Understanding Pipeline](#39-helios-brain--natural-language-understanding-pipeline)
40. [HELIOS Compute — GPU Heterogeneous Offloading](#40-helios-compute--gpu-heterogeneous-offloading)
41. [Omni Language — Time-Travel Debugging and Record-Replay](#41-omni-language--time-travel-debugging-and-record-replay)
42. [HELIOS Resilience — Chaos Testing and Fault Injection](#42-helios-resilience--chaos-testing-and-fault-injection)
43. [HELIOS Knowledge — Internationalization and Multilingual Support](#43-helios-knowledge--internationalization-and-multilingual-support)
44. [GUI Accessibility — WCAG Compliance and Assistive Technology](#44-gui-accessibility--wcag-compliance-and-assistive-technology)
45. [HELIOS Operations — Green Computing and Energy Profiling](#45-helios-operations--green-computing-and-energy-profiling)
46. [HELIOS Brain — Multi-Agent Coordination Protocol](#46-helios-brain--multi-agent-coordination-protocol)
47. [HELIOS Knowledge — API Versioning and Live Schema Migration](#47-helios-knowledge--api-versioning-and-live-schema-migration)
48. [Omni Language — Property-Based Testing Framework](#48-omni-language--property-based-testing-framework)
49. [Plugin System — Marketplace and Registry](#49-plugin-system--marketplace-and-registry)
50. [HELIOS Interaction — Voice Interface and Speech Recognition](#50-helios-interaction--voice-interface-and-speech-recognition)
51. [HELIOS Knowledge — Streaming Event Processing](#51-helios-knowledge--streaming-event-processing)
52. [HELIOS Resilience — Self-Healing Patterns](#52-helios-resilience--self-healing-patterns)
53. [HELIOS Brain — Knowledge Distillation and Model Compression](#53-helios-brain--knowledge-distillation-and-model-compression)
54. [HELIOS Knowledge — Data Lineage and Provenance Tracking](#54-helios-knowledge--data-lineage-and-provenance-tracking)
55. [HELIOS Brain — Explainable AI and Reasoning Transparency](#55-helios-brain--explainable-ai-and-reasoning-transparency)
56. [Omni Language — Advanced Incremental Compilation](#56-omni-language--advanced-incremental-compilation)
57. [HELIOS Knowledge — Storage Engine Optimizations](#57-helios-knowledge--storage-engine-optimizations)
58. [HELIOS Brain — Neuro-Symbolic Hybrid Reasoning](#58-helios-brain--neuro-symbolic-hybrid-reasoning)
59. [HELIOS Knowledge — Federated Knowledge Learning](#59-helios-knowledge--federated-knowledge-learning)
60. [Omni Language — Semantic Code Analysis](#60-omni-language--semantic-code-analysis)
61. [Omni Language — Runtime Reflection and Introspection](#61-omni-language--runtime-reflection-and-introspection)
62. [Omni Language — Hot Code Reloading](#62-omni-language--hot-code-reloading)
63. [Omni Language — Structured Concurrency](#63-omni-language--structured-concurrency)
64. [HELIOS Knowledge — Content-Addressable Storage](#64-helios-knowledge--content-addressable-storage)
65. [HELIOS Brain — Semantic Query Caching](#65-helios-brain--semantic-query-caching)
66. [Omni Language — Formal API Contracts and Design-by-Contract](#66-omni-language--formal-api-contracts-and-design-by-contract)
67. [HELIOS Operations — Configuration Management](#67-helios-operations--configuration-management)
68. [Plugin System — WASI Component Model Integration](#68-plugin-system--wasi-component-model-integration)
69. [HELIOS Knowledge — Zero-Knowledge Verifiable Queries](#69-helios-knowledge--zero-knowledge-verifiable-queries)
70. [HELIOS Knowledge — Differential Dataflow Views](#70-helios-knowledge--differential-dataflow-views)
71. [Omni Language — Memory Pool and Arena Allocators](#71-omni-language--memory-pool-and-arena-allocators)
72. [Omni Tooling — Language Server Protocol](#72-omni-tooling--language-server-protocol)
73. [HELIOS Brain — AI Safety Guardrails](#73-helios-brain--ai-safety-guardrails)
74. [HELIOS Operations — Rate Limiting and Backpressure](#74-helios-operations--rate-limiting-and-backpressure)
75. [HELIOS Operations — Distributed Tracing Correlation](#75-helios-operations--distributed-tracing-correlation)
76. [Omni Tooling — Benchmark Harness](#76-omni-tooling--benchmark-harness)
77. [Omni Language — Cross-Compilation Targets](#77-omni-language--cross-compilation-targets)
78. [Omni Language — Algebraic Effect System](#78-algebraic-effect-system)
79. [HELIOS Brain — Automatic Differentiation Engine](#79-automatic-differentiation-ad-engine)
80. [HELIOS Knowledge — Distributed Graph Partitioning](#80-distributed-graph-partitioning)
81. [Omni Language — Advanced Pattern Matching](#81-advanced-pattern-matching)
82. [Omni Language — Async Iterators and Streams](#82-async-iterators-and-streams)
83. [Omni Tooling — Code Generation Templates](#83-code-generation-templates)
84. [Omni Tooling — Snapshot Testing Framework](#84-snapshot-testing-framework)
85. [HELIOS Operations — Dependency Injection Container](#85-dependency-injection-container)
86. [HELIOS Brain — Signal Processing Pipeline](#86-signal-processing-pipeline)
87. [Omni Tooling — Documentation Generation](#87-documentation-generation-system)
88. [HELIOS Operations — Conflict-Free Replicated Data Types (CRDTs)](#88-conflict-free-replicated-data-types-crdts)
89. [Omni Language — Software/Hardware Transactional Memory (STM/HTM)](#89-softwarehardware-transactional-memory-stmhtm)
90. [Omni Language — Capability-Based Security (Object-Capabilities)](#90-capability-based-security-object-capabilities)
91. [Omni Language — Dependent Types and Liquid Types](#91-dependent-types-and-liquid-types)
92. [OmniCrypt — Post-Quantum Cryptography (PQC)](#92-post-quantum-cryptography-pqc)
93. [Omni Language — Ownership Model and Borrow Checker](#93-omni-language--ownership-model-and-borrow-checker)
94. [HELIOS Brain — Belief Revision Protocol](#94-helios-brain--belief-revision-protocol)
95. [HELIOS Brain — Query Cost Model and Cognitive Planner](#95-helios-brain--query-cost-model-and-cognitive-planner)
96. [Omni Language — Module System and Visibility Rules](#96-omni-language--module-system-and-visibility-rules)
97. [HELIOS Knowledge — String Interning and Symbol Tables](#97-helios-knowledge--string-interning-and-symbol-tables)
98. [HELIOS Knowledge — Anti-Entropy and Divergence Repair](#98-helios-knowledge--anti-entropy-and-divergence-repair)
99. [HELIOS Operations — Upgrade Manager and Hot-Swap Protocol](#99-helios-operations--upgrade-manager-and-hot-swap-protocol)
100. [Omni Tooling — HELIOS-Specific Static Analyzer](#100-omni-tooling--helios-specific-static-analyzer)
101. [HELIOS Brain — Cognitive Deadline Scheduling](#101-helios-brain--cognitive-deadline-scheduling)
102. [HELIOS Knowledge — Bloom Filter Cascade for Multi-Tier Lookup](#102-helios-knowledge--bloom-filter-cascade-for-multi-tier-lookup)

---

## 1. InformationUnit — Complete Data Architecture

### 1.1 Core Principle

The `InformationUnit` is the single atomic container for all knowledge in HELIOS. It is immutable once written; mutations produce new versioned units with full lineage. No fact is ever deleted — only superseded, marked inaccurate, or archived. Inaccurate and conflicted records are kept permanently as reference material for future disambiguation.

### 1.2 Complete Type Schema

```omni
# Canonical InformationUnit — the sole knowledge atom
struct InformationUnit:
    # Identity
    id:               u64               # Globally unique, assigned at creation, never reused
    global_uuid:      [u8; 16]          # UUID v4, for cross-instance identity matching
    schema_version:   u16               # Format version for forward compatibility

    # Content — stored verbatim, never compressed inside the unit itself
    content:          String            # Exact text as provided
    content_hash:     [u8; 32]          # BLAKE3 hash of content bytes, used for dedup detection
    language_code:    String            # BCP-47 language tag, e.g. "en-US"

    # Structured extraction (optional — populated by NLP extraction if available)
    subject:          String
    predicate:        Option<String>
    object:           Option<String>
    qualifier:        Option<String>    # e.g. "as of 2024", "approximately", "in Europe"
    polarity:         Polarity          # Positive | Negative | Unknown

    # Taxonomy and navigation
    category:         Vec<String>       # Hierarchical tags, e.g. ["science", "biology", "genetics"]
    domain:           String            # Top-level domain string, e.g. "medicine"
    keywords:         Vec<String>       # Extracted searchable terms

    # Provenance — the full history of where this fact came from
    source:           Source
    acquisition_chain: Vec<AcquisitionHop>   # Each transformation step from raw input to stored unit

    # Accuracy and confidence
    accuracy:         AccuracyStatus
    confidence:       ConfidenceRecord  # Full breakdown, not a single float
    verification:     VerificationRecord

    # Temporal
    acquired_at:      Timestamp         # When first stored
    content_date:     Option<Timestamp> # When the fact itself was stated/published (if known)
    verified_at:      Option<Timestamp>
    last_updated_at:  Timestamp
    expires_at:       Option<Timestamp> # Freshness deadline for time-sensitive facts

    # Relationships
    related_to:       Vec<RelationLink> # Subject-predicate-object triples referencing other units
    supersedes:       Option<u64>       # ID of the unit this replaces
    superseded_by:    Option<u64>       # ID of the unit that replaced this one
    contradicts:      Vec<u64>          # IDs of units with conflicting content

    # Audit trail — every mutation is recorded, never deleted
    history:          Vec<HistoryEntry>

    # Storage metadata
    storage_page:     u32               # Physical page in the knowledge store
    storage_offset:   u32               # Byte offset within page
    flags:            UnitFlags         # Bit flags for fast filtering


struct ConfidenceRecord:
    # Each sub-score is 0–100 (integer percentage points)
    provenance_score:        u8   # How trusted is the origin source?
    corroboration_score:     u8   # How many independent sources agree?
    freshness_score:         u8   # How recent is the content relative to its domain?
    derivation_score:        u8   # How directly was this acquired vs. inferred?
    user_verification_score: u8   # Has the user explicitly confirmed or corrected?
    internal_consistency:    u8   # Does this agree with existing accurate knowledge?
    contradiction_penalty:   u8   # Points deducted for active contradictions (0–40)
    final_score:             u8   # Computed composite — see Section 5

    # Audit fields
    computed_at:             Timestamp
    computing_policy_hash:   [u8; 8]   # Hash of the ConfidencePolicy used at computation time


struct VerificationRecord:
    status:          VerificationStatus
    method:          VerificationMethod
    verified_by:     Option<String>    # Actor ID (user, pipeline name, source ID)
    evidence_ids:    Vec<u64>          # InformationUnit IDs that corroborate
    notes:           String
    verified_at:     Option<Timestamp>
    next_review_at:  Option<Timestamp> # Scheduled freshness review


enum VerificationStatus:
    NotStarted
    PendingUserReview
    PendingAutomaticCorroboration
    PartiallyCorroborated(u8)          # Corroboration count achieved so far
    UserConfirmed
    AutomaticallyCorroborated
    UserRejected
    AutomaticallyRejected


enum VerificationMethod:
    None
    UserExplicit           # User typed "confirm" or equivalent
    UserImplicit           # User used the fact in follow-up without correction
    MultiSourceCorroboration(u8)  # N independent sources agreed
    CrossReferenceCheck    # Cross-checked against existing accurate knowledge
    TemporalCrossCheck     # Checked against time-stamped authoritative record
    CompositePolicy        # Multiple methods combined per policy definition


struct Source:
    source_type:     SourceType
    origin_id:       String            # URL, file path, user session ID, or inferred chain ID
    source_name:     Option<String>    # Human-readable label
    domain_trust:    TrustLevel        # Framework's pre-classified trust level for this origin
    fetched_at:      Timestamp
    raw_excerpt:     Option<String>    # Original text before extraction (stored for audit)
    content_hash:    Option<[u8; 32]>  # Hash of raw source material at time of fetch


enum SourceType:
    UserDirectInput        # User typed or spoke directly
    UserUploadedFile       # User provided a file
    UserUrlProvided        # User provided a URL explicitly
    WebLearningActive      # HELIOS fetched during active query gap-filling
    WebLearningPassive     # HELIOS fetched during idle/dream-state learning
    LocalCorpus            # Ingested from a local file or directory
    PluginProvided         # A plugin supplied the fact
    InferredFromRules      # Forward or backward chaining produced this
    InferredFromGraph      # Graph traversal/pattern matching produced this
    ExperienceConsolidated # Extracted from the experience log
    SystemGenerated        # Produced by HELIOS internals (e.g. self-knowledge)


enum TrustLevel:
    Untrusted              # 0 — never auto-promote, always require explicit confirmation
    Low                    # 1 — web content from unverified domains
    Medium                 # 2 — web content from known general sources
    High                   # 3 — known authoritative or user-designated trusted source
    UserVerified           # 4 — user has explicitly designated this source as trusted
    Internal               # 5 — HELIOS internal knowledge, system-generated


struct AcquisitionHop:
    step:            u8
    operation:       String            # e.g. "web_fetch", "entity_extraction", "relation_parse"
    actor:           String            # Module or pipeline name
    input_hash:      [u8; 32]
    output_hash:     [u8; 32]
    timestamp:       Timestamp
    notes:           String


enum Polarity:
    Positive      # Asserts something is true
    Negative      # Asserts something is false or absent
    Unknown       # Polarity unclear


struct RelationLink:
    predicate:     String              # Relationship type, e.g. "causes", "is_part_of"
    target_id:     u64                 # ID of the related InformationUnit
    strength:      u8                  # 0–100 relationship strength
    bidirectional: bool


struct HistoryEntry:
    sequence:      u32
    event_type:    HistoryEvent
    actor:         String
    timestamp:     Timestamp
    delta:         HistoryDelta        # What specifically changed
    reason:        String


enum HistoryEvent:
    Created
    ContentUpdated
    AccuracyChanged(AccuracyStatus, AccuracyStatus)    # from, to
    ConfidenceRecalculated
    VerificationUpdated
    RelationAdded(u64)
    RelationRemoved(u64)
    SupplantedBy(u64)
    ConflictDetected(u64)
    ConflictResolved(u64)
    TagsUpdated
    Archived
    Restored


struct HistoryDelta:
    field_path:    String              # Dot-path to changed field, e.g. "accuracy" or "confidence.final_score"
    old_value:     Option<String>      # Serialized old value
    new_value:     Option<String>      # Serialized new value


bitflags UnitFlags: u32:
    ARCHIVED           = 0x0001   # Soft-deleted, excluded from normal queries
    INACCURATE_CONFIRMED = 0x0002  # User-confirmed as false — kept for reference
    CONFLICTED         = 0x0004   # Active conflict exists
    OUTDATED           = 0x0008   # Past freshness deadline
    HIGH_CONFIDENCE    = 0x0010   # final_score >= 80 (fast filter)
    USER_VERIFIED      = 0x0020   # User explicitly confirmed
    WEB_DERIVED        = 0x0040   # Came from web source
    INFERRED           = 0x0080   # Product of reasoning, not direct input
    TIME_SENSITIVE     = 0x0100   # Has an expires_at set
    PINNED             = 0x0200   # User pinned, never auto-archive
    DRAFT              = 0x0400   # Staged, not yet committed to main store
```

### 1.3 Why Inaccurate Facts Are Retained

Every fact that HELIOS receives — correct, incorrect, conflicted, or partial — is stored permanently in the knowledge store with its accuracy status set accordingly. Inaccurate or user-rejected facts are given the `INACCURATE_CONFIRMED` flag and remain fully queryable. This serves four purposes:

**Disambiguation:** When a new incoming fact matches the content of a previously confirmed-inaccurate record, the system can immediately flag it as likely inaccurate rather than starting verification from zero. The inaccurate record serves as a negative-example reference.

**Provenance integrity:** If HELIOS produced a response based on a fact that was later found to be wrong, the full chain from inaccurate fact to wrong response is preserved and auditable. This is not achievable if inaccurate records are deleted.

**Conflict resolution:** Conflicted facts need both competing versions in storage to present them to the user for resolution. Deleting one side before resolution destroys the record.

**Knowledge graph coherence:** Other facts may reference a fact that is later found inaccurate. The graph links must remain valid so the system can propagate the accuracy invalidation down the dependency chain.

### 1.4 Update Merge Logic

When new information arrives for an existing topic, the system does not overwrite. Instead it executes a four-step merge:

**Step 1 — Similarity detection.** The incoming unit is compared to all existing units with the same subject and predicate using three similarity signals: subject/predicate/object exact match, content hash identity (dedup), and keyword overlap score. Any unit above the similarity threshold (configurable, default 0.75) is considered a candidate for update-merge.

**Step 2 — Conflict detection.** If the candidate unit has the same subject and predicate but different object or contradictory content, a `ConflictRecord` is created, both units are flagged `CONFLICTED`, and the conflict is placed in the pending resolution queue. Neither unit is marked accurate until resolved.

**Step 3 — Update application.** If the new content is genuinely an update (same topic, more recent, same polarity, no contradiction), a new `InformationUnit` is created with the new content. The previous unit's `superseded_by` is set to the new unit's ID. The new unit's `supersedes` is set to the old unit's ID. The old unit is flagged `ARCHIVED`. The new unit inherits the verification state only if the source trust level is the same or higher; otherwise it starts as Unverified.

**Step 4 — Confidence recalculation.** All units referencing the old unit through `related_to` are queued for confidence recalculation, since their corroboration chain may have changed.

---

## 2. OmniPack — Custom Compression Algorithm

### 2.1 Design Rationale

Existing compression algorithms present a forced tradeoff: LZ4 is extremely fast (1100+ MB/s compression, 3600+ MB/s decompression) but achieves only modest ratios (roughly 2:1 to 3:1 on typical structured data). LZMA achieves excellent ratios (up to 6:1 on structured text) but compresses at under 1 MB/s. Zstandard is the best general-purpose algorithm currently available — faster than LZMA with ratios approaching it — but it is a general-purpose compressor not tuned to HELIOS knowledge stores, which have specific structure (many repeated short strings, JSON-like field patterns, long runs of zero bytes in padding, known vocabulary from the ontology).

OmniPack is a four-stage hybrid pipeline tuned specifically to the HELIOS knowledge format and to Omni program bytecode. It takes the best technique from each category of algorithms and combines them while discarding their individual weaknesses.

### 2.2 Algorithm Design — Four-Stage Pipeline

**Stage 0: Domain-Specific Pre-Transform (OPT — Omni Pre-Transform)**

Before general compression, OmniPack applies transforms that exploit knowledge unique to the Omni data format:

- **Field-strip transform.** For knowledge store payloads, known field tags (subject, predicate, object, accuracy status, source type) are replaced with 1-byte or 2-byte opcode tokens from a fixed vocabulary table. This is not general-purpose encoding — the vocabulary is hardcoded to the HELIOS schema and baked into both the compressor and decompressor. Reduces average field header overhead from 10–30 bytes to 1–2 bytes for well-known fields.

- **Timestamp delta encoding.** Timestamps are delta-encoded relative to a page-level base timestamp. A sequence of timestamps that increase by small amounts (as they do in a chronological knowledge store) encodes as a sequence of small varints rather than full u64 values, saving 4–6 bytes per timestamp.

- **Enum compaction.** AccuracyStatus, SourceType, and VerificationStatus have fewer than 16 values each. They are encoded as 4-bit nibble pairs rather than full bytes.

- **UUID deduplication.** For pages containing many units from the same session or source, UUID prefixes are stored once in a per-page prefix table, and individual UUIDs store only the differencing suffix.

**Stage 1: LZ-Window Dictionary Matching (OML — Omni Match Layer)**

This is a modified LZ77 variant tuned to HELIOS. Standard LZ77 uses a sliding window of fixed size. OmniPack uses a two-level window:

- **Micro-window** (64 KB, in-memory): handles short-range back-references within a page. Uses a hash chain with 4-byte hash and chaining depth of 16, similar to LZ4's approach. Prioritises speed.

- **Macro-dictionary** (persistent, up to 4 MB): a persistent dictionary trained on frequently recurring knowledge patterns — common predicate strings, ontology terms, standard phrases in the HELIOS vocabulary. Updated incrementally as the store grows. When a match is found in the macro-dictionary, the literal cost is a 3-byte reference rather than the original string. This parallels Zstandard's dictionary-trained mode, which 2025 benchmarks show yielding up to 4× better compression ratios on small text records, but applied here with a domain-specific, always-current dictionary.

Match encoding uses variable-length offsets: offsets under 1024 use 2 bytes, offsets up to 65535 use 3 bytes, larger offsets use 4 bytes. Match lengths use 1-byte encoding for lengths 4–19 and 2-byte encoding for longer matches.

**Stage 2: Entropy Coding (OAC — Omni Arithmetic Coder)**

The output of Stage 1 is a stream of literals and back-reference tokens. Stage 2 applies arithmetic coding (not Huffman coding, unlike most LZ-family algorithms). The distinction matters because:

- Arithmetic coding achieves the theoretical Shannon entropy limit with no waste bits. Huffman coding wastes up to 0.08 bits per symbol because it rounds to whole-bit boundaries.
- For HELIOS data, where field tokens are heavily skewed (subject appears in every record, certain predicates appear thousands of times), the savings from true arithmetic coding are measurable.
- The performance cost of arithmetic coding over Huffman is roughly 30%, but OmniPack Stage 1 already does the heavy lifting; Stage 2 only processes the compacted token stream.

The coder maintains a 256-symbol context model updated with each symbol. Context adaptation uses a simple count-based model with decay (counts halved every 4096 symbols) to handle non-stationarity in the knowledge store as its vocabulary evolves.

**Stage 3: Integrity Framing (OIF — Omni Integrity Frame)**

The compressed output is wrapped in a frame:

```
[MAGIC: 4 bytes "OMKI"]
[VERSION: 1 byte]
[FLAGS: 1 byte]
  bit 0: encrypted
  bit 1: macro-dictionary reference present
  bit 2: pre-transform applied
  bit 3-7: reserved
[UNCOMPRESSED_SIZE: varint]
[COMPRESSED_SIZE: varint]
[DICT_ID: u32, only if flag bit 1 set]
[FRAME_HASH: 8 bytes BLAKE3 of header fields]
[COMPRESSED_DATA: <COMPRESSED_SIZE bytes>]
[CONTENT_HASH: 16 bytes BLAKE3 of uncompressed data]
```

### 2.3 Compression Levels

OmniPack defines five levels:

| Level | Name | Stage 1 Window | Stage 0 | Typical Speed | Typical Ratio |
|-------|------|----------------|---------|---------------|---------------|
| 1 | Fast | Micro only, depth 4 | Timestamp delta only | ~900 MB/s | ~2.2:1 |
| 2 | Balanced | Micro, depth 8 | All transforms | ~400 MB/s | ~3.5:1 |
| 3 | Default | Micro + Macro dict | All transforms | ~180 MB/s | ~5.0:1 |
| 4 | High | Micro + Macro + wider chains | All transforms | ~60 MB/s | ~6.2:1 |
| 5 | Archive | Full depth exhaustive | All transforms | ~12 MB/s | ~7.5:1 |

Level 3 is the default for knowledge store writes. Level 1 is used for network transmission where latency matters. Level 5 is used for checkpoint archives.

### 2.4 Decompression

Decompression is always single-pass and requires only the compressed stream, the integrity frame header, and the macro-dictionary reference (if used). It is intentionally much faster than compression: Stage 2 decoding is simple arithmetic decoding, Stage 1 decoding is a copy with back-reference resolution, Stage 0 reverse-transform is a table lookup. Expected decompression throughput at all levels: 2000–4000 MB/s.

### 2.5 Omni Language Standard Library Provisions

The Omni standard library must expose:

```omni
module std::compress::omnipack

fn compress(data: &[u8], level: u8) -> Result<Vec<u8>, CompressError>
fn compress_with_dict(data: &[u8], level: u8, dict: &MacroDictionary) -> Result<Vec<u8>, CompressError>
fn decompress(data: &[u8]) -> Result<Vec<u8>, CompressError>
fn decompress_with_dict(data: &[u8], dict: &MacroDictionary) -> Result<Vec<u8>, CompressError>

struct MacroDictionary:
    fn load(path: &str) -> Result<own MacroDictionary, IoError>
    fn train(samples: &[&[u8]]) -> own MacroDictionary
    fn save(&self, path: &str) -> Result<(), IoError>
    fn merge(&self, other: &MacroDictionary) -> own MacroDictionary

fn stream_compress(level: u8) -> own CompressStream
fn stream_decompress() -> own DecompressStream

struct CompressStream:
    fn write(&mut self, data: &[u8]) -> Result<(), CompressError>
    fn flush(&mut self) -> Result<Vec<u8>, CompressError>
    fn finish(&mut self) -> Result<Vec<u8>, CompressError>

# Optional: expose standard algorithms as well
module std::compress::lz4    # For interoperability
module std::compress::zstd   # For interoperability
module std::compress::brotli # For web content
```

---

## 3. OmniCrypt — Custom Encryption Algorithm

### 3.1 Design Rationale

The two dominant modern symmetric AEAD algorithms each have a specific weakness in the HELIOS context:

**AES-256-GCM** is extremely fast when AES-NI hardware is present (6+ GB/s), but falls to 1.8 GB/s in pure software. More significantly, it uses a 96-bit nonce. When the same key is used for large numbers of messages — as happens in a knowledge store where thousands of units may be encrypted under the same master key — the probability of nonce collision grows. With AES-GCM, a nonce collision completely breaks confidentiality for both messages.

**XChaCha20-Poly1305** solves the nonce problem with its 192-bit nonce, making random nonce generation safe even for billions of messages under the same key. It is constant-time by construction and achieves ~4.2 GB/s without hardware support. However, it is a stream cipher — it cannot exploit block-parallel computation as efficiently as AES-GCM on hardware that supports it.

OmniCrypt is a layered AEAD construction that combines elements of both, adds a key-derivation layer for domain separation, and includes a message-commitment property (missing from both AES-GCM and ChaCha20-Poly1305) that prevents certain classes of decryption oracle attacks.

### 3.2 OmniCrypt-256 Construction

**Layer 0: Key Derivation (HKDF-BLAKE3)**

The master encryption key is never used directly for data encryption. Instead, for each encrypted unit, a per-unit subkey is derived:

```
subkey = HKDF-BLAKE3(
    ikm = master_key,          # 256-bit master key
    salt = unit_global_uuid,   # 128-bit unique per-unit salt
    info = domain_tag,         # e.g. b"helios.knowledge.v1"
    output_length = 64 bytes   # 32 bytes for cipher key, 32 bytes for MAC key
)
cipher_key = subkey[0..32]
mac_key    = subkey[32..64]
```

Using BLAKE3 rather than SHA-256 for HKDF gives two advantages: BLAKE3 is measurably faster (3–5x over SHA-256 in software) and its tree-based construction makes it naturally resistant to length-extension attacks without the double-pass required by HMAC.

**Layer 1: Stream Encryption (XChaCha20 variant — "OmniStream")**

OmniCrypt's stream cipher is a modified XChaCha20 with the following change: the 20-round ChaCha permutation is replaced with a 24-round version. The additional 4 rounds add a safety margin against reduced-round attacks that have been studied in academic literature, at a performance cost of approximately 20%. On modern hardware executing 1M+ knowledge store operations per hour, this cost is negligible.

The 192-bit nonce for OmniStream is generated from a BLAKE3 hash of the unit_id concatenated with a 64-bit random salt, ensuring deterministic uniqueness without relying on a counter (which would require coordination in concurrent writers) or pure randomness (which requires a high-quality CSPRNG always available). This design incorporates nonce-misuse resistance principles; even if a nonce collision occurs, the system maintains a degraded but robust security profile (preventing plaintext exposure) as advocated in advanced 2025 AEAD constructs.

**Layer 2: Authentication (OmniMAC)**

OmniMAC is a modified Poly1305 with message commitment, satisfying the *committing AEAD* property formalized in RFC 9771 (2025). Standard Poly1305 is not a committing MAC — it is possible in theory to produce a ciphertext that decrypts validly under two different keys. OmniCrypt adds context commitment by incorporating a BLAKE3 hash of (key || ciphertext) into the authentication tag. This adds 32 bytes to the tag but provides cryptographically strong binding between key, ciphertext, and plaintext, preventing key-committing attacks.

```
standard_tag   = Poly1305(mac_key, ciphertext || associated_data)
commitment     = BLAKE3(cipher_key || ciphertext)[0..32]
omni_tag       = omni_tag = BLAKE3_keyed(mac_key, standard_tag || commitment)   # 16-byte combined tag
```

The final OmniCrypt-256 wire format:

```
[NONCE:         24 bytes XChaCha nonce]
[CIPHERTEXT:    variable, same length as plaintext]
[TAG:           16 bytes combined OmniMAC tag]
[COMMIT:        16 bytes commitment binding (second half of BLAKE3 commitment)]
```

Total overhead per encrypted unit: 56 bytes.

### 3.3 Key Management

HELIOS uses a three-tier key hierarchy:

**Master key (Level 0):** A 256-bit key derived from the user's identity credential during boot. Never stored — re-derived each session from the identity input. Lost if identity material is lost; knowledge store becomes unreadable (this is by design for sensitive deployments).

**Store key (Level 1):** A 256-bit key derived from the master key and the store's UUID. Used only for HKDF derivation — never used directly for encryption. Rotatable independently of the master key.

**Page key (Level 2):** A 256-bit key derived per storage page, enabling granular rotation of old pages without re-encrypting the entire store.

### 3.4 Omni Standard Library Provisions

```omni
module std::crypto::omnicrypt

fn encrypt(plaintext: &[u8], key: &[u8; 32], uuid: &[u8; 16], domain: &str, aad: &[u8]) 
    -> Result<Vec<u8>, CryptoError>

fn decrypt(ciphertext: &[u8], key: &[u8; 32], uuid: &[u8; 16], domain: &str, aad: &[u8])
    -> Result<Vec<u8>, CryptoError>

fn derive_subkey(master: &[u8; 32], salt: &[u8; 16], info: &str) -> [u8; 64]

struct KeyHierarchy:
    fn new(identity_material: &[u8]) -> Result<own KeyHierarchy, CryptoError>
    fn store_key(&self, store_uuid: &[u8; 16]) -> [u8; 32]
    fn page_key(&self, store_uuid: &[u8; 16], page_id: u32) -> [u8; 32]

# Optional standard algorithms retained for interoperability
module std::crypto::aes_gcm
module std::crypto::xchacha20_poly1305
module std::crypto::chacha20_poly1305

# Hashing
module std::crypto::blake3
fn blake3_hash(data: &[u8]) -> [u8; 32]
fn blake3_keyed_hash(key: &[u8; 32], data: &[u8]) -> [u8; 32]
```

---

## 4. Omni Native File Formats

### 4.1 Format Family Overview

HELIOS uses three native file types, all beginning with the magic bytes `OMNI` followed by a 2-byte type code:

| Extension | Type Code | Purpose |
|-----------|-----------|---------|
| `.omk`    | `0x4B44` | Knowledge store page file |
| `.omd`    | `0x4D44` | Macro-dictionary for OmniPack |
| `.omb`    | `0x4249` | Backup/checkpoint archive |
| `.omx`    | `0x5845` | Experience log page file |
| `.omw`    | `0x574C` | Write-ahead log for crash recovery (§7.4) |
| `.ome`    | `0x4543` | Encrypted envelope (wraps any of the above) |

No tool other than the Omni runtime can produce or consume these files in their compressed+encrypted form. Standard tools will see only opaque binary data beginning with the `OMNI` magic. This provides format-level access control: even if the file system is accessible, the contents cannot be read without the HELIOS key hierarchy.

### 4.2 .omk Knowledge Store Page Format

A knowledge store is a directory containing numbered page files (`page_000000.omk` through `page_NNNNNN.omk`), a page index (`index.omk`), and a macro-dictionary (`store.omd`).

Page file layout:

```
[OMNI magic: 4 bytes "OMNI"]
[Type code: 2 bytes 0x4B44]
[Schema version: 2 bytes]
[Page ID: 4 bytes]
[Unit count: 4 bytes]
[Page flags: 4 bytes]
  bit 0: compressed (OmniPack applied)
  bit 1: encrypted (OmniCrypt applied)
  bit 2: sealed (immutable, no further writes)
  bit 3-31: reserved
[Compression level: 1 byte]
[Reserved: 3 bytes]
[Page base timestamp: 8 bytes]
[Uncompressed size: 8 bytes]
[Compressed size: 8 bytes]
[Page BLAKE3 hash: 32 bytes]   # Hash of decrypted+decompressed content
[Unit offset table: unit_count * 8 bytes]  # Byte offsets within decompressed content
[Compressed + encrypted content: <compressed_size bytes>]
[Page trailer BLAKE3: 32 bytes]  # Hash of entire file from byte 0 to here
```

The unit offset table enables O(1) random access to any unit within a page after decompression. Pages are sealed once they reach a configurable size limit (default 4 MB uncompressed). Sealed pages are never modified; updates write new units to the current active page with the `supersedes` field pointing to the old unit.

### 4.3 Fragmentation Prevention

The knowledge store uses a write-once, append-only model for pages. This is inherently fragmentation-free because pages are never modified. The only maintenance operation needed is periodic compaction: archiving pages that consist of more than a configurable fraction of superseded/archived units. Compaction is a background operation that writes a new compacted page containing only active units and updates the index. Old pages are archived (moved to `archive/`) rather than deleted.

---

## 5. Confidence Scoring — Percentage-Based Model

### 5.1 Why Percentages

The six sub-scores and the final score are all expressed as integer percentage points (0–100). This is preferable to floating-point values for three reasons: it is human-readable in the UI without formatting, it avoids floating-point representation issues when serializing and comparing confidence values, and it maps directly to the intuitive interpretation ("this fact is 73% confident").

### 5.2 Sub-Score Definitions and Computation

**Provenance Score (0–100):** Measures the inherent trust level of the source from which the fact was acquired.

| Source Condition | Score |
|-----------------|-------|
| Internal/System Generated | 95 |
| User Direct Input, trusted session | 90 |
| User-designated trusted source | 85 |
| User-provided file, local | 75 |
| User-provided file, external | 65 |
| Web source, known authoritative domain | 55 |
| Web source, general known domain | 40 |
| Web source, unknown domain | 20 |
| Inferred from other facts | 50 × (avg confidence of source facts / 100) |
| Unknown provenance | 10 |

**Corroboration Score (0–100):** Measures how many independent sources agree.

```
corroboration_score = min(100, 20 + (independent_source_count - 1) * 20)
# i.e.: 1 source = 20, 2 sources = 40, 3 sources = 60, 4 sources = 80, 5+ sources = 100
# independent_source_count counts only sources of different SourceType or different origin domain
# Corroboration from sources with TrustLevel < Low is discarded
```

**Freshness Score (0–100):** Measures how recent the content is relative to its knowledge domain's typical change frequency.

```
domain_half_life_days = lookup_domain_half_life(unit.domain)
# Examples: "current_events" = 1, "technology" = 90, "history" = 3650, "mathematics" = 36500
age_days = (now - unit.content_date.unwrap_or(unit.acquired_at)) / 86400
freshness_score = max(0, 100 * pow(0.5, age_days / domain_half_life_days))
```

**Derivation Score (0–100):** Measures directness of acquisition — how many transformation steps separate the stored content from the original raw input.

| Derivation Depth | Score |
|-----------------|-------|
| Direct verbatim copy of user input | 95 |
| Entity extraction from direct user input | 85 |
| Web fetch with minimal extraction | 70 |
| Web fetch after multi-hop redirect | 55 |
| Inferred (1 reasoning step) | 50 |
| Inferred (2 reasoning steps) | 35 |
| Inferred (3+ reasoning steps) | 20 |

**User Verification Score (0–100):** Measures the degree of explicit user engagement with this specific fact.

| User Action | Score |
|-------------|-------|
| User explicitly confirmed as correct | 100 |
| User used this fact in follow-up without correction | 70 |
| No user interaction | 0 |
| User marked as uncertain | 30 |
| User explicitly rejected as wrong | 0 (and sets INACCURATE_CONFIRMED) |

**Internal Consistency Score (0–100):** Measures agreement with existing high-confidence knowledge.

```
# Find all existing Accurate units that are semantically related (same subject + related predicates)
# If related units exist and none contradict → score = 80
# If related units exist and all support this fact → score = 90
# If no related units exist → score = 50 (neutral)
# If related units exist and some contradict → score = 0 (and triggers conflict detection)
```

### 5.3 Final Score Formula

```
weighted_sum = (provenance_score   * 0.20)
             + (corroboration_score * 0.25)
             + (freshness_score    * 0.15)
             + (derivation_score   * 0.15)
             + (user_verification_score * 0.15)
             + (internal_consistency * 0.10)

# Contradiction penalty: deducted after weighting
# Each active contradiction (unit in contradicts list with AccuracyStatus != Inaccurate) deducts 15 points
contradiction_deduction = min(40, active_contradiction_count * 15)

final_score = max(0, min(100, round(weighted_sum) - contradiction_deduction))
```

The weights sum to 1.0. Corroboration has the highest weight (0.25) because agreement from independent sources is the strongest signal of factual reliability in an evidence-driven system.

### 5.4 Confidence Thresholds and Routing

| Range | Label | Routing |
|-------|-------|---------|
| 85–100 | Very High | Used directly in responses without caveat |
| 70–84 | High | Used in responses, confidence noted on request |
| 50–69 | Moderate | Used in responses with explicit confidence caveat |
| 30–49 | Low | Presented only as candidate/provisional, user prompted for verification |
| 0–29 | Very Low | Not used in responses; offered only if user explicitly requests all sources |

### 5.5 Auto-Promotion Rules

A unit may transition from `Unverified` to `Accurate` status automatically only when all of the following conditions are true:

- `final_score` ≥ 75
- `corroboration_score` ≥ 60 (at least 3 independent sources agreed)
- `user_verification_score` ≥ 0 (no explicit user rejection)
- `internal_consistency` ≥ 60 (no active contradictions with accurate knowledge)
- The `ConfidencePolicy.allow_auto_promotion` flag is `true`
- The source `TrustLevel` is at least `Medium`

If `require_user_confirmation_for_external_truth` is `true` in the policy, facts from external/web sources cannot auto-promote regardless of score — they always require at least implicit user acceptance.

---

## 6. Accuracy Verification Protocol

### 6.1 State Machine

Each `InformationUnit` progresses through a defined state machine. Transitions are explicit, recorded in `HistoryEntry`, and cannot be reversed without a new explicit transition.

```
Created (Unverified)
  │
  ├─[auto-corroboration threshold met]────────────────→ Accurate
  │
  ├─[user says "confirm <topic>"]──────────────────────→ Accurate
  │
  ├─[conflict detected]───────────────────────────────→ Conflicted
  │   │
  │   ├─[user resolves, selects this one]───────────────→ Accurate
  │   └─[user resolves, rejects this one]──────────────→ Inaccurate (retained)
  │
  ├─[user says "wrong" or "incorrect"]──────────────────→ Inaccurate (retained)
  │
  ├─[freshness deadline passes]─────────────────────────→ Outdated
  │   │
  │   ├─[re-verified after refresh]──────────────────────→ Accurate
  │   └─[refresh finds contradiction]──────────────────→ Conflicted → Inaccurate
  │
  └─[superseded by new version]─────────────────────────→ Archived
```

### 6.2 Verification Queue

A verification queue (`verification_queue.omk`) holds all units in `PendingUserReview` or `PendingAutomaticCorroboration` state. The queue is processed in priority order:

- Priority 1: Units whose `accuracy` is `Conflicted` (resolution needed)
- Priority 2: Units whose `final_score` is in the 30–49 range and have been used in at least one response
- Priority 3: Units in `PendingAutomaticCorroboration` with high provenance scores (likely to auto-resolve)
- Priority 4: All other pending units in age order

### 6.3 Multi-Source Corroboration Protocol

When a unit enters `PendingAutomaticCorroboration`:

1. The system records the unit's subject + predicate + object as a "verification target."
2. All future incoming facts with matching subject + predicate are evaluated against the target.
3. Each confirming fact from a distinct domain (different origin URL or source type) increments the corroboration count.
4. Once the count meets the configured threshold (default 2 from different domains), the unit transitions to `Accurate` if `allow_auto_promotion` is true, otherwise to `PendingUserReview`.
5. Each contradicting fact immediately transitions the unit to `Conflicted`.

### 6.4 User Interaction Commands for Verification

The REPL and GUI both support:

```
verify <fact-id>           → present the fact and ask for confirm/reject/unsure
verify-topic <subject>     → present all unverified facts about a subject
conflicts                  → show all conflicted facts requiring resolution
resolve <fact-id> keep     → mark this version as accurate, archive the conflicting version
resolve <fact-id> discard  → mark this version as inaccurate, mark conflicting version as accurate
confirm <topic>            → heuristically confirm all unverified Unverified facts about topic
```

---

## 7. Knowledge CRUD Tracking, Paging, and Indexing

### 7.1 CRUD Audit Log

Every operation on every `InformationUnit` is recorded in a separate immutable CRUD audit log (`crud_audit.omx`, same page format as experience log). This is distinct from the `history` field inside the unit itself, which records semantic changes. The CRUD log records storage-level events:

```omni
struct CrudEvent:
    event_id:        u64
    unit_id:         u64
    operation:       CrudOperation
    actor:           String         # Module, capability, or user session ID
    timestamp:       Timestamp
    page_before:     Option<u32>
    page_after:      Option<u32>
    byte_size:       u32
    checksum:        [u8; 8]        # BLAKE3 first 8 bytes of serialized unit


enum CrudOperation:
    Created
    Read(ReadPurpose)              # Reads are audited for sensitive units
    Updated(UpdateReason)
    Archived
    Restored
    Superseded(u64)                # Superseded by which unit
    ConflictFlagged(u64)
    ConflictResolved
    VerificationStatusChanged
    AccuracyStatusChanged
    PageMigrated(u32, u32)         # Source page, destination page during compaction
    Exported
    Imported


enum ReadPurpose:
    QueryResponse
    ReasoningInput
    ConflictCheck
    VerificationCheck
    Export
    Diagnostic
```

### 7.2 Index Architecture

The knowledge store maintains five index files:

**Primary index (`index.omk`):** Maps unit_id → (page_id, page_offset). Implemented as a B-tree with branching factor 64, stored as a fixed set of .omk pages. Each leaf node holds 256 entries. Supports O(log N) lookup by unit_id.

**Subject index (`idx_subject.omk`):** Maps subject string → list of unit_ids. Implemented as a hash-based bucket index with open addressing. Each bucket holds up to 16 entries; overflow chains to an extended bucket. Supports O(1) average lookup for subject queries.

**Keyword inverted index (`idx_words.omk`):** Maps word token → posting list of unit_ids. The posting list is delta-encoded (each ID stored as delta from the previous). This is the full-text search index. Words are normalized: lowercase, stop words removed, stemmed using a simple suffix-stripping stemmer (no external NLP required). Supports O(K) lookup where K is the number of matching units for a term.

**Accuracy index (`idx_accuracy.omk`):** Maps AccuracyStatus → bitmap of unit_ids. Stored as a Roaring Bitmap for efficient set operations. Supports O(1) count and O(N) enumeration by status.

**Temporal index (`idx_time.omk`):** Maps time window → sorted list of unit_ids. Implemented as a skip list ordered by `acquired_at`. Supports O(log N) range queries.

### 7.3 Memory Management

The knowledge store uses a tiered memory model to prevent large stores from exhausting RAM:

**Hot tier (in-memory):** The most recently accessed 1000 units and the full index structures. All units accessed in the current session are retained here. Estimated memory: 2–20 MB depending on unit sizes.

**Warm tier (memory-mapped pages):** The current active write page and the 5 most recently read pages are memory-mapped. The OS manages physical memory backing and can page them out under pressure. Estimated memory: 0–40 MB physical depending on OS paging.

**Cold tier (on-disk):** All sealed pages are on disk. Units in cold tier require a page read on access. Page reads are async; the query system returns a Future that resolves when the page is available.

The memory allocator for in-memory units uses a **slab allocator** with fixed-size buckets (64B, 256B, 1KB, 4KB, 16KB) to minimize fragmentation from units of varying size. Units larger than 16KB (very long content strings) are allocated directly from the OS heap. The slab allocator maintains per-bucket freelists with O(1) allocation and deallocation. Each slab pre-allocates a contiguous 2MB arena from the OS and subdivides it into fixed-size slots, eliminating per-allocation system call overhead.

An additional **arena allocator** (bump allocator) is used for short-lived allocations during cognitive layer processing: AST nodes during OQL parsing, RETE network intermediate match sets, and reasoning chain working memory. The arena is reset in O(1) at the end of each query execution cycle, avoiding per-node deallocation overhead entirely. This separation — slab for long-lived Knowledge Store units, arena for ephemeral reasoning temporaries — is critical for meeting the L0–L4 latency budgets defined in §101.

### 7.4 Page Compaction Schedule

A background compaction task runs when:
- A sealed page has more than 60% of its units marked as ARCHIVED or SUPERSEDED, or
- The store has more than 1000 sealed pages, or
- The user explicitly requests compaction via `compact-knowledge`.

Compaction is non-blocking: it reads old pages, writes new compacted pages, updates the index atomically (by replacing the index file), and then moves old pages to the archive directory. Live queries continue against the old index until the atomic swap completes.

**Write-Ahead Log (WAL) for crash safety:** Before beginning any compaction or page write operation, the Knowledge Store writes the intended operation to a WAL file (`wal.omk`). The WAL uses the standard write-ahead protocol: (1) log the operation, (2) fsync the WAL, (3) apply the operation, (4) checkpoint. On recovery after a crash, the WAL is replayed in an analysis→redo→undo sequence: committed operations are replayed, uncommitted operations are rolled back, and the store is restored to its last consistent state. WAL entries are compressed with OmniPack (§2) and each entry includes a BLAKE3 checksum for corruption detection.

**Recovery time objective:** WAL replay for a store with up to 1M units must complete in under 10 seconds on the reference hardware (Appendix R).

---

## 8. Web Learning Staging and Verification Pipeline

### 8.1 Pipeline Overview

The web learning pipeline is a six-stage sequential processor with explicit handoffs between stages. Each stage produces a typed output that becomes the input to the next. The pipeline can be invoked in two modes: **Active** (triggered by a knowledge gap during query processing) and **Passive** (triggered by idle-state scheduling).

```
Stage 1: Gap Detection or Schedule Trigger
   ↓
Stage 2: Source Selection and Safety Check
   ↓  
Stage 3: Fetch and Raw Content Acquisition
   ↓
Stage 4: Extraction and Structuring
   ↓
Stage 5: Staging and Conflict Pre-Check
   ↓
Stage 6: Queue for Verification and Consolidation
```

### 8.2 Stage 1 — Gap Detection

A knowledge gap is detected when a query response would require a fact with `final_score < 30` or no matching fact exists. Gap detection produces a `KnowledgeGap` record:

```omni
struct KnowledgeGap:
    gap_id:          u64
    query_subject:   String
    query_predicate: Option<String>
    detected_at:     Timestamp
    triggering_query: String
    urgency:         GapUrgency          # Immediate | Background | Opportunistic
    policy_cleared:  bool
    search_terms:    Vec<String>         # Generated by the query processor for web search
```

Gap detection consults the safety policy before proceeding: if the subject is on a restricted domain list (configurable), the gap is marked as policy-blocked and no web learning occurs.

### 8.3 Stage 2 — Source Selection

The source selector constructs a prioritized list of candidate sources:

1. Domains in the trusted source registry (user-configured).
2. Domains in the general allowed list (framework default: major encyclopedias, government sites, scientific publishers).
3. General web search results (DuckDuckGo or user-configured search API), filtered through the domain blocklist.

Each candidate source is assigned an initial trust level from the domain registry. Sources below `TrustLevel::Low` are discarded before fetching. Rate limiting is applied per domain (configurable, default 10 requests per domain per hour) and globally (configurable, default 60 requests per hour across all domains).

### 8.4 Stage 3 — Fetch and Raw Content Acquisition

```omni
struct RawAcquisition:
    source_url:      String
    final_url:       String            # After redirect resolution
    redirect_count:  u8
    http_status:     u16
    content_type:    String
    raw_bytes:       Vec<u8>
    fetched_at:      Timestamp
    response_hash:   [u8; 32]          # BLAKE3 of raw_bytes
    headers:         HashMap<String, String>
    fetch_latency_ms: u32
```

Fetching is async with a 10-second timeout. Fetches exceeding 5 MB are truncated after 5 MB with a flag set. Redirects are followed up to 5 hops. TLS verification is mandatory — unverified TLS does not proceed.

The fetcher respects `robots.txt` by maintaining a per-domain robots cache. Domains that disallow crawling are not fetched even if they are in the allowed list.

### 8.5 Stage 4 — Extraction and Structuring

The extractor converts raw HTML/JSON/plain-text content into candidate `InformationUnit` records:

**Step 4a — Format detection:** Determine content type from `Content-Type` header and content sniffing. Support: HTML, plain text, JSON, Markdown.

**Step 4b — Text extraction:** For HTML, extract the main content text using a readability heuristic (similar to Mozilla's Readability algorithm): identify the element with the highest text-to-HTML ratio in a block of 200+ words, extract its text nodes, discard navigation, headers, footers.

**Step 4c — Sentence segmentation:** Split extracted text into sentences using a rule-based sentence boundary detector (punctuation + capitalization heuristics, no NLP model required).

**Step 4d — Entity and relation extraction:** For each sentence, attempt to extract subject-predicate-object triples using a rule-based shallow parser. The parser recognizes common English clause patterns: `<NP> <VP> <NP>`, `<NP> is|are <NP>`, `<NP> was|were <NP>`, `<NP> has|have <NP>`. This is not a full NLP pipeline — it is a deterministic rule-based extractor that errs on the side of leaving predicate/object as None rather than guessing incorrectly.

**Step 4e — Candidate unit construction:** Each extracted sentence becomes one candidate `InformationUnit` in `DRAFT` status with `AccuracyStatus::Unverified` and confidence computed from the source's trust level.

### 8.6 Stage 5 — Staging and Conflict Pre-Check

Each candidate unit is run through the conflict pre-checker before being committed to the staging area:

1. **Dedup check:** If `content_hash` matches any existing unit, discard the candidate (no new information).
2. **Subject overlap check:** Find all existing units with the same subject. For each, compute semantic similarity between their content and the candidate's content using keyword overlap. If similarity > 0.85, classify as an update candidate.
3. **Contradiction pre-check:** If an update candidate has the same subject and predicate but clearly opposite polarity or different object, flag as `potential_conflict` and route to the conflict resolution queue rather than the main staging area.
4. **Commit to staging:** Surviving candidates are written to the staging area (`staging/pending.omk`) as DRAFT units.

```omni
struct StagingEntry:
    candidate:       InformationUnit     # In DRAFT status
    conflict_with:   Option<u64>         # ID of existing unit if potential conflict
    update_of:       Option<u64>         # ID of existing unit this updates
    staged_at:       Timestamp
    pipeline_run_id: u64
    reviewer_notes:  String
```

### 8.7 Stage 6 — Verification Queue Entry

Staged units are added to the verification queue with a priority determined by:

- The urgency of the original gap (Immediate gaps get Priority 1)
- The initial confidence score of the candidate
- Whether the user's query is still waiting for this information

The verification queue processor runs on a configurable schedule (default: every 30 seconds for Priority 1, every 5 minutes for Priority 2+). It processes entries by attempting automatic corroboration first: for each candidate, it searches the existing knowledge store for agreeing facts from different sources. If the threshold is met, the unit is committed. Otherwise it is presented to the user at the next interaction opportunity.

### 8.8 Passive Learning (Dream-State)

Passive learning activates when:
- The system has been idle for more than the `idle_threshold` (default 10 minutes)
- The current session has pending verification queue items or a `background_learning_topic_list` is configured
- Safety policy allows passive learning

During passive learning, the pipeline runs at reduced priority with a stricter rate limit (default 6 requests per domain per hour instead of 10). The learning queue is populated from three sources: (1) topics the user has asked about but where existing knowledge had `final_score < 60`, (2) topics in the user-configured background learning list, and (3) related topics discovered through the knowledge graph during previous active learning cycles.

Passive learning never presents results to the user directly. All acquired facts go through the full staging and verification pipeline. The user is notified at next interaction with a summary: "During background learning, I acquired N new candidate facts about X topics. N are ready for your review."

---

## 9. Phase H2 Brain — Complete Cognitive Implementation

### 9.1 Architecture Overview

HELIOS's brain is a five-layer cognitive system inspired by two established cognitive architectures — **SOAR** and **ACT-R** — but adapted to the non-neural, evidence-driven, deterministic constraints of the project. The five layers correspond to well-defined information processing speeds:

| Layer | Name | Latency Target | Analogous to |
|-------|------|----------------|-------------|
| L0 | Reflex | < 1ms | SOAR automatic rules / ACT-R procedural memory |
| L1 | Pattern Match | < 10ms | SOAR elaboration phase / ACT-R declarative retrieval |
| L2 | Inference | < 100ms | SOAR deliberate reasoning / ACT-R production compilation |
| L3 | Deep Reasoning | < 500ms | SOAR subgoaling / ACT-R problem solving |
| L4 | Background Cognition | Async | SOAR chunking / ACT-R learning |

Processing a user query traverses layers from L0 upward, stopping as soon as a satisfactory answer is found. A "satisfactory" answer is one with `final_score >= 70` and no unresolved conflicts. If L3 is reached without a satisfactory answer, the query either returns a low-confidence provisional answer or triggers a web learning cycle.

### 9.2 Working Memory

Working Memory (WM) holds the context for the current cognitive cycle. It is cleared between unrelated queries but can be explicitly maintained across turns in a session.

```omni
struct WorkingMemory:
    cycle_id:           u64
    current_goal:       Goal
    active_facts:       Vec<u64>          # InformationUnit IDs loaded into context
    intermediate_results: Vec<InferredFact>
    active_rules:       Vec<RuleId>        # Rules being tested in current cycle
    pending_subgoals:   Vec<Goal>
    conflict_set:       Vec<ActivatedRule> # Rules that matched in this cycle
    focus_stack:        Vec<String>        # Current attention focus — subject stack
    cycle_start:        Timestamp
    max_inference_depth: u8               # Prevents infinite loops


struct Goal:
    goal_type:   GoalType
    subject:     String
    predicate:   Option<String>
    constraints: Vec<Constraint>
    priority:    u8


enum GoalType:
    Answer(String)          # Answer a question
    Verify(u64)             # Verify a specific fact
    Explain(u64)            # Explain a fact's provenance and reasoning
    Resolve(u64, u64)       # Resolve a conflict between two units
    Infer(String)           # Try to infer a new fact
    Plan(String)            # Produce a plan of actions
```

### 9.3 RETE Network — Forward Chaining Implementation

The RETE network is the core of L1 and L2 reasoning. It maintains a compiled network of nodes that matches production rules against facts in Working Memory.

**Node types:**

```omni
enum ReteNode:
    RootNode                              # Entry point for all fact assertions
    AlphaNode(AlphaTest)                  # Single-fact filter: tests one attribute of one WME
    AlphaMemory(Vec<WorkingMemoryElement>) # Stores WMEs passing an alpha test
    BetaJoinNode(JoinSpec)               # Joins two partial matches
    BetaMemory(Vec<PartialMatch>)        # Stores partial matches
    NegativeNode(JoinSpec)               # Negative condition: matches absence of facts
    ProductionNode(ProductionId)          # Terminal: rule has fully matched


struct AlphaTest:
    field:      WmeField    # Subject | Predicate | Object | Confidence | Accuracy | ...
    operator:   TestOp      # Equal | NotEqual | Greater | Less | Contains | Matches
    value:      TestValue


struct JoinSpec:
    left_memory:  NodeId
    right_memory: NodeId
    binding_map:  Vec<(Variable, WmeField)>   # Unification bindings


struct WorkingMemoryElement:
    unit_id:   u64
    subject:   String
    predicate: Option<String>
    object:    Option<String>
    accuracy:  AccuracyStatus
    confidence: u8
    timestamp:  Timestamp


struct PartialMatch:
    tokens:    Vec<WorkingMemoryElement>
    bindings:  HashMap<Variable, String>
    timestamp: Timestamp
```

**RETE cycle — match-resolve-act:**

1. **Assert:** When a new `InformationUnit` is committed, it is converted to a `WorkingMemoryElement` and injected into the root node.

2. **Alpha propagation:** Each alpha node tests the WME against its condition. Passing WMEs are added to the alpha memory and propagated to connected beta join nodes.

3. **Beta propagation:** Each beta join node tests all existing partial matches in its left memory against the new WME from the right memory. Successful joins create new partial matches added to the beta memory and propagated further.

4. **Production activation:** When a production node receives a complete match (all conditions satisfied), it creates an `ActivatedRule` and adds it to the conflict set.

5. **Conflict resolution:** The conflict resolver orders the conflict set by:
   - Rule salience (explicitly declared priority)
   - Recency (prefer rules triggered by the most recently asserted WME)
   - Specificity (prefer rules with more conditions)
   
6. **Fire:** The highest-priority activated rule fires. Its actions may: assert new facts (which re-enter step 1), modify existing facts, retract facts, or produce output. Firing continues until the conflict set is empty.

**Retraction handling:** When a fact is superseded or archived, it is retracted from WM. Retraction propagates backward through the RETE network: partial matches dependent on the retracted WME are removed, which may remove production activations from the conflict set. Truth maintenance is automatic.

**Incremental computation note:** The RETE network inherently operates as an incremental dataflow engine — when a single fact is asserted or retracted, only the affected alpha/beta path segments are re-evaluated, not the entire rule base. This mirrors the differential dataflow paradigm (Materialize/Timely Dataflow, 2025) where computations update outputs based on deltas rather than full recomputation. For HELIOS, this means asserting one new fact into a 10,000-rule RETE network has cost proportional to the number of rules that reference the changed fact’s attributes, not proportional to the total rule count — critical for meeting the L1 < 10ms latency target.

### 9.4 Backward Chaining — Resolution-Based

Backward chaining is used when the system has a specific goal (e.g., "prove or disprove that X is true") rather than a data-driven inference. The algorithm is resolution-based, operating over the production rule set:

```
backward_chain(goal: Goal) -> Option<InferredFact>:
    // Base case: goal directly satisfies from existing accurate knowledge
    if let Some(unit) = knowledge_store.find_accurate(goal):
        return Some(unit.to_inferred())
    
    // Inductive step: find rules whose consequent matches the goal
    matching_rules = rule_base.find_rules_with_consequent(goal)
    
    for rule in matching_rules:
        // Try to prove all antecedents of this rule
        all_proved = true
        sub_bindings = {}
        
        for antecedent in rule.antecedents:
            bound_antecedent = apply_bindings(antecedent, sub_bindings)
            if let Some(proof) = backward_chain(bound_antecedent):
                sub_bindings.merge(proof.bindings)
            else:
                all_proved = false
                break
        
        if all_proved:
            result = instantiate_rule_consequent(rule, sub_bindings)
            result.confidence = compute_derived_confidence(rule, sub_bindings)
            return Some(result)
    
    None  // Goal could not be proved
```

The maximum recursion depth is `WorkingMemory.max_inference_depth` (default 8) to prevent infinite loops from cyclic rule dependencies.

### 9.5 Knowledge Graph Operations

The knowledge graph is not a separate database — it is the set of `RelationLink` edges stored inside `InformationUnit` records, with the graph algorithms operating over the knowledge store's subject index.

**BFS (Breadth-First Search):** Used for "what is connected to X within N hops" queries. Standard queue-based BFS over the relation graph, with cycle detection using a visited-set.

**Dijkstra's Shortest Path:** Used for "what is the shortest reasoning path between X and Y" queries. Edge weights are computed as `(1 - strength/100)` so high-strength relations are "shorter." The path is used to construct an explanation trace.

**Topological Sort (Kahn's Algorithm):** Used for dependency ordering when building inference chains. When HELIOS needs to prove a conclusion from multiple prerequisite facts, it topologically orders the facts to ensure prerequisites are established before dependents.

**Cycle Detection (DFS-based Tarjan's SCC):** Used before adding a new relation link to prevent the knowledge graph from developing circular dependencies that would trap the backward chainer.

**Subgraph Pattern Matching (VF2 Algorithm):** Used for "find instances of this relational pattern in the knowledge graph" queries. VF2 is an established graph isomorphism algorithm. Implementation:

```
vf2_match(pattern_graph: Graph, knowledge_graph: Graph) -> Vec<Mapping>:
    state = VF2State::new(pattern_graph, knowledge_graph)
    results = []
    vf2_recurse(state, &mut results)
    results

vf2_recurse(state: VF2State, results: &mut Vec<Mapping>):
    if state.is_complete():
        results.push(state.current_mapping.clone())
        return
    
    for (p_node, k_node) in state.candidate_pairs():
        if state.is_feasible(p_node, k_node):
            state.add_pair(p_node, k_node)
            vf2_recurse(state, results)
            state.remove_pair(p_node, k_node)
```

### 9.6 Memory Architecture — Four Tiers

Inspired by ACT-R's declarative/procedural split and SOAR's semantic/episodic/procedural split, HELIOS implements four tiers:

**Semantic Memory (Long-Term Knowledge):** The full `InformationUnit` knowledge store. Contains general world knowledge, domain facts, and accumulated learning. Accessed through the knowledge store query API. Never expires unless freshness deadline passes.

**Episodic Memory (Session + Experience History):** The experience log. Contains the record of everything that has happened — queries, responses, learning events, capability invocations. Queryable by time range, session, and content. Supports "what did I tell you about X last Tuesday" queries. Accessed through the experience log query API.

**Procedural Memory (Rules and Capabilities):** The production rule base and the capability registry. Contains compiled RETE network rules and capability handlers. Does not store facts — stores how to act on facts. Loaded at startup and updated only through governed capability/rule addition proposals.

**Working Memory (Transient Computation):** The current cognitive cycle's active context (see Section 9.2). Cleared after each query unless explicitly persisted. Limited to 64 active WMEs by default to keep reasoning within time budgets.

### 9.7 Cognitive Cortex Pipeline

The cognitive cortex (`brain/cognitive_cortex.omni`) is the top-level orchestrator that receives a query and coordinates the five layers:

```
query arrives
    │
    ├─ L0: Check reflexes (exact match in WM, cached recent answer)
    │       → if hit: return immediately with source/confidence trace
    │
    ├─ L1: RETE forward chain over current WM + accurate knowledge
    │       → if satisfactory answer: return with reasoning trace
    │
    ├─ L2: Backward chain from query goal over rule base
    │       → if satisfactory answer: return with proof tree
    │
    ├─ L3: Expand WM with moderate-confidence knowledge, re-run L1+L2
    │      → if satisfactory answer: return with confidence-annotated trace
    │
    ├─ L4 trigger (async): if answer was low-confidence or not found:
    │      → trigger active web learning for gap filling
    │      → return provisional answer now, update when learning resolves
    │
    └─ compose response:
            ← answer text
            ← supporting fact IDs
            ← confidence breakdown
            ← reasoning trace (which rules fired, in what order)
            ← freshness status of each supporting fact
            ← contradiction warnings if any
            ← provenance for each source fact
```

### 9.8 Cognitive Inference — Abductive and Analogical

Beyond forward and backward chaining, `brain/cognitive_inference.omni` implements two additional reasoning modes:

**Abductive reasoning (inference to best explanation):** Given an observation O, find the most plausible explanation E such that E → O would hold. Implementation: the system searches for all facts in the knowledge store that have "causes" or "leads_to" relations pointing toward the subject of O. It ranks these candidates by their confidence score and the strength of the causal link.

**Analogical reasoning (structure mapping):** Given a known relationship A→B, infer a probable relationship C→D when A is structurally analogous to C. Implementation: the VF2 subgraph matcher finds known relational patterns (A→B) in the knowledge graph. When the system encounters C which shares structural properties with A (same domain, similar predicates), it proposes the analogous relationship C→D as an inference candidate with a derivation score adjusted for the analogical distance.

### 9.9 Deep Thought Layer

`brain/deep_thought/` contains modules for computationally expensive reasoning that cannot meet the 500ms L3 deadline. These run asynchronously and return results through the experience log:

- **Consistency checker:** Scans the entire knowledge store for latent contradictions (pairs of facts that should conflict but haven't been explicitly linked as contradictions). Runs nightly.
- **Confidence decay processor:** Recalculates confidence scores for all time-sensitive facts whose freshness has changed. Runs hourly.
- **Pattern learner:** Analyzes the experience log to identify recurring query patterns and proposes new production rules to the user for approval.
- **Knowledge graph enricher:** Attempts to discover implicit relation links between existing facts through multi-hop traversal. Proposes new `RelationLink` additions to the user for approval.

---

## 10. Plugin System

### 10.1 What a Plugin Is

A plugin in HELIOS is a compiled Omni module that extends the system's capabilities at runtime without requiring a restart or rebuild of the core framework. Plugins are first-class citizens, not afterthoughts — they use the same capability, safety, and audit infrastructure as the core system. Every plugin is sandboxed, versioned, signed, and auditable.

A plugin can be thought of as a "certified extension" — it declares what it needs (permissions), what it can do (typed functions), how it should be isolated (sandbox profile), and what to do if something goes wrong (rollback hook). The user approves its installation, can audit everything it does, and can uninstall it with a guaranteed rollback.

### 10.2 What Plugins Can Do

Plugins extend HELIOS in five capability categories:

**Data source plugins:** Provide new knowledge acquisition channels. Examples: a plugin that queries a local SQL database and translates results to InformationUnits, a plugin that monitors a directory for new documents and ingests them, a plugin that connects to an authenticated API (corporate knowledge base, medical database) the user has credentials for.

**Reasoning plugins:** Add new inference strategies. Examples: a mathematical theorem prover plugin that handles numeric reasoning, a domain-specific expert system plugin for medical or legal reasoning, a temporal logic plugin for reasoning about events over time.

**Output formatter plugins:** Transform HELIOS responses into specialized output formats. Examples: a LaTeX formatter for academic citation output, a code generator that produces Omni or other language code from knowledge, a structured report generator.

**Capability plugins:** Add new HELIOS capabilities (like the built-in capabilities for file operations, search, etc.) for new domains. Examples: a calendar integration plugin, an email plugin, a code execution plugin with sandboxed OVM execution.

**UI plugins:** Add new panels or views to the HELIOS GUI. Examples: a knowledge graph visualization panel, a timeline view of the experience log, a domain-specific dashboard.

### 10.3 Plugin Manifest — Complete Schema

```omni
struct PluginManifest:
    # Identity
    id:             String              # Reverse-domain: "com.example.sql-connector"
    name:           String              # Display name
    version:        SemanticVersion     # e.g. "1.2.3"
    description:    String
    author:         String
    author_url:     Option<String>
    license:        String              # SPDX identifier

    # Entry point
    entry_module:   String              # Omni module path, e.g. "sql_connector::PluginEntry"
    min_helios:     SemanticVersion     # Minimum HELIOS version required

    # Permissions — must be declared; undeclared permissions are denied at call time
    permissions:    Vec<Permission>

    # Sandbox profile
    sandbox:        SandboxProfile

    # Function exports — typed signatures that HELIOS uses to call into the plugin
    exports:        Vec<PluginFunction>

    # Rollback
    rollback_hook:  Option<String>      # Module function path for cleanup on uninstall

    # Provenance
    signed_by:      Option<[u8; 64]>    # Ed25519 signature of manifest hash by author
    content_hash:   [u8; 32]            # BLAKE3 of compiled plugin binary


enum Permission:
    ReadKnowledgeStore              # Read any InformationUnit
    WriteKnowledgeStore             # Add new InformationUnits (always Unverified)
    ReadExperienceLog               # Access experience records
    ExecuteCapability(String)       # Invoke a named core capability
    NetworkAccess(Vec<String>)      # Outbound HTTP/HTTPS to listed domains only
    FileRead(Vec<String>)           # Read files matching listed path patterns
    FileWrite(Vec<String>)          # Write files matching listed path patterns
    SpawnProcess(Vec<String>)       # Spawn processes matching listed executable paths
    AccessUserInterface             # Add elements to the GUI
    ReadSelfModel                   # Read the SelfModel
    ProposeSelfModification         # Propose (but not apply) self-modification
    AccessBiometrics                # Request biometric verification


struct SandboxProfile:
    memory_limit_mb:    u32         # Max heap memory
    cpu_time_limit_ms:  u32         # Max CPU time per invocation
    wall_time_limit_ms: u32         # Max wall-clock time per invocation
    thread_limit:       u8          # Max threads
    network_policy:     NetworkPolicy
    file_policy:        FilePolicy
    ipc_allowed:        bool        # Can the plugin communicate with other plugins?


enum NetworkPolicy:
    Denied                          # No network access
    AllowListOnly(Vec<String>)      # Specific domains only
    AllowAll                        # Unrestricted (requires explicit approval)


struct PluginFunction:
    name:           String
    description:    String
    parameters:     Vec<TypedParam>
    return_type:    String          # Omni type name
    requires_approval: bool         # Must user approve each invocation?
    audit_level:    AuditLevel      # None | Summary | Full
```

### 10.4 Plugin IPC Architecture

Plugins run in isolated processes (one OVM instance per plugin). Communication with the HELIOS host process uses a typed message-passing protocol over anonymous pipes. The host and plugin share a schema that both compile against; schema mismatches fail at plugin load time before any messages are exchanged.

```
Plugin Process                    HELIOS Host Process
     │                                    │
     │  ← InvocationRequest             │
     │  (function name, typed args)      │
     │                                    │
     │  [plugin executes in sandbox]     │
     │                                    │
     │  → InvocationResponse            │
     │  (typed result or Error)           │
     │                                    │
     │  → AuditEvent                    │
     │  (what the plugin did)            │
```

### 10.5 Plugin Installation and Approval Flow

1. User provides plugin package (`.ompkg` file — a ZIP containing the compiled `.ovc` bytecode and `manifest.json`).
2. HELIOS validates the content hash and optional signature.
3. HELIOS presents the manifest to the user: all declared permissions are listed in plain language.
4. User approves or rejects each permission individually (can deny specific permissions; plugin may fail if required permissions are denied).
5. HELIOS records the approval in the audit log and writes the plugin to `plugins/<plugin-id>/`.
6. HELIOS loads the plugin OVM instance and runs the entry point's `initialize()` function.
7. Plugin is available from this point forward.

### 10.6 Plugin Rollback

On uninstall or on detected failure:

1. `rollback_hook()` is called in the plugin process.
2. Any `InformationUnit` records with `SourceType::PluginProvided` attributed to this plugin are flagged `ARCHIVED` (not deleted).
3. The plugin's OVM instance is terminated.
4. The plugin directory is moved to `plugins/archived/<plugin-id>-<timestamp>/`.
5. The audit log records the uninstall event, actor, timestamp, and reason.

---

## 11. Experience Log — Complete Design

### 11.1 What the Experience Log Is For

The experience log is HELIOS's episodic memory — the complete, exact, and ordered record of everything that has happened during the life of the system. It serves four distinct functions that distinguish it from the knowledge store:

**Episodic recall:** "What did you tell me about X on Tuesday" — the experience log provides exact session-level recall of what was said, in what order, with what confidence, in response to what query.

**Behavioral audit:** Every capability invocation, plugin call, self-modification, and web learning cycle is logged. The log is the answer to "what did HELIOS do and why."

**Provenance chain and Trust scoring:** Addressing 2025 AI reliability standards, the log establishes an immutable provenance chain for all knowledge. By tracking `AcquisitionHop` sequences back to the source, the system calculates dynamic trust scores and defends against the algorithmic misinformation cycle.

**Feedback loop:** User feedback events (corrections, approvals, rejections) in the experience log feed back into the knowledge store's verification pipeline. A user saying "that was wrong" is an experience event that triggers a knowledge accuracy re-evaluation.

**Learning signal:** The pattern learner in the Deep Thought layer analyzes the experience log to propose new production rules based on recurring reasoning patterns (see Section 9.9).

### 11.2 Experience Record Taxonomy

Every event that occurs in HELIOS produces exactly one experience record. The taxonomy:

```omni
enum ExperienceRecord:
    # Interaction
    QueryReceived(QueryRecord)
    ResponseGenerated(ResponseRecord)
    ResponseRated(RatingRecord)              # User gave feedback on a response
    SessionStarted(SessionRecord)
    SessionEnded(SessionRecord)

    # Knowledge
    KnowledgeAdded(KnowledgeEventRecord)     # Unit committed to store
    KnowledgeUpdated(KnowledgeEventRecord)   # Unit superseded
    KnowledgeConflictDetected(ConflictEventRecord)
    KnowledgeConflictResolved(ConflictEventRecord)
    KnowledgeVerified(VerificationEventRecord)
    KnowledgeRejected(VerificationEventRecord)
    KnowledgeArchived(KnowledgeEventRecord)

    # Reasoning
    InferencePerformed(InferenceRecord)      # A forward/backward chain produced a result
    ReasoningFailed(ReasoningFailRecord)     # Reasoning couldn't answer the query
    KnowledgeGapDetected(GapRecord)

    # Web Learning
    WebLearningTriggered(LearningRecord)
    WebFetchPerformed(FetchRecord)
    StagingEntryCreated(StagingRecord)
    StagingEntryPromoted(StagingRecord)
    StagingEntryRejected(StagingRecord)

    # Capabilities
    CapabilityInvoked(CapabilityRecord)
    CapabilitySucceeded(CapabilityRecord)
    CapabilityFailed(CapabilityRecord)
    CapabilityApprovalRequested(ApprovalRecord)
    CapabilityApprovalGranted(ApprovalRecord)
    CapabilityApprovalDenied(ApprovalRecord)

    # Plugins
    PluginInstalled(PluginEventRecord)
    PluginUninstalled(PluginEventRecord)
    PluginInvoked(PluginInvocationRecord)
    PluginFailed(PluginEventRecord)

    # Governance
    SelfModificationProposed(ProposalRecord)
    SelfModificationApproved(ProposalRecord)
    SelfModificationApplied(ProposalRecord)
    SelfModificationRejected(ProposalRecord)
    SelfModificationRolledBack(ProposalRecord)

    # Safety
    SafetyCheckTriggered(SafetyRecord)
    SafetyCheckPassed(SafetyRecord)
    SafetyCheckFailed(SafetyRecord)
    IdentityVerificationRequested(IdentityRecord)
    IdentityVerificationPassed(IdentityRecord)
    IdentityVerificationFailed(IdentityRecord)

    # System
    SystemStarted(SystemRecord)
    SystemStopped(SystemRecord)
    CheckpointCreated(CheckpointRecord)
    CheckpointRestored(CheckpointRecord)
    ConfigurationChanged(ConfigRecord)
    ErrorOccurred(ErrorRecord)
```

### 11.3 Retention Policy

Experience records are never deleted. They may be:

- **Active:** Retained in the active log pages, queryable in full detail.
- **Archived:** Moved to compressed archive pages after the session retention window (default 90 days for detailed records).
- **Summarized:** For very old records (default >1 year), the raw records may be replaced with a summary record that captures the high-level event without verbatim content, controlled by the `SummarizationPolicy`.

The user can override retention at any category level. Governance records (approvals, modifications) are permanently retained regardless of policy.

### 11.4 Experience Log as Feedback Loop

The feedback loop runs as follows:

1. User rates a response as incorrect (`ResponseRated` with `FeedbackType::Incorrect`).
2. The experience processor extracts the unit IDs cited in the `ResponseRecord` for that response.
3. Each cited unit is queued for user verification with high priority.
4. If the user subsequently confirms the inaccuracy, each cited unit transitions to `AccuracyStatus::Inaccurate`.
5. All facts inferred from those units through forward chaining are recursively marked as requiring re-evaluation (confidence recalculated, status set to Conflicted or Unverified depending on depth).

---

## 12. GUI Implementation Plan

### 12.1 Architecture — Windows First

The HELIOS GUI is implemented as a native Windows desktop application in Phase H7. Cross-platform support (Linux, then macOS) follows in Phase H8+. A new OS installation package is the final target.

The Windows implementation uses **WinUI 3** (Windows App SDK) as the UI framework. WinUI 3 is the current Microsoft-recommended native UI stack, provides access to all modern Windows controls, integrates with the Windows notification system and system tray, and produces applications that look and behave as first-class Windows citizens. It is not a web wrapper or an Electron application — it is native compiled code.

The HELIOS GUI process is distinct from the HELIOS service process. They communicate through a named pipe IPC channel (see Section 12.4). This separation means the GUI can crash without affecting the HELIOS service, and the service can run headlessly without the GUI.

### 12.2 Main Window Layout

The main window is 1400×900 minimum, resizable, with a persistent left sidebar and a main content area that switches between panels.

**Left sidebar (fixed 280px):**
- HELIOS logo and version badge
- Navigation icons for each panel (active panel highlighted)
- Confidence posture mini-indicator (green/amber/red badge)
- Pending verification count badge
- Pending conflict count badge
- Background learning status indicator (pulsing when active)

**Main content area — five panels:**

**Panel 1: Conversation**
The primary interaction surface. Shows the full chronological exchange for the current session. Each HELIOS response is expandable to show:
- The answer text (top level)
- Confidence indicator (color-coded bar: green ≥70, amber 50–69, red <50)
- Supporting fact IDs with accuracy icons
- "Show reasoning" button that expands the full cognitive trace (see below)
- "Show sources" button that lists all source URLs/IDs
- Conflict warnings (if any supporting fact is Conflicted)

The input box is at the bottom. Supports multi-line input. Has a "learning mode" toggle that changes how facts entered in the input box are handled (direct knowledge entry vs. query).

**Panel 2: Cognitive Trace Viewer**
A real-time view of what HELIOS is doing. When a query is being processed, this panel shows:
- Which cognitive layer is currently active (L0–L4), animated
- Which RETE rules fired, in order, with the facts that triggered them
- For backward chaining: the goal tree with proved/failed branches
- For web learning: the fetch and staging pipeline progress
- The final confidence breakdown for the answer

The trace is retained for the last 50 queries and can be replayed.

**Panel 3: Knowledge Browser**
A browsable, searchable view of the knowledge store:
- Search bar with live results
- Filter panel: by accuracy status, confidence range, source type, date range, domain
- Results list: each unit shows subject, first 80 chars of content, accuracy badge, confidence bar
- Unit detail view: the full InformationUnit with all fields, history timeline, relation graph
- Verification action buttons: Confirm, Reject, Investigate

The relation graph for each unit is rendered as a force-directed mini-graph showing the unit and its directly related units with labeled edges.

**Panel 4: Learning Status**
Shows the web learning pipeline state:
- Active learning: currently pending gaps, estimated time to resolution
- Staging area: all DRAFT units waiting for verification, with review actions
- Background learning: queue topics, last run time, next scheduled run
- Domain statistics: how many facts from each source domain, trust level breakdown

**Panel 5: Settings and Governance**
Shows:
- ConfidencePolicy current values with sliders to adjust
- GovernanceProfile toggles
- Plugin list with enable/disable and uninstall
- Pending self-modification proposals with Approve/Reject
- Capability list and per-capability audit history
- Identity management (PIN change, lockout settings)
- Data management: checkpoint creation, store compaction trigger, export

### 12.3 Accessibility and Theming

The HELIOS native GUI framework enforces strict adherence to WCAG 2.2 accessibility standards, ensuring semantic ARIA roles, verified contrast ratios (minimum 4.5:1), and full keyboard navigation. Furthermore, dark mode is implemented as a foundational design token system — not an afterthought — aligning with 2025 native OS-level display baseline expectations.

### 12.4 Showing Framework Execution to the User

HELIOS is designed to be transparent. Everything that happens internally is visible on request. The execution visibility model has three tiers:

**Tier 1 — Summary (always shown):** Every response includes a one-line status bar: "Answered from 3 accurate facts at 82% confidence. [Show details]"

**Tier 2 — Response Detail (on click):** The confidence breakdown table, the list of supporting fact IDs with their accuracy, the list of contradictions if any.

**Tier 3 — Full Cognitive Trace (on click from Tier 2):** The full step-by-step trace from the Cognitive Trace Viewer Panel 2, showing exactly which layers activated, which rules fired, which facts were loaded into working memory, and which facts were excluded (with reasons).

The cognitive tracing system emits structured telemetry following conventions inspired by OpenTelemetry’s 2025 GenAI semantic conventions: each cognitive operation (L0 lookup, L1 RETE match, L2 backward chain, L3 analogy search) is a **span** with attributes for layer ID, duration, fact count, rule firings, confidence delta, and outcome. Spans form a hierarchical trace tree rooted at the user query, enabling latency attribution across cognitive layers and facilitating budget enforcement (§101).

The user can pin the Cognitive Trace panel to always be visible alongside the Conversation panel.

### 12.5 Service IPC Protocol

The GUI connects to the HELIOS service via a named pipe:

**Windows:** `\\.\pipe\HeliosService-<user-sid>`  
**Linux (future):** Unix socket at `$XDG_RUNTIME_DIR/helios.sock`

Wire format: 4-byte little-endian length prefix, followed by MessagePack-encoded message body. MessagePack is chosen over JSON for the service protocol because it is significantly faster to encode/decode (up to 4x) and more compact, while remaining self-describing. The IPC pipeline utilizes zero-copy MessagePack deserialization where possible, allowing the Omni runtime to read directly from the named pipe buffer without intermediate allocations, maximizing local service throughput.

**Message types:**

```omni
enum ServiceMessage:
    # GUI → Service
    Query(QueryRequest)
    LearnFact(LearnFactRequest)
    VerifyFact(VerifyFactRequest)
    InvokeCapability(CapabilityRequest)
    GetStatus(StatusRequest)
    GetKnowledge(KnowledgeQuery)
    GetExperiences(ExperienceQuery)
    GetPendingVerifications
    GetPendingConflicts
    ApproveProposal(u64)
    RejectProposal(u64)
    TriggerCompaction
    CreateCheckpoint(String)

    # Service → GUI
    QueryResponse(QueryResult)
    StatusResponse(SystemStatus)
    KnowledgeResponse(Vec<InformationUnit>)
    ExperiencesResponse(Vec<ExperienceRecord>)
    PendingVerificationsResponse(Vec<VerificationItem>)
    PendingConflictsResponse(Vec<ConflictItem>)
    CognitiveTraceUpdate(TraceEvent)     # Streamed during query processing
    LearningProgressUpdate(LearningStatus) # Streamed during web learning
    ProposalNotification(ProposalRecord)
    Error(ServiceError)
```

Streaming messages (`CognitiveTraceUpdate`, `LearningProgressUpdate`) are sent unsolicited by the service and have a `stream_id` field so the GUI can associate them with the relevant query.

### 12.5 Cross-Platform Strategy

**Linux (Phase H8+):**
Replace WinUI 3 with **GTK 4** for the native UI. GTK 4 supports the same panel layout and provides native Linux look and feel. Named pipe → Unix socket. The service IPC protocol is identical. All business logic in the service process is already platform-abstracted through `os-hooks/sal.omni`.

**macOS (Phase H9+):**
Replace WinUI 3 with **AppKit/SwiftUI** via Omni FFI bindings, or — more practically — use **Tauri** (a lightweight Rust+WebView shell) for the macOS GUI while keeping the core HELIOS service in Omni. The GUI panel content can be rendered as local HTML+JS communicating to the service over the Unix socket, providing a write-once GUI implementation for macOS and Linux.

**Unified approach for Linux and macOS:**
After the Windows native GUI is stable, implement a **Tauri-backed web GUI** as the cross-platform version. The web GUI uses the same service IPC protocol via WebSocket bridge. This avoids maintaining three native GUI codebases while preserving the native Windows experience.

### 12.6 New OS Target

The OS target is the final frontier. The intent is to create a bootable operating system that includes HELIOS as its primary interface and cognitive framework. This is a long-term milestone (well beyond Phase H9) but the architecture supports it through:

**`kernel/hot_swap.omni`:** The hot-swap kernel module provides the foundation for a microkernel architecture where HELIOS itself is the user-space supervisor. The hot-swap mechanism enables live updates to HELIOS components without rebooting.

**`os-hooks/sal.omni`:** The System Abstraction Layer is already designed as a hardware-facing abstraction. For a new OS, SAL's implementations point directly at hardware via device drivers rather than at an existing OS kernel.

**Omni as the OS language:** With Omni self-hosted and having a native linker, HELIOS OS can be built entirely in Omni — bootloader, kernel, drivers, and the HELIOS framework itself — with no dependency on any existing OS.

The OS product would boot to a HELIOS conversation interface as the primary UI, with a file manager, knowledge browser, and settings panel as the "shell." It installs on bare metal like a Linux distribution and ships as an ISO. The first platform target is x86-64. The OS includes:

- UEFI bootloader written in Omni
- Microkernel written in Omni (memory management, process scheduling, IPC)
- HELIOS framework as the primary user-space process
- WinUI-equivalent UI framework for the desktop
- Standard applications (file manager, text editor, terminal) implemented as HELIOS plugins

---

## 13. Self-Model Design

### 13.1 What the Self-Model Is

The self-model, as shown in `helios/self_model.omni`, is HELIOS's first-person description of itself. It is not metadata stored in the knowledge store — it is a live, derived view of the system's actual current state. The distinction matters: the knowledge store holds world knowledge; the self-model holds HELIOS's knowledge about HELIOS.

The self-model serves five functions:

**Answering "what are you?" questions:** When a user asks HELIOS what it can do, what it knows, or how confident it is in its own abilities, the answer comes from the self-model, not from a hardcoded string.

**Capability awareness:** Before attempting a capability invocation, the system checks the self-model to confirm the capability exists and is currently enabled.

**Confidence posture awareness:** The self-model's `KnowledgeSummary` tells the system what fraction of its knowledge is verified, unverified, conflicted, etc. This affects how HELIOS phrases its responses — a system with 80% unverified knowledge should caveat its answers differently than one with 80% accurate knowledge.

**Governance self-check:** Before processing a request that may require approval or identity verification, the system consults the `GovernanceProfile` in the self-model.

**Session identity:** The self-model's `identity_statement` and `operating_principles` are what HELIOS returns when asked to introduce itself or explain its values.

### 13.2 Live Derivation vs. Cached State

The `SelfModel.refresh()` method (as shown in the attached file) must be called:
- At startup
- After any capability is added or removed
- After any knowledge store compaction changes the counts
- After any `GovernanceProfile` change
- After any `ConfidencePolicy` change
- After any plugin is installed or removed

The `last_refresh_at` timestamp is used to detect stale self-model state. Any query that depends on the self-model and finds `last_refresh_at` older than 60 seconds triggers an automatic refresh before use.

### 13.3 Self-Model Extensions

The self-model in the attached file covers the essential fields. The following additions are needed for completeness:

```omni
# Additions to SelfModel struct:

    plugin_summaries:     Vec<String>           # Names of installed and active plugins
    pending_verifications: usize                # Count of facts awaiting verification
    pending_conflicts:    usize                 # Count of active conflicts
    active_capabilities:  Vec<String>           # Currently enabled capability names
    learning_status:      LearningStatus        # Current web learning state
    session_id:           u64                   # Current session ID
    uptime_seconds:       u64                   # Time since startup
    omni_version:         String                # Compiler/runtime version
    platform:             String                # OS and architecture


struct LearningStatus:
    passive_learning_active: bool
    pending_gap_count:       usize
    staging_area_count:      usize
    last_web_fetch_at:       Option<Timestamp>
    domains_learned_from:    usize
```

### 13.4 Self-Model in Responses

When the self-model is consulted during response composition, its state contributes to the response's framing. Examples:

- If `pending_conflicts > 0`: response includes "Note: I currently have N unresolved knowledge conflicts that may affect this answer."
- If `knowledge_summary.unverified_facts > knowledge_summary.accurate_facts`: response is framed with "My knowledge on this topic is mostly unverified — treat this as provisional."
- If `confidence_policy.allow_auto_promotion == false`: response includes an explicit note that no facts from external sources are ever automatically trusted.

---

## 14. Cross-Cutting Omni Language Provisions

### 14.1 Required Standard Library Modules

The Omni standard library must include the following modules to support HELIOS without external dependencies:

```
std::compress::omnipack       # OmniPack implementation (Section 2)
std::compress::lz4            # Interoperability
std::compress::zstd           # Interoperability
std::compress::brotli         # Web content

std::crypto::omnicrypt        # OmniCrypt implementation (Section 3)
std::crypto::aes_gcm          # Interoperability
std::crypto::xchacha20_poly1305 # Interoperability
std::crypto::blake3           # Hashing
std::crypto::ed25519          # Plugin signature verification

std::collections::btree_map   # B-tree for index structures
std::collections::roaring_bitmap # Efficient set operations for indices
std::collections::skip_list   # Temporal index
std::collections::hash_map    # General HashMap
std::collections::bloom_filter # For dedup pre-checks

std::net::http                # HTTP/HTTPS client for web learning
std::net::tls                 # TLS support
std::net::pipe                # Named pipe IPC

std::time                     # Timestamps, duration, scheduling
std::fs                       # File system operations
std::io                       # Input/output
std::serialize::json          # JSON for human-readable export
std::serialize::msgpack       # MessagePack for IPC
std::serialize::omni          # Native Omni binary serialization

std::async                    # Async task executor, futures
std::sync                     # Mutex, RWLock, channel, semaphore
std::process                  # Process spawning for plugin isolation

std::text::regex              # Pattern matching in text
std::text::tokenize           # Sentence and word segmentation
std::text::similarity         # Keyword overlap and basic similarity scoring

std::log                      # Structured logging
std::config::toml             # TOML configuration loading

std::sync::crdt               # CRDT primitives: G-Counter, PN-Counter, OR-Set, LWW-Register (Section 88)
std::concurrency::stm         # Software Transactional Memory: TVar, atomic blocks (Section 89)
std::sec::pola                # Capability-based security: Realm, Cap<T> (Section 90)
std::typing::liquid           # Liquid type refinements with SMT backend (Section 91)
std::crypto::pqc              # Post-Quantum: ML-KEM, ML-DSA, SLH-DSA, HQC (Section 92)
std::net::quic                # QUIC/Multipath QUIC transport for federation (Section 35)
std::net::mdns                # mDNS/DNS-SD peer discovery (Section 35.4)
std::collections::counting_bloom # Counting Bloom Filter with 4-bit counters (Section 102)
std::alloc::arena             # Arena (bump) allocator for ephemeral allocations (Section 7.3)
std::alloc::slab              # Slab allocator with fixed-size bucket pools (Section 7.3)
```

### 14.2 Async Model

HELIOS is inherently concurrent (GUI thread, service thread, reasoning thread, background learning thread, plugin processes). The Omni async model must provide:

- **Cooperative multitasking** via async/await syntax and an executor.
- **Work-stealing thread pool** for the executor backend (configurable pool size, default = logical CPU count).
- **Structured concurrency** via `TaskGroup` scopes: all child tasks spawned within a group are guaranteed to complete or be cancelled before the group exits. Cancellation propagates downward automatically. This eliminates orphaned background tasks and ensures deterministic cleanup (modeled after JDK 25 `StructuredTaskScope` / JEP 505 and Swift `TaskGroup`).
- **Async channels** for communication between cognitive layers.
- **Async file I/O** for non-blocking knowledge store page reads.
- **Timeouts** on all async operations (required for meeting cognitive layer latency budgets).

```omni
# Example: structured concurrency for parallel reasoning with cancellation
let result = TaskGroup::scoped(|group| {
    group.spawn(|| reasoning_engine.backward_chain(goal));
    group.spawn(|| reasoning_engine.analogy_search(goal));
    // TaskGroup waits for all children; first failure cancels siblings
}).await;

# Example cognitive layer invocation with timeout
let result = timeout(Duration::millis(100), reasoning_engine.backward_chain(goal)).await
    .unwrap_or_else(|_| InferenceResult::TimedOut)
```

### 14.3 Build System Requirements

The Omni build system (`opm`) must support:

- **Workspace build:** Build `omni-lang/` and `helios-framework/` as a single workspace with shared compilation cache.
- **Feature flags:** Enable/disable GUI, web learning, plugin system, biometrics as compile-time features.
- **Platform targets:** `native-windows`, `native-linux`, `native-macos`, `ovm` (bytecode).
- **Cross-compilation:** Build for `ovm` target from any host, build for native targets only from matching or cross-capable hosts.
- **Incremental compilation:** Only rebuild changed modules and their dependents.
- **Test runner integration:** `opm test` runs the full test suite with per-module and integration test categorization.

---

## 15. Omni Language — Type System and Effect Enhancements

### 15.1 Effect System for Capability Tracking

The Omni type system must be extended with effect annotations to track what side effects a function is permitted to perform. This is essential for HELIOS's plugin sandbox and cognitive pipeline, where functions may be declared as pure but must not perform network calls, file IO, or process spawning.

Effect annotations are declared as a set of capabilities appended to function signatures:

```omni
# Pure function — may not perform any effects
fn pure_computation(x: i32, y: i32) -> i32 { x + y }

# Function that performs IO — red flag in cognitive layers L0–L2
fn read_config() !Io !File -> Result<String, IoError>

# Function that performs network access — prohibited in some contexts
fn fetch_url(url: &str) !Net !Spawn -> Result<Vec<u8>, NetError>

# Function composition — effects propagate upward
fn query_and_fetch(topic: &str) !Net !Io !File -> Result<String, Error> {
    let gap = query_knowledge_base(topic)!  // Inherits !Net from caller
    fetch_url(&gap.source)?
}
```

**Effect checking rules:**
- Calling a function with effects you have not declared is a compile-time error.
- Effects propagate upward through the call chain: if `f()` calls `g() !Net`, then `f()` must declare `!Net`.
- Default functions without effect annotations are pure: `!{}` (empty effect set).
- The compiler implements effect polymorphism, allowing generic functions to abstract over effects while maintaining safety.

**Standard effect set for HELIOS:**
- `!Io` — File system access
- `!Net` — Network operations
- `!Spawn` — Process/thread spawning
- `!Mut` — Mutable state beyond local variables
- `!Crypto` — Cryptographic and random number operations
- `!Panic` — May panic (unrecoverable errors)
- `!Unsafe` — Invokes unsafe code blocks

### 15.2 Refinement Types — Lightweight Verification

Full dependent types are too complex for a systems language, but refinement types offer the right tradeoff: the ability to express constraints directly in types while erasing them at runtime.

```omni
# Confidence scores must always be 0–100
type ConfidenceScore = u8 where value <= 100 and value >= 0

# Non-empty collections
type NonEmpty<T> = Vec<T> where len(self) > 0

# Bounded ranges
type PageCount = u32 where value <= 1_000_000 and value > 0

# User input must be non-empty after trimming
type TrimmedString = String where len(self.trim()) > 0

struct InformationUnit:
    confidence: ConfidenceScore     # Compile-time: never assigned out-of-range value
    content: TrimmedString          # Compile-time: never stores-only-whitespace string
```

**Compiler behavior:**
- At assignment, the compiler inserts proof obligations: `assign unit.confidence = 95` generates a proof obligation `95 <= 100 land 95 >= 0`, discharged immediately.
- For untrusted sources (user input, web fetches), refinement checks are explicit: `confidence = clamp(web_value, 0, 100)`.
- Fully discharged refinements are erased at compile-time; there is no runtime overhead.
- Integration: refinement violations are reported with the same syntax as type errors: `error: 101 does not satisfy ConfinceCeScore bounds`.

### 15.3 Compile-Time Contracts

Functions maintain data integrity invariants through compile-time checkable contracts:

```omni
fn add_unit(&mut self, unit: InformationUnit) 
    #[requires(unit.content.len() > 0)]
    #[ensures(self.count() == old(self.count()) + 1)]
    #[ensures(self.index_contains(unit.id))]
    -> Result<(), StoreError>
{
    self.units.push(unit);
    self.index.insert(unit.id, self.units.len() - 1);
    Ok(())
}
```

**Contract system:**
- `#[requires(condition)]` — Precondition checked at call site. Caller's responsibility.
- `#[ensures(condition)]` — Postcondition checked after function returns. Callee's responsibility.
- `#[invariant(condition)]` — Struct invariant maintained after every mutation.
- `old(expr)` — Refers to the value of `expr` at function entry time.
- Contracts are checked in debug builds and optimized away in release builds (with a `contracts` config flag to retain them).

HELIOS-specific invariants:
```omni
struct KnowledgeStore:
    #[invariant(units.len() == index.count())]
    #[invariant(units.iter().all(|u| u.accuracy in [Unverified, Accurate, Inaccurate]))]
    #[invariant(no two units have identical (subject, predicate, object) unless one is ARCHIVED)]
    units: Vec<InformationUnit>,
    index: BTreeIndex,
```


### 15.x Variance Annotations for Generic Type Parameters

Omni generics specify variance explicitly using `+T` (covariant), `-T` (contravariant), and `T` (invariant, the default):

```omni
// Covariant (+T): safe for read-only containers
// If Dog <: Animal, then Vec<+Dog> <: Vec<+Animal>
struct ReadOnlyList<+T>:
    items: &[T]    // Only produces T values (read-only)

// Contravariant (-T): safe for write-only / consumer positions
// If Dog <: Animal, then Consumer<-Animal> <: Consumer<-Dog>
trait Processor<-T>:
    fn process(&self, item: T)

// Invariant (T, default): required for read-write containers
// Vec<Dog> is NOT a subtype of Vec<Animal>
struct MutableList<T>:
    items: Vec<T>  // Both produces and consumes T

// Standard library variance:
// Vec<T>      — invariant (default, read-write)
// &T          — covariant in T
// &mut T      — invariant in T (prevents unsound aliasing)
// fn(T) -> U  — contravariant in T, covariant in U
// Capability<T> — invariant (linear types must not be cast)
```

Variance is critical for the plugin ABI (§27/§68) where typed containers cross sandbox boundaries: a plugin returning `Vec<+SpecificUnit>` can be safely upcast to `Vec<+InformationUnit>` at the boundary, but a mutable `Vec<SpecificUnit>` cannot.

---

## 16. Omni Language — Concurrency, Metaprogramming, and Error Handling

### 16.1 Structured Concurrency

Unstructured concurrency (where tasks can outlive the scope that spawned them) is the source of most async bugs. Omni adopts structured concurrency as a language primitive:

```omni
struct Query { /* ... */ }

fn process_cognitive_layers(query: Query) -> Result<Response, QueryError> {
    scope {
        // All tasks spawned in this scope must complete or be cancelled before scope exits
        let l0_handle = spawn { reflex_layer.process(query.clone()).await }
        let l1_handle = spawn { pattern_match.process(query.clone()).await }
        let l2_handle = spawn { inference_engine.process(query.clone()).await }
        
        // Automatically cancels incomplete tasks if parent times out
        let (l0, l1, l2) = timeout(Duration::millis(500), async {
            join_all([l0_handle, l1_handle, l2_handle]).await
        }).await?
    }
    // Scope guaranteed to exit only when all tasks are done or cancelled
}
```

**Structured concurrency guarantees:**
- Tasks spawned within a `scope { }` block cannot outlive the scope.
- Scopes nest: a child scope is cancelled if its parent is cancelled.
- Cancellation is automatic: calling `cancel_scope()` or scope timeout cancels all children.
- No resource leaks: scope exit runs all cleanup handlers of spawned tasks in LIFO order.
- Integration: works with `timeout()`, `.await`, and `.cancel()` combinators.

Implementation: The compiler tracks task lifetimes and generates code that joins all spawned tasks on scope exit. Scopes compile to try-catch blocks where the finally handler cancels all pending tasks.

### 16.2 Compile-Time Metaprogramming — Hygienic Macros

Omni macros operate over the AST and are hygienic — they cannot accidentally capture names from the calling context:

```omni
#[macro]
fn derive_serialize($Type:ty) {
    impl Serialize for $Type {
        fn to_msgpack(&self) -> Vec<u8>;
    }
}

#[derive(Serialize)]  // Expands derive_serialize!(InformationUnit)
struct InformationUnit {
    id: u64,
    content: String,
    confidence: ConfidenceScore,
    // ...
}
```

**Macro features:**
- `#[derive(...)]` — Expand to auto-generated trait implementations.
- `#[macro]` — Define a compile-time function that operates on the AST.
- `$Name:ty` — Captures a type AST node.
- `$Name:expr` — Captures an expression AST node.
- `$Name:ident` — Captures an identifier.
- Hygiene: generated names are namespaced (`__gen_0`, `__gen_1`, etc.) to avoid collisions.

**HELIOS use cases:**
- `#[derive(Serialize, Deserialize)]` — Auto-generate MessagePack code for all types.
- `#[derive(Audit)]` — Auto-generate CRUD audit logging for structs.
- `#[derive(Eq, Hash)]` — Auto-generate equality and hashing for knowledge store keys.

### 16.3 Compile-Time Schema Validation

Service IPC requires schema compatibility between plugin and host. A compile-time macro detects mismatches:

```omni
#[schema_validate]
mod ipc_protocol {
    // Both plugin and host compile against this schema
    enum Message {
        Query(String),
        Response(Vec<InformationUnit>),
        // ...
    }
}

// Compiler hashes the schema and embeds it in the binary
// Mismatches cause linker error: "Schema hash mismatch: plugin has 0xABC, host expects 0xDEF"
```

### 16.4 Error Context Propagation

Errors automatically accumulate context through the call stack:

```omni
fn inner() -> Result<(), Error> {
    read_page(4872)
        .context("failed to read knowledge page")
        .context("decompression failed")
}

fn middle() -> Result<(), Error> {
    inner().context("OmniPack stage 1 processing")
}

fn outer() -> Result<(), Error> {
    middle().context("Knowledge store query")
}

// Error reported to user:
// Knowledge store query → OmniPack stage 1 processing → decompression failed → 
// failed to read knowledge page → byte offset 4872
```

The context chain is built automatically by the `?` operator without explicit `context()` calls:

```omni
fn auto_context() -> Result<(), Error> {
    let page = read_page(4872)?  // Auto: implicit context from file: line
    let decompressed = decompress(page)?  // Auto: context added
    Ok(())
}
```

---

## 17. Omni Language — Tooling, Verification, and Syntax

### 17.1 Fuzzing Infrastructure

The Omni test runner integrates coverage-guided fuzzing:

```bash
opm fuzz knowledge_store::deserialize --corpus seeds/
opm fuzz std::crypto::omnicrypt::decrypt --corpus corpus-crypt/
```

**Fuzz targets:** Natural targets for HELIOS:
- Knowledge store page deserialization (malformed pages must not crash or corrupt state)
- OmniPack decompression (invalid compressed data must decompress safely or error)
- OmniCrypt decryption (invalid authentication must fail safely)
- IPC message deserialization (corrupt plugin messages must not crash host)
- Web fetch response parsing (malformed web content must not panic extraction)

**Implementation:** Omni fuzzing uses libFuzzer-style corpus-guided mutation. The compiler generates instrumentation to track code coverage. Fuzzer output reports crashing inputs and minimal reproductions.

### 17.2 Property-Based Testing

Beyond unit tests, Omni supports property-based testing to verify mathematical properties:

```omni
#[test]
property! {
    confidence_formula_property(
        prov: u8,
        corr: u8,
        fresh: u8,
        deriv: u8,
        user: u8,
        cons: u8
    ) in prov <= 100 && corr <= 100 && fresh <= 100 && deriv <= 100 && user <= 100 && cons <= 100 {
        let score = compute_confidence(prov, corr, fresh, deriv, user, cons);
        assert!(score >= 0 && score <= 100, "confidence {score} out of bounds [0, 100]")
    }
}
```

**Property test coverage for HELIOS:**
- Confidence formula always produces [0, 100]
- RETE network produces the same result regardless of fact assertion order
- Backward chaining with empty rule base returns None
- Knowledge store after flush-reload has identical content hashes
- Knowledge store indices are consistent: `for all unit_id, index[unit_id] returns correct unit`

**Tool:** `opm test --properties` runs property tests with 1000+ random generated inputs per property.

### 17.3 Formal Verification Integration

For safety-critical components, Omni supports formal verification via SMT solvers:

```omni
#[ verify]
fn confidence_formula_proof(
    prov: u8,
    corr: u8,
    fresh: u8,
    deriv: u8,
    user: u8,
    cons: u8,
) {
    requires(prov <= 100 && corr <= 100 && fresh <= 100 && deriv <= 100 && user <= 100 && cons <= 100);
    
    let weighted = (prov * 20 + corr * 25 + fresh * 20 + deriv * 20 + user * 10 + cons * 5) / 100;
    let final_score = max(0, min(100, weighted));
    
    ensures(final_score >= 0 && final_score <= 100);
}
```

**Verified components for HELIOS:**
- Confidence formula: proven to never produce out-of-range values
- Safety gate: proven that restricted capabilities cannot be bypassed
- BLAKE3 collision resistance: assumed properties proven correct

### 17.4 Incremental Compilation Correctness

The Omni incremental compiler tracks dependencies at declaration-level granularity:

**Problem:** If a type definition changes, all modules that transitively depend on it must be recompiled. File-level tracking is too coarse; function-level is too fine.

**Solution:** Omni tracks "declaration dependencies" — when a module uses a declaration from another module, the compiler records that dependency. On recompilation, only modules affected by actually-changed declarations are rebuilt, not modules that import the same module but don't use changed declarations.

```
confidence.rs:
  - fn compute_confidence(...) [line 1-50]
  - fn confidence_thresholds(...) [line 52-80]
  
query.rs imports confidence:
  uses compute_confidence() — requires recompile if confidence formula changes
  doesn't use confidence_thresholds() — no recompile needed if thresholds change
```

### 17.5 Pattern Guards and Exhaustiveness Checking

Complex enum matching requires exhaustiveness verification:

```omni
match interaction {
    ExperienceRecord::QueryResult(q, r) if q.confidence < 50 => {
        // Handle low-confidence responses
    },
    ExperienceRecord::UserFeedback(f) if f.feedback_type == Incorrect => {
        // Handle corrections
    },
    ExperienceRecord::PluginInvoked(p) if !p.permissions.contains(FileIO) => {
        // Handle permission-denied cases
    },
    // Exhaustiveness check catches any missing variants
}
```

The compiler maintains a database of all enum variants and flags non-exhaustive matches at compile time.

### 17.6 Protocol Types — Session Type Encoding

Service IPC and plugin IPC require message sequence protocols. Protocol types enforce correct sequencing:

```omni
type ConnectionState<S>;

// Start state — only Connect is valid
impl ConnectionState<Disconnected> {
    fn connect(&self, url: &str) -> ConnectionState<Connected> {
        // Returns a new type that only allows Query, Approve, etc.
    }
}

impl ConnectionState<Connected> {
    fn send_query(&mut self, q: Query) -> QueryState {
        // Transitions to a state where response is expected
    }
}

// Compiler prevents: in Disconnected state, calling send_query() is a type error
let conn = ConnectionState::new();  // : ConnectionState<Disconnected>
conn.send_query(q)  // ERROR: ConnectionState<Disconnected> doesn't have send_query
```

### 17.7 Numeric Tower Extensions

The Omni numeric type system includes:

- `u128`, `i128` — For UUID arithmetic and large ID spaces
- `f16` — Compact floating-point for confidence value arrays (significant space savings in bulk knowledge store operations)
- `Decimal<P, S>` — Arbitrary-precision decimal, not floating-point (for scientific and financial domains)
- `Duration` — Distinct from `u64`, prevents accidental timestamp arithmetic
- `Timestamp` — Distinct from `u64`, prevents mixing absolute and relative times

```omni
struct InformationUnit {
    id: u128,              // Large ID space, no overflow risk
    confidence: f16,       // Array of 1M confidences = 2MB instead of 4MB
    acquired_at: Timestamp,
    expires_at: Option<Timestamp>,
}

// ERROR: cannot add two Timestamps
let invalid = unit1.acquired_at + unit2.acquired_at;

// CORRECT: subtract to get Duration
let age: Duration = now() - unit.acquired_at;
```

---

## 18. HELIOS Confidence — Advanced Reasoning Modes

### 18.1 Dempster-Shafer Theory as Secondary Evidence Mode

The current confidence model works well with complete consistent evidence but struggles to represent ignorance as distinct from uncertainty. Dempster-Shafer Theory (DST) provides this distinction through belief mass functions:

```omni
struct DempsterShaferRecord:
    belief:       f32          # 0.0–1.0: Mass supporting true
    disbelief:    f32          # 0.0–1.0: Mass supporting false
    uncertainty:  f32          # 0.0–1.0: Mass representing ignorance
    # Invariant: belief + disbelief + uncertainty = 1.0
    
    # Additional fields from base confidence record
    provenance:   u8
    corroboration: u8


enum ConfidenceMode:
    Standard                   # Traditional weighted sum model
    DempsterShafer            # DST model when evidence is limited/conflicted
```

**When to use DST mode:**
1. Query domain has very limited evidence (< 3 independent sources)
2. Sources actively contradict each other (conflicted units in the knowledge graph)
3. User explicitly requests "express uncertainty" in response phrasing

**DST Dempster combination rule:** When two independent belief masses $m_1$ and $m_2$ combine:

$$m_{combined}(A) = \frac{1}{1-K} \sum_{X \cap Y = A} m_1(X) \cdot m_2(Y)$$

where $K = \sum_{X \cap Y = \emptyset} m_1(X) \cdot m_2(Y)$ (conflict measure).

**Response generation difference:**
- Standard model on 40% confidence: "I have moderate confidence in this answer."
- DST model with belief=0.4, disbelief=0.1, uncertainty=0.5: "My sources are divided, and I have substantial doubt. The answer might be X, but consider alternative Y."

### 18.2 Counterfactual Reasoning

Counterfactual queries explore "what if" scenarios by hypothetically retracting facts:

```omni
struct CounterfactualQuery:
    current_goal: Goal
    hypothetical_retractions: Vec<u64>  # Unit IDs to temporarily remove
    
fn counterfactual_query(
    goal: Goal,
    retraction_ids: Vec<u64>,
) -> Result<CounterfactualResult, QueryError>


struct CounterfactualResult:
    factual_answer: String       # Answer under normal knowledge
    delta_facts: Vec<u64>        # Facts whose support changed
    counterfactual_answer: String  # Answer if retractions applied
    changed_conclusions: Vec<String>  # Inferences that would differ
```

**Algorithm:**
1. Save current Working Memory state
2. Remove specified units from the knowledge store (temporarily)
3. Run cognitive pipeline and capture results
4. Restore state
5. Compute delta (factual vs counterfactual)
6. Report: "Normally: X. If fact #4521 were false: Y. Additionally, fact #6342 would lose support."

---

## 19. HELIOS Knowledge — Temporal Reasoning and Ontologies

### 19.1 Temporal Knowledge Quadruples

The current `InformationUnit` model treats facts as timeless. Temporal Knowledge Graphs require facts to be **quadruples** `(subject, predicate, object, time_interval)`:

```omni
struct InformationUnit:
    # ... existing fields ...
    
    # New temporal fields
    valid_from:   Option<Timestamp>   # When this fact became true
    valid_until:  Option<Timestamp>   # When this fact ceased to be true
    
    # Enhancement: time-scoped reasoning
    temporal_context: Option<TemporalConstraint>


enum TemporalConstraint:
    Point(Timestamp)                    # True at exactly this moment
    Interval(Timestamp, Timestamp)      # True between start and end
    OpenStart(Timestamp)                # True from this point onward
    OpenEnd(Timestamp)                  # True up to this point
    Always                              # Always true (no time bounds)


# Example: "The capital of France is Paris, valid_from=1870, valid_until=None"
unit = InformationUnit {
    subject: "France",
    predicate: "capital",
    object: "Paris",
    valid_from: Some(ts_1870),
    valid_until: None,  // Still true
    # ...
}
```

**Temporal RETE queries:**
```
# What was France's capital in 1910?
FIND facts WHERE subject="France" AND predicate="capital" 
    AND valid_from <= ts_1910 AND (valid_until >= ts_1910 OR valid_until NULL)
```

**Temporal inference rule example:**
```
IF subject:Place predicate:"capital" object:City valid_from=T1 valid_until=T2
   AND subject:City predicate:"population" object:N valid_from=T1 valid_until=T2
THEN subject:Place predicate:"capital_population" object:N valid_from=T1 valid_until=T2
```

### 19.2 Ontology Support — Concept Hierarchies

Ontologies define knowledge vocabulary and relationships. HELIOS implements ontologies as special `InformationUnit` records:

```omni
# Ontology relationships stored as units with special predicates
unit1 = InformationUnit {
    subject: "dog",
    predicate: "subclass_of",
    object: "mammal",
    accuracy: Accurate,
    confidence: 95,
}

unit2 = InformationUnit {
    subject: "mammal",
    predicate: "subclass_of",
    object: "animal",
    accuracy: Accurate,
    confidence: 95,
}
```

**Ontology query expansion:**
```
FIND facts WHERE subject = "Labrador Retriever" AND predicate="behavior"
→ Auto-expanded to:
FIND facts WHERE subject IN ("Labrador Retriever", "dog", "mammal", "animal") 
    AND predicate="behavior"
```

**Property inheritance:**
```
dog: is_a → canine
canine: property → has_tail (true)

QUERY: Does Labrador have tail?
→ Labrador -> dog -> canine -> has_tail = true ✓
```

**Ontology type support:**
- `is_a` — Class membership (individual → class)
- `subclass_of` — Class hierarchy (class → class)
- `part_of` — Meronymy (component relationships)
- `same_as` — Equivalence (for synonym handling)
- `inverse_of` — Predicate inversion (e.g., "child_of" inverse is "parent_of")

---

## 20. HELIOS Knowledge — Versioning, Federation, and Interchange

### 20.1 Knowledge Versioning — Snapshots and Branches

Users need to revert knowledge stores to known-good states and experiment safely:

```omni
# Create a snapshot
knowledge_store.snapshot("before-news-feed-ingestion")

# Later, roll back if needed
knowledge_store.restore_snapshot("before-news-feed-ingestion")

# Create an experiment branch
branch = knowledge_store.branch("hypothesis-all-staged-facts")
branch.accept_all_staged_facts()
# Hypothetical branch never committed to main
```

**Implementation:** Since pages are write-once, a snapshot is just a reference to the current page index BLAKE3 hash plus timestamp. No data duplication needed. Restoration means reloading that index version.

**Version timeline:**
```
main: ----[snapshot S1]----[10 new facts]----[snapshot S2]----[rollback]→S1
                                                   ↓
                            branch: [accept hypothesis facts]---[experiment]
```

### 20.2 Knowledge Federation — Multi-Device Sync

Users running HELIOS on desktop, laptop, and new OS need synchronized knowledge stores:

```omni
struct FederationSync:
    source_instance_id: [u8; 16]
    last_sync_at: Timestamp
    units_modified_since: Vec<(u64, Timestamp)>  # unit_id, modification_time


struct FederationMergePolicy:
    strategy: MergeStrategy,  // LastWriteWins | KeepLocal | KeepRemote | Manual
    selective_sync: bool,    // Some units are LOCAL_ONLY and don't sync
#

impl FederationMergePolicy {
    fn merge_unit(&self, local: &InformationUnit, remote: &InformationUnit) -> InformationUnit {
        match self.strategy {
            LastWriteWins => {
                if remote.last_updated_at > local.last_updated_at {
                    remote.clone()
                } else {
                    local.clone()
                }
            },
            // ...
        }
    }
}
```

**Sync protocol:**
1. Instance A queries Instance B: "Send me all units modified after <timestamp>."
2. Instance B sends paginated chunks of modified units.
3. Instance A merges using MergePolicy.
4. Instance A sends back: "Send me units from <timestamp> to now on your end."
5. Bidirectional sync complete.

**LOCAL_ONLY flag:**
```omni
unit = InformationUnit {
    // ... standard fields ...
    flags: {
        LOCAL_ONLY,  // Don't sync to other devices
    }
}
```

### 20.3 Knowledge Import/Export Standards

Interoperability requires standard interchange formats:

**JSON-LD Export:**
```json
{
  "@context": "https://schema.org",
  "@type": "Thing",
  "name": "Aspirin",
  "description": "A common pain reliever",
  "url": "helios://knowledge/unit/12345",
  "relatedLink": [
    {
      "@context": "https://helios.example/ns",
      "predicate": "treats",
      "target": "unit/67890"
    }
  ]
}
```

**Markdown Import:**
HELIOS ingests Markdown (Obsidian, Logseq) files:
```markdown
# Climate Change

[[Greenhouse Gas Effect]] causes [[Global Warming]]

- Related: [[IPCC Reports]]
- Source: Wikipedia
```

Transforms to:
```
subject: "Climate Change"
predicate: "causes"
object: "Global Warming"

subject: "Climate Change"
predicate: "related_to"
object: "IPCC Reports"
```

**CSV/TSV Import:**
```
subject,predicate,object,confidence,source
Aspirin,treats,Headache,85,Medical_Authority
Aspirin,compounds,Salicylic Acid,95,Chemistry_Handbook
```

---

## 21. HELIOS Capabilities — Multimodal Input and Query Language

### 21.1 Multimodal Input Processing

HELIOS extends knowledge acquisition to images, audio, and PDFs:

```omni
enum MultimediaSource:
    ImageOcr { file_path: String, format: String }      // JPG, PNG, PDF document scan
    AudioTranscript { file_path: String, provider: String }  // Local transcription
    PdfMixed { file_path: String }                        // PDF with text + images + tables
    VideoCaption { file_path: String }                    // Extracted video captions


fn process_image_ocr(image_path: &str) -> Result<Vec<InformationUnit>, ProcessError> {
    // OCR pipeline: read image → extract text → parse into units
    // Result: source marked as SourceType::UserUploadedFile(image_path)
}

fn process_audio(audio_path: &str) -> Result<Vec<InformationUnit>, ProcessError> {
    // Transcription: audio → text → units
    // Local transcription engine (no cloud dependency)
}

fn process_pdf_mixed(pdf_path: &str) -> Result<Vec<InformationUnit>, ProcessError> {
    // Extract: text pages + scanned images (OCR) + tables (structured)
    // Each becomes a unit with source lineage to PDF
}
```

### 21.2 Structured Query Language — OQL

Users need a human-readable query language for the REPL and Knowledge Browser:

```omni
# Module: std::knowledge::oql

enum OqlQuery:
    Find { where_clause: WhereClause }
    FindPath { from: String, to: String, hops: u8 }
    FindConflicts { where_clause: Option<WhereClause> }
    Count { where_clause: WhereClause }


# Examples:
FIND facts WHERE subject="climate change" AND confidence > 70 AND accuracy="Accurate"
FIND facts WHERE domain="medicine" AND acquired_after="2024-01-01"
FIND path FROM "aspirin" TO "anti-inflammatory" IN 3 HOPS
FIND conflicts WHERE subject CONTAINS "vaccine"
COUNT facts WHERE source_type="WebLearningActive" AND accuracy="Unverified"


fn parse_oql(query_str: &str) -> Result<OqlQuery, ParseError>
fn execute_oql(query: OqlQuery, store: &KnowledgeStore) -> Result<Vec<InformationUnit>, QueryError>
```

---

## 22. HELIOS Safety — Information Classification and Governance

### 22.1 Information Hazard Classification

Facts themselves (independent of source) may be sensitive or dangerous:

```omni
enum InformationClass:
    Public              // No restrictions
    Sensitive           // Not included in unauthenticated responses
    Confidential         // Requires identity verification before use
    Restricted          // Cannot be used in responses; audit-only
    Hazardous           // Flagged for review; may be quarantined


struct InformationUnit:
    // ... existing fields ...
    classification: InformationClass,  // Orthogonal to accuracy_status


impl KnowledgeStore {
    fn query_response(&self, goal: Goal, session: &Session) -> Result<Response, Error> {
        facts = self.find_relevant_facts(goal)?
        
        // Filter by classification
        usable_facts = facts.filter(|f| {
            match f.classification {
                Public => true,
                Sensitive => session.authenticated,
                Confidential => session.identity_verified,
                Restricted => false,  // Never use in responses
                Hazardous => session.has_hazard_consent,
            }
        })
        
        // Construct response from usable facts only
        self.build_response(usable_facts, goal)
    }
}
```

### 22.2 Audit Replay

Complete audit log allows deterministic reconstruction of past states:

```omni
struct AuditReplayRequest:
    target_timestamp: Timestamp
    delta_comparison_with: Option<Timestamp>  // Compare two states


fn replay_at_timestamp(
    request: AuditReplayRequest
) -> Result<ReconstructedKnowledgeStore, ReplayError> {
    // Load checkpoint before target_timestamp
    checkpoint = load_latest_checkpoint_before(target_timestamp)?
    
    // Replay CRUD events up to target_timestamp
    for event in audit_log.events_from(checkpoint.timestamp) {
        if event.timestamp > target_timestamp { break }
        apply_crud_event(&mut checkpoint, event)
    }
    
    Ok(checkpoint)
}
```

**Use case:** "Show me what HELIOS knew on March 1st vs. today, and what caused the difference."

### 22.3 Rate-of-Change Alerting

Monitor knowledge store for anomalous modification patterns:

```omni
struct RateChangeAlert:
    subject: String
    super_units_modified: u32
    time_window: Duration
    threshold: u32


// Alert if 500+ facts about same subject modified in 1 hour
if units_modified_per_subject.max() > 500 && time_window < 1_hour {
    alert: "Anomalous ingestion rate for subject X — pausing auto-learning"
}
```

---

## 23. GUI Enhancements — Visualization and Explainability

### 23.1 Full Knowledge Graph Visualization

The Knowledge Browser includes a comprehensive graph visualizer:

```omni
# GPU-accelerated rendering using Direct2D (Windows) or Cairo (Linux)

struct GraphVisualizerState:
    zoom_level: f32          // 0.1 to 10.0
    node_filter: FilterLens  // By confidence, accuracy, source, date
    layout_algorithm: LayoutAlgorithm,  // Fruchterman-Reingold, Barnes-Hut
    selected_nodes: Vec<u64>,


enum LayoutAlgorithm:
    FruchtermanReingold  // Force-directed; good for sparse graphs
    BarnesHut            // Force-directed; better for dense graphs
    TreeLayout           // For ontology hierarchies
```

**Semantic zoom:**
- **Low zoom (city view):** Domain-level clusters (Medicine, Technology, History)
- **Medium zoom (street view):** Concept nodes with labels and confidence badges
- **High zoom (address view):** InformationUnit nodes with full text snippets

**Interactive features:**
- Click two nodes → visualizer computes shortest path and animates it
- Filter lens pulls nodes outside configuration and fades them
- Force layout updates incrementally as new nodes are added (not recomputed from scratch)

### 23.2 Session Comparison

Compare knowledge states across time:

```omni
fn compare_sessions(
    session1_at: Timestamp,
    session2_at: Timestamp,
) -> SessionComparison {
    state1 = replay_at_timestamp(session1_at)?
    state2 = replay_at_timestamp(session2_at)?
    
    delta = compute_diff(state1, state2)
    
    SessionComparison {
        units_added: delta.added.len(),
        units_removed: delta.removed.len(),
        units_modified: delta.modified.len(),
        added_facts: delta.added,
        removed_facts: delta.removed,
        // ...
    }
}
```

**UI:** Side-by-side knowledge browser showing state at T1 and T2, with changed units highlighted.

### 23.3 Notification System

Native OS notifications for key events:

```omni
enum NotificationEvent:
    WebLearningComplete { facts_acquired: u32, require_review: u32 }
    ConflictDetected { subjects: Vec<String> }
    FreshnessAlert { facts_outdated: u32 }
    SelfModificationProposal { proposal_id: u64 }
    PluginError { plugin_id: String, error: String }


// Windows: Toast notification via WinAppSDK
// Linux: libnotify
// macOS: NSUserNotificationCenter

notify_user(
    NotificationEvent::WebLearningComplete { 
        facts_acquired: 47, 
        require_review: 12 
    }
)
// Notification text: "Acquired 47 new facts about climate science. 12 require your review."
// Double-click: Navigate to Learning Status panel
```

### 23.4 Explainability Mode — Natural Language Traces

Translate technical cognitive traces into English:

```omni
struct ExplainabilityTrace:
    trace_events: Vec<TraceEvent>,  // RETE firings, BCs steps, layer transitions
    

fn explain_trace(trace: ExplainabilityTrace) -> String {
    // Template-based NLG — not a language model
    
    "I answered this question by first recalling that [fact A] from [source] \
     ([confidence]% confident). I then applied the rule 'if A is true and B is true, \
     then C follows' using [fact B] ([confidence]% confident). My final answer has a \
     confidence of [score]% because: the two facts I used came from different sources \
     (good corroboration), but one of them has not been verified by you and was \
     fetched from the web [age] ago ([freshness assessment])."
}
```

---

## Appendix A — Phase Implementation Sequence

The implementation sequence for the above specifications, ordered to minimize dependency blockers:

**Phase A — Foundation (prerequisite for everything)**
1. Implement BLAKE3 hash in Omni standard library (used by OmniPack, OmniCrypt, and audit).
2. Implement OmniPack Level 1 (fast path only, no macro-dictionary).
3. Implement OmniCrypt (full implementation — security cannot be partial).
4. Define and freeze the `.omk` page format.
5. Implement the B-tree primary index.

**Phase B — InformationUnit and Knowledge Store**
1. Implement the full `InformationUnit` schema from Section 1.2.
2. Implement the five index structures from Section 7.2.
3. Implement the CRUD audit log.
4. Implement the confidence formula from Section 5.3.
5. Implement the accuracy state machine from Section 6.1.

**Phase C — Brain Core (can parallel with Phase B)**
1. Implement Working Memory struct.
2. Implement the RETE network (alpha nodes, alpha memory, beta join nodes, beta memory, production nodes).
3. Implement the match-resolve-act cycle.
4. Implement backward chaining with depth limit.
5. Implement the Cognitive Cortex pipeline.
6. Write and pass acceptance tests: load 20 facts + 5 rules, assert correct derived facts appear.

**Phase D — Web Learning**
1. Implement HTTP client in std::net::http.
2. Implement Stages 1–4 of the web learning pipeline.
3. Implement the staging area.
4. Implement the verification queue.
5. Implement automatic corroboration.

**Phase E — Experience Log**
1. Implement the experience record taxonomy.
2. Implement the three-index experience log store.
3. Implement the feedback loop (experience → knowledge accuracy).

**Phase F — Plugins**
1. Implement plugin manifest parsing and validation.
2. Implement sandbox OVM instance spawning.
3. Implement typed IPC protocol.
4. Implement the approval flow.
5. Test with one real plugin (e.g., a local file corpus ingester).

**Phase G — GUI**
1. Implement the service IPC protocol (named pipe + MessagePack).
2. Implement the Conversation panel.
3. Implement the Cognitive Trace Viewer panel.
4. Implement the Knowledge Browser panel.
5. Implement the Learning Status panel.
6. Implement the Settings panel.

**Phase H — Self-Model and Governance**
1. Implement SelfModel with live derivation.
2. Implement self-modification proposal generation and approval.
3. Implement identity verification.

**Phase I — OS Target (long-term)**
1. Complete Omni self-hosting.
2. Implement Omni-native bootloader.
3. Implement microkernel.
4. Port HELIOS framework to bare-metal SAL.
5. Ship bootable ISO.

---

## Appendix B — Acceptance Test Specifications Per Phase

**Phase B acceptance:** `create-knowledge-store`, add 100 units with varying accuracy and confidence, run queries for exact matches, fuzzy matches, by accuracy filter, by confidence range. All queries return correct results. Check CRUD audit log contains 100 creation events. Flush to disk, reload from disk, verify all 100 units and all indices are intact with identical content hashes.

**Phase C acceptance:** Load knowledge base with at least 20 facts and 5 production rules. Run `cognitive_cortex.process_query("what is X")` for a query whose answer requires exactly 2 forward-chaining rule firings. Assert the returned answer is correct, the confidence score is ≥ 50, the reasoning trace contains exactly 2 rule firing events, and the response time is < 100ms. Run backward-chain test: assert the backward chainer correctly proves a 3-step inference chain.

**Phase D acceptance:** Simulate web learning for a known gap. Mock the HTTP client to return a known document. Assert the pipeline stages all produce correct typed outputs, the staging area contains the expected candidate unit, and the auto-corroboration count updates correctly when a second corroborating fact is added.

**Phase G acceptance:** Start the HELIOS service. Connect the GUI. Type a query in the Conversation panel. Assert the response appears with correct confidence indicator. Open the Cognitive Trace panel. Assert it shows at least one RETE rule firing event. Open the Knowledge Browser. Assert it shows units that were used in the query response.

---

---

## 24. HELIOS Reasoning — Causal Inference and Interventional Queries

### 24.1 Causal Knowledge Graph Extension

The current knowledge graph stores associative relationships (`causes`, `is_part_of`, `leads_to`). A Causal Knowledge Graph (CKG) extension adds formal causal semantics, enabling interventional and counterfactual reasoning beyond simple correlation. This is achieved by marking relationship types as directional causal links with causal strength metrics.

```omni
enum CausalRelationType:
    DirectCause          # A directly causes B
    IndirectCause        # A causes B through intermediary chain
    ContributingFactor   # A increases probability of B but doesn't deterministically cause it
    Inhibitor            # A decreases probability of B
    Necessary            # A is necessary for B
    Sufficient           # A is sufficient for B
    NecessaryAndSufficient

struct CausalLink:
    source_id:      u64
    target_id:      u64
    causal_type:    CausalRelationType
    strength:       u8                   # 0–100 causal strength
    evidence_ids:   Vec<u64>             # InformationUnit IDs supporting this causal claim
    confounders:    Vec<u64>             # Known confounding factor unit IDs
    interventional: bool                 # Can this cause be intervened upon?
    confidence:     u8                   # Confidence in the causal relationship itself

# Addition to InformationUnit:
    causal_links:   Vec<CausalLink>      # Outgoing causal edges
```

### 24.2 Interventional Query Engine

Interventional queries model the do-calculus from Pearl's causal inference framework: "What happens if we **do** X?" — as opposed to "What happens if we **observe** X?"

```omni
struct InterventionalQuery:
    intervention:     Vec<(u64, String)>  # (unit_id, forced_value) — set these facts to specific values
    observe:          Vec<u64>            # Unit IDs to observe the effect on
    control_for:      Vec<u64>            # Confounders to control for

struct InterventionalResult:
    pre_intervention:   Vec<(u64, String, u8)>   # (unit_id, value, confidence) before intervention
    post_intervention:  Vec<(u64, String, u8)>   # (unit_id, value, confidence) after intervention
    causal_chain:       Vec<CausalLink>          # The causal path from intervention to effect
    confounders_found:  Vec<u64>                 # Confounders discovered during analysis
    explanation:        String                   # NLG explanation of the causal reasoning

fn interventional_query(
    query: InterventionalQuery,
    store: &KnowledgeStore,
    graph: &CausalGraph,
) -> Result<InterventionalResult, QueryError>
```

**Algorithm:**
1. Construct the causal DAG from `CausalLink` relations in the knowledge graph.
2. Apply Pearl's do-operator: remove all incoming edges to intervened variables (graph surgery).
3. Propagate the intervention value through the modified DAG using belief propagation.
4. Compare pre- and post-intervention distributions for observed variables.
5. Report the causal chain and confounders.

### 24.3 Causal Discovery Module

For domains where causal structure is unknown, a causal discovery module infers causal relationships from temporal observations:

```omni
struct CausalDiscoveryConfig:
    algorithm:          CausalDiscoveryAlgorithm
    significance_level: f32    # p-value threshold for edge inclusion (default 0.05)
    max_conditioning:   u8     # Maximum conditioning set size (default 3)
    require_approval:   bool   # Require user approval for discovered causal links

enum CausalDiscoveryAlgorithm:
    PC                          # Peter-Clark algorithm — constraint-based
    GES                         # Greedy Equivalence Search — score-based
    FCI                         # Fast Causal Inference — handles latent confounders
    GrangerCausality            # For time-series data in temporal knowledge quadruples
```

---

## 25. HELIOS Reasoning — Probabilistic Inference and Bayesian Networks

### 25.1 Bayesian Network Integration

While the RETE network handles deterministic rule-based reasoning, many real-world knowledge queries require probabilistic inference. HELIOS integrates a Bayesian Network (BN) engine as a complement to the RETE forward/backward chainer:

```omni
struct BayesianNetwork:
    nodes:              Vec<BayesNode>
    edges:              Vec<(usize, usize)>          # Directed edges (parent → child)
    cpt_tables:         HashMap<usize, CPT>          # Conditional Probability Tables

struct BayesNode:
    id:                 usize
    unit_id:            Option<u64>                  # Linked InformationUnit, if any
    variable_name:      String
    states:             Vec<String>                  # Possible values (e.g., ["true", "false"])

struct CPT:
    parent_ids:         Vec<usize>
    probabilities:      Vec<f32>                     # Flattened conditional probability table
    # Layout: P(child_state | parent_states) in lexicographic order of parent state combinations

fn infer_bayesian(
    network: &BayesianNetwork,
    evidence: HashMap<usize, String>,                # Observed variable → state
    query_node: usize,
) -> Result<Vec<(String, f32)>, InferenceError>      # State → posterior probability
```

### 25.2 Belief Propagation Engine

Inference is performed using the Sum-Product Message Passing (Belief Propagation) algorithm. For tree-structured networks this yields exact results; for loopy networks, Loopy Belief Propagation (LBP) is used with convergence detection:

```omni
struct BeliefPropagationConfig:
    max_iterations:     u32        # Maximum LBP iterations (default 100)
    convergence_eps:    f32        # Convergence threshold (default 1e-6)
    damping_factor:     f32        # Message damping to improve convergence (default 0.5)
    schedule:           BPSchedule

enum BPSchedule:
    Synchronous                    # All messages updated simultaneously
    Asynchronous                   # Messages updated one at a time, residual-based priority
    TreeDecomposition              # Convert to junction tree for exact inference on loopy graphs
```

### 25.3 Bayesian-Knowledge Store Bridge

The BN engine integrates with the knowledge store through automatic BN construction from causal links:

```
Knowledge Store                    Bayesian Network
    CausalLink(A→B, strength=80)   →   Edge A→B, P(B|A) = 0.80
    CausalLink(A→B, strength=80)
    + CausalLink(C→B, strength=60) →   CPT for B given parents {A, C}
    
    New fact corroborating A        →   Evidence: A=true → re-run inference → update B's confidence
```

When a new fact arrives or an existing fact changes confidence, the BN engine re-runs inference on affected nodes and propagates confidence updates through the causal graph. This provides a principled alternative to the weighted-sum confidence formula for domains with rich causal structure.

---

## 26. HELIOS Knowledge — Graph Embeddings and Link Prediction

> **Architectural Note:** Knowledge Graph Embeddings (TransE, RotatE) in this section are trained **offline** on a static snapshot of the knowledge graph and loaded as immutable embedding tables. No gradient descent occurs at runtime. The embeddings serve as a pre-computed similarity index for analogy detection and link prediction, consumed as read-only lookup tables by the reasoning layers.


### 26.1 Knowledge Graph Embedding (KGE) Engine

Knowledge graph embeddings map entities and relations to continuous vector spaces. HELIOS uses KGE for three purposes: (1) link prediction — discovering missing relationships, (2) semantic similarity — finding conceptually related facts even without explicit links, and (3) clustering — identifying knowledge clusters for visualization.

```omni
struct KGEConfig:
    model:              KGEModel
    embedding_dim:      u16          # Embedding vector dimensionality (default 128)
    learning_rate:      f32          # SGD learning rate (default 0.01)
    margin:             f32          # Margin for ranking loss (default 1.0)
    batch_size:         u32          # Training batch size (default 256)
    negative_samples:   u8           # Negative samples per positive (default 5)
    epochs:             u32          # Training epochs (default 100)

enum KGEModel:
    TransE              # Translation-based: h + r ≈ t
    TransR              # Relation-specific projection: M_r * h + r ≈ M_r * t
    DistMult            # Bilinear diagonal: h ⊙ r ⊙ t
    ComplEx             # Complex-valued for asymmetric relations
    RotatE              # Rotation-based for composition patterns

struct EmbeddingStore:
    entity_embeddings:  HashMap<u64, Vec<f32>>       # unit_id → embedding vector
    relation_embeddings: HashMap<String, Vec<f32>>   # predicate → embedding vector
    last_trained_at:    Timestamp
    training_loss:      f32

fn train_embeddings(store: &KnowledgeStore, config: KGEConfig) -> Result<EmbeddingStore, KGEError>
fn predict_links(embeddings: &EmbeddingStore, head: u64, relation: &str, top_k: u8) -> Vec<(u64, f32)>
fn find_similar(embeddings: &EmbeddingStore, unit_id: u64, top_k: u8) -> Vec<(u64, f32)>
```

### 26.2 Entity Alignment for Federation

When synchronizing knowledge stores between HELIOS instances (Section 20.2), entity alignment identifies equivalent entities across stores that may use different naming:

```omni
struct EntityAlignmentConfig:
    method:             AlignmentMethod
    similarity_threshold: f32       # Minimum similarity to consider alignment (default 0.85)
    use_embeddings:     bool        # Use KGE embeddings for alignment (recommended)
    use_string_matching: bool       # Use string similarity as fallback

enum AlignmentMethod:
    MTransE             # Cross-store embedding alignment via translation
    JointEmbedding      # Train embeddings jointly across both stores
    StringBased         # Edit distance + keyword overlap only

struct AlignmentResult:
    alignments:         Vec<(u64, u64, f32)>  # (local_id, remote_id, confidence)
    unmatched_local:    Vec<u64>
    unmatched_remote:   Vec<u64>
```

---

## 27. Plugin System — WASM-Based Sandboxing and Component Model

### 27.1 WebAssembly Plugin Runtime

The current plugin system (Section 10) runs plugins in isolated OVM processes. A superior isolation model uses WebAssembly (WASM) with the Component Model, providing language-agnostic sandboxing, zero-trust security, and near-native performance:

```omni
struct WasmPluginRuntime:
    engine:             WasmEngine           # Compiled WASM engine instance
    component:          WasmComponent        # Validated WASM Component Model module
    capabilities:       Vec<WasiCapability>  # WASI capabilities granted to this plugin
    memory_limit:       u32                  # Maximum linear memory in bytes
    fuel_limit:         u64                  # Instruction count limit per invocation

enum WasiCapability:
    Stdout                                   # Write to stdout
    Stderr                                   # Write to stderr
    FilesystemRead(Vec<String>)              # Read from specific paths
    FilesystemWrite(Vec<String>)             # Write to specific paths
    HttpOutgoing(Vec<String>)                # HTTP requests to specific domains
    Sockets(Vec<String>)                     # Socket access to specific endpoints
    Clocks                                   # Access to system time
    Random                                   # Access to CSPRNG
    # No capability = no access (whitelist model)
```

### 27.2 WASM Component Model Plugin Interface

Plugins define their interface using the WIT (WASM Interface Type) format. HELIOS provides a host interface that plugins can call through:

```wit
// helios-plugin.wit — the interface every plugin component implements
package helios:plugin@1.0.0;

interface plugin-api {
    // Plugin lifecycle
    initialize: func() -> result<_, string>;
    shutdown: func();

    // Plugin declares what it provides
    record plugin-info {
        name: string,
        version: string,
        description: string,
        capabilities: list<string>,
    }
    get-info: func() -> plugin-info;
}

// Host interface — what HELIOS provides to plugins
interface host-api {
    // Read knowledge store (requires ReadKnowledgeStore permission)
    query-facts: func(subject: string, predicate: option<string>) -> list<fact-summary>;

    // Write candidate facts (always Unverified, requires WriteKnowledgeStore)
    propose-fact: func(content: string, subject: string, predicate: option<string>, object: option<string>) -> result<u64, string>;

    // Logging
    log: func(level: log-level, message: string);
}
```

### 27.3 Advantages over OVM Isolation

| Property | OVM Process Isolation (v3) | WASM Component Model (v4) |
|----------|---------------------------|---------------------------|
| Granularity | Process-level | Function-level |
| Memory overhead | Full OVM per plugin | Shared WASM engine, sandboxed linear memory |
| Startup time | ~100ms (process spawn) | ~1ms (module instantiation) |
| Language support | Omni only | Any language compiling to WASM |
| Capability model | Declared permissions | Zero-trust whitelist (WASI) |
| Security audit | Review plugin source + manifest | Validate WASM binary + WIT interface |
| IPC overhead | Message passing over pipes | Direct function calls across component boundaries |

The OVM process isolation model is retained as a fallback for plugins requiring capabilities not yet supported by WASI (e.g., direct GPU access, raw socket creation).

---

## 28. HELIOS Observability — Distributed Tracing and Telemetry

### 28.1 OpenTelemetry Integration

HELIOS adopts the OpenTelemetry (OTel) standard for internal observability. Every cognitive operation — from query receipt to response generation — produces structured traces, metrics, and logs correlated by trace ID.

```omni
struct HeliosTracer:
    tracer:             OtelTracer
    meter:              OtelMeter
    logger:             OtelLogger

# Trace a cognitive query through all layers
fn trace_query(tracer: &HeliosTracer, query: Query) -> Response {
    let span = tracer.start_span("cognitive_query", SpanKind::Server);
    span.set_attribute("query.subject", query.subject);
    span.set_attribute("query.text", query.text);
    
    let l0_span = tracer.start_child_span("L0_reflex", &span);
    let l0_result = reflex_layer.process(&query);
    l0_span.set_attribute("l0.hit", l0_result.is_some());
    l0_span.end();
    
    if l0_result.is_none() {
        let l1_span = tracer.start_child_span("L1_rete_forward", &span);
        let l1_result = rete_network.forward_chain(&query);
        l1_span.set_attribute("l1.rules_fired", l1_result.rules_fired);
        l1_span.set_attribute("l1.confidence", l1_result.confidence);
        l1_span.end();
        // ... continue through L2, L3, L4
    }
    
    span.set_attribute("response.confidence", response.confidence);
    span.set_attribute("response.fact_count", response.supporting_facts.len());
    span.end();
    response
}
```

### 28.2 Cognitive Metrics

HELIOS exposes the following metrics via OTel:

| Metric | Type | Description |
|--------|------|-------------|
| `helios.query.duration` | Histogram | Query processing time in ms, labeled by layer reached |
| `helios.query.confidence` | Histogram | Final confidence score distribution |
| `helios.rete.rules_fired` | Counter | Total RETE rule firings |
| `helios.rete.cycle_time` | Histogram | Time per RETE match-resolve-act cycle |
| `helios.knowledge.unit_count` | Gauge | Total InformationUnits by accuracy status |
| `helios.knowledge.conflict_count` | Gauge | Active conflicts |
| `helios.web_learning.fetch_count` | Counter | Web fetches performed |
| `helios.web_learning.staging_count` | Gauge | Units in staging area |
| `helios.plugin.invocation_duration` | Histogram | Plugin call duration by plugin ID |
| `helios.memory.working_memory_size` | Gauge | Current WME count |
| `helios.inference.depth` | Histogram | Backward chaining depth reached |

### 28.3 Structured Logging with Trace Correlation

All HELIOS log records include the active trace ID and span ID, enabling correlation between logs and traces:

```omni
struct HeliosLogRecord:
    timestamp:      Timestamp
    level:          LogLevel       # Trace | Debug | Info | Warn | Error
    message:        String
    trace_id:       Option<[u8; 16]>
    span_id:        Option<[u8; 8]>
    module:         String         # e.g., "brain.rete", "web_learning.stage4"
    attributes:     HashMap<String, String>
```

---

## 29. HELIOS Privacy — Differential Privacy and Federated Knowledge

### 29.1 Differential Privacy for Knowledge Queries

When HELIOS exposes knowledge through plugins, APIs, or federated sync, sensitive information may leak through query responses. Differential privacy (DP) adds calibrated noise to aggregate query results to prevent individual fact inference:

```omni
struct DPConfig:
    epsilon:            f32         # Privacy budget (lower = more private, default 1.0)
    delta:              f32         # Probability of privacy breach (default 1e-5)
    mechanism:          DPMechanism
    per_query_budget:   f32         # Fraction of total epsilon consumed per query

enum DPMechanism:
    Laplace             # Additive Laplace noise for numeric queries
    Gaussian            # Additive Gaussian noise (requires delta > 0)
    Exponential         # For non-numeric query results (selection from candidates)

# Example: "How many facts about topic X have confidence > 70?"
fn dp_count_query(
    store: &KnowledgeStore,
    filter: &WhereClause,
    config: &DPConfig,
) -> Result<f64, PrivacyError> {
    let true_count = store.count(filter)?;
    let noise = laplace_noise(1.0 / config.epsilon);  # Sensitivity = 1 for counting queries
    Ok(true_count as f64 + noise)
}
```

### 29.2 Sensitivity Classification for DP

Not all facts require DP protection. The `InformationClass` (Section 22) determines which facts contribute to DP accounting:

```
InformationClass    DP Requirement
Public              No DP needed — results are exact
Sensitive           DP applied to aggregate queries involving these facts
Confidential        DP applied with stricter epsilon (epsilon/2)
Restricted          Not included in any query results, DP moot
Hazardous           Not included in any query results, DP moot
```

### 29.3 Privacy Budget Tracking

HELIOS maintains a global privacy budget tracker to ensure cumulative queries do not exceed the configured privacy guarantee:

```omni
struct PrivacyBudget:
    total_epsilon:      f32
    consumed_epsilon:   f32
    total_delta:        f32
    consumed_delta:     f32
    query_log:          Vec<PrivacyQueryLog>

struct PrivacyQueryLog:
    query_hash:         [u8; 8]
    epsilon_spent:      f32
    timestamp:          Timestamp
    actor:              String     # Who issued the query (plugin, federation peer, etc.)
```

When the budget is exhausted, further queries against sensitive data return `PrivacyBudgetExhausted` — the system refuses to answer rather than compromise privacy.

---

## 30. OmniCrypt — Post-Quantum Cryptography Readiness

### 30.1 Hybrid Key Encapsulation

As quantum computing advances threaten classical public-key cryptography, OmniCrypt adopts a hybrid approach: pairing the existing XChaCha20-based symmetric encryption (Section 3) with a post-quantum Key Encapsulation Mechanism (KEM) for key exchange in federated sync and cross-instance scenarios.

```omni
struct HybridKEM:
    classical_kem:      X25519             # Existing ECDH key exchange
    pq_kem:             MLKEM768           # NIST FIPS 203 (formerly CRYSTALS-Kyber)
    combiner:           KEMCombiner

enum PQKEMAlgorithm:
    MLKEM512            # ML-KEM-512 — fastest, NIST Security Level 1
    MLKEM768            # ML-KEM-768 — recommended default, NIST Security Level 3
    MLKEM1024           # ML-KEM-1024 — highest security, NIST Security Level 5
    HQC128              # HQC — code-based alternative for algorithm diversity (NIST 2025)

fn hybrid_encapsulate(
    peer_public_classical: &[u8; 32],
    peer_public_pq: &[u8],
) -> Result<(Vec<u8>, [u8; 32]), CryptoError> {
    let (ct_classical, ss_classical) = x25519_encapsulate(peer_public_classical)?;
    let (ct_pq, ss_pq) = mlkem_encapsulate(peer_public_pq)?;
    
    // Combine shared secrets: both must be compromised for the combined key to be broken
    let combined_secret = blake3_hash(&[ss_classical, ss_pq].concat());
    let combined_ciphertext = [ct_classical, ct_pq].concat();
    
    Ok((combined_ciphertext, combined_secret))
}
```

### 30.2 Committing AEAD with AEGIS

The v3 OmniCrypt construction adds commitment via BLAKE3 XOR with Poly1305 (Section 3.2). Research shows AEGIS-256 is a natively key-committing AEAD with superior performance (up to 2× faster than AES-GCM on AES-NI hardware). HELIOS adds AEGIS as an alternative cipher backend:

```omni
enum AEADBackend:
    OmniStream24        # v3 default: modified XChaCha20-24 + committed Poly1305
    AEGIS256            # v4 addition: natively committing, AES-NI accelerated
    Ascon128a           # v4 addition: lightweight committing AEAD for embedded/constrained targets

# Selection logic:
fn select_aead_backend() -> AEADBackend {
    if cpu_supports_aes_ni() {
        AEADBackend::AEGIS256    // 2× faster than OmniStream on AES-NI hardware
    } else {
        AEADBackend::OmniStream24  // Constant-time without hardware support
    }
}
```

### 30.3 Post-Quantum Signature Scheme for Plugin Verification

Plugin signatures (Section 10.3) currently use Ed25519. For post-quantum resistance, HELIOS adds ML-DSA (formerly CRYSTALS-Dilithium) as a hybrid signature:

```omni
struct HybridSignature:
    ed25519_sig:        [u8; 64]
    mldsa_sig:          Vec<u8>          # ML-DSA-65 signature (NIST FIPS 204)
    combined_valid:     bool             # Both must verify

fn verify_plugin_signature(manifest_hash: &[u8; 32], sig: &HybridSignature, pubkeys: &HybridPublicKey) -> bool {
    let ed25519_ok = ed25519_verify(manifest_hash, &sig.ed25519_sig, &pubkeys.ed25519_pk);
    let mldsa_ok = mldsa_verify(manifest_hash, &sig.mldsa_sig, &pubkeys.mldsa_pk);
    ed25519_ok && mldsa_ok  // Both required — defense in depth
}
```

---

## 31. OmniPack — Next-Generation Compression Enhancements

### 31.1 Configurable Compressor Graph (OpenZL-Inspired)

Instead of a fixed four-stage pipeline (Section 2.2), v4 models OmniPack as a dynamically configurable **directed acyclic graph (DAG) of compression stages**. Different data types route through different subsets of stages:

```omni
struct CompressorGraph:
    stages:             Vec<CompressionStage>
    edges:              Vec<(usize, usize)>       # DAG of stage connections
    data_type_routes:   HashMap<DataType, Vec<usize>>  # Which stages to use for each data type

enum CompressionStage:
    OPT(OPTConfig)          # Stage 0: Domain-Specific Pre-Transform (existing)
    OML(OMLConfig)          # Stage 1: LZ Dictionary Matching (existing)
    OAC(OACConfig)          # Stage 2: Arithmetic Coding (existing)
    OIF(OIFConfig)          # Stage 3: Integrity Framing (existing)
    BPE(BPEConfig)          # Stage NEW: Byte Pair Encoding for repeated token patterns
    DeltaFilter(DeltaConfig) # Stage NEW: Delta filter for time-series confidence arrays
    RLE(RLEConfig)          # Stage NEW: Run-length encoding for sparse flag arrays

enum DataType:
    KnowledgeStorePage      # Route: OPT → OML → OAC → OIF (existing default)
    ConfidenceArray         # Route: DeltaFilter → RLE → OAC → OIF (new: 30% better ratio)
    ExperienceLogPage       # Route: BPE → OML → OAC → OIF (new: 15% better ratio)
    PluginBytecode          # Route: OML → OAC → OIF (skip OPT — not schema-aware)
    FederationPayload       # Route: OPT → OML → OAC → OIF (same as knowledge pages)
```

### 31.2 Searchable Compression

Inspired by Crystal (domain-specific log compression), OmniPack supports Bloom filter indexing over compressed data to answer "does this page contain a fact about subject X?" without decompression:

```omni
struct SearchableCompressedPage:
    compressed_data:    Vec<u8>              # Standard OmniPack compressed payload
    subject_bloom:      BloomFilter          # Bloom filter over subject strings in this page
    predicate_bloom:    BloomFilter          # Bloom filter over predicate strings
    keyword_bloom:      BloomFilter          # Bloom filter over keyword tokens
    bloom_false_positive_rate: f32           # Target FPR (default 0.01)

fn page_may_contain_subject(page: &SearchableCompressedPage, subject: &str) -> bool {
    page.subject_bloom.may_contain(subject)
    // If true: decompress and search. If false: guaranteed absent — skip this page.
}
```

This avoids decompressing cold-tier pages during knowledge store queries, improving query latency for large stores by 5–10× on average.

---

## 32. OQL — Query Optimization and Execution Planning

### 32.1 Cost-Based Query Planner

The OQL query engine (Section 21.2) gains a cost-based query planner that selects the most efficient execution strategy based on index statistics:

```omni
struct QueryPlan:
    steps:              Vec<QueryStep>
    estimated_cost:     f64              # Estimated I/O + CPU cost units
    index_usage:        Vec<IndexName>   # Indices used in this plan

enum QueryStep:
    IndexScan(IndexName, ScanRange)      # Use a specific index
    SequentialScan(PageRange)            # Full page scan (last resort)
    BloomFilterPrecheck(PageId)          # Check Bloom filter before decompressing
    HashJoin(QueryStep, QueryStep)       # Join two intermediate results
    MergeJoin(QueryStep, QueryStep)      # Sort-merge join on sorted index results
    Filter(Predicate)                    # Apply filter to intermediate result
    Limit(u32)                           # Stop after N results

fn plan_query(query: &OqlQuery, stats: &IndexStatistics) -> QueryPlan {
    // 1. Generate candidate plans (index scan vs. sequential scan for each predicate)
    // 2. Estimate cost for each plan using index cardinality and selectivity statistics
    // 3. Select lowest-cost plan
    // 4. Apply Bloom filter prechecks for cold-tier pages
}
```

### 32.2 Index Statistics Maintenance

The query planner requires up-to-date statistics about index cardinality, distribution, and selectivity:

```omni
struct IndexStatistics:
    subject_cardinality:    u64          # Number of distinct subjects
    predicate_cardinality:  u64          # Number of distinct predicates
    total_units:            u64          # Total InformationUnit count
    accuracy_distribution:  [u64; 5]     # Count per AccuracyStatus
    confidence_histogram:   [u64; 10]    # Count per 10-point confidence bucket
    domain_cardinality:     u64          # Number of distinct domains
    last_updated_at:        Timestamp

# Statistics are updated incrementally with each write operation
# and fully recalculated during page compaction
```

### 32.3 Parallel Query Execution

For queries spanning multiple pages, the query engine parallelizes page reads and decompression:

```omni
fn execute_parallel_query(plan: &QueryPlan, store: &KnowledgeStore) -> Vec<InformationUnit> {
    scope {
        let page_results: Vec<_> = plan.pages_to_scan()
            .par_iter()
            .map(|page_id| {
                let page = store.read_page(page_id).await;
                let decompressed = omnipack::decompress(&page.data)?;
                plan.apply_filters(&decompressed)
            })
            .collect();
        
        merge_results(page_results, plan.sort_order)
    }
}
```

---

## 33. HELIOS Brain — Adaptive Cognitive Learning and RETE Optimizations

### 33.1 RETE-NT Optimizations

The v3 RETE network (Section 9.3) is enhanced with optimizations from Rete-NT research:

**Alpha-Node Hashing:** Instead of linear scans through alpha nodes, alpha tests are indexed by a hash of the tested field + operator + value. When a WME enters the network, the system looks up only the alpha nodes whose hash matches, reducing alpha propagation from O(N) to O(1) average.

```omni
struct AlphaNetwork:
    hash_index:     HashMap<AlphaTestHash, Vec<AlphaNodeId>>   # Hash-indexed alpha nodes
    fallback_nodes: Vec<AlphaNodeId>                           # Nodes that can't be hashed (e.g., range tests)

fn propagate_alpha(wme: &WorkingMemoryElement, network: &AlphaNetwork) {
    let hash = compute_alpha_hash(wme);
    for node_id in network.hash_index.get(&hash).iter().flatten() {
        evaluate_alpha_node(node_id, wme);
    }
    for node_id in &network.fallback_nodes {
        evaluate_alpha_node(node_id, wme);
    }
}
```

**Beta-Node Indexing:** Beta join nodes maintain hash indices on their binding variables. When a new partial match arrives from the left memory, the join looks up matching entries in the right memory via hash index rather than scanning all entries:

```omni
struct BetaJoinNode:
    left_index:     HashMap<BindingValue, Vec<PartialMatch>>   # Index on join variable
    right_index:    HashMap<BindingValue, Vec<WorkingMemoryElement>>
    join_spec:      JoinSpec
```

### 33.2 Adaptive Rule Compilation

The pattern learner (Section 9.9) is enhanced to not only propose new rules but also optimize existing ones based on runtime statistics:

```omni
struct RulePerformanceStats:
    rule_id:            RuleId
    activation_count:   u64          # How many times this rule has been activated
    firing_count:       u64          # How many times this rule has actually fired
    avg_match_time_us:  f64          # Average time to match this rule's conditions
    useful_fire_ratio:  f32          # Fraction of firings that led to used conclusions

# Rules with high activation but low firing are candidates for condition reordering
# Rules with low useful_fire_ratio are candidates for salience adjustment or deactivation
```

### 33.3 Interactive Task Learning

Inspired by SOAR's interactive task learning, HELIOS gains the ability to learn new rules from natural language instruction:

```omni
fn learn_from_instruction(instruction: &str) -> Result<ProposedRule, LearningError> {
    // Parse: "When a fact has confidence below 30 and source is web, mark it as requiring review"
    // ↓ Extract:
    //   Condition 1: unit.confidence < 30
    //   Condition 2: unit.source.source_type == WebLearningActive || WebLearningPassive
    //   Action: set unit.verification.status = PendingUserReview
    // ↓ Generate:
    ProposedRule {
        conditions: vec![
            AlphaTest { field: Confidence, operator: Less, value: 30 },
            AlphaTest { field: SourceType, operator: OneOf, value: [WebLearningActive, WebLearningPassive] },
        ],
        actions: vec![
            SetField { path: "verification.status", value: "PendingUserReview" },
        ],
        proposed_salience: 50,
        requires_approval: true,   // User must approve before this rule goes live
    }
}
```

---

## 34. Omni Language — Advanced Type Features and Algebraic Effects

### 34.1 Algebraic Effects

Beyond the v3 effect annotation system (Section 15.1), Omni adopts full algebraic effects as a language-level abstraction for managing computational effects. Algebraic effects separate the **declaration** of an effect from its **handling**, enabling modular and composable effect management:

```omni
# Declare an effect
effect KnowledgeAccess:
    fn read_unit(id: u64) -> Option<InformationUnit>
    fn write_unit(unit: InformationUnit) -> u64

# Use an effect in a function — the function doesn't know HOW the effect is implemented
fn query_reasoning(goal: Goal) -> Result<Answer, QueryError> with KnowledgeAccess {
    let facts = KnowledgeAccess.read_unit(goal.subject_id)?;
    // ... reasoning logic ...
}

# Handle the effect — provides the actual implementation
handler InMemoryKnowledgeHandler for KnowledgeAccess:
    fn read_unit(id: u64) -> Option<InformationUnit> {
        self.memory_store.get(id)
    }
    fn write_unit(unit: InformationUnit) -> u64 {
        self.memory_store.insert(unit)
    }

handler DiskKnowledgeHandler for KnowledgeAccess:
    fn read_unit(id: u64) -> Option<InformationUnit> {
        self.page_reader.read(id)
    }
    fn write_unit(unit: InformationUnit) -> u64 {
        self.page_writer.append(unit)
    }

# Call with a specific handler — handler is "injected" at the call site
let answer = with InMemoryKnowledgeHandler::new(store) {
    query_reasoning(goal)?
};
```

**Benefits for HELIOS:**
- **Testing**: Use in-memory handlers during tests, disk handlers in production — same logic code.
- **Plugins**: Plugins declare effects they need; the host provides handlers (sandboxed).
- **Cognitive layers**: Each layer can use different knowledge access patterns without code changes.

### 34.2 Linear Types for Resource Safety

Omni adds linear types to guarantee that resources (file handles, network connections, page locks) are used exactly once:

```omni
# A linear type — must be consumed (used) exactly once
linear struct PageLock:
    page_id:    u32
    lock_token: u64

fn acquire_page_lock(page_id: u32) -> linear PageLock {
    // Acquire lock, return linear handle
}

fn write_to_page(lock: linear PageLock, data: &[u8]) -> Result<(), WriteError> {
    // Lock is consumed here — cannot be used again
    // Compiler error if PageLock is dropped without being passed to write or release
}

fn release_page_lock(lock: linear PageLock) {
    // Explicit release — lock is consumed
}

// Compiler ERROR: PageLock was not consumed
fn bad_code() {
    let lock = acquire_page_lock(42);
    // Function exits without using `lock` — compile error: linear value not consumed
}
```

### 34.3 Const Generics for Compile-Time Array Sizing

Inspired by Rust's const generics evolution, Omni adds const generic parameters to types:

```omni
# Fixed-size buffer with compile-time known capacity
struct FixedBuffer<const N: usize>:
    data:   [u8; N]
    len:    usize

# BLAKE3 hash output is always 32 bytes — encode this in the type
type Blake3Hash = FixedBuffer<32>
type UUID = FixedBuffer<16>

# Page offset table with compile-time known unit count
struct PageOffsetTable<const UNIT_COUNT: u32>:
    offsets: [u32; UNIT_COUNT]

# Compile-time enforcement:
fn process_hash(hash: Blake3Hash) { ... }
let uuid: UUID = get_uuid();
process_hash(uuid)  // COMPILE ERROR: expected FixedBuffer<32>, got FixedBuffer<16>
```

---

## 35. HELIOS Knowledge — Cross-Instance Federation Protocol

### 35.1 Federation Wire Protocol

The v3 federation (Section 20.2) described high-level sync semantics. v4 specifies the complete wire protocol for bidirectional knowledge synchronization between HELIOS instances:

```omni
enum FederationMessage:
    # Handshake
    Hello { instance_id: [u8; 16], protocol_version: u16, capabilities: Vec<FederationCapability> }
    HelloAck { instance_id: [u8; 16], accepted_capabilities: Vec<FederationCapability> }

    # Sync negotiation
    SyncRequest { since_timestamp: Timestamp, domains: Option<Vec<String>> }
    SyncManifest { unit_count: u64, total_bytes: u64, checksum: [u8; 32] }
    SyncAccept
    SyncReject { reason: String }

    # Data transfer
    UnitBatch { batch_id: u32, units: Vec<SerializedUnit>, has_more: bool }
    UnitBatchAck { batch_id: u32, accepted: u32, rejected: u32, conflicts: Vec<u64> }

    # Conflict resolution
    ConflictReport { conflicts: Vec<FederationConflict> }
    ConflictResolution { resolutions: Vec<(u64, ConflictAction)> }

    # Finalization
    SyncComplete { units_transferred: u64, elapsed_ms: u64 }
    SyncError { error: String }

enum FederationCapability:
    DeltaSync               # Only transfer modified units
    CompressedTransfer      # OmniPack-compressed unit batches
    EncryptedTransfer       # OmniCrypt-encrypted unit batches
    EmbeddingAlignment      # Use KGE for entity alignment (Section 26.2)
    DifferentialPrivacy     # Apply DP to transferred aggregates (Section 29)
    PostQuantum             # Use hybrid KEM for key exchange (Section 30)
    MultipathQuic           # Multipath QUIC (RFC 9369) for bonded LAN/WAN transfer
    ZeroRttResume           # 0-RTT session resumption for reconnecting peers

struct FederationConflict:
    local_unit_id:      u64
    remote_unit_id:     u64
    conflict_type:      ConflictType       # SameSubjectDifferentObject | ContradictoryPolarity | VersionMismatch
    local_confidence:   u8
    remote_confidence:  u8
```

### 35.2 Conflict-Free Replicated Data Types (CRDT) for Metadata

While fact content conflicts require resolution (human or policy-based), metadata fields use CRDTs for automatic conflict-free merging. Delta-state CRDTs (§88) stream incremental updates over the QUIC federation channel rather than full state snapshots:

```omni
# G-Counter CRDT for corroboration_score across instances
struct GCounter:
    counts: HashMap<[u8; 16], u64>          # instance_id → local count

impl GCounter:
    fn increment(&mut self, instance_id: [u8; 16]) {
        *self.counts.entry(instance_id).or_insert(0) += 1;
    }
    fn merge(&mut self, other: &GCounter) {
        for (id, count) in &other.counts {
            let local = self.counts.entry(*id).or_insert(0);
            *local = max(*local, *count);
        }
    }
    fn value(&self) -> u64 {
        self.counts.values().sum()
    }

# LWW-Register CRDT for last_updated_at
struct LWWRegister<T>:
    value:      T
    timestamp:  Timestamp
    instance:   [u8; 16]

impl<T> LWWRegister<T>:
    fn merge(&mut self, other: &LWWRegister<T>) {
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.instance = other.instance;
        }
    }
```

### 35.3 Federation Security

All federation communication is secured with:
1. **Mutual TLS** with certificate pinning per trusted peer.
2. **Hybrid KEM** key exchange (Section 30.1) for forward secrecy.
3. **Per-batch BLAKE3 authentication** to detect tampering.
4. **Rate limiting** to prevent federation flooding attacks.
5. **Differential privacy** on aggregate statistics shared during sync negotiation.

---

## Appendix C — Phase Implementation Sequence (v4.0 Additions)

The v4.0 additions extend the existing phase sequence (Appendix A) with additional implementation phases:

**Phase J — Causal and Probabilistic Reasoning**
1. Implement `CausalLink` structure and causal graph construction.
2. Implement Pearl's do-operator and graph surgery algorithm.
3. Implement Bayesian Network engine with belief propagation.
4. Implement causal discovery (PC algorithm).
5. Integrate BN with confidence recalculation pipeline.
6. Acceptance test: causal chain from 3 facts, interventional query produces correct delta.

**Phase K — Knowledge Graph Embeddings**
1. Implement TransE embedding training.
2. Implement link prediction API.
3. Implement entity alignment for federation.
4. Background training scheduled via Deep Thought layer.
5. Acceptance test: link prediction recall@10 ≥ 0.70 on test knowledge store.

**Phase L — WASM Plugin Runtime**
1. Integrate wasmtime or wasmer as WASM engine.
2. Implement WIT-based host/plugin interface.
3. Implement WASI capability granting and zero-trust model.
4. Port example plugin from OVM to WASM.
5. Acceptance test: plugin operates within memory and instruction limits; denied capabilities fail cleanly.

**Phase M — Observability and Privacy**
1. Implement OpenTelemetry tracing integration.
2. Implement cognitive metrics exporter.
3. Implement differential privacy engine for aggregate queries.
4. Implement privacy budget tracking.
5. Acceptance test: query trace has correct parent-child spans; DP queries satisfy ε guarantee.

**Phase N — Post-Quantum and Compression Upgrades**
1. Implement ML-KEM-768 hybrid encapsulation.
2. Implement AEGIS-256 AEAD backend.
3. Implement compressor graph with data-type routing.
4. Implement searchable compression (Bloom filter pages).
5. Acceptance test: hybrid KEM produces shared secret; Bloom filter precheck has ≤ 1% FPR.

**Phase O — Query Optimization and Adaptive Learning**
1. Implement cost-based query planner.
2. Implement index statistics maintenance.
3. Implement parallel query execution.
4. Implement RETE alpha-node hashing and beta-node indexing.
5. Implement interactive task learning (natural language → rule).
6. Acceptance test: query planner selects index scan over sequential scan for selective queries.

---

## Appendix D — Acceptance Test Specifications (v4.0 Additions)

**Phase J acceptance:** Create a causal graph with 5 causal links. Run an interventional query that sets one variable. Assert the post-intervention confidence of downstream variables changes according to the causal chain. Run Bayesian inference with 3 evidence nodes. Assert posterior probabilities sum to 1.0 and match hand-computed values within ±0.01.

**Phase K acceptance:** Train TransE embeddings on a knowledge store with 200 entities and 500 relations. Predict top-10 tail entities for a given (head, relation) pair. Assert at least 7 of the actual tails appear in the top-10 predictions (recall@10 ≥ 0.70). Run entity alignment between two stores with 50 shared entities using different names. Assert ≥ 40 correct alignments.

**Phase L acceptance:** Load a WASM plugin compiled from a simple Rust program. Invoke the plugin's query function. Assert the plugin receives the expected host API calls and returns valid results. Attempt a denied capability (network access not granted). Assert the call fails with a permission error without crashing the host.

**Phase M acceptance:** Process 10 queries with OpenTelemetry tracing enabled. Assert each query has a root span with child spans for each cognitive layer. Export traces to a JSON file. Assert all spans have valid trace IDs and parent references. Run 1000 DP count queries with ε=1.0. Assert the noise distribution approximates Laplace(1.0) and no individual fact can be inferred from the aggregate results.

**Phase N acceptance:** Perform hybrid KEM key exchange between two HELIOS instances. Assert both derive identical shared secrets. Encrypt and decrypt a knowledge store page using AEGIS-256. Assert decrypted content matches original. Compress a knowledge store page with searchable compression. Query the Bloom filter for 10 known subjects (all should return true) and 10 absent subjects (≤ 1 should return true).

**Phase O acceptance:** Create a knowledge store with 10,000 units. Run an OQL query `FIND facts WHERE subject="test" AND confidence > 70`. Assert the query planner uses the subject index rather than sequential scan. Assert query completes in < 50ms. Teach HELIOS a new rule via natural language instruction. Assert the proposed rule has correct conditions and actions. Assert the rule fires correctly after approval and matching fact insertion.

---

*End of v4.0 sections — v5.0 sections follow.*


### 35.4 Peer Discovery

HELIOS federation peers are discovered through two mechanisms:

1. **Local network (zero-config):** Use mDNS/DNS-SD (multicast DNS with Service Discovery, the same mechanism underlying Apple Bonjour and Linux Avahi) to announce and discover HELIOS instances on the local network. Each instance registers a `_helios._tcp.local` service record containing its federation endpoint (QUIC port) and instance UUID. Discovery is automatic and requires no user configuration.

2. **Static registry (fallback):** Known remote peer endpoints are configured in `helios.toml` under `[federation.peers]` for WAN scenarios where mDNS is not available:

```toml
[federation.peers]
endpoints = [
    "quic://192.168.1.50:4433",
    "quic://helios-office.example.com:4433",
]
```

**Out of scope (future extension):** STUN/TURN for NAT traversal in complex network topologies. Current federation assumes direct reachability between peers (LAN or pre-configured WAN endpoints).

---

## 36. HELIOS Reasoning — GNN-Based Knowledge Graph Reasoning

> **Architectural Note:** GNN models referenced in this section are trained **offline** as auxiliary learned tools and loaded as frozen, read-only inference artifacts. They do not perform gradient-based training at runtime. The cognitive reasoning pipeline consumes GNN outputs as pre-computed relational features, analogous to a compiled index. This is distinct from the removed §95 (MARL), which proposed runtime gradient-based policy learning integrated into the cognitive pipeline.


### 36.1 Graph Neural Network Reasoning Layer

The existing knowledge graph reasoning relies on symbolic approaches (RETE forward chaining, backward chaining, graph algorithms). A Graph Neural Network (GNN) reasoning layer adds a learned, sub-symbolic complement that excels at inductive reasoning over unseen entities and multi-hop relation inference:

```omni
struct GNNReasoningConfig:
    architecture:       GNNArchitecture
    num_layers:         u8               # Number of message-passing layers (default 3)
    hidden_dim:         u16              # Hidden representation dimension (default 256)
    attention_heads:    u8               # For GAT/GraphTransformer (default 4)
    dropout:            f32              # Dropout rate (default 0.1)
    aggregation:        AggregationType  # How neighbor messages are combined

enum GNNArchitecture:
    GCN                 # Graph Convolutional Network — fast, isotropic
    GAT                 # Graph Attention Network — learned attention weights per edge
    GraphSAGE           # Inductive: samples and aggregates from neighbors
    RGCN                # Relational GCN: relation-specific weight matrices
    CompGCN             # Composition-based: jointly embeds entities and relations
    GraphTransformer    # Transformer attention over graph structure

enum AggregationType:
    Mean                # Average neighbor representations
    Sum                 # Sum neighbor representations
    Max                 # Max-pool neighbor representations
    Attention           # Weighted sum via learned attention (GAT/Transformer)
```

### 36.2 Inductive Link Prediction

Unlike TransE/RotatE (Section 26) which are transductive (require retraining for new entities), the GNN layer supports inductive link prediction — predicting relationships for entities never seen during training:

```omni
struct InductivePrediction:
    entity_id:          u64
    context_subgraph:   Vec<(u64, String, u64)>    # Local k-hop subgraph around entity
    predicted_links:    Vec<(String, u64, f32)>    # (relation, target_id, confidence)

fn predict_links_inductive(
    model: &GNNModel,
    entity: u64,
    store: &KnowledgeStore,
    k_hops: u8,                                    # Subgraph radius (default 2)
) -> Vec<InductivePrediction>
```

### 36.3 GNN-RAG Integration

Combining GNN retrieval with the knowledge store query engine (GNN-RAG) enables multi-hop question answering by using the GNN to identify relevant subgraphs and then reasoning over them:

```omni
struct GNNRAGQuery:
    question:           String
    max_hops:           u8              # Maximum reasoning hops (default 3)
    top_k_paths:        u8              # Number of reasoning paths to retrieve (default 5)

struct GNNRAGResult:
    answer:             String
    supporting_paths:   Vec<Vec<(u64, String, u64)>>   # Reasoning paths through the KG
    confidence:         u8
    explanation:        String          # NLG explanation of the reasoning path
```

---

## 37. HELIOS Knowledge — GraphRAG Retrieval-Augmented Knowledge

### 37.1 Hybrid Retrieval Architecture

HELIOS employs a hybrid retrieval strategy combining dense vector search with structured knowledge graph traversal for maximum recall and precision:

```omni
struct HybridRetriever:
    vector_index:       VectorIndex          # Dense embedding index for semantic search
    kg_traverser:       KGTraverser          # Structured graph traversal engine
    fusion_strategy:    FusionStrategy       # How to combine results from both retrievers
    reranker:           Option<Reranker>     # Optional learned reranker for result ordering

enum FusionStrategy:
    ReciprocalRankFusion { k: u32 }          # RRF: score = Σ(1/(k + rank_i))
    WeightedLinear { vector_weight: f32, kg_weight: f32 }
    LearnedFusion                            # Trained combiner model

struct VectorIndex:
    embeddings:         HashMap<u64, Vec<f32>>   # unit_id → dense vector
    index_type:         VectorIndexType
    distance_metric:    DistanceMetric

enum VectorIndexType:
    HNSW { m: u16, ef_construction: u16 }    # Hierarchical Navigable Small World
    IVFFlat { n_lists: u32 }                 # Inverted File with Flat quantization
    ProductQuantization { n_subvectors: u8 }  # Compressed vectors for memory efficiency

enum DistanceMetric:
    Cosine
    DotProduct
    Euclidean
```

### 37.2 Knowledge Graph Index Construction

The GraphRAG pipeline builds a hierarchical graph index from the knowledge store, enabling community-based summarization and global queries:

```omni
struct GraphRAGIndex:
    communities:        Vec<KnowledgeCommunity>    # Detected knowledge communities
    community_summaries: HashMap<usize, String>    # NLG summaries per community
    global_summary:     String                     # System-wide knowledge summary
    hierarchy_levels:   u8                         # Levels of community hierarchy (default 3)

struct KnowledgeCommunity:
    community_id:       usize
    member_unit_ids:    Vec<u64>
    central_entities:   Vec<u64>                   # Most connected entities in this community
    density:            f32                        # Edge density within community
    parent_community:   Option<usize>              # Parent in hierarchy

fn build_graph_rag_index(store: &KnowledgeStore) -> Result<GraphRAGIndex, IndexError> {
    // 1. Extract entity-relation graph from knowledge store
    // 2. Run Leiden community detection algorithm
    // 3. Build hierarchy by recursively clustering communities
    // 4. Generate NLG summaries for each community
    // 5. Create global summary from top-level communities
}
```

### 37.3 Adaptive Query Routing

The retriever automatically selects the optimal strategy based on query type:

```
Query Type              Retrieval Strategy          Rationale
Point lookup            KG traversal only           Direct fact retrieval, no semantic gap
Semantic similarity     Vector search only          Embedding distance is sufficient
Multi-hop reasoning     GNN-RAG (Section 36.3)      GNN paths + chain reasoning
Global/thematic         GraphRAG community search   Community summaries answer global queries
Hybrid complex          Full fusion pipeline        Combines all strategies for maximum recall
```

---

## 38. HELIOS Knowledge — Transactional Store with MVCC

### 38.1 Multi-Version Concurrency Control

The knowledge store (Section 7) gains full MVCC support, allowing concurrent readers and writers without blocking. Every write creates a new version of the InformationUnit rather than overwriting in place:

```omni
struct VersionedUnit:
    unit_id:            u64
    version:            u64              # Monotonically increasing version number
    created_at:         Timestamp
    created_by_txn:     TxnId            # Transaction that created this version
    expired_by_txn:     Option<TxnId>    # Transaction that superseded this version (None = current)
    data:               InformationUnit  # The actual unit data

struct Transaction:
    txn_id:             TxnId
    start_timestamp:    Timestamp
    snapshot_version:   u64              # Global version counter at txn start
    read_set:           HashSet<(u64, u64)>   # (unit_id, version) — versions read
    write_set:          HashMap<u64, VersionedUnit>  # unit_id → new version written
    status:             TxnStatus

enum TxnStatus:
    Active
    Committed
    Aborted
    Preparing                            # In two-phase commit (federation)

fn begin_transaction(store: &KnowledgeStore) -> Transaction
fn read_unit(txn: &Transaction, unit_id: u64) -> Option<&InformationUnit>  # Reads from snapshot
fn write_unit(txn: &mut Transaction, unit: InformationUnit) -> u64
fn commit(txn: Transaction, store: &mut KnowledgeStore) -> Result<(), ConflictError>
fn abort(txn: Transaction, store: &mut KnowledgeStore)
```

### 38.2 Serializable Snapshot Isolation (SSI)

To guarantee serializable consistency while maintaining MVCC performance, the store implements Serializable Snapshot Isolation with write-skew detection:

```omni
struct SSIConflictDetector:
    rw_conflicts:       Vec<(TxnId, TxnId, u64)>   # (reader_txn, writer_txn, unit_id)
    dangerous_structures: Vec<(TxnId, TxnId, TxnId)> # Potential serialization anomalies

fn check_serializable(detector: &SSIConflictDetector, committing_txn: TxnId) -> Result<(), SerializationError> {
    // Detect "dangerous structures": T1 →rw T2 →rw T3 where T1 committed before T3
    // If found, abort T3 to prevent write-skew anomaly
}
```

### 38.3 Write-Ahead Log for Crash Recovery

All knowledge store mutations are logged to a write-ahead log (WAL) before being applied, ensuring crash recovery to a consistent state:

```omni
struct WALEntry:
    lsn:                u64              # Log Sequence Number
    txn_id:             TxnId
    operation:          WALOperation
    timestamp:          Timestamp
    checksum:           u32              # CRC32 of this entry

enum WALOperation:
    InsertUnit { unit_id: u64, data: Vec<u8> }
    UpdateUnit { unit_id: u64, old_version: u64, new_version: u64, data: Vec<u8> }
    DeleteUnit { unit_id: u64, version: u64 }
    CommitTxn { txn_id: TxnId }
    AbortTxn { txn_id: TxnId }
    Checkpoint { active_txns: Vec<TxnId> }

struct WALConfig:
    sync_mode:          WALSyncMode
    segment_size:       u32              # Bytes per WAL segment (default 16 MB)
    checkpoint_interval: u32             # Pages between checkpoints (default 1000)

enum WALSyncMode:
    Fsync                                # Durable: fsync after every commit
    GroupCommit { interval_ms: u16 }     # Batch: fsync every N ms (better throughput)
    NoSync                               # Unsafe: OS decides when to flush (testing only)
```


### 38.4 MVCC Version Garbage Collection (Vacuum)

MVCC creates a new `VersionedUnit` on every write. Without a vacuum process, expired versions accumulate indefinitely, wasting storage and degrading scan performance. Vacuum operations are WAL-logged (§7.4) to ensure crash consistency — if a crash occurs during vacuum, the WAL replay restores the pre-vacuum state.

```omni
struct VacuumConfig:
    min_retained_versions:  u32    # Always keep at least N versions (default: 2)
    max_version_age:        Duration  # Versions older than this are candidates (default: 7 days)
    vacuum_interval:        Duration  # How often the vacuum task runs (default: 1 hour)

struct VacuumTask:
    config:             VacuumConfig
    oldest_active_txn:  TransactionId  # Snapshot of the oldest currently-running transaction

fn vacuum(store: &mut KnowledgeStore, config: &VacuumConfig):
    let oldest_snapshot = store.get_oldest_active_transaction_snapshot();
    
    for page in store.pages_mut():
        let reclaimable = page.versions()
            .filter(|v| v.expired_by_txn.is_some())
            .filter(|v| v.expired_by_txn.unwrap() < oldest_snapshot)
            .filter(|v| v.version_count_for_unit() > config.min_retained_versions)
            .collect::<Vec<_>>();
        
        if reclaimable.len() > 0:
            page.remove_versions(reclaimable);
            page.compact();  // Reclaim freed space within the page
    
    store.update_freelist();  // Update page freelist with any fully-empty pages
```

**Acceptance Criteria:**
- After 10,000 write operations creating new versions, vacuum reclaims at least 80% of expired versions that are older than `oldest_active_txn`.
- Versions referenced by any active transaction snapshot are never reclaimed (safety invariant).
- Vacuum runs as a background maintenance task (§101 `TaskPriority::Maintenance`) and does not block interactive queries.

---

## 39. HELIOS Brain — Natural Language Understanding Pipeline

### 39.1 Query Understanding Pipeline

User queries pass through a structured NLU pipeline before reaching the cognitive layers:

```omni
struct NLUPipeline:
    tokenizer:          Tokenizer
    intent_classifier:  IntentClassifier
    entity_extractor:   EntityExtractor
    context_manager:    ContextManager
    query_rewriter:     QueryRewriter

struct NLUResult:
    original_text:      String
    intent:             QueryIntent
    entities:           Vec<ExtractedEntity>
    context:            ConversationContext
    rewritten_query:    Option<String>       # Clarified version of the query

enum QueryIntent:
    FactLookup                               # "What is X?"
    RelationQuery                            # "How are X and Y related?"
    CausalQuery                              # "Why does X cause Y?"
    TemporalQuery                            # "When did X change?"
    CountingQuery                            # "How many X have property Y?"
    ComparisonQuery                          # "Is X bigger than Y?"
    HypotheticalQuery                        # "What if X were true?"
    TeachingIntent                           # "Remember that X is Y"
    CorrectionIntent                         # "Actually, X is not Y"
    MetaQuery                                # "How confident are you about X?"

struct ExtractedEntity:
    text:               String               # Surface form in the query
    entity_type:        EntityType           # Person, Place, Concept, Date, Number, etc.
    resolved_unit_id:   Option<u64>          # Linked to existing InformationUnit if possible
    confidence:         u8

enum EntityType:
    Person | Organization | Location | Date | Time | Duration
    Number | Percentage | Currency | Concept | Event | Product
```

### 39.2 Context Manager

The context manager maintains conversational history to resolve references and ambiguities:

```omni
struct ConversationContext:
    session_id:         u64
    turn_history:       Vec<ConversationTurn>
    active_topic:       Option<u64>          # Current InformationUnit being discussed
    coreference_map:    HashMap<String, u64> # "it", "that" → resolved unit_id
    emotional_tone:     Option<EmotionalTone>

struct ConversationTurn:
    turn_id:            u32
    user_input:         String
    system_response:    String
    entities_mentioned: Vec<u64>

enum EmotionalTone:
    Neutral | Curious | Frustrated | Urgent | Casual
```

### 39.3 Privacy-Conscious Entity Recognition

Entity extraction respects privacy classifications (Section 22). PII entities detected in input are handled according to the information classification:

```omni
struct PIIDetectionConfig:
    detect_names:       bool     # Default true
    detect_emails:      bool     # Default true
    detect_phone:       bool     # Default true
    detect_addresses:   bool     # Default true
    action:             PIIAction

enum PIIAction:
    Anonymize                     # Replace PII with [REDACTED] tokens in logs
    Classify                      # Tag as Sensitive/Confidential, store normally
    Reject                        # Refuse to process PII-containing queries
```

---

## 40. HELIOS Compute — GPU Heterogeneous Offloading

### 40.1 Compute Backend Abstraction

HELIOS uses a unified compute backend abstraction layer that dispatches workloads to the most efficient available processor:

```omni
struct ComputeDispatcher:
    backends:           Vec<ComputeBackend>
    policy:             DispatchPolicy

enum ComputeBackend:
    CPU { threads: u16 }
    CUDADevice { device_id: u8, memory_mb: u32 }     # NVIDIA GPU via CUDA
    VulkanDevice { device_id: u8, memory_mb: u32 }    # Cross-platform GPU via Vulkan Compute
    MetalDevice { device_id: u8, memory_mb: u32 }     # Apple GPU via Metal
    NPU { device_id: u8 }                             # Neural Processing Unit

enum DispatchPolicy:
    PreferGPU                    # Use GPU when available, fallback to CPU
    EnergyOptimal                # Choose lowest energy cost backend (Section 45)
    LatencyOptimal               # Choose lowest latency backend
    Manual(ComputeBackend)       # User-specified backend
```

### 40.2 Offloadable Workloads

Specific HELIOS workloads are marked as GPU-offloadable:

```omni
enum OffloadableWorkload:
    KGETraining(KGEConfig)                   # Section 26: Embedding training
    GNNInference(GNNReasoningConfig)         # Section 36: GNN-based reasoning
    VectorIndexBuild(VectorIndexType)        # Section 37: HNSW/IVF construction
    BeliefPropagation(BayesianNetwork)       # Section 25: Large network inference
    BloomFilterBatch(Vec<String>)            # Section 31: Batch membership checks
    BatchEmbeddingLookup(Vec<u64>)           # Section 26: Batch cosine similarity

fn offload_workload(
    dispatcher: &ComputeDispatcher,
    workload: OffloadableWorkload,
) -> Result<ComputeResult, ComputeError> {
    let backend = dispatcher.select_backend(&workload);
    match backend {
        ComputeBackend::CUDADevice { .. } => execute_cuda(workload),
        ComputeBackend::VulkanDevice { .. } => execute_vulkan(workload),
        ComputeBackend::MetalDevice { .. } => execute_metal(workload),
        _ => execute_cpu(workload),
    }
}
```

### 40.3 Memory Management for GPU Offloading

For large knowledge graphs that exceed GPU memory, HELIOS implements chunked offloading with prefetching:

```omni
struct GPUMemoryManager:
    gpu_memory_total:   u64
    gpu_memory_used:    u64
    prefetch_queue:     VecDeque<MemoryChunk>
    eviction_policy:    EvictionPolicy

enum EvictionPolicy:
    LRU                                      # Least Recently Used
    LFU                                      # Least Frequently Used
    Priority                                 # Based on workload priority
```

---

## 41. Omni Language — Time-Travel Debugging and Record-Replay

### 41.1 Execution Recording

The Omni Virtual Machine (OVM) supports deterministic execution recording, capturing all non-deterministic inputs (I/O, timing, RNG) to enable exact replay:

```omni
struct ExecutionRecording:
    recording_id:       u64
    program_hash:       [u8; 32]             # BLAKE3 hash of the program binary
    start_timestamp:    Timestamp
    events:             Vec<RecordedEvent>
    total_instructions: u64

enum RecordedEvent:
    Syscall { id: u32, args: Vec<u64>, result: Vec<u8>, instruction_count: u64 }
    FileRead { fd: u32, data: Vec<u8>, instruction_count: u64 }
    FileWrite { fd: u32, len: u32, instruction_count: u64 }
    NetworkRecv { socket: u32, data: Vec<u8>, instruction_count: u64 }
    TimerRead { value: u64, instruction_count: u64 }
    RandomBytes { data: Vec<u8>, instruction_count: u64 }
    ThreadSwitch { from_tid: u32, to_tid: u32, instruction_count: u64 }

struct RecordingConfig:
    enabled:            bool
    max_size_mb:        u32              # Maximum recording size (default 1024 MB)
    compression:        bool             # Compress recording with OmniPack (default true)
    include_memory:     bool             # Include memory snapshots at checkpoints
    checkpoint_interval: u64             # Instructions between memory checkpoints
```

### 41.2 Reverse Execution

During replay, the debugger supports reverse execution commands:

```omni
enum DebugCommand:
    // Forward
    Continue
    StepForward
    StepOver
    StepOut

    // Reverse (time-travel)
    ReverseContinue                      # Run backward to previous breakpoint
    ReverseStep                          # Step back one instruction
    ReverseStepOver                      # Step back over the previous function call
    GotoInstruction(u64)                 # Jump to specific instruction count
    GotoTimestamp(Timestamp)             # Jump to specific wall-clock time

    // Search
    FindLastWrite { address: u64 }       # Find the last instruction that wrote to address
    FindLastCall { function: String }    # Find the last call to a named function
    WatchpointReverse { address: u64 }   # Run backward until this address was written
```

### 41.3 Checkpointed State Snapshots

For efficient reverse navigation over long recordings, periodic checkpoints store full VM state:

```omni
struct VMCheckpoint:
    instruction_count:  u64
    memory_snapshot:    Vec<u8>          # Compressed VM memory state
    register_state:     Vec<u64>        # All OVM registers
    stack_state:        Vec<u8>         # Stack contents
    heap_metadata:      Vec<(u64, u64)> # Allocated regions (start, size)

// To go backward: find nearest checkpoint before target, replay forward from there.
// With checkpoints every 10M instructions, reverse navigation is O(10M) worst case.
```

---

## 42. HELIOS Resilience — Chaos Testing and Fault Injection

### 42.1 Fault Injection Framework

HELIOS includes a built-in chaos testing framework for verifying system resilience under adverse conditions:

```omni
struct ChaosExperiment:
    experiment_id:      u64
    name:               String
    fault:              FaultType
    target:             ChaosTarget
    duration:           Duration
    blast_radius:       BlastRadius
    steady_state:       SteadyStateDefinition
    rollback_on_failure: bool

enum FaultType:
    PageCorruption { corruption_rate: f32 }          # Flip random bits in knowledge pages
    SlowIO { delay_ms: u32 }                         # Inject latency into page reads/writes
    NetworkPartition { peers: Vec<[u8; 16]> }        # Simulate federation disconnect
    PluginCrash { plugin_id: String }                # Kill a running plugin mid-execution
    MemoryPressure { limit_bytes: u64 }              # Restrict available memory
    ClockSkew { drift_ms: i64 }                      # Shift system clock (tests temporal reasoning)
    CPUStarvation { available_pct: u8 }              # Throttle CPU availability
    WALCorruption { segment: u32 }                   # Corrupt a WAL segment (tests recovery)
    RETEOverload { rules_per_sec: u64 }              # Flood RETE network with WMEs

enum ChaosTarget:
    KnowledgeStore
    BrainCognitiveLayers
    PluginRuntime
    FederationSyncEngine
    WebLearningPipeline

enum BlastRadius:
    SingleComponent                                  # Affect only the target
    SubSystem                                        # Affect target + direct dependencies
    FullSystem                                       # Affect everything (used sparingly)
```

### 42.2 Steady-State Verification

Each chaos experiment defines what "normal" looks like, and the framework continuously verifies the system remains within bounds during the experiment:

```omni
struct SteadyStateDefinition:
    metrics:            Vec<SteadyStateMetric>
    check_interval_ms:  u32

struct SteadyStateMetric:
    metric_name:        String           # e.g., "helios.query.duration"
    operator:           Comparison       # Less, Greater, Equal, InRange
    threshold:          f64              # e.g., 500.0 (ms)
    tolerance_pct:      f32              # Allowed deviation (default 10%)
```

### 42.3 AI-Driven Experiment Design

The chaos framework uses HELIOS's own reasoning capabilities to design targeted experiments:

```omni
fn suggest_chaos_experiments(
    architecture: &SystemArchitecture,
    incident_history: &Vec<IncidentLog>,
) -> Vec<ChaosExperiment> {
    // 1. Analyze system architecture for single points of failure
    // 2. Review past incidents for patterns
    // 3. Generate experiments targeting untested failure modes
    // 4. Rank by impact × likelihood × coverage gap
}
```

---

## 43. HELIOS Knowledge — Internationalization and Multilingual Support

### 43.1 Unicode-Aware Knowledge Storage

All text fields in InformationUnit and related structures are stored as Unicode (UTF-8). The knowledge store maintains locale metadata for multilingual operation:

```omni
struct LocaleMetadata:
    primary_locale:     Locale           # e.g., Locale::en_US
    content_locale:     Locale           # Locale of the fact's content text
    translation_status: TranslationStatus
    translations:       HashMap<Locale, TranslatedUnit>

struct TranslatedUnit:
    translated_content: String
    translator:         TranslationSource
    quality_score:      u8               # 0-100, confidence in translation quality
    last_updated:       Timestamp

enum TranslationSource:
    Human                                # Verified human translation
    MachineTranslation { model: String } # LLM or MT engine
    UserProvided                         # User supplied the translation

enum TranslationStatus:
    OriginalOnly                         # No translations exist
    PartiallyTranslated                  # Some target locales translated
    FullyTranslated                      # All configured target locales translated
    NeedsReview                          # Translation exists but may be stale
```

### 43.2 CLDR-Based Formatting

All date, time, number, and currency formatting uses Unicode CLDR rules:

```omni
struct CLDRFormatter:
    locale:             Locale
    calendar:           Calendar         # Gregorian, Japanese, Islamic, etc.
    number_system:      NumberSystem     # Latin, Arabic-Indic, Thai, etc.

fn format_timestamp(formatter: &CLDRFormatter, ts: Timestamp) -> String
fn format_number(formatter: &CLDRFormatter, n: f64) -> String
fn format_confidence(formatter: &CLDRFormatter, confidence: u8) -> String
// "85%" in en_US, "85 %" in fr_FR, "٨٥٪" in ar_EG
```

### 43.3 Cross-Lingual Entity Resolution

When ingesting facts from multilingual sources, the knowledge store identifies entities across languages:

```omni
struct CrossLingualResolver:
    alignment_model:    MultilingualEmbedding    # Multilingual sentence-transformers
    similarity_threshold: f32                    # Default 0.80
    disambiguation_rules: Vec<DisambiguationRule>

fn resolve_cross_lingual(
    resolver: &CrossLingualResolver,
    local_entity: &str,
    local_locale: Locale,
    store: &KnowledgeStore,
) -> Vec<(u64, f32)>   // Matching unit_ids with similarity scores
```

---

## 44. GUI Accessibility — WCAG Compliance and Assistive Technology

### 44.1 Accessibility Architecture

The HELIOS GUI (Section 12, WinUI 3) is designed for WCAG 2.2 AA compliance:

```omni
struct AccessibilityConfig:
    wcag_level:         WCAGLevel
    screen_reader:      ScreenReaderSupport
    keyboard_nav:       KeyboardNavConfig
    visual_config:      VisualAccessibilityConfig

enum WCAGLevel:
    A                   # Minimum: essential features accessible
    AA                  # Target: broad accessibility (HELIOS default)
    AAA                 # Maximum: highest level of accessibility

struct ScreenReaderSupport:
    aria_labels:        bool             # All interactive elements have ARIA labels
    live_regions:       bool             # Dynamic content updates announced
    landmark_roles:     bool             # Major sections marked with ARIA landmarks
    reading_order:      bool             # DOM order matches visual reading order
```

### 44.2 Keyboard Navigation

Every GUI feature is fully operable via keyboard:

```omni
struct KeyboardNavConfig:
    tab_order:          TabOrderMode
    focus_indicators:   FocusStyle
    skip_links:         bool             # Skip-to-main-content link at page top
    shortcut_keys:      HashMap<KeyCombo, GUIAction>

enum TabOrderMode:
    Automatic                            # Follow DOM/visual order
    Explicit(Vec<ElementId>)             # Custom tab order

struct FocusStyle:
    outline_color:      Color            # High-contrast outline (default: system accent)
    outline_width:      u8               # Pixels (default: 3)
    contrast_ratio:     f32              # Minimum 3:1 against adjacent colors (WCAG 2.4.13)

// Standard keyboard shortcuts
// F6:        Cycle between panes
// Ctrl+F:    Open search
// Ctrl+/:    Open command palette
// Escape:    Close modal/dialog
// Arrow keys: Navigate within knowledge graph visualization
```

### 44.3 High-Contrast Theme System

```omni
struct ThemeSystem:
    themes:             Vec<Theme>
    active_theme:       ThemeId
    respect_system:     bool             # Follow Windows High Contrast Mode

struct Theme:
    id:                 ThemeId
    name:               String
    mode:               ThemeMode
    colors:             ThemeColors

enum ThemeMode:
    Light
    Dark
    HighContrastLight
    HighContrastDark
    SystemDefault

struct ThemeColors:
    text_primary:       Color            # Must achieve 4.5:1 ratio vs background
    text_secondary:     Color            # Must achieve 4.5:1 ratio vs background
    background:         Color
    surface:            Color
    accent:             Color
    error:              Color
    border:             Color            # Must achieve 3:1 ratio vs background
    focus_ring:         Color            # Must achieve 3:1 ratio vs adjacent colors
```

---

## 45. HELIOS Operations — Green Computing and Energy Profiling

### 45.1 Energy Profiler

HELIOS tracks energy consumption across all subsystems to enable optimization and carbon-aware scheduling:

```omni
struct EnergyProfiler:
    sampling_interval_ms: u32           # How often to sample power (default 100)
    estimator:          EnergyEstimator
    budget:             Option<EnergyBudget>

enum EnergyEstimator:
    RAPL                                 # Intel Running Average Power Limit (hardware counters)
    SoftwareModel                        # Estimated from CPU utilization + instruction mix
    ExternalMeter                        # External power meter via API

struct EnergyProfile:
    total_joules:       f64
    per_subsystem:      HashMap<String, f64>  # "brain.rete" → joules, "web_learning" → joules
    per_query:          Vec<QueryEnergyEntry>
    co2_estimate_g:     f64              # Estimated CO₂ emissions (using grid carbon intensity)

struct QueryEnergyEntry:
    query_id:           u64
    joules:             f64
    cognitive_layer:    u8               # Which layer consumed most energy
    duration_ms:        f64
```

### 45.2 Carbon-Aware Scheduling

Background workloads (embedding training, compaction, web learning) are scheduled based on grid carbon intensity:

```omni
struct CarbonAwareScheduler:
    carbon_api:         CarbonIntensityAPI
    background_tasks:   Vec<SchedulableTask>
    threshold:          CarbonThreshold

enum CarbonIntensityAPI:
    ElectricityMaps { api_key: String, zone: String }
    WattTime { username: String, password: String }
    Static { gco2_per_kwh: f32 }         # Fixed estimate (offline operation)

struct CarbonThreshold:
    high_carbon_gco2:   f32              # Above this: defer non-urgent tasks (default 400)
    low_carbon_gco2:    f32              # Below this: run all background tasks (default 200)
```

### 45.3 Energy Budget Enforcement

Operators can set energy budgets per time period:

```omni
struct EnergyBudget:
    period:             Duration         # e.g., 24 hours
    max_joules:         f64              # Maximum energy consumption per period
    current_consumed:   f64
    policy_on_exceed:   EnergyExceedPolicy

enum EnergyExceedPolicy:
    ThrottleCognition                    # Reduce to L0/L1 responses only — skip deep reasoning
    DeferBackground                      # Stop background tasks, serve queries normally
    AlertOnly                            # Log alert but don't change behavior
    Shutdown                             # Graceful shutdown (extreme conservation)
```

---

## 46. HELIOS Brain — Multi-Agent Coordination Protocol

### 46.1 Agent Communication Interface

HELIOS implements the Agent Communication Protocol (ACP) for interoperation with external AI agents:

```omni
struct AgentCard:
    agent_id:           [u8; 16]
    name:               String
    description:        String
    capabilities:       Vec<AgentCapability>
    endpoint:           String           # HTTP/gRPC endpoint
    protocol_version:   u16
    authentication:     AgentAuth

enum AgentCapability:
    FactQuery                            # Can answer factual queries
    Reasoning { types: Vec<String> }     # Can perform specific reasoning types
    WebSearch                            # Can search the web
    CodeExecution                        # Can execute code
    ImageAnalysis                        # Can analyze images
    Translation { languages: Vec<Locale> } # Can translate between languages
    SpecializedDomain(String)            # Domain-specific expertise

enum AgentAuth:
    APIKey
    OAuth2 { token_url: String }
    MutualTLS
    None                                 # Trusted internal agents only
```

### 46.2 Task Delegation Protocol

HELIOS can delegate sub-tasks to specialized agents and aggregate their results:

```omni
struct AgentTask:
    task_id:            u64
    requester:          [u8; 16]         # Requesting agent ID
    task_type:          AgentTaskType
    payload:            String           # JSON-encoded task parameters
    deadline:           Option<Timestamp>
    priority:           u8               # 0-100

enum AgentTaskType:
    FactVerification { claim: String }                   # "Is this fact true?"
    InformationGathering { topic: String, depth: u8 }    # "Find information about X"
    TranslationRequest { text: String, target: Locale }  # "Translate X to Y"
    ReasoningRequest { goal: String, method: String }    # "Reason about X using method Y"

struct AgentTaskResult:
    task_id:            u64
    responder:          [u8; 16]
    status:             TaskResultStatus
    result:             Option<String>   # JSON-encoded result
    confidence:         u8
    sources:            Vec<String>      # Source citations from the responding agent

enum TaskResultStatus:
    Completed
    PartialResult
    Failed { error: String }
    Timeout
    Declined { reason: String }
```

### 46.3 Swarm Intelligence for Group Reasoning

When a query requires capabilities beyond HELIOS's own, it orchestrates multiple agents in a swarm pattern:

```omni
struct SwarmConfig:
    min_agents:         u8               # Minimum agents to consult (default 2)
    max_agents:         u8               # Maximum agents to consult (default 5)
    consensus_strategy: ConsensusStrategy
    timeout_ms:         u32              # Per-agent timeout (default 5000)

enum ConsensusStrategy:
    MajorityVote                         # Accept the most common answer
    WeightedByConfidence                 # Weight by each agent's confidence score
    HighestConfidence                    # Accept the single most confident answer
    UnionWithConflictResolution          # Merge all results, resolve conflicts
```

---

## 47. HELIOS Knowledge — API Versioning and Live Schema Migration

### 47.1 Semantic Versioning for Knowledge Schema

The InformationUnit schema follows semantic versioning. Every breaking change increments the major version:

```omni
struct SchemaVersion:
    major:              u16              # Breaking changes (field removal, type change)
    minor:              u16              # Backward-compatible additions
    patch:              u16              # Bug fixes in default values or constraints

const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion { major: 5, minor: 0, patch: 0 };
```

### 47.2 Live Schema Migration

Schema migrations are applied without downtime using a phased dual-write approach:

```omni
struct SchemaMigration:
    from_version:       SchemaVersion
    to_version:         SchemaVersion
    migration_type:     MigrationType
    steps:              Vec<MigrationStep>
    rollback_steps:     Vec<MigrationStep>

enum MigrationType:
    Additive                             # New optional fields — no dual-write needed
    Transform                            # Field type change — requires dual-write
    Destructive                          # Field removal — requires deprecation period

enum MigrationStep:
    AddField { field_name: String, field_type: String, default_value: String }
    RenameField { old_name: String, new_name: String }
    ChangeFieldType { field_name: String, old_type: String, new_type: String, converter: String }
    RemoveField { field_name: String, deprecation_date: Timestamp }
    AddIndex { field_name: String, index_type: String }
    DropIndex { field_name: String }
    BackfillData { query: String, transform: String }
```

### 47.3 Dual-Write Migration Protocol

For non-additive migrations, the dual-write protocol ensures zero read failures during migration:

```
Phase 1 (Prepare):     Add new field/format alongside old. Write to both.
Phase 2 (Backfill):    Migrate existing data in background batches.
Phase 3 (Switch):      Change readers to prefer new format. Old remains readable.
Phase 4 (Cleanup):     After grace period, remove old field/format. Compact pages.
```

```omni
struct DualWriteState:
    migration_id:       u64
    current_phase:      MigrationPhase
    backfill_progress:  f32              # 0.0 to 1.0
    start_time:         Timestamp
    grace_period:       Duration         # How long to keep old format after switch (default 7 days)

enum MigrationPhase:
    Preparing
    DualWriting
    Backfilling
    Switched
    CleaningUp
    Completed
```

### 47.4 Breaking Change Detection

An automated tool detects breaking changes between schema versions:

```omni
fn detect_breaking_changes(
    old_schema: &SchemaDefinition,
    new_schema: &SchemaDefinition,
) -> Vec<BreakingChange> {
    let mut changes = Vec::new();
    // Detect removed fields
    // Detect type changes
    // Detect required field additions (without defaults)
    // Detect renamed fields without aliases
    // Detect changed enum variants
    changes
}

struct BreakingChange:
    field_path:         String
    change_type:        BreakingChangeType
    severity:           Severity
    migration_hint:     String

enum BreakingChangeType:
    FieldRemoved
    TypeChanged { from: String, to: String }
    RequiredFieldAdded { field: String }
    EnumVariantRemoved { variant: String }
    FieldRenamed { from: String, to: String }
```

---

## Appendix E — Phase Implementation Sequence (v5.0 Additions)

**Phase P — GNN Reasoning and GraphRAG**
1. Implement GNN message-passing framework with configurable architectures (GCN, GAT, GraphSAGE, RGCN).
2. Implement inductive link prediction using subgraph extraction.
3. Implement GNN-RAG pipeline for multi-hop question answering.
4. Implement GraphRAG hybrid retriever with vector index (HNSW) and Leiden community detection.
5. Implement reciprocal rank fusion for result combining.
6. Acceptance test: GNN predicts correct links for 3 entities unseen during training. GraphRAG correctly answers a 3-hop question.

**Phase Q — Transactional Knowledge Store**
1. Implement MVCC with versioned units and snapshot reads.
2. Implement SSI conflict detection with write-skew prevention.
3. Implement WAL with configurable sync modes (fsync, group commit).
4. Implement crash recovery from WAL replay.
5. Acceptance test: 100 concurrent readers and 10 concurrent writers produce serializable results. WAL recovery correctly restores state after simulated crash.

**Phase R — NLU Pipeline and GPU Offloading**
1. Implement tokenization, intent classification, and entity extraction pipeline.
2. Implement conversation context manager with coreference resolution.
3. Implement compute backend abstraction layer with CPU/CUDA/Vulkan dispatch.
4. Implement GPU memory manager with chunked offloading and prefetching.
5. Acceptance test: NLU correctly classifies 10 different intent types. GPU offloading trains embeddings 5× faster than CPU-only.

**Phase S — Time-Travel Debugging**
1. Implement OVM execution recording with deterministic event capture.
2. Implement checkpoint-based reverse execution.
3. Implement reverse continue, reverse step, and watchpoint-reverse commands.
4. Acceptance test: Record execution of 100K instructions. Reverse to instruction 50K. Assert memory state matches forward execution at that point.

**Phase T — Chaos Resilience and Internationalization**
1. Implement fault injection framework with 9 fault types.
2. Implement steady-state verification during experiments.
3. Implement Unicode-aware knowledge storage with locale metadata.
4. Implement CLDR-based formatting and cross-lingual entity resolution.
5. Acceptance test: System recovers from simulated WAL corruption. Cross-lingual resolver matches "東京" to "Tokyo" with confidence > 0.85.

**Phase U — GUI Accessibility and Green Computing**
1. Implement WCAG 2.2 AA compliance in all GUI components.
2. Implement keyboard navigation with visible focus indicators.
3. Implement high-contrast theme system with Windows HWCM support.
4. Implement energy profiler with RAPL/software estimation.
5. Implement carbon-aware background task scheduler.
6. Acceptance test: All GUI elements navigable by keyboard alone. Screen reader correctly announces all interactive elements. Energy budget enforcement throttles background tasks when limit reached.

**Phase V — Multi-Agent Coordination**
1. Implement AgentCard discovery and ACP-compatible communication.
2. Implement task delegation with async result handling.
3. Implement swarm intelligence consensus for group queries.
4. Acceptance test: HELIOS delegates a translation task to a mock agent and correctly integrates the result. Swarm consensus correctly selects the most-supported answer from 3 agents.

**Phase W — API Versioning and Schema Migration**
1. Implement schema version tracking in knowledge store pages.
2. Implement additive, transform, and destructive migration types.
3. Implement dual-write migration with phased rollout.
4. Implement breaking change detection tool.
5. Acceptance test: Apply a transform migration (field type change) while concurrent reads continue. Assert zero read failures during migration. Breaking change detector correctly identifies all 5 types of breaking changes.

---

## Appendix F — Acceptance Test Specifications (v5.0 Additions)

**Phase P acceptance:** Train a 3-layer GAT model on a knowledge graph with 500 entities. Test inductive prediction for 5 entities withheld from training. Assert ≥ 3 of 5 entities have correct top-1 link predictions. Build a GraphRAG index with Leiden communities. Query "What connects entity A to entity D through intermediaries?" Assert the answer includes the correct 3-hop path.

**Phase Q acceptance:** Start 100 concurrent reader transactions and 10 concurrent writer transactions against a knowledge store with 10,000 units. Assert all transactions produce serializable results (no write-skew anomalies detected). Kill the process after 50 writer commits. Restart and verify WAL recovery restores exactly 50 committed writes with zero data loss.

**Phase R acceptance:** Feed 20 diverse queries through the NLU pipeline. Assert intent classification accuracy ≥ 90% (18/20 correct). Assert entity extraction F1 score ≥ 0.85. Train KGE on GPU using CUDA backend. Assert training is ≥ 5× faster than CPU-only baseline on the same hardware.

**Phase S acceptance:** Record execution of a program performing 200,000 OVM instructions with 3 threads. Set a forward breakpoint at instruction 150,000. Run reverse-continue from instruction 200,000. Assert the debugger stops at the correct breakpoint. Assert all register and memory state at instruction 150,000 matches the forward execution state.

**Phase T acceptance:** Run a chaos experiment injecting WAL corruption on segment 2. Assert the system detects corruption, skips the affected segment, and recovers using the previous checkpoint. Run a chaos experiment with 200ms I/O latency injection. Assert queries still complete within 2× normal latency (graceful degradation, not failure). Create two InformationUnits: one in English ("Tokyo is in Japan") and one in Japanese ("東京は日本にある"). Assert the cross-lingual resolver identifies them as referring to the same entities with similarity > 0.85.

**Phase U acceptance:** Automated accessibility audit of all GUI panels using Axe or Accessibility Insights. Assert zero critical WCAG 2.2 AA violations. Navigate the entire knowledge graph visualization panel using only keyboard (Tab, Arrow keys, Enter). Assert all nodes and edges are reachable and interactive. Set energy budget to 1000 joules/hour. Run continuous queries for 15 minutes. Assert background tasks (embedding training, web learning) are paused when budget is 80% consumed. Assert query handling continues uninterrupted.

**Phase V acceptance:** Register 3 mock agents with different AgentCapabilities (FactQuery, Translation, Reasoning). Delegate a fact verification task. Assert the correct agent is selected. Delegate the same question to all 3 agents. Assert weighted-by-confidence consensus selects the answer with the highest weighted score.

**Phase W acceptance:** Apply a `Transform` migration that changes `confidence` from `u8` to `u16`. Assert dual-write produces valid data in both formats. Assert reader transparently reads both old (u8) and new (u16) units. After backfill completion and grace period, assert cleanup removes all old-format data. Run breaking change detection on a schema diff. Assert it correctly flags: 1 field removal, 1 type change, 1 required field addition without default, and 1 enum variant removal.

---

*End of v5.0 sections — v6.0 sections follow.*

---

## 48. Omni Language — Property-Based Testing Framework

### 48.1 Generators and Arbitraries

The Omni standard library includes a property-based testing framework inspired by QuickCheck/Hypothesis, with integrated generators for all Omni types:

```omni
struct PropertyTest:
    name:               String
    property:           fn(Vec<Arbitrary>) -> PropertyResult
    num_cases:          u32              # Number of random inputs (default 100)
    max_shrinks:        u32              # Maximum shrink attempts (default 1000)
    seed:               Option<u64>      # Reproducible seed

enum PropertyResult:
    Passed
    Failed { input: Vec<Arbitrary>, shrunk_input: Vec<Arbitrary> }
    Discarded                            # Input didn't meet precondition

trait Generator<T>:
    fn generate(rng: &mut RNG, size: u32) -> T
    fn shrink(value: T) -> Vec<T>        # Produce smaller counterexamples

// Built-in generators for all Omni primitive and compound types
fn arb_u8() -> Generator<u8>
fn arb_string(max_len: u32) -> Generator<String>
fn arb_vec<T>(elem: Generator<T>, max_len: u32) -> Generator<Vec<T>>
fn arb_option<T>(elem: Generator<T>) -> Generator<Option<T>>
fn arb_information_unit() -> Generator<InformationUnit>   # Domain-specific
```

### 48.2 Coverage-Guided Fuzzing Integration

Property tests can optionally enable coverage-guided fuzzing to find deeper bugs:

```omni
struct FuzzConfig:
    mode:               FuzzMode
    corpus_dir:         String           # Directory to store interesting inputs
    max_runtime:        Duration
    sanitizers:         Vec<Sanitizer>

enum FuzzMode:
    Random                               # Pure random generation
    CoverageGuided                       # Track code coverage, guide toward new paths
    Hybrid                               # Alternate between random and guided

enum Sanitizer:
    AddressSanitizer                     # Memory safety violations
    UndefinedBehavior                    # UB detection
    ThreadSanitizer                      # Data race detection
```

---

## 49. Plugin System — Marketplace and Registry

### 49.1 Plugin Registry

HELIOS hosts a plugin registry for discovering, installing, and managing community-contributed plugins:

```omni
struct PluginRegistry:
    registry_url:       String           # Central registry endpoint
    local_cache:        PathBuf          # Local plugin cache directory
    signing_key:        PublicKey        # Registry signing key for verification
    update_policy:      UpdatePolicy

struct PluginListing:
    plugin_id:          String           # Unique identifier (e.g., "org.example.translator")
    name:               String
    author:             String
    version:            SchemaVersion
    description:        String
    capabilities:       Vec<PluginCapability>   # Section 10 capabilities required
    downloads:          u64
    rating:             f32              # Community rating 0-5
    signature:          Vec<u8>          # Cryptographic signature (Section 30)

enum UpdatePolicy:
    Manual                               # User must explicitly update
    AutoPatch                            # Auto-update patch versions
    AutoMinor                            # Auto-update minor versions
    Pinned(SchemaVersion)                # Lock to specific version
```

### 49.2 Dependency Resolution

Plugin dependencies are resolved using a SAT-solver approach to find compatible version sets:

```omni
struct DependencySpec:
    plugin_id:          String
    version_constraint: VersionConstraint

enum VersionConstraint:
    Exact(SchemaVersion)
    Range { min: SchemaVersion, max: SchemaVersion }
    Compatible(SchemaVersion)            # ^major.minor — semver compatible
    Any

fn resolve_dependencies(
    specs: &[DependencySpec],
    registry: &PluginRegistry,
) -> Result<Vec<(String, SchemaVersion)>, DependencyConflict>
```

---

## 50. HELIOS Interaction — Voice Interface and Speech Recognition

### 50.1 Speech Recognition Pipeline

HELIOS supports voice input as an alternative to text queries:

```omni
struct VoiceInterface:
    recognizer:         SpeechRecognizer
    wake_word:          WakeWordDetector
    tts_engine:         TextToSpeech
    noise_filter:       NoiseFilter

struct SpeechRecognizer:
    model:              SpeechModel
    language:           Locale           # Primary recognition language
    additional_languages: Vec<Locale>    # Additional languages for multilingual mode
    beam_width:         u8               # Beam search width (default 5)

enum SpeechModel:
    Local { model_path: PathBuf }        # On-device model (privacy-preserving)
    CloudAPI { endpoint: String }        # Cloud-based recognition
    Hybrid                               # Local for wake word, cloud for full recognition

struct WakeWordDetector:
    wake_phrase:        String           # Default: "Hey HELIOS"
    sensitivity:        f32              # 0.0 (strict) to 1.0 (lenient), default 0.5
    always_listening:   bool             # If true, continuously listen for wake word
```

### 50.2 Voice Command Routing

Voice commands are routed through the NLU pipeline (Section 39) with speech-specific preprocessing:

```omni
struct VoiceQueryResult:
    transcript:         String           # Raw speech-to-text output
    confidence:         f32              # Recognition confidence
    alternatives:       Vec<(String, f32)>  # Alternative transcriptions
    nlu_result:         NLUResult        # Section 39 pipeline result
    response_text:      String           # HELIOS response text
    response_audio:     Option<Vec<u8>>  # TTS audio response
```

---

## 51. HELIOS Knowledge — Streaming Event Processing

### 51.1 Knowledge Event Stream

All knowledge store mutations are published as a real-time event stream for downstream consumers:

```omni
struct KnowledgeEvent:
    event_id:           u64
    event_type:         KnowledgeEventType
    unit_id:            u64
    timestamp:          Timestamp
    actor:              ActorId
    metadata:           HashMap<String, String>

enum KnowledgeEventType:
    UnitCreated { content: InformationUnit }
    UnitUpdated { old_version: u64, new_version: u64, diff: UnitDiff }
    UnitDeleted { reason: String }
    ConfidenceChanged { old: u8, new: u8 }
    RelationAdded { target: u64, relation_type: String }
    RelationRemoved { target: u64, relation_type: String }
    VerificationCompleted { result: VerificationResult }
```

### 51.2 Complex Event Processing Rules

Users can define CEP rules to detect patterns in the knowledge event stream:

```omni
struct CEPRule:
    rule_id:            u64
    name:               String
    pattern:            EventPattern
    action:             CEPAction
    window:             TimeWindow

enum EventPattern:
    Single(KnowledgeEventType)
    Sequence(Vec<KnowledgeEventType>)    # Events in order within window
    All(Vec<KnowledgeEventType>)         # All events within window (any order)
    Absence(KnowledgeEventType)          # Event did NOT occur within window
    Frequency { event: KnowledgeEventType, min_count: u32 }

enum CEPAction:
    Alert { message: String }
    TriggerRule(u64)                     # Fire a RETE rule
    ExecuteOQL(String)                   # Run an OQL query
    CallWebhook { url: String }

struct TimeWindow:
    duration:           Duration         # Window size
    slide:              Option<Duration> # Sliding window step (None = tumbling)
```

---

## 52. HELIOS Resilience — Self-Healing Patterns

### 52.1 Circuit Breaker

The circuit breaker pattern protects HELIOS subsystems from cascading failures:

```omni
struct CircuitBreaker:
    name:               String
    state:              CircuitState
    failure_threshold:  u32              # Failures before opening (default 5)
    success_threshold:  u32              # Successes in half-open to close (default 3)
    timeout:            Duration         # Time in open state before half-open (default 30s)
    failure_count:      u32
    last_failure:       Option<Timestamp>

enum CircuitState:
    Closed                               # Normal operation, requests flow
    Open                                 # Failures exceeded threshold, reject requests
    HalfOpen                             # Allow limited requests to test recovery

fn call_with_circuit_breaker<T>(
    breaker: &mut CircuitBreaker,
    operation: fn() -> Result<T, Error>,
    fallback: fn() -> T,
) -> T
```

### 52.2 Retry with Exponential Backoff

```omni
struct RetryPolicy:
    max_retries:        u8               # Maximum attempts (default 3)
    initial_delay:      Duration         # First retry delay (default 100ms)
    max_delay:          Duration         # Cap on backoff (default 30s)
    multiplier:         f32              # Backoff multiplier (default 2.0)
    jitter:             bool             # Add random jitter (default true)
    retryable_errors:   Vec<ErrorKind>   # Only retry these error types
```

### 52.3 Graceful Degradation

```omni
struct DegradationPolicy:
    levels:             Vec<DegradationLevel>
    current_level:      usize
    escalation_trigger: EscalationTrigger

struct DegradationLevel:
    name:               String           # e.g., "Normal", "Reduced", "Minimal"
    disabled_features:  Vec<Feature>     # Features disabled at this level
    max_cognitive_depth: u8              # Maximum cognitive layer invoked (L0-L4)
    cache_only:         bool             # Serve only cached responses

enum EscalationTrigger:
    ErrorRate { threshold_pct: f32, window: Duration }
    Latency { p99_ms: u32 }
    MemoryPressure { threshold_pct: f32 }
    Manual
```

---

## 53. HELIOS Brain — Knowledge Distillation and Model Compression

### 53.1 Model Distillation for Edge Deployment

HELIOS can distill its learned models (GNN, KGE, NLU classifiers) into compact versions for edge/mobile deployment:

```omni
struct DistillationConfig:
    teacher_model:      ModelRef
    student_architecture: StudentArch
    loss_function:      DistillationLoss
    temperature:        f32              # Softmax temperature (default 3.0)
    epochs:             u32
    target_size_mb:     f32              # Target student model size

enum StudentArch:
    SameArchSmaller { reduction_factor: f32 }  # Same architecture, fewer layers/dims
    DifferentArch(String)                      # Specify a different architecture
    AutoSearch                                 # Neural architecture search for best fit

enum DistillationLoss:
    KLDivergence                         # Standard KD loss
    MSE                                  # Mean squared error on logits
    CombinedKDCE { alpha: f32 }          # α×KD + (1-α)×CrossEntropy
```

### 53.2 Quantization Support

```omni
struct QuantizationConfig:
    method:             QuantizationMethod
    target_precision:   Precision
    calibration_data:   Option<Vec<Vec<u8>>>  # Calibration dataset for PTQ

enum QuantizationMethod:
    PostTraining                         # PTQ: quantize after training
    QuantizationAwareTraining            # QAT: quantize during training
    DynamicQuantization                  # Quantize at inference time

enum Precision:
    FP32                                 # Full precision (baseline)
    FP16                                 # Half precision
    INT8                                 # 8-bit integer
    INT4                                 # 4-bit integer (aggressive)
```

---

## 54. HELIOS Knowledge — Data Lineage and Provenance Tracking

### 54.1 Provenance Graph

Every InformationUnit in the knowledge store carries full provenance metadata forming a DAG:

```omni
struct ProvenanceRecord:
    unit_id:            u64
    origin:             DataOrigin
    transformations:    Vec<TransformationStep>
    lineage_hash:       [u8; 32]         # BLAKE3 hash of the full lineage chain
    created_by:         ActorId
    created_at:         Timestamp

enum DataOrigin:
    UserInput { session_id: u64 }
    WebLearning { url: String, crawl_id: u64 }
    PluginGenerated { plugin_id: String }
    Inference { rule_id: u64, inputs: Vec<u64> }  # Derived from other units
    Federation { source_instance: [u8; 16] }
    Import { format: String, file_path: String }

struct TransformationStep:
    step_id:            u32
    operation:          String           # e.g., "confidence_update", "merge", "split"
    input_units:        Vec<u64>
    output_units:       Vec<u64>
    actor:              ActorId
    timestamp:          Timestamp
    justification:      Option<String>
```

### 54.2 Lineage Queries

OQL (Section 21) is extended with lineage-specific operators:

```omni
// Trace full provenance of a unit
FIND LINEAGE OF unit WHERE id = 42

// Find all units derived from a specific source
FIND facts WHERE DERIVED_FROM(id, 42)

// Show transformation chain
FIND TRANSFORMATIONS OF unit WHERE id = 42 ORDER BY timestamp ASC
```

---

## 55. HELIOS Brain — Explainable AI and Reasoning Transparency

### 55.1 Explanation Generation

Every HELIOS reasoning result includes a structured explanation:

```omni
struct ReasoningExplanation:
    query:              String           # Original user query
    answer:             String           # HELIOS response
    confidence:         u8
    reasoning_chain:    Vec<ReasoningStep>
    feature_attributions: Vec<FeatureAttribution>  # SHAP-like feature importance
    alternative_answers: Vec<(String, u8)>  # Other considered answers + confidence

struct ReasoningStep:
    step_id:            u32
    layer:              u8               # Cognitive layer that produced this step (L0-L4)
    rule_or_method:     String           # Rule name, algorithm, or method used
    inputs:             Vec<u64>         # InformationUnit IDs used as input
    output:             String           # Intermediate conclusion
    confidence_delta:   i8               # How this step changed overall confidence

struct FeatureAttribution:
    feature:            String           # e.g., "source_reliability", "temporal_recency"
    attribution_score:  f32              # -1.0 (strongly against) to 1.0 (strongly for)
    explanation:        String           # "Source Reuters has 95% historical accuracy"
```

### 55.2 Explainability Modes

```omni
enum ExplainabilityMode:
    Silent                               # No explanation (fastest)
    Summary                              # One-sentence explanation
    Detailed                             # Full reasoning chain
    Debug                                # All intermediate states + feature attributions
    Interactive                          # Step-by-step: user can drill into each step
```

---

## 56. Omni Language — Advanced Incremental Compilation

### 56.1 Stateful Compiler Architecture

The Omni compiler retains state between invocations for true incremental compilation:

```omni
struct CompilerState:
    dependency_graph:   DAG<ModuleId, DependencyEdge>
    ast_cache:          HashMap<ModuleId, CachedAST>
    type_cache:         HashMap<ModuleId, TypeCheckResult>
    ir_cache:           HashMap<ModuleId, IRModule>
    content_hashes:     HashMap<ModuleId, [u8; 32]>  # BLAKE3 of source content
    last_build:         Timestamp

struct CachedAST:
    ast:                AST
    source_hash:        [u8; 32]
    parse_time:         Duration

fn incremental_compile(
    state: &mut CompilerState,
    changed_files: &[PathBuf],
) -> CompileResult {
    // 1. Compute content hashes for changed files
    // 2. Identify invalidated modules via dependency graph traversal
    // 3. Re-parse only invalidated modules
    // 4. Re-typecheck only modules whose dependencies changed types
    // 5. Re-generate IR only for modules with changed type-checked output
    // 6. Link incrementally
}
```

### 56.2 Distributed Build Cache

```omni
struct DistributedCache:
    backend:            CacheBackend
    namespace:          String           # Project-specific cache isolation
    compression:        bool             # Compress cached artifacts (default true)

enum CacheBackend:
    Local { path: PathBuf }
    HTTP { endpoint: String, auth: Option<String> }
    S3 { bucket: String, prefix: String }

fn cache_lookup(cache: &DistributedCache, key: [u8; 32]) -> Option<CachedArtifact>
fn cache_store(cache: &mut DistributedCache, key: [u8; 32], artifact: CachedArtifact)
```

---

## 57. HELIOS Knowledge — Storage Engine Optimizations

### 57.1 Tiered Storage with MMIO

The knowledge store uses memory-mapped I/O for hot pages and tiered storage for cost-efficiency:

```omni
struct TieredStorage:
    hot_tier:           HotTier          # Memory-mapped, frequently accessed pages
    warm_tier:          WarmTier         # SSD-backed, less frequent access
    cold_tier:          ColdTier         # Compressed archive storage

struct HotTier:
    mmap_region:        *mut u8          # Memory-mapped region
    capacity_pages:     u32
    map_ahead:          u32              # Pages to prefetch (default 8)
    eviction:           EvictionPolicy   # LRU/LFU from Section 40.3

struct WarmTier:
    storage_path:       PathBuf
    page_size:          u32              # Bytes per page (default 16 KB)
    bloom_filters:      HashMap<u32, BloomFilter>  # Per-page Bloom filters (Section 31)

struct ColdTier:
    archive_path:       PathBuf
    compression:        CompressionLevel # OmniPack compression level
    searchable:         bool             # Maintain Bloom filter index even for cold pages
```

### 57.2 B-Tree Index Enhancements

```omni
struct BTreeConfig:
    order:              u16              # B-tree order (default 128)
    leaf_compression:   bool             # Compress leaf nodes (default true)
    simd_search:        bool             # Use SIMD for intra-node key search (default true)
    write_optimization: WriteOptMode

enum WriteOptMode:
    Standard                             # In-place updates
    LSMBacked                            # Write to LSM buffer, merge on compaction
    Deferred { batch_size: u32 }         # Batch writes, apply periodically
```

### 57.3 LSM Tree for Write-Heavy Workloads

For scenarios with high write throughput (web learning, federation sync), an LSM tree variant is available:

```omni
struct LSMConfig:
    memtable_size:      u32              # Bytes before flush to L0 (default 64 MB)
    level_multiplier:   u8               # Size ratio between levels (default 10)
    compaction_strategy: CompactionStrategy
    bloom_bits_per_key: u8               # Bloom filter bits per key (default 10)

enum CompactionStrategy:
    SizeTiered                           # Group similarly-sized SSTables
    Leveled                              # Maintain sorted levels (better reads)
    FIFO                                 # Time-based expiration (for temporal data)
    UniversalCompaction                  # Adaptive between size-tiered and leveled
```

---

## Appendix G — Phase Implementation Sequence (v6.0 Additions)

**Phase X — Testing and Plugin Ecosystem**
1. Implement property-based testing framework with generators, shrinking, and coverage-guided fuzzing.
2. Implement plugin registry with cryptographic signature verification.
3. Implement SAT-solver dependency resolution for plugins.
4. Acceptance test: Property test discovers a known bug in a sample Omni function within 100 test cases. Plugin registry resolves 3 plugins with overlapping version constraints correctly.

**Phase Y — Voice, Streaming, and Resilience**
1. Implement speech recognition pipeline with wake word detection and multilingual support.
2. Implement knowledge event stream with CEP rule engine.
3. Implement circuit breaker, retry with exponential backoff, and graceful degradation.
4. Acceptance test: Voice interface correctly transcribes 10 queries with ≥ 90% word accuracy. CEP detects a sequence of 3 events within a 5-second window. Circuit breaker opens after 5 consecutive failures and closes after 3 successes in half-open state.

**Phase Z — Distillation, Lineage, and Explainability**
1. Implement teacher-student knowledge distillation with KL divergence loss.
2. Implement provenance graph with full lineage tracking for every InformationUnit.
3. Implement reasoning explanation generation with feature attributions.
4. Implement explainability modes (Silent, Summary, Detailed, Debug, Interactive).
5. Acceptance test: Distilled student model retains ≥ 90% of teacher model accuracy at ≤ 50% parameter count. Lineage query correctly traces a derived unit back to its original web-learned source. Detailed explanation includes all reasoning steps with correct confidence deltas.

**Phase AA — Compiler and Storage Optimizations**
1. Implement stateful incremental compilation with content-addressable caching.
2. Implement distributed build cache with S3/HTTP backends.
3. Implement tiered storage (MMIO hot tier, SSD warm tier, compressed cold tier).
4. Implement B-tree SIMD search and LSM tree compaction strategies.
5. Acceptance test: Incremental recompilation of a single changed file in a 100-module project completes in < 500ms (vs. full rebuild baseline). Tiered storage auto-promotes a cold page to hot tier after 10 accesses within 1 minute. LSM compaction maintains read latency < 5ms p99 under sustained 10,000 writes/sec.

---

## Appendix H — Acceptance Test Specifications (v6.0 Additions)

**Phase X acceptance:** Define a property `∀ x: u32, reverse(reverse(list(x))) == list(x)`. Run 100 generated inputs. Assert all pass. Introduce a known bug (off-by-one in reverse). Assert the property test finds and shrinks to a minimal counterexample. Install a mock plugin from the registry with 2 dependencies. Assert both dependencies are resolved, downloaded, verified (signature check), and sandbox-loaded successfully.

**Phase Y acceptance:** Speak "Hey HELIOS, what is the capital of France?" into the voice interface. Assert wake word detected with confidence > 0.9. Assert transcript matches within 2 edit distance characters. Assert NLU identifies FactLookup intent and "France" entity. Create a CEP rule detecting 3 ConfidenceChanged events within 10 seconds. Emit exactly 3 such events. Assert the rule fires. Emit only 2 events. Assert the rule does not fire. Configure a circuit breaker with failure threshold 3. Force 3 consecutive failures on a subsystem. Assert circuit opens. Wait timeout period. Assert half-open state allows one request through.

**Phase Z acceptance:** Train a 3-layer GNN teacher model (256 hidden dim). Distill into a 2-layer student (128 hidden dim) with temperature 3.0 for 50 epochs. Assert student's link prediction MRR is ≥ 90% of teacher's. Assert student's parameter count is ≤ 50% of teacher's. Create an InformationUnit via web learning, then merge it with a user-provided correction. Query LINEAGE for the merged unit. Assert the provenance graph shows both the web-learning origin and the user-correction transformation step. Ask HELIOS a factual question in Debug explainability mode. Assert the response includes: reasoning chain with ≥ 2 steps, feature attributions with ≥ 3 features, and at least 1 alternative considered answer.

**Phase AA acceptance:** Create a 100-module Omni project with intermodule dependencies. Perform a full build to warm the compiler state. Modify a single leaf module (no dependents). Assert incremental rebuild completes in < 500ms. Modify a root module (all modules depend on it). Assert incremental rebuild recompiles all dependent modules but reuses cached ASTs where source is unchanged. Insert 10,000 InformationUnits using LSM-backed writes. Assert all units are queryable within 5ms p99. Run compaction. Assert storage size after compaction is ≤ 70% of pre-compaction size thanks to deduplication. Access a cold-tier page 10 times within 1 minute. Assert the page is auto-promoted to hot tier. Assert subsequent access latency is < 0.1ms (memory-mapped speed).

---

*End of v6.0 sections — v7.0 sections follow.*

---

## 58. HELIOS Brain — Neuro-Symbolic Hybrid Reasoning

### 58.1 Dual-Process Architecture

HELIOS implements a Kahneman-inspired dual-process reasoning architecture combining fast neural pattern matching (System 1) with deliberate symbolic reasoning (System 2):

```omni
struct NeuroSymbolicEngine:
    neural_layer:       NeuralReasonerConfig
    symbolic_layer:     SymbolicReasonerConfig
    arbitrator:         ReasoningArbitrator
    fallback_policy:    FallbackPolicy

struct NeuralReasonerConfig:
    model:              NeuralModel          # GNN, transformer, or embedding model
    confidence_threshold: f32                # Below this → escalate to symbolic (default 0.7)
    max_latency_ms:     u32                  # Time budget for neural inference (default 50)

struct SymbolicReasonerConfig:
    rule_engine:        RETEConfig           # Section 8 RETE network
    logic_prover:       LogicProver          # Forward/backward chaining
    ontology:           OntologyConfig       # Section 17 class hierarchy

enum FallbackPolicy:
    NeuralFirst                              # Try neural, escalate to symbolic if low confidence
    SymbolicFirst                            # Try symbolic, use neural for gaps
    Parallel                                 # Run both, merge results
    AdaptiveRouting                          # ML-based router selects per-query
```

### 58.2 Knowledge-Grounded Neural Inference

Neural models are grounded against the symbolic knowledge store to prevent hallucination:

```omni
struct GroundingConfig:
    grounding_mode:     GroundingMode
    fact_check_threshold: f32                # Minimum KG match score (default 0.8)
    max_hops:           u8                   # KG traversal depth for grounding (default 2)

enum GroundingMode:
    StrictGrounding                          # Only return answers with KG support
    SoftGrounding                            # Return answer + grounding confidence
    Ungrounded                               # Neural-only (fastest, least reliable)
```

---

## 59. HELIOS Knowledge — Federated Knowledge Learning

### 59.1 Federated Training Protocol

Multiple HELIOS instances collaboratively train shared models without sharing raw knowledge:

```omni
struct FederatedLearningConfig:
    role:               FederatedRole
    aggregation:        AggregationStrategy
    privacy:            FLPrivacyConfig
    communication:      FLCommunication
    rounds:             u32                  # Number of federated rounds

enum FederatedRole:
    Coordinator                              # Aggregates updates from participants
    Participant                              # Trains locally, sends updates

enum AggregationStrategy:
    FedAvg                                   # Federated Averaging (weighted by dataset size)
    FedProx { mu: f32 }                      # Proximal term for heterogeneous data
    SecureAggregation                        # Cryptographic aggregation (SMC)

struct FLPrivacyConfig:
    differential_privacy: Option<DPConfig>   # Section 29 DP mechanisms
    gradient_clipping:  f32                  # Max gradient norm (default 1.0)
    noise_multiplier:   f32                  # Gaussian noise scale (default 0.1)
    secure_aggregation: bool                 # Use homomorphic encryption (default true)

struct FLCommunication:
    compression:        GradientCompression
    frequency:          UpdateFrequency

enum GradientCompression:
    None
    TopK { k_pct: f32 }                      # Send only top-K% of gradients
    Quantized { bits: u8 }                   # Quantize gradients to N bits
```

---

## 60. Omni Language — Semantic Code Analysis

### 60.1 Abstract Interpretation Framework

The Omni compiler includes an abstract interpretation engine for sound static analysis:

```omni
struct AbstractInterpreter:
    domains:            Vec<AbstractDomain>
    widening_threshold: u8                   # Iterations before widening (default 5)
    narrowing_passes:   u8                   # Narrowing refinement passes (default 2)

enum AbstractDomain:
    Interval                                 # Numeric range analysis [lo, hi]
    Sign                                     # Positive/Negative/Zero/Top/Bottom
    PointerAnalysis                          # Points-to sets for references
    NullabilityDomain                        # Definitely null / Maybe null / Not null
    TaintTracking                            # Track untrusted data flow

struct AnalysisResult:
    warnings:           Vec<AnalysisWarning>
    proven_safe:        Vec<SafetyProperty>  # Properties proven to always hold
    unreachable_code:   Vec<(u32, u32)>      # Line ranges proved unreachable

enum AnalysisWarning:
    PossibleNullDeref { line: u32, variable: String }
    PossibleOverflow { line: u32, expression: String }
    PossibleDivByZero { line: u32 }
    TaintedDataUsed { line: u32, source: String, sink: String }
    UnusedVariable { line: u32, variable: String }
```

---

## 61. Omni Language — Runtime Reflection and Introspection

### 61.1 Compile-Time Reflection

Omni favors compile-time reflection over runtime reflection for zero-cost introspection:

```omni
// Derive macro generates reflection metadata at compile time
#[derive(Reflect)]
struct MyStruct:
    name:   String
    value:  u32

// At compile time, the Reflect derive generates:
impl Reflectable for MyStruct:
    fn type_name() -> &str { "MyStruct" }
    fn field_count() -> usize { 2 }
    fn field_names() -> &[&str] { &["name", "value"] }
    fn field_types() -> &[TypeId] { &[TypeId::of::<String>(), TypeId::of::<u32>()] }

// Use for automatic serialization, deserialization, debug printing
fn serialize_any<T: Reflectable>(value: &T) -> Vec<u8>
fn deserialize_any<T: Reflectable>(data: &[u8]) -> Result<T, DeserializeError>
```

### 61.2 Runtime Type Information (RTTI)

For dynamic scenarios (plugins, federation), limited RTTI is available:

```omni
struct TypeInfo:
    name:               String
    module:             String
    size:               usize
    alignment:          usize
    fields:             Vec<FieldInfo>
    is_enum:            bool

struct FieldInfo:
    name:               String
    type_id:            TypeId
    offset:             usize
    is_public:          bool
```

---

## 62. Omni Language — Hot Code Reloading

### 62.1 Live Module Replacement

The OVM (Omni Virtual Machine) supports replacing modules at runtime for development and live patching:

```omni
struct HotReloadConfig:
    enabled:            bool
    watch_dirs:         Vec<PathBuf>         # Directories to watch for changes
    debounce_ms:        u32                  # Debounce file change events (default 200)
    strategy:           ReloadStrategy

enum ReloadStrategy:
    ModuleReplace                            # Replace entire module, preserve state
    FunctionPatch                            # Patch individual functions via jump tables
    FullRestart                              # Restart with new code (safest, slowest)

struct ReloadResult:
    reloaded_modules:   Vec<String>
    preserved_state:    bool
    reload_time_ms:     f64
    errors:             Vec<ReloadError>

enum ReloadError:
    TypeMismatch { old_type: String, new_type: String }
    StateIncompatible { reason: String }
    CompileError(String)
```

---

## 63. Omni Language — Structured Concurrency

### 63.1 Task Groups with Hierarchical Cancellation

Omni adopts structured concurrency where all spawned tasks are bound to their parent scope:

```omni
// Task group: all child tasks complete before the group exits
async fn fetch_related_knowledge(unit_id: u64) -> Vec<InformationUnit> {
    task_group! {
        let related = spawn fetch_relations(unit_id);
        let similar = spawn find_similar(unit_id);
        let temporal = spawn find_temporal_neighbors(unit_id);
        // All three run concurrently. Group waits for all to complete.
        // If any task panics, siblings are cancelled.
    }
    merge(related.await, similar.await, temporal.await)
}

struct TaskGroup<T>:
    tasks:              Vec<TaskHandle<T>>
    cancellation:       CancellationToken
    error_policy:       GroupErrorPolicy

enum GroupErrorPolicy:
    CancelOnFirstError                       # Cancel all siblings on first failure
    CollectAll                               # Wait for all, collect all errors
    CancelOnFirstSuccess                     # Cancel remaining after first success (race)

struct CancellationToken:
    is_cancelled:       AtomicBool
    parent:             Option<Arc<CancellationToken>>   # Propagates up
    children:           Vec<Arc<CancellationToken>>      # Propagates down
```

---

## 64. HELIOS Knowledge — Content-Addressable Storage

### 64.1 Content-Addressed Units

InformationUnits can optionally be stored by content hash, enabling automatic deduplication:

```omni
struct ContentAddressedUnit:
    content_hash:       [u8; 32]             # BLAKE3 hash of canonical content
    unit:               InformationUnit
    reference_count:    u32                  # Number of references to this content
    first_seen:         Timestamp

fn store_content_addressed(
    store: &mut KnowledgeStore,
    unit: InformationUnit,
) -> [u8; 32] {
    let hash = blake3::hash(&canonicalize(&unit));
    if store.has_content(hash) {
        store.increment_ref(hash);
    } else {
        store.insert(hash, unit);
    }
    hash
}

fn deduplicate_store(store: &mut KnowledgeStore) -> DeduplicationReport {
    // Scan all units, compute hashes, merge duplicates
    // Preserve highest-confidence version, update references
}

struct DeduplicationReport:
    units_scanned:      u64
    duplicates_found:   u64
    bytes_saved:        u64
    merge_conflicts:    Vec<(u64, u64)>      # Pairs that differed in confidence/metadata
```

---

## 65. HELIOS Brain — Semantic Query Caching

### 65.1 Embedding-Based Cache Lookup

HELIOS caches query results and uses semantic similarity (not exact match) for cache hits:

```omni
struct SemanticCache:
    cache_entries:      Vec<CacheEntry>
    embedding_model:    EmbeddingModel
    similarity_threshold: f32                # Cosine similarity for cache hit (default 0.92)
    max_entries:        u32                  # Maximum cache size (default 10000)
    ttl:                Duration             # Time-to-live per entry (default 1 hour)

struct CacheEntry:
    query_embedding:    Vec<f32>             # Dense vector of the original query
    query_text:         String
    result:             String
    confidence:         u8
    created_at:         Timestamp
    hit_count:          u32

fn cache_lookup(
    cache: &SemanticCache,
    query: &str,
) -> Option<&CacheEntry> {
    let query_emb = cache.embedding_model.encode(query);
    cache.cache_entries.iter()
        .filter(|e| cosine_similarity(&query_emb, &e.query_embedding) >= cache.similarity_threshold)
        .max_by(|a, b| a.hit_count.cmp(&b.hit_count))
}
```

---

## 66. Omni Language — Formal API Contracts and Design-by-Contract

### 66.1 Preconditions, Postconditions, and Invariants

Omni supports Design-by-Contract with compiler-enforced contracts:

```omni
fn binary_search(arr: &[i32], target: i32) -> Option<usize>
    requires arr.is_sorted()                 # Precondition: array must be sorted
    requires arr.len() > 0                   # Precondition: non-empty
    ensures result.is_none() || arr[result.unwrap()] == target  # Postcondition
{
    // Implementation...
}

struct BoundedQueue<T>:
    items:      Vec<T>
    capacity:   usize
    invariant self.items.len() <= self.capacity  # Class invariant: never exceed capacity

    fn enqueue(&mut self, item: T)
        requires self.items.len() < self.capacity
        ensures self.items.len() == old(self.items.len()) + 1
    {
        self.items.push(item);
    }
```

### 66.2 Contract Enforcement Modes

```omni
enum ContractMode:
    Enabled                                  # Check all contracts at runtime (debug builds)
    AssertionsOnly                            # Check preconditions only (release builds)
    StaticVerification                        # Verify at compile time via SMT solver (Section 11)
    Disabled                                 # No checks (maximum performance)
```

---

## 67. HELIOS Operations — Configuration Management

### 67.1 Typed Configuration System

HELIOS uses a strongly-typed, hierarchical configuration system:

```omni
struct ConfigurationSystem:
    layers:             Vec<ConfigLayer>      # Priority-ordered config sources
    schema:             ConfigSchema          # Expected types and defaults
    change_listeners:   Vec<fn(&ConfigChange)>

enum ConfigLayer:
    Defaults                                 # Built-in defaults (lowest priority)
    File { path: PathBuf, format: ConfigFormat }
    Environment                              # Environment variables (HELIOS_*)
    CommandLine                              # CLI arguments
    Remote { endpoint: String }              # Remote config server (highest priority)

enum ConfigFormat:
    TOML
    YAML
    JSON

struct ConfigChange:
    key:                String
    old_value:          Option<String>
    new_value:          String
    source:             ConfigLayer
    timestamp:          Timestamp
```

### 67.2 Live Configuration Reload

Configuration changes can be applied without restart:

```omni
struct LiveConfigPolicy:
    hot_reloadable:     Vec<String>          # Config keys that can change at runtime
    requires_restart:   Vec<String>          # Config keys that need restart
    validation:         fn(&Config) -> Result<(), ConfigError>  # Validate before applying
```

---

## Appendix I — Phase Implementation Sequence (v7.0 Additions)

**Phase BB — Neuro-Symbolic and Federated Learning**
1. Implement dual-process System 1/System 2 reasoning architecture.
2. Implement knowledge grounding with strict/soft modes.
3. Implement federated learning protocol with FedAvg and SecureAggregation.
4. Implement gradient compression and differential privacy for FL.
5. Acceptance test: Neural-first query escalates to symbolic when confidence < 0.7. Federated training across 3 simulated instances converges within 10 rounds. No raw knowledge is transmitted between instances.

**Phase CC — Omni Language Advanced Features**
1. Implement abstract interpretation with interval, nullability, and taint tracking domains.
2. Implement compile-time reflection via derive macros.
3. Implement hot code reloading with module replacement and function patching.
4. Implement structured concurrency with task groups and hierarchical cancellation.
5. Acceptance test: Abstract interpreter detects a known null-deref bug in sample code. Hot reload replaces a function while preserving module state. Task group correctly cancels sibling tasks when one fails.

**Phase DD — Storage and Caching**
1. Implement content-addressable storage with automatic deduplication.
2. Implement semantic query caching with embedding-based similarity lookup.
3. Acceptance test: Storing 100 duplicate units results in 1 stored unit with ref_count 100. Semantic cache hits on rephrased query with cosine similarity > 0.92. Cache miss for semantically unrelated query.

**Phase EE — Contracts and Configuration**
1. Implement Design-by-Contract with preconditions, postconditions, and invariants.
2. Implement contract enforcement modes (Enabled, AssertionsOnly, StaticVerification, Disabled).
3. Implement typed hierarchical configuration with live reload.
4. Acceptance test: Contract violation raises a clear error with the violated condition. Static verification mode proves a simple function's postcondition at compile time. Live config reload changes a hot-reloadable key without restart. Changing a requires-restart key logs a warning.

---

## Appendix J — Acceptance Test Specifications (v7.0 Additions)

**Phase BB acceptance:** Query "What causes rain?" with neural confidence 0.5. Assert neural-first mode escalates to symbolic reasoning. Assert symbolic layer produces an answer grounded in KG with confidence ≥ 0.8. Set up 3 HELIOS instances with disjoint knowledge. Run FedAvg for 10 rounds on a shared KGE model. Assert the global model's link prediction MRR exceeds any individual instance's MRR. Assert no raw InformationUnit content was transmitted — only gradient updates.

**Phase CC acceptance:** Write Omni code with a known null-dereference path. Run abstract interpreter with NullabilityDomain. Assert it reports PossibleNullDeref at the correct line. Apply `#[derive(Reflect)]` to a struct with 5 fields. Assert compile-time reflection correctly reports all 5 field names and types. Assert `serialize_any()` produces valid output. Run an Omni program, modify a function while running, trigger hot reload. Assert the new function code executes on the next invocation. Assert module state is preserved. Create a task group with 3 tasks. Make task 2 fail. Assert tasks 1 and 3 are cancelled via CancellationToken propagation.

**Phase DD acceptance:** Insert the InformationUnit "Paris is the capital of France" 50 times via content-addressed storage. Assert storage contains exactly 1 unit with reference_count 50. Run deduplication on a store with 1000 units (200 duplicates). Assert 200 duplicates merged and bytes_saved > 0. Ask "What is France's capital?" and cache the result. Then ask "Capital city of France?" Assert semantic cache hit (cosine similarity ≥ 0.92). Ask "What is 2+2?" Assert cache miss.

**Phase EE acceptance:** Call `binary_search` on an unsorted array. Assert contract violation error mentioning `arr.is_sorted()`. Call `enqueue` on a full BoundedQueue. Assert contract violation mentioning capacity. Switch to StaticVerification mode. Write a function with a provable postcondition. Assert compiler verifies it without runtime checks. Create a config file with `log_level = "debug"`. Add `log_level` to hot_reloadable list. Change file to `log_level = "info"`. Assert live reload applies without restart. Assert ConfigChange event has correct old and new values.

---

*End of v7.0 sections — v8.0 sections follow.*

---

## 68. Plugin System — WASI Component Model Integration

### 68.1 Component-Based Plugin Architecture

Plugins can be distributed as WASI components, enabling language-agnostic, capability-sandboxed execution:

```omni
struct WASIPluginConfig:
    component_path:     PathBuf              # Path to .wasm component
    capabilities:       Vec<WASICapability>  # Granted system capabilities
    memory_limit:       u32                  # Max linear memory in MB (default 64)
    fuel_limit:         Option<u64>          # Instruction fuel limit (None = unlimited)

enum WASICapability:
    FileRead { paths: Vec<PathBuf> }         # Read access to specific paths
    FileWrite { paths: Vec<PathBuf> }
    NetworkConnect { hosts: Vec<String> }    # Connect to specific hosts
    EnvironmentRead { vars: Vec<String> }   # Read specific env vars
    Stdout                                   # Write to stdout
    RandomSource                             # Access random number generator
    Clocks                                   # Access monotonic/wall clocks
```

### 68.2 Inter-Component Communication

WASI components communicate via strongly-typed interfaces defined in WIT (WebAssembly Interface Types):

```omni
// Auto-generated Omni bindings from WIT interface
trait PluginInterface:
    fn process_unit(unit: &InformationUnit) -> PluginResult
    fn get_capabilities() -> Vec<PluginCapability>
    fn health_check() -> bool
```

---

## 69. HELIOS Knowledge — Zero-Knowledge Verifiable Queries

### 69.1 Verifiable Query Proofs

HELIOS can produce cryptographic proofs that query results are correct without revealing the underlying knowledge:

```omni
struct VerifiableQueryConfig:
    proof_system:       ZKProofSystem
    circuit_cache:      HashMap<String, CompiledCircuit>  # Cache compiled query circuits

enum ZKProofSystem:
    Groth16                                  # zkSNARK: compact proofs, trusted setup
    PLONK                                    # zkSNARK: universal trusted setup
    STARK                                    # zkSTARK: transparent, quantum-resistant
    Bulletproofs                              # No trusted setup, range proofs

struct VerifiableQueryResult:
    result:             String
    proof:              Vec<u8>              # Cryptographic proof of correctness
    public_inputs:      Vec<u8>              # Public circuit inputs (query hash)
    verification_key:   Vec<u8>              # Key for independent verification

fn verify_query_proof(
    result: &VerifiableQueryResult,
    verification_key: &[u8],
) -> bool
```

---

## 70. HELIOS Knowledge — Differential Dataflow Views

### 70.1 Incremental Materialized Views

Knowledge queries can be maintained as incrementally-updated materialized views:

```omni
struct MaterializedView:
    view_id:            u64
    query:              String               # OQL query defining the view
    result_set:         Vec<InformationUnit>
    last_updated:       Timestamp
    update_strategy:    ViewUpdateStrategy

enum ViewUpdateStrategy:
    DifferentialDataflow                     # Process only deltas (insertions/deletions)
    PeriodicRefresh { interval: Duration }   # Full recomputation on schedule
    OnDemand                                 # Recompute only when queried

struct ViewDelta:
    insertions:         Vec<InformationUnit>
    deletions:          Vec<u64>              # Unit IDs removed from view
    modifications:      Vec<(u64, UnitDiff)> # Modified units with diffs
```

---

## 71. Omni Language — Memory Pool and Arena Allocators

### 71.1 Arena Allocator

Omni provides a built-in arena allocator for batch allocation of objects with the same lifetime:

```omni
struct Arena:
    chunks:             Vec<*mut u8>
    chunk_size:         usize                # Default 64 KB
    current_offset:     usize
    total_allocated:    usize

impl Arena:
    fn alloc<T>(&mut self) -> &mut T         # Bump-allocate a T
    fn alloc_slice<T>(&mut self, len: usize) -> &mut [T]
    fn reset(&mut self)                      # Free all allocations at once
    fn bytes_used(&self) -> usize

// Usage: AST nodes during compilation
let arena = Arena::new(64 * 1024);
let node = arena.alloc::<ASTNode>();
// All nodes freed at once when arena is dropped
```

### 71.2 Slab Allocator for InformationUnits

```omni
struct SlabAllocator<T>:
    slabs:              Vec<Slab<T>>
    free_list:          Vec<usize>           # Indices of free slots
    slab_capacity:      usize               # Objects per slab (default 256)

struct Slab<T>:
    data:               Vec<Option<T>>
    used_count:         usize

impl SlabAllocator<T>:
    fn allocate(&mut self) -> (usize, &mut T)  # Returns (slot_id, reference)
    fn deallocate(&mut self, slot_id: usize)
    fn utilization(&self) -> f32             # Fraction of slots in use
```

---

## 72. Omni Tooling — Language Server Protocol

### 72.1 Omni LSP Server

The Omni compiler ships with a built-in LSP server for IDE integration:

```omni
struct OmniLSPConfig:
    capabilities:       Vec<LSPCapability>
    diagnostics_delay:  Duration             # Debounce diagnostics (default 300ms)
    semantic_tokens:    bool                 # Enable semantic token highlighting (default true)

enum LSPCapability:
    Completion                               # Auto-complete suggestions
    Hover                                    # Type information on hover
    GoToDefinition                           # Navigate to symbol definition
    FindReferences                           # Find all references to a symbol
    Rename                                   # Project-wide rename
    SemanticTokens                           # Type-aware syntax highlighting
    CodeActions                              # Quick fixes and refactorings
    Diagnostics                              # Real-time error/warning reporting
    InlayHints                               # Inlay type hints and parameter names
    SignatureHelp                            # Function signature display
```

---

## 73. HELIOS Brain — AI Safety Guardrails

### 73.1 Input and Output Filtering

HELIOS implements multi-layered safety guardrails for all reasoning outputs:

```omni
struct SafetyGuardrails:
    input_filters:      Vec<InputFilter>
    output_filters:     Vec<OutputFilter>
    red_team_tests:     Vec<RedTeamTest>
    audit_log:          bool                 # Log all filtered content (default true)

enum InputFilter:
    PromptInjectionDetector                  # Detect adversarial prompt manipulation
    PIIScanner                               # Section 39.5 PII detection
    ToxicityClassifier { threshold: f32 }    # Block toxic queries
    TopicBlocklist(Vec<String>)              # Block queries on specific topics

enum OutputFilter:
    FactualGrounding                         # Section 58.2 grounding check
    HallucinationDetector { threshold: f32 }
    BiasScanner                              # Detect demographic bias in responses
    ConfidenceGate { min_confidence: u8 }    # Suppress low-confidence answers
    ContentPolicyEnforcer(ContentPolicy)     # Enforce org-specific content policies

struct ContentPolicy:
    allowed_topics:     Option<Vec<String>>  # Whitelist (None = all allowed)
    blocked_topics:     Vec<String>          # Blacklist
    max_response_length: Option<u32>
    require_citations:  bool                 # Must cite source InformationUnit IDs
```

---

## 74. HELIOS Operations — Rate Limiting and Backpressure

### 74.1 Query Rate Limiting

```omni
struct RateLimiter:
    algorithm:          RateLimitAlgorithm
    limits:             Vec<RateLimit>

enum RateLimitAlgorithm:
    TokenBucket { capacity: u32, refill_rate: f32 }   # Tokens per second
    SlidingWindow { window: Duration, max_requests: u32 }
    LeakyBucket { rate: f32 }

struct RateLimit:
    scope:              RateLimitScope
    limit:              u32                  # Requests per window
    window:             Duration
    action:             RateLimitAction

enum RateLimitScope:
    Global                                   # All queries
    PerUser(ActorId)
    PerPlugin(String)
    PerQueryType(String)

enum RateLimitAction:
    Reject                                   # Return error immediately
    Queue                                    # Queue and process when capacity available
    Degrade                                  # Apply degradation policy (Section 52.3)
```

---

## 75. HELIOS Operations — Distributed Tracing Correlation

### 75.1 Trace Context Propagation

Every HELIOS operation carries a trace context for end-to-end observability:

```omni
struct TraceContext:
    trace_id:           [u8; 16]             # W3C Trace Context trace-id
    span_id:            [u8; 8]              # Current span-id
    parent_span_id:     Option<[u8; 8]>
    trace_flags:        u8                   # Sampled flag
    baggage:            HashMap<String, String>  # Cross-cutting metadata

struct Span:
    context:            TraceContext
    operation_name:     String               # e.g., "query.execute", "rule.fire"
    start_time:         Timestamp
    end_time:           Option<Timestamp>
    attributes:         HashMap<String, String>
    events:             Vec<SpanEvent>
    status:             SpanStatus

enum SpanStatus:
    Ok
    Error { message: String }
    Unset
```

---

## 76. Omni Tooling — Benchmark Harness

### 76.1 Built-in Benchmarking

Omni includes a benchmarking framework for reproducible performance measurement:

```omni
struct Benchmark:
    name:               String
    setup:              Option<fn()>
    teardown:           Option<fn()>
    routine:            fn()
    iterations:         BenchmarkIterations
    warmup:             u32                  # Warmup iterations (default 10)

enum BenchmarkIterations:
    Auto                                     # Framework determines optimal count
    Fixed(u32)                               # Run exactly N iterations
    TimeBudget(Duration)                     # Run for at least this duration

struct BenchmarkResult:
    name:               String
    mean_ns:            f64
    median_ns:          f64
    std_dev_ns:         f64
    min_ns:             f64
    max_ns:             f64
    iterations:         u32
    throughput:         Option<f64>          # Operations per second
```

---

## 77. Omni Language — Cross-Compilation Targets

### 77.1 Target Triple System

The Omni compiler supports cross-compilation to multiple platforms:

```omni
struct CompilationTarget:
    arch:               Architecture
    os:                 OperatingSystem
    abi:                Option<ABI>

enum Architecture:
    X86_64
    AArch64
    RISCV64
    WASM32                                   # WebAssembly 32-bit
    WASM64                                   # WebAssembly 64-bit

enum OperatingSystem:
    Linux
    Windows
    MacOS
    WASI                                     # WebAssembly System Interface
    Bare                                     # No OS (embedded/kernel)

enum ABI:
    GNU
    MSVC
    Musl                                     # Static linking on Linux
    EABI                                     # Embedded ABI
```

---

## Appendix K — Phase Implementation Sequence (v8.0 Additions)

**Phase FF — Component Model and Cryptographic Integrity**
1. Implement WASI component model plugin loading with capability-based sandboxing.
2. Implement WIT interface bindings for Omni ↔ WASM interop.
3. Implement zero-knowledge proof generation for OQL queries (Groth16/STARK).
4. Acceptance test: Load a WASI plugin, grant FileRead to one directory only. Assert plugin can read that directory but not others. Generate a ZK proof for a factual query. Verify the proof independently. Assert verification succeeds.

**Phase GG — Dataflow, Allocators, and Tooling**
1. Implement differential dataflow for incremental materialized views.
2. Implement arena and slab allocators in the Omni standard library.
3. Implement Omni LSP server with 10 capabilities.
4. Acceptance test: Create a materialized view. Insert 100 units. Assert view contains correct results. Insert 1 more unit. Assert only the delta (1 insertion) is processed. Allocate 10,000 objects via arena. Assert total time < 1ms. Verify LSP returns correct completions, hover info, and diagnostics.

**Phase HH — Safety, Rate Limiting, and Observability**
1. Implement AI safety guardrails with input/output filters.
2. Implement rate limiting with token bucket and sliding window algorithms.
3. Implement W3C-compatible distributed tracing with span propagation.
4. Acceptance test: Submit a prompt injection attempt. Assert InputFilter blocks it. Submit a query exceeding rate limit. Assert rejection or queueing. Execute a multi-step query. Assert trace contains correct parent-child span hierarchy.

**Phase II — Benchmarking and Cross-Compilation**
1. Implement benchmark harness with auto-iteration and statistical reporting.
2. Implement cross-compilation for x86_64, AArch64, RISC-V, WASM32, and WASI targets.
3. Acceptance test: Benchmark a known O(n²) function. Assert mean time scales quadratically with input size. Cross-compile a Hello World program for WASM32. Assert output runs in Wasmtime.

---

## Appendix L — Acceptance Test Specifications (v8.0 Additions)

**Phase FF acceptance:** Load a WASI component plugin that reads a file. Grant FileRead to `/data/allowed`. Assert plugin successfully reads `/data/allowed/test.txt`. Assert plugin fails to read `/data/secret/private.txt` with capability denied error. Assert plugin's memory usage stays under the configured limit. Generate a zkSTARK proof for the query `FIND facts WHERE topic = "physics" AND confidence > 80`. Provide the proof and verification key to an independent verifier. Assert verification passes. Modify one byte of the proof. Assert verification fails.

**Phase GG acceptance:** Create a materialized view: `SELECT * FROM units WHERE confidence > 90`. Insert 100 units: 50 with confidence > 90, 50 with confidence ≤ 90. Assert view contains exactly 50 units. Insert 1 unit with confidence 95. Assert differential update processes only 1 delta (not re-scanning 101 units). Assert view now contains 51 units. Allocate 100,000 `InformationUnit`-sized objects via slab allocator. Assert allocation throughput > 10M ops/sec. Assert all slots are deallocatable and reusable. Start the Omni LSP server. Open a file with a type error. Assert diagnostics report the error within 500ms. Type a partial function name. Assert completion list includes the correct function with signature.

**Phase HH acceptance:** Configure guardrails with ToxicityClassifier(threshold=0.7). Send a clearly toxic query. Assert it's blocked before reaching the reasoning engine. Send a benign query. Assert it passes all filters and returns a result. Configure TokenBucket(capacity=5, refill_rate=1). Send 6 requests within 1 second. Assert the 6th request is rate-limited. Wait 1 second. Assert the next request succeeds. Execute a query that invokes NLU → RETE → GNN. Assert the trace contains 3 spans with correct parent-child relationships. Assert trace_id is consistent across all spans.

**Phase II acceptance:** Define a benchmark for `vec_sort(n)`. Run with n=100, 1000, 10000. Assert mean_ns scales as O(n log n). Assert std_dev_ns < 10% of mean_ns for each size. Cross-compile `fn main() { print("Hello HELIOS") }` for targets: x86_64-linux-gnu, aarch64-linux-gnu, wasm32-wasi. Assert all three binaries are produced. Assert WASM32 binary runs successfully in Wasmtime and prints "Hello HELIOS".

---

*End of v8.0 sections — v9.0 sections follow.*

---

## 78. Omni Language — Algebraic Effect System

> **Cross-reference:** This section refines and extends §34 (Algebraic Effects, v5.0) with the `handler X for Effect` sugar syntax and additional effect composition patterns. §34 defines the foundational effect type declarations and handler semantics; this section adds the ergonomic surface syntax and advanced composition rules. Implementations should treat §34 as the canonical effect system specification and this section as its syntactic and compositional extension.


### 78.1 Effect Annotations

Omni tracks computational side effects in function signatures using an algebraic effect system:

```omni
// Effects are declared as interfaces
effect IO:
    fn read_line() -> String
    fn print(msg: &str)

effect State<S>:
    fn get() -> S
    fn put(s: S)

effect Exception<E>:
    fn raise(e: E) -> !

// Functions declare their effects after a / separator
fn greet_user() / IO:
    let name = perform IO.read_line()
    perform IO.print("Hello, " ++ name)

fn safe_divide(a: f64, b: f64) / Exception<DivByZero>:
    if b == 0.0:
        perform Exception.raise(DivByZero)
    a / b
```

### 78.2 Effect Handlers

Effects are interpreted by handlers, enabling different implementations for the same effect:

```omni
// Handler: interpret IO effect for console
handler ConsoleIO for IO:
    fn read_line() -> String { stdin.read_line() }
    fn print(msg: &str) { stdout.write(msg) }

// Handler: interpret IO effect for testing
handler MockIO for IO:
    inputs: Vec<String>
    outputs: Vec<String>
    fn read_line() -> String { self.inputs.pop() }
    fn print(msg: &str) { self.outputs.push(msg.to_string()) }
```

---

## 79. HELIOS Brain — Automatic Differentiation Engine

### 79.1 Dual-Mode AD

HELIOS includes an automatic differentiation engine for training and fine-tuning internal models:

```omni
struct AutoDiffEngine:
    mode:               ADMode
    tape:               Option<ComputationTape>  # Reverse-mode tape
    gradient_checkpointing: bool             # Trade compute for memory

enum ADMode:
    ForwardMode                              # Propagate tangents forward (few inputs)
    ReverseMode                              # Record tape, backpropagate (many inputs)

struct ComputationTape:
    nodes:              Vec<TapeNode>
    gradients:          HashMap<u64, Tensor>

struct TapeNode:
    id:                 u64
    op:                 Operation
    inputs:             Vec<u64>
    output:             Tensor
    local_grad:         fn(&[Tensor]) -> Vec<Tensor>  # Local gradient function

fn gradient<F: Fn(Tensor) -> Tensor>(f: F, x: &Tensor) -> Tensor
fn jacobian<F: Fn(Tensor) -> Tensor>(f: F, x: &Tensor) -> Tensor
fn hessian<F: Fn(Tensor) -> Tensor>(f: F, x: &Tensor) -> Tensor
```

---

## 80. HELIOS Knowledge — Distributed Graph Partitioning

### 80.1 Knowledge Graph Sharding

Large knowledge graphs can be partitioned across multiple HELIOS instances:

```omni
struct PartitionConfig:
    strategy:           PartitionStrategy
    num_partitions:     u16
    rebalance_threshold: f32                 # Imbalance ratio triggering rebalance (default 0.2)

enum PartitionStrategy:
    EdgeCut                                  # Partition vertices, edges may cross
    VertexCut                                # Partition edges, vertices may replicate
    HashPartition { key: PartitionKey }      # Hash-based assignment
    CommunityDetection                       # Partition by detected communities (Louvain)

enum PartitionKey:
    SubjectHash                              # Hash on subject entity
    PredicateHash                            # Hash on relation type
    DomainHash                               # Hash on topic/domain

struct PartitionMetrics:
    partition_sizes:    Vec<u64>             # Units per partition
    edge_cut_ratio:     f32                  # Fraction of edges crossing partitions
    replication_factor: f32                  # Average vertex replication (vertex-cut only)
    load_imbalance:     f32                  # Max/avg partition size ratio
```

---

## 81. Omni Language — Advanced Pattern Matching

### 81.1 Exhaustive Pattern Matching with Guards

Omni extends pattern matching with guards, nested patterns, and exhaustiveness checking:

```omni
fn classify_unit(unit: &InformationUnit) -> UnitCategory:
    match (unit.confidence, unit.source, unit.relations.len()):
        (90..=100, Source::Verified(_), n) if n > 5 =>
            UnitCategory::HighValueHub
        (70..=89, _, n) if n > 0 =>
            UnitCategory::Connected
        (c, Source::UserInput(user), _) if user.is_trusted() =>
            UnitCategory::TrustedUserContent
        (c, _, _) if c < 30 =>
            UnitCategory::LowConfidence
        _ =>
            UnitCategory::Standard

// Compiler enforces exhaustiveness: all patterns must be covered
// Compiler warns on unreachable patterns
```

---

## 82. Omni Language — Async Iterators and Streams

### 82.1 Async Iterator Protocol

Omni supports async iteration for streaming data processing:

```omni
trait AsyncIterator<T>:
    async fn next(&mut self) -> Option<T>

trait AsyncStream<T>: AsyncIterator<T>:
    fn map<U>(self, f: fn(T) -> U) -> MappedStream<T, U>
    fn filter(self, pred: fn(&T) -> bool) -> FilteredStream<T>
    fn take(self, n: usize) -> TakeStream<T>
    fn buffer(self, size: usize) -> BufferedStream<T>
    async fn collect(self) -> Vec<T>
    async fn for_each(self, f: fn(T))

// Usage: stream knowledge updates
async fn process_updates(store: &KnowledgeStore):
    store.watch_changes()
        .filter(|change| change.confidence > 50)
        .map(|change| enrich(change))
        .buffer(100)
        .for_each(|enriched| index(enriched))
        .await
```

---

## 83. Omni Tooling — Code Generation Templates

### 83.1 Typed Template Engine

The Omni toolchain includes a code generation system for boilerplate reduction:

```omni
struct CodeTemplate:
    name:               String
    parameters:         Vec<TemplateParam>
    template_body:      String               # Template with {{ placeholder }} syntax
    output_path:        PathBuf

struct TemplateParam:
    name:               String
    param_type:         TemplateType
    default:            Option<String>

enum TemplateType:
    StringParam
    BoolParam
    ListParam(Box<TemplateType>)
    EnumParam(Vec<String>)

// CLI usage:
// omni generate --template crud_module --name User --fields "name:String,email:String,age:u32"
```

---

## 84. Omni Tooling — Snapshot Testing Framework

### 84.1 Snapshot Assertions

Omni includes a snapshot testing framework for testing complex output:

```omni
struct SnapshotTest:
    name:               String
    snapshot_dir:       PathBuf              # Directory for .snap files

fn assert_snapshot(name: &str, value: &str)  # Compare value against stored snapshot
fn update_snapshots()                        # Re-record all snapshots (--update-snapshots flag)

// Usage in tests
#[test]
fn test_query_output():
    let result = execute_query("FIND facts WHERE topic = 'physics'");
    assert_snapshot("physics_query", &format_result(&result));
    // First run: creates physics_query.snap
    // Subsequent runs: compares against snapshot, fails on diff
```

---

## 85. HELIOS Operations — Dependency Injection Container

### 85.1 Service Registry

HELIOS uses a DI container for modular service composition:

```omni
struct ServiceContainer:
    registrations:      HashMap<TypeId, ServiceRegistration>

enum ServiceLifetime:
    Singleton                                # Single instance for entire application
    Scoped                                   # One instance per scope (e.g., per query)
    Transient                                # New instance every time

struct ServiceRegistration:
    lifetime:           ServiceLifetime
    factory:            fn(&ServiceContainer) -> Box<dyn Any>

impl ServiceContainer:
    fn register<T: 'static>(&mut self, lifetime: ServiceLifetime, factory: fn(&Self) -> T)
    fn resolve<T: 'static>(&self) -> &T
    fn create_scope(&self) -> ScopedContainer
```

---

## 86. HELIOS Brain — Signal Processing Pipeline

### 86.1 Time-Series Knowledge Signals

HELIOS can process temporal knowledge patterns as signals:

```omni
struct SignalPipeline:
    stages:             Vec<SignalStage>

enum SignalStage:
    WindowFunction { window: WindowType, size: usize }
    FFT                                      # Fast Fourier Transform
    InverseFFT
    LowPassFilter { cutoff_hz: f32 }
    HighPassFilter { cutoff_hz: f32 }
    BandPassFilter { low_hz: f32, high_hz: f32 }
    MovingAverage { window: usize }
    AnomalyDetection { z_threshold: f32 }    # Flag values beyond Z standard deviations

enum WindowType:
    Rectangular
    Hamming
    Hanning
    Blackman

// Usage: detect anomalous confidence trends
fn detect_confidence_anomalies(
    units: &[InformationUnit],
    window: usize,
) -> Vec<AnomalyEvent>
```

---

## 87. Omni Tooling — Documentation Generation

### 87.1 Doc Comments and API Documentation

Omni supports doc comments that generate searchable API documentation:

```omni
/// Stores an InformationUnit in the knowledge store.
///
/// # Arguments
/// * `store` - The target knowledge store
/// * `unit` - The unit to store
///
/// # Returns
/// The assigned unit ID, or an error if validation fails.
///
/// # Example
/// ```omni
/// let id = store_unit(&mut store, unit)?;
/// ```
fn store_unit(store: &mut KnowledgeStore, unit: InformationUnit) -> Result<u64, StoreError>

struct DocGenConfig:
    output_format:      DocFormat
    include_private:    bool                 # Include private items (default false)
    include_source:     bool                 # Link to source code (default true)
    search_index:       bool                 # Generate search index (default true)

enum DocFormat:
    HTML                                     # Static HTML site (like rustdoc)
    Markdown                                 # Markdown files
    JSON                                     # Machine-readable JSON API
```

---

## Appendix M — Phase Implementation Sequence (v9.0 Additions)

**Phase JJ — Language Semantics and AI Compute**
1. Implement algebraic effect system with effect annotations, declarations, and handlers.
2. Implement automatic differentiation with forward and reverse modes, tape recording, and gradient computation.
3. Implement graph partitioning with 4 strategies (edge-cut, vertex-cut, hash, community detection).
4. Acceptance test: Declare an IO effect. Handle it with ConsoleIO and MockIO handlers. Assert MockIO captures all outputs. Compute gradient of `f(x) = x³` at x=2. Assert gradient = 12.0. Partition a 10,000-node graph into 4 partitions. Assert load imbalance < 0.2.

**Phase KK — Language Features and Developer UX**
1. Implement advanced pattern matching with guards, ranges, nested patterns, and exhaustiveness checking.
2. Implement async iterators with map/filter/take/buffer/collect combinators.
3. Implement code generation templates with typed parameters.
4. Acceptance test: Write a match expression missing a case. Assert compiler error for non-exhaustive pattern. Stream 1000 items through filter+map+buffer pipeline. Assert correct output and no items lost. Generate a CRUD module from a template. Assert output compiles successfully.

**Phase LL — Testing, DI, and Analytics**
1. Implement snapshot testing with snapshot creation, comparison, and update modes.
2. Implement dependency injection container with singleton/scoped/transient lifetimes.
3. Implement signal processing pipeline for time-series knowledge analysis.
4. Implement documentation generation from doc comments.
5. Acceptance test: Run a snapshot test. Assert it creates the .snap file on first run. Modify output. Assert test fails on second run. Update snapshots. Assert test passes. Register a service as Singleton. Resolve it twice. Assert same instance. Register as Transient. Resolve twice. Assert different instances. Feed 100 confidence values with one outlier through anomaly detection. Assert the outlier is flagged. Generate HTML docs for a module. Assert all public functions have documentation pages.

---

## Appendix N — Acceptance Test Specifications (v9.0 Additions)

**Phase JJ acceptance:** Define `effect Log: fn log(msg: &str)`. Define `handler TestLog for Log` that appends to a Vec. Run a function that performs 3 log operations under TestLog. Assert Vec contains exactly 3 messages. Define `f(x) = x² + 3x + 2`. Compute gradient via reverse-mode AD at x=5. Assert gradient = 13.0 (2*5 + 3). Assert forward-mode produces the same result. Create a knowledge graph with 50,000 units and 200,000 relations. Partition using CommunityDetection into 8 partitions. Assert all partitions have between 4,000 and 8,000 units. Assert edge_cut_ratio < 0.15.

**Phase KK acceptance:** Write `match x: (1..=5, _) => "low", (6..=10, true) => "high-active"` without a default. Assert compiler reports non-exhaustive pattern for cases like `(6..=10, false)`. Create an AsyncStream of integers 1..100. Apply `.filter(|x| x % 2 == 0).map(|x| x * 10).take(5).collect().await`. Assert result is `[20, 40, 60, 80, 100]`. Define a template "entity" with params name:String, fields:List<String>. Generate code. Assert output contains a struct with the correct name and fields. Assert output compiles.

**Phase LL acceptance:** Create a snapshot test `assert_snapshot("hello", "Hello World")`. Run once. Assert `hello.snap` is created with content "Hello World". Change to "Hello HELIOS". Run again. Assert test fails with diff showing "World" → "HELIOS". Run with `--update-snapshots`. Assert test passes and .snap is updated. Register `KnowledgeStore` as Singleton. Create two scopes. Resolve in both. Assert same instance. Register `QueryHandler` as Scoped. Resolve twice in same scope: same instance. Resolve in different scope: different instance. Generate 200 confidence readings: 199 values between 70-80, 1 value at 150. Apply AnomalyDetection(z_threshold=2.0). Assert exactly 1 anomaly flagged at the outlier value. Generate docs with `DocFormat::HTML`. Assert output directory contains `index.html` and at least one function page. Assert search index JSON is valid.

---

## 88. Conflict-Free Replicated Data Types (CRDTs)

**Problem:** Edge and Local-First HELIOS deployments experience network partitions. Decentralized synchronization requires automatic conflict resolution without relying on a central coordinator.
**Solution:** Implement native state-based and operation-based CRDTs (Conflict-Free Replicated Data Types) in the Omni standard library for seamless offline-first synchronization.

**Omni Specification:**
```omni
use sync::crdt::{LWWRegister, ORSet, AutomergeDoc};

// An Observed-Remove Set for distributed tagging
let mut tags = ORSet::<String>::new(replica_id);
tags.add("urgent");

// A text document synced via Automerge-style sequence CRDT
let mut doc = AutomergeDoc::new();
doc.insert(0, "HELIOS");

// Merge states from another disconnected node
tags.merge(remote_tags);
```

**Implementation details:**
*   Implement standard CRDT primitives: G-Counter, PN-Counter, LWW-Register, OR-Set, and Sequence CRDTs (RGA or YATA algorithm). Use Delta State CRDTs (Almeida et al., 2018) to send only per-mutation deltas rather than full state, reducing bandwidth by orders of magnitude for large replicated collections.
*   Add a local-first network layer that gossips CRDT delta-state vectors over TCP/QUIC connections (QUIC preferred for low-latency LAN discovery; TCP fallback for WAN reconnection) when HELIOS instances discover each other. Use delta vector clocks rather than full vector clocks for causality tracking metadata to minimize per-node memory overhead. CRDTs are scoped to the federation protocol defined in Section 35 and do not require a central coordinator.
*   Ensure the Knowledge Store can materialize views over distributed CRDT collections. Leverage Merkle-CRDT integration with §98's Merkle tree anti-entropy and §64's content-addressable storage for transport deduplication and scalable synchronization.
*   **Byzantine CRDT note (future extension):** Current ORSet/G-Counter primitives assume non-adversarial replicas. A future extension should implement signed-hash "blocklace" equivocation prevention for federation scenarios involving untrusted HELIOS instances. This is reserved for a future version and is not part of the initial implementation.

---

## 89. Software/Hardware Transactional Memory (STM/HTM)

**Problem:** Complex concurrent manipulations of the Knowledge Graph using traditional locks lead to deadlocks, priority inversion, and poor composability.
**Solution:** Introduce Transactional Memory at the language level. Use STM with atomic blocks, transparently falling back to HTM (Hardware Transactional Memory — ARM TME and equivalent CPU facilities where available and microcode-enabled) when hardware supports it. The STM path is always correct; HTM is a best-effort fast path only.

**Omni Specification:**
```omni
use concurrency::stm::{TVar, atomically};

let account_a = TVar::new(100.0);
let account_b = TVar::new(50.0);

// Atomic block: executes entirely or retries if conflicts occur
atomically(|| {
    let a_bal = account_a.read()?;
    if a_bal >= 20.0 {
        account_a.write(a_bal - 20.0)?;
        let b_bal = account_b.read()?;
        account_b.write(b_bal + 20.0)?;
    }
    Ok(())
})
```

**Implementation details:**
*   Implement a lock-free STM manager (inspired by Clojure STM and Haskell STM) using a global version clock and per-transaction read/write sets for optimistic concurrency control and conflict detection. HTM is probed at startup and used as an opportunistic fast path; failed HTM transactions fall back to STM automatically.
*   Add an HTM fast-path in the OVM backend using CPU transaction start/commit instructions on platforms where hardware transactional memory is available (e.g., ARM TME on Armv9-A). Detect support at runtime via platform-appropriate capability flags (e.g., `HWCAP2_TME` on ARM). If the hardware transaction aborts due to capacity limits or interrupts, fall back to the software STM implementation.
*   **ARM TME flattened nesting note:** ARM TME uses flattened nesting where nested transactions are subsumed by the outer transaction — when a nested `tbegin` aborts, it causes the entire outer transaction to abort. The `atomically()` block implementation must account for this: nested `atomically()` calls within an HTM-accelerated outer block are collapsed into the single outer hardware transaction, and any abort retries the entire outer block in STM mode.

---

## 90. Capability-Based Security (Object-Capabilities)

**Problem:** Traditional Access Control Lists (ACLs) are vulnerable to the confused deputy problem and struggle with granular, dynamic access delegation in multi-agent environments.
**Solution:** Enforce the Principle of Least Authority (POLA) at the language and runtime level through Object-Capabilities. Security properties are tied to object references rather than user identities.

**Omni Specification:**
```omni
use sec::pola::{Realm, Capability};

// A highly restricted file write capability
resource interface FileWriteCap {
    fn write(&self, data: &[u8]) -> Result<()>;
}

// Agents are passed only the capabilities they need, not ambient authority
fn agent_task(fs_writer: Capability<FileWriteCap>, ds_reader: Capability<DataSetReader>) {
    let data = ds_reader.fetch();
    fs_writer.write(&data).unwrap();
    // Cannot read FS, cannot write to network.
}
```

**Implementation details:**
*   Remove ambient authority (e.g., global `std::fs` or `net`) from the Omni sandbox environment.
*   Implement unforgeable capabilities as language primitives using linear types to ensure capabilities cannot be duplicated if they represent exclusive access.
*   Provide a "Realm" API as the language-level primitive through which Sections 27 and 68 (WASM/WASI plugin sandboxing) derive their capability enforcement. This section defines the underlying Omni type-system primitive; Section 68 defines the plugin-loader integration that consumes it.

---

## 91. Dependent Types and Liquid Types

**Problem:** Critical system invariants (e.g., array bounds, state machine transitions) are typically checked at runtime, introducing overhead and risk of production crashes.
**Solution:** Enhance Omni's type system with Liquid Types (Logically Qualified Data Types)—lightweight dependent types checked by a background SMT solver to prove invariants at compile time without heavy manual proofs.

**Omni Specification:**
```omni
use typing::liquid::*;

// Liquid type refinement: an integer strictly greater than zero
type PositiveInt = { n: i32 | n > 0 };
type SafeIndex<T> = { i: usize | i < T.len() };

fn get_element(arr: &Vec<i32>, idx: SafeIndex<arr>) -> i32 {
    // Array bounds check is ELIMINATED at runtime!
    // Compiler guarantees idx < arr.len() via SMT solver
    unsafe_get(arr, idx) 
}
```

**Implementation details:**
*   Abstract the SMT solver behind an `SmtBackend` trait with Z3 as the default backend and Bitwuzla as an alternative (Bitwuzla outperforms cvc5 in bit-vector and array theory benchmarks directly relevant to `SafeIndex<T>` refinements). The compiler loads the backend specified in the project's `omni.toml` configuration, defaulting to Z3 if unspecified.
*   Implement Predicate Abstraction to automatically infer liquid types during type checking.
*   Allow developers to specify type refinements in method signatures and let the compiler verify that all call sites satisfy the preconditions.
*   Cache SMT solver results keyed by normalized refinement predicate hash to avoid re-solving identical constraints across incremental compilation runs.

---

## 92. Post-Quantum Cryptography (PQC)

**Problem:** "Store now, decrypt later" attacks using future quantum computers threaten the long-term confidentiality of HELIOS distributed knowledge and communications.
**Solution:** Migrate all HELIOS cryptographic infrastructure to NIST standard Post-Quantum Cryptography (PQC) algorithms to ensure quantum safety.

**Omni Specification:**
```omni
use crypto::pqc::{MLKEM, MLDSA};

// Post-Quantum Key Encapsulation (FIPS 203)
let (public_key, secret_key) = MLKEM::generate_keypair(MLKEM_768);
let (ciphertext, shared_secret) = MLKEM::encapsulate(&public_key);

// Post-Quantum Digital Signature (FIPS 204)
let signature = MLDSA::sign(&message, &secret_key);
assert!(MLDSA::verify(&message, &signature, &public_key));
```

**Implementation details:**
*   Begin a **phased deprecation** of RSA and standard ECC in Omni's `crypto` module: Phase 1 — mark deprecated in documentation, emit compiler warnings on use; Phase 2 — disable by default but allow opt-in via `#[allow(deprecated_crypto)]`; Phase 3 — remove after a minimum 3-version transition window. All new connections use hybrid PQ/classical TLS (specifically `X25519MLKEM768` per IETF draft) during Phases 1–2, as specified in Section 30.
*   Implement bindings to NIST standard algorithms (finalized August 2024): ML-KEM / FIPS 203 (Kyber) for key encapsulation/exchange, ML-DSA / FIPS 204 (Dilithium) for digital signatures, and SLH-DSA / FIPS 205 (SPHINCS+) for stateless hash-based signatures.
*   **FN-DSA (FALCON) — pending standard:** FN-DSA is in draft standardization as FIPS 206, offering compact signatures with high throughput well-suited for bandwidth-constrained HELIOS federation channels (§35). Add FN-DSA bindings to `crypto::pqc` once FIPS 206 is finalized.
*   **HQC backup KEM:** In March 2025, NIST selected HQC (Hamming Quasi-Cyclic, a code-based KEM) as a backup to ML-KEM for standardization (expected finalization 2027). The `crypto::pqc` module must be designed for **crypto-agility** — if lattice-based weaknesses are found in ML-KEM, HELIOS can swap to HQC without architectural changes via the `KemBackend` trait abstraction.
*   Update the Multi-Agent Coordination Protocol (ACP) to use hybrid PQ/Classical TLS handshakes with `X25519MLKEM768` as the target hybrid cipher suite during the migration phase, transitioning to pure PQC.


### 92.4 CNSA 2.0 Compliance Timeline

HELIOS’s PQC migration aligns with NSA’s Commercial National Security Algorithm Suite 2.0 (CNSA 2.0):

| Milestone | Date | HELIOS Action |
|-----------|------|---------------|
| CNSA 2.0 published | September 2022 | Begin PQC research and §92 specification |
| FIPS 203/204/205 finalized | August 2024 | Implement ML-KEM, ML-DSA, SLH-DSA bindings |
| New systems must be CNSA 2.0 compliant | January 1, 2027 | All new HELIOS deployments use PQC by default (Phase 2 of §92 deprecation) |
| FIPS 206 (FN-DSA) expected | 2025–2026 | Add FN-DSA bindings when finalized |
| HQC standardization expected | 2027 | Add HQC as backup KEM via `KemBackend` trait |
| Full CNSA 2.0 compliance mandatory | 2033 | Phase 3: remove all classical-only code paths |

HELIOS deployments targeting national-security-adjacent use cases (government knowledge bases, defense analytics) **must** complete the Phase 1→2 transition before January 1, 2027. The 3-version phased deprecation window (§92 implementation details) is designed to complete within this timeline.

---

## 93. Omni Language — Ownership Model and Borrow Checker

### 93.1 Ownership Rules

Omni enforces memory safety at compile time through a single-ownership model with explicit borrowing. No garbage collector is required:

```omni
// Ownership: each value has exactly one owner
let unit = InformationUnit::new(content);  // `unit` owns the value
let moved = unit;                          // Ownership transferred; `unit` is now invalid
// store(unit);   // ERROR: use of moved value

// Cloning is explicit; the original remains valid
let cloned = unit.clone();
store(cloned);
store(unit);   // OK — `unit` still owns its original value
```

### 93.2 Borrowing and Lifetimes

```omni
// Immutable borrow: many readers, no writers
fn read_content(unit: &InformationUnit) -> &str {
    &unit.content   // Lifetime of return value tied to `unit`
}

// Mutable borrow: exclusive access, no concurrent reads
fn update_confidence(unit: &mut InformationUnit, new_score: u8) {
    unit.confidence.final_score = new_score;
}

// Lifetime annotations for struct fields referencing borrowed data
struct QueryContext<'a>:
    query:        &'a str
    working_mem:  &'a mut WorkingMemory

// Compiler error: cannot have &mut and & active simultaneously
fn bad_alias(store: &mut KnowledgeStore, id: u64):
    let ref1 = store.get(id);         // immutable borrow
    store.insert(new_unit);            // ERROR: mutable borrow while `ref1` alive
```

### 93.3 Lifetime Elision Rules

Omni follows three elision rules that cover the majority of cases, avoiding explicit annotation noise:

1. Each `&` input parameter gets its own implicit lifetime parameter.
2. If there is exactly one input lifetime, it is assigned to all output lifetimes.
3. If one of the input lifetimes is `&self` or `&mut self`, it is assigned to all output lifetimes.

```omni
// Explicit:
fn get_subject<'a>(unit: &'a InformationUnit) -> &'a str { &unit.subject }

// Elided (equivalent):
fn get_subject(unit: &InformationUnit) -> &str { &unit.subject }
```

### 93.4 Drop and Resource Cleanup

Types implement `Drop` for deterministic cleanup — no finalizer uncertainty:

```omni
trait Drop:
    fn drop(&mut self)

impl Drop for KnowledgeStore:
    fn drop(&mut self):
        self.flush_dirty_pages()
        self.close_file_handles()
        // Called automatically when `KnowledgeStore` goes out of scope
```

### 93.5 Acceptance Criteria

- `omnic` rejects any program where a moved value is used after the move.
- `omnic` rejects programs with simultaneous `&mut` and `&` borrows of the same value.
- `omnic` correctly elides lifetimes per the three-rule system for all HELIOS API surface types.
- `Drop` implementations are invoked in reverse declaration order for values in the same scope, verifiable by injecting `println` into drop implementations in tests.

### 93.6 Non-Lexical Lifetimes (NLL) and Pin/Unpin

Omni uses Non-Lexical Lifetimes (NLL), where a borrow’s lifetime ends at its **last use**, not at the end of its lexical scope. This matches the Polonius research model adopted by Rust 2018+:

```omni
let mut store = KnowledgeStore::new();
let id = store.get_id();        // immutable borrow of `store` ends HERE (last use of `id`)
store.insert(new_unit);         // OK under NLL — no active borrow of `store`
// Under lexical lifetimes this would fail because `id` is still “in scope”

// Two-phase borrows: the compiler distinguishes reservation from activation
let mut vec = Vec::new();
vec.push(vec.len());            // OK under two-phase: `vec.len()` is a reservation borrow,
                                // activated only when `push` needs `&mut vec`
```

**Pin\<T\> and Unpin for self-referential types:**

Async futures (§14, §63) compile to state machines that may contain self-references. `Pin<T>` guarantees the pinned value will not be moved in memory, making self-referential futures sound:

```omni
use core::pin::{Pin, Unpin};

// Most types are Unpin (movable) by default
struct SimpleQuery: Unpin { query: String }

// Self-referential future state machines are !Unpin
// Pin<Box<T>> ensures the future stays at a fixed memory address
async fn query_store(store: &KnowledgeStore) -> QueryResult {
    let future = store.async_query("France capital");  // returns impl Future + !Unpin
    let pinned: Pin<Box<dyn Future>> = Box::pin(future);
    pinned.await
}
```

**Acceptance Criteria (NLL):**
- The code example `let id = store.get_id(); store.insert(new_unit);` compiles successfully under NLL.
- Under a `--lexical-lifetimes` debug flag, the same code is rejected, confirming NLL is the default behavior.
- `Pin<Box<dyn Future>>` prevents calling `std::mem::swap` on the pinned value — compile-time error.
---

## 94. HELIOS Brain — Belief Revision Protocol

### 94.1 AGM Postulates Applied to InformationUnit Retraction

When a user marks an InformationUnit as `Inaccurate` or `UserRejected`, HELIOS must propagate the retraction to all derived facts. The revision obeys AGM postulates (Alchourrón–Gärdenfors–Makinson) adapted for the knowledge store:

```omni
enum RevisionTrigger:
    UserRejection { unit_id: u64 }
    AutomaticallyRejected { unit_id: u64 }
    ContradictionConfirmed { unit_id: u64, superseded_by: u64 }
    ExpiredFact { unit_id: u64 }

struct RevisionPlan:
    trigger:            RevisionTrigger
    directly_affected:  Vec<u64>    # Units whose source or acquisition_chain includes the retracted unit
    transitively_derived: Vec<u64>  # Units derived from directly_affected via InferredFromRules
    proposed_actions:   Vec<RevisionAction>

enum RevisionAction:
    QueueForVerification(u64)       # Re-verify against other sources
    DemoteConfidence { id: u64, new_score: u8 }
    MarkOutdated(u64)               # Source was valid but superseded
    MarkConflicted(u64)             # Awaiting human resolution
    RetractInference(u64)           # Remove InferredFromRules unit that depended on retracted fact
```

### 94.2 Revision Cascade Algorithm

```omni
fn compute_revision_plan(
    store: &KnowledgeStore,
    trigger: RevisionTrigger,
) -> RevisionPlan:
    let root_id = trigger.unit_id();
    
    // 1. BFS over acquisition_chain and supersedes links
    let directly_affected = bfs_dependents(store, root_id, max_depth=3);
    
    // 2. Identify InferredFromRules units citing affected units
    let transitively_derived = store.index
        .find_inferred_from(directly_affected.ids());
    
    // 3. Score each affected unit
    let actions = directly_affected.union(transitively_derived).map(|id| {
        let unit = store.get(id);
        match unit.source.source_type:
            InferredFromRules if unit.confidence.final_score < 50 =>
                RevisionAction::RetractInference(id)
            InferredFromRules =>
                RevisionAction::QueueForVerification(id)
            _ if unit.corroboration_count >= 2 =>
                // Independently corroborated — downgrade but don't retract
                RevisionAction::DemoteConfidence { id, new_score: unit.confidence.final_score / 2 }
            _ =>
                RevisionAction::QueueForVerification(id)
    });
    
    RevisionPlan { trigger, directly_affected, transitively_derived, proposed_actions: actions }
```

### 94.3 Acceptance Criteria

- Marking unit #X as `UserRejected` produces a `RevisionPlan` within 200ms for stores up to 1,000,000 units.
- All `InferredFromRules` units with confidence < 50 derived solely from #X are retracted (marked `Archived`) atomically.
- Units with independent corroboration (≥ 2 sources) are demoted to half confidence but not retracted.
- The full revision plan is recorded as a `RevisionApplied` event in the Experience Log for audit replay.

---

## 95. HELIOS Brain — Query Cost Model and Cognitive Planner

### 95.1 Layer Cost Estimates

Before dispatching a user query into the cognitive pipeline, the Cognitive Planner selects the minimum-depth layer sufficient to answer it, preserving latency budgets:

```omni
struct CognitivePlan:
    query:              ParsedQuery
    selected_layer:     CognitiveLayer       # L0..L4
    estimated_cost:     CostEstimate
    fallback_layer:     Option<CognitiveLayer>  # If primary layer times out
    rationale:          String               # Human-readable in trace

struct CostEstimate:
    estimated_ms:       u32
    knowledge_units_accessed: u32
    rule_firings_expected: u32
    web_fetch_required: bool

enum CognitivePlannerHeuristic:
    Reflexive          # L0: pure keyword, no reasoning needed
    PatternMatch       # L1: query is a known RETE template
    InferenceRequired  # L2: needs backward chaining
    DeepReason         # L3: requires WM expansion + multi-hop inference
    BackgroundDeferred # L4: too expensive for real-time, schedule async
```

### 95.2 Planner Decision Tree

```omni
fn plan_query(query: &ParsedQuery, store: &KnowledgeStore) -> CognitivePlan:
    // Reflexive check: is it a simple lookup?
    if query.is_direct_lookup() and store.hot_tier_contains(&query.subject):
        return plan(query, CognitiveLayer::L0)

    // Pattern match: does the query match a cached RETE production?
    if let Some(production) = rete_network.find_matching_production(query):
        let cost = estimate_rete_cost(&production, store);
        if cost.estimated_ms <= 10:
            return plan(query, CognitiveLayer::L1)

    // Inference: does it require backward chaining?
    let goal_depth = estimate_backward_chain_depth(query, store, max_depth=8);
    if goal_depth <= 4 and store.indexed_unit_count() < 100_000:
        return plan(query, CognitiveLayer::L2)

    // Deep reasoning: WM expansion needed
    if store.indexed_unit_count() < 1_000_000:
        return plan_with_fallback(query, CognitiveLayer::L3, CognitiveLayer::L2)

    // Background: too expensive, defer
    schedule_background(query);
    return plan(query, CognitiveLayer::L4)
```

### 95.3 Budget Enforcement

```omni
struct LayerBudget:
    l0_ms:  u32  # Default 1
    l1_ms:  u32  # Default 10
    l2_ms:  u32  # Default 100
    l3_ms:  u32  # Default 500
    l4_ms:  u32  # Default unlimited (async)

// Each layer runs inside a deadline-aware executor:
fn execute_with_deadline<T>(
    layer: CognitiveLayer,
    budget: &LayerBudget,
    operation: async fn() -> T,
) -> Result<T, CognitiveTimeout>:
    let deadline = Instant::now() + budget.for_layer(layer);
    timeout(deadline, operation).await
        .map_err(|_| CognitiveTimeout { layer, budget_ms: budget.for_layer(layer).as_millis() })
```

### 95.4 Acceptance Criteria

- A direct fact-lookup query (`FIND facts WHERE subject = "France" AND predicate = "capital"`) is planned as L0 if the unit is in the hot tier, completing in < 1ms p99.
- A 3-hop inference query is planned as L2 and completes within 100ms p99 on a store of 500,000 units.
- If L2 exceeds its budget, the planner automatically falls back to L1 with partial results and a `PartialAnswer` flag.
- All planner decisions are recorded in the Cognitive Trace (Section 12) with rationale text.

---

## 96. Omni Language — Module System and Visibility Rules

### 96.1 Module Declaration and Nesting

```omni
// File: helios/brain/mod.omni
module helios::brain:
    pub module rete        // Public sub-module (visible outside helios::brain)
    pub module backward    // Public sub-module
    module working_memory  // Private: only visible inside helios::brain
    module planner         // Private

// File: helios/brain/rete/mod.omni
module helios::brain::rete:
    pub struct ReteNetwork    // Exported from module
    pub fn build_network() -> ReteNetwork
    struct InternalNode       // Private: not visible outside this module
```

### 96.2 Import Resolution

```omni
// Absolute import
use helios::brain::rete::ReteNetwork;

// Relative import (from within helios::brain)
use super::working_memory::WorkingMemory;
use self::backward::BackwardChainer;

// Glob import (use sparingly — triggers a linter warning)
use helios::brain::rete::*;

// Aliased import
use helios::knowledge::store::KnowledgeStore as Store;

// Conditional import (feature flag)
#[cfg(feature = "gpu")]
use helios::compute::gpu::GpuBackend;
```

### 96.3 Visibility Modifiers

```omni
pub struct PublicType        // Visible everywhere
pub(crate) struct CrateType  // Visible within the same crate only
pub(super) struct SuperType  // Visible in parent module and its children
struct PrivateType           // Visible only within this module

// Field visibility can differ from struct visibility:
pub struct InformationUnit:
    pub id:          u64     // Public field
    pub content:     String  // Public field
    pub(crate) flags: UnitFlags  // Crate-internal field
    storage_page:    u32     // Private field (module-internal)
```

### 96.4 Cyclic Dependency Detection

The Omni compiler performs Kahn's topological sort on the module import graph at compile time:

```omni
// This will be rejected at compile time:
// module A imports module B, module B imports module A
// Error: CyclicDependencyError: A → B → A
// Suggestion: extract shared types into a common module C
//             used by both A and B without a cycle
```

### 96.5 Acceptance Criteria

- The compiler correctly resolves all `use` statements across a 50-module project with 5 levels of nesting within 2 seconds.
- A circular import `A → B → A` is detected and reported with the full cycle path in the error message.
- `pub(crate)` items are inaccessible from a different crate — verified by attempting external access and asserting a `VisibilityError`.
- Glob imports (`use mod::*`) emit a linter `GlobImport` warning by default; suppressed with `#[allow(glob_import)]`.

---

## 97. HELIOS Knowledge — String Interning and Symbol Tables

### 97.1 Motivation

InformationUnits contain repeated high-cardinality strings in `subject`, `predicate`, `domain`, and `keywords`. Across 1,000,000 units, subjects like `"France"` or predicates like `"capital_of"` would be stored thousands of times without interning, wasting significant memory. The Symbol Table provides O(1) intern and resolve operations with a global deduplicated string store.

### 97.2 SymbolTable Design

```omni
struct SymbolTable:
    arena:      StringArena        # Contiguous memory region for all string bytes
    index:      HashMap<str, SymbolId>  # Hash map for intern lookup
    reverse:    Vec<ArenaRef>      # SymbolId → ArenaRef (offset + length)
    lock:       RwLock             # Read-concurrent, write-exclusive

type SymbolId = u32                # 4-byte token replacing String in InformationUnit

struct ArenaRef:
    offset:     u32                # Byte offset in arena
    length:     u16                # String byte length (max 65535 bytes)

impl SymbolTable:
    fn intern(&mut self, s: &str) -> SymbolId:
        if let Some(id) = self.index.get(s):
            return *id
        let arena_ref = self.arena.append(s);
        let id = SymbolId(self.reverse.len() as u32);
        self.reverse.push(arena_ref);
        self.index.insert(arena_ref.as_str(&self.arena), id);
        id

    fn resolve(&self, id: SymbolId) -> &str:
        let aref = &self.reverse[id.0 as usize];
        self.arena.get(aref)
```

### 97.3 Integration with InformationUnit

```omni
// InformationUnit stores SymbolIds for frequently repeated fields:
struct InformationUnitCompact:
    id:         u64
    subject:    SymbolId           # Interned
    predicate:  Option<SymbolId>   # Interned
    object:     Option<SymbolId>   # Interned
    domain:     SymbolId           # Interned
    keywords:   Vec<SymbolId>      # Each interned independently
    content:    ContentRef         # Content stored in Content Addressable Store (§64)
    // ... remaining fields unchanged
```

### 97.4 Acceptance Criteria

- Interning 1,000,000 strings drawn from a realistic HELIOS workload (high repetition of subjects and predicates) consumes < 60% of the raw `String` heap allocation.
- `intern()` completes in < 200ns per call under concurrent read load of 32 threads.
- `resolve(intern(s)) == s` holds for all valid UTF-8 strings up to 65535 bytes.
- The `SymbolTable` survives a checkpoint-restore cycle: ids before and after persist save are identical.

---

## 98. HELIOS Knowledge — Anti-Entropy and Divergence Repair

### 98.1 Problem

Federation (Section 35) synchronizes knowledge across HELIOS instances, but long partition events or partial sync failures can leave stores in divergent states that CRDT merge alone cannot detect — specifically, *missing deletions* (superseded units not propagated) and *silent corruption* (on-disk bit errors in cold pages).

### 98.2 Merkle Tree Anti-Entropy

Metadata fields managed by CRDTs (§88, §35.2) are self-converging and excluded from Merkle-tree anti-entropy comparison. Anti-entropy applies only to content fields and non-CRDT attributes.

```omni
struct MerkleTree:
    root:       [u8; 32]       # BLAKE3 root over all page hashes
    page_hashes: Vec<[u8; 32]> # Leaf: BLAKE3 of each page's content

fn build_merkle_tree(store: &KnowledgeStore) -> MerkleTree:
    let page_hashes = store.pages().par_map(|page| blake3::hash(page.bytes()));
    MerkleTree {
        root: blake3::hash_all(&page_hashes),
        page_hashes,
    }

// Anti-entropy exchange protocol between two peers:
fn anti_entropy_sync(local: &mut KnowledgeStore, remote: &RemotePeer):
    let local_tree = build_merkle_tree(local);
    let remote_root = remote.fetch_merkle_root().await;
    if local_tree.root == remote_root:
        return   // Stores are identical, no sync needed

    // Bisect to find diverged page ranges
    let diverged_pages = merkle_bisect(local_tree, remote.fetch_merkle_tree().await);
    for page_id in diverged_pages:
        let remote_page = remote.fetch_page(page_id).await;
        reconcile_page(local, page_id, remote_page);
```

### 98.3 Reconciliation Strategy

```omni
fn reconcile_page(
    local: &mut KnowledgeStore,
    page_id: u32,
    remote_page: &Page,
):
    let local_units = local.read_page(page_id).units();
    let remote_units = remote_page.units();

    // Union of unit IDs present in either page
    let all_ids = local_units.ids().union(remote_units.ids());
    for id in all_ids:
        match (local_units.get(id), remote_units.get(id)):
            (Some(l), Some(r)) =>
                // Both have the unit: keep highest confidence version
                if r.confidence.final_score > l.confidence.final_score:
                    local.update_unit(r.clone())
            (None, Some(r)) =>
                // Remote has it, local doesn't: ingest with TrustLevel::Medium
                local.insert_unit(r.clone())
            (Some(_), None) =>
                // Local-only: no action (remote may not have seen this yet)
                ()
            (None, None) =>
                unreachable!()
```

### 98.4 Corruption Detection

```omni
fn verify_store_integrity(store: &KnowledgeStore) -> IntegrityReport:
    let mut errors = Vec::new();
    for page in store.pages():
        let computed_hash = blake3::hash(page.content_bytes());
        if computed_hash != page.header.content_hash:
            errors.push(IntegrityError::PageCorruption { page_id: page.id, expected: page.header.content_hash, actual: computed_hash });
        for unit in page.units():
            let computed_unit_hash = blake3::hash(unit.content.as_bytes());
            if computed_unit_hash != unit.content_hash:
                errors.push(IntegrityError::UnitCorruption { unit_id: unit.id, page_id: page.id });
    IntegrityReport { pages_checked: store.page_count(), errors }
```

### 98.5 Acceptance Criteria

- Anti-entropy between two 500,000-unit stores that differ by 500 units completes in < 5 seconds on a 100 Mbit local network.
- Diverged pages are identified via Merkle bisect in O(log N) round trips regardless of store size.
- A page with an artificially corrupted BLAKE3 hash is reported in `IntegrityReport.errors` within 30 seconds of the nightly verification run.

---

## 99. HELIOS Operations — Upgrade Manager and Hot-Swap Protocol

### 99.1 Upgrade Lifecycle

HELIOS supports in-process hot-swap upgrades to avoid service interruption. The 7-step upgrade pipeline ensures zero data loss and rollback capability:

```omni
enum UpgradeState:
    Idle
    Downloading { version: Version, progress: f32 }
    Verifying { version: Version }
    Staging { version: Version }
    Draining                         # Stop accepting new requests; drain in-flight
    Swapping { old: Version, new: Version }
    Completing
    RollingBack { reason: String }

struct UpgradePackage:
    version:    Version
    binary:     PathBuf              # New HELIOS binary
    schema_migration: Option<MigrationScript>  # Optional schema delta (§47)
    rollback_binary: PathBuf         # Previous binary for rollback
    checksum:   [u8; 32]             # BLAKE3 of binary content
    signature:  [u8; 64]             # Ed25519 / ML-DSA signature
```

### 99.2 The 7-Step Hot-Swap Pipeline

```omni
async fn execute_upgrade(pkg: UpgradePackage) -> UpgradeResult:
    // Step 1: Download and checksum verify
    let binary = download_verified(&pkg).await?;

    // Step 2: Signature verification (Ed25519 or ML-DSA per §92)
    verify_signature(&binary, &pkg.signature, &HELIOS_PUBLIC_KEY)?;

    // Step 3: Stage — copy to staging area, do NOT overwrite running binary
    let staged_path = stage_binary(&binary).await?;

    // Step 4: Schema migration preflight (dry-run)
    if let Some(migration) = &pkg.schema_migration:
        migration.dry_run(&current_store).await?;

    // Step 5: Drain — mark service as draining, reject new queries, wait for in-flight
    set_state(UpgradeState::Draining);
    wait_for_drain(timeout=Duration::seconds(30)).await?;

    // Step 6: Apply schema migration (if any)
    if let Some(migration) = pkg.schema_migration:
        migration.apply(&mut current_store).await?;

    // Step 7: Exec-replace (Unix execve or Windows CreateProcess+TerminateProcess)
    exec_replace(staged_path, &current_args)
    // The new process inherits the store file descriptors and resumes from state file
```

### 99.3 Rollback Protocol

```omni
fn rollback(pkg: &UpgradePackage, reason: &str):
    // Reverse schema migration if it was applied
    if let Some(migration) = &pkg.schema_migration:
        migration.rollback(&mut current_store);

    // Restore previous binary
    exec_replace(&pkg.rollback_binary, &current_args);
    log_rollback_event(reason);
```

### 99.4 Acceptance Criteria

- A binary upgrade completes with zero lost queries: all in-flight queries submitted before drain start receive responses.
- A corrupted upgrade package (wrong checksum) is rejected in Step 1 with `ChecksumMismatch` error before any state is modified.
- A schema migration that fails dry-run is rejected before drain begins, leaving the service fully operational.
- Rollback after a successful swap restores the previous version within 10 seconds and all knowledge store contents remain intact.

---

## 100. Omni Tooling — HELIOS-Specific Static Analyzer

### 100.1 Motivation

General-purpose linters do not understand HELIOS's epistemic contracts — specifically, the rules around confidence thresholds, unit provenance requirements, web-fetch staging obligations, and plugin capability escalation. This section defines `omni-helios-lint`: a domain-aware static analyzer for HELIOS codebases.

### 100.2 Analyzer Architecture

```omni
struct HeliosAnalyzer:
    ast:            &AST
    type_info:      &TypeCheckResult
    symbol_table:   &SymbolTable
    enabled_rules:  Vec<LintRule>

struct LintDiagnostic:
    rule_id:        String           # e.g., "H001"
    severity:       LintSeverity     # Error | Warning | Info
    message:        String
    span:           SourceSpan
    suggestion:     Option<String>   # Auto-fix suggestion

enum LintSeverity:
    Error          # Blocks compilation
    Warning        # Reported, does not block
    Info           # Advisory only
```

### 100.3 Built-In HELIOS Lint Rules

```omni
// H001: Unverified knowledge in response — Error
// Fires when a call to respond() or format_response() receives an InformationUnit
// whose AccuracyStatus is NOT Accurate or UserConfirmed AND confidence < 50.
rule H001_UnverifiedResponseContent: LintRule

// H002: Unstaged web fetch — Error
// Fires when web-fetched content is inserted directly into KnowledgeStore::commit()
// without passing through the StagingArea. Bypasses the 6-stage verification pipeline.
rule H002_UnstagedWebContent: LintRule

// H003: Plugin capability escalation — Error
// Fires when plugin code accesses a resource type not declared in its PluginManifest
// permissions list, detectable via CapabilityToken flow analysis.
rule H003_CapabilityEscalation: LintRule

// H004: Cognitive layer effect violation — Warning
// Fires when a function annotated with a cognitive layer performs an effect not
// permitted for that layer (e.g., L0 reflex code calling a network operation).
rule H004_CognitiveLayerEffectViolation: LintRule

// H005: Missing contradiction check — Warning
// Fires when a new InformationUnit is committed without calling
// check_contradictions() first, which could silently introduce conflicts.
rule H005_MissingContradictionCheck: LintRule

// H006: Confidence arithmetic overflow — Error
// Fires when confidence sub-scores are combined without bounds checking,
// risking u8 wrap-around past 100.
rule H006_ConfidenceOverflow: LintRule

// H008: Deprecated cryptography enforcement — Error/Warning
// Enforces CNSA 2.0 phase deadlines from §92.4. In Phase 2 (2028-2030),
// flags any code path using non-hybrid classical-only key exchange as a Warning.
// In Phase 3 (2030+), flags it as an Error.
rule H008_DeprecatedCrypto: LintRule

// H007: Stale self-model reference — Info
// Fires when code reads from SelfModel fields without checking stale_after
// (Section 13), potentially using outdated self-knowledge.
rule H007_StaleSelfModel: LintRule

// H008: Deprecated cryptographic primitive use — Error
// Fires when code in any module uses std::crypto::aes_gcm, rsa, or classic ECC
// after Phase 1 of the §92 PQC deprecation schedule. Enforces the phased
// deprecation timeline — without this rule, deprecation has no compiler enforcement.
rule H008_DeprecatedCryptoPrimitive: LintRule

// H009: Resource handle without Drop implementation — Error
// Fires when a struct holding a field typed as a file handle, network socket,
// or PageLock (§34.2 linear type) does not implement Drop. Catches resource
// leaks that the linear type system alone won't catch if the struct is on the heap.
rule H009_ResourceHandleWithoutDrop: LintRule
```

### 100.4 Invocation

```bash
# Run all rules against the HELIOS source tree
omni lint --profile helios-strict ./helios-framework/

# Run specific rules only
omni lint --rules H001,H002,H003 ./helios-framework/

# Auto-fix where suggestions are available
omni lint --fix ./helios-framework/

# CI integration: exit code 1 on any Error-severity finding
omni lint --error-on-warning ./helios-framework/
```

### 100.5 Acceptance Criteria

- A test codebase with 3 injected H001 violations (low-confidence units used in responses) reports exactly 3 H001 errors.
- A test codebase with an H002 violation (direct commit without staging) reports the violation with a suggestion pointing to the staging API.
- A clean HELIOS source tree (no violations) produces exit code 0 with zero diagnostics.
- Analysis of the full `helios-framework/` source tree (estimated 50,000 lines) completes in < 30 seconds.

---

## 101. HELIOS Brain — Cognitive Deadline Scheduling

### 101.1 Motivation

Sections 9 and 95 define cognitive layer latency budgets (L0: 1ms, L1: 10ms, L2: 100ms, L3: 500ms). These budgets must be enforced at runtime by a scheduler that can preempt long-running reasoning operations and gracefully degrade to partial answers rather than hanging.

### 101.2 Deadline-Aware Executor

Cognitive tasks are spawned within `TaskGroup` scopes (§14.2); deadline cancellation triggers the TaskGroup's cancellation propagation, ensuring all associated subtasks are cleaned up.

```omni
struct DeadlineScheduler:
    task_queue:     PriorityQueue<ScheduledTask>  # Min-heap by deadline
    active_tasks:   Vec<TaskHandle>
    budget_policy:  LayerBudget                   # From §95

struct ScheduledTask:
    task_id:        u64
    layer:          CognitiveLayer
    deadline:       Instant               # Absolute deadline
    priority:       TaskPriority
    cancel_token:   CancellationToken
    payload:        CognitiveOperation

enum TaskPriority:
    Interactive     # User is waiting — strict deadline enforcement
    Background      # Async deep thought — soft deadline
    Maintenance     # Compaction, anti-entropy — lowest priority

fn schedule(
    scheduler: &mut DeadlineScheduler,
    operation: CognitiveOperation,
    layer: CognitiveLayer,
    priority: TaskPriority,
) -> TaskHandle:
    let deadline = Instant::now() + scheduler.budget_policy.for_layer(layer);
    let task = ScheduledTask { task_id: next_id(), layer, deadline, priority, ... };
    scheduler.task_queue.push(task);
    spawn_with_deadline(task, deadline)
```

### 101.3 Timeout and Graceful Degradation

```omni
enum CognitiveResult<T>:
    Complete(T)
    PartialTimeout { partial: T, layer_reached: CognitiveLayer, budget_ms: u32 }
    HardTimeout { last_layer: CognitiveLayer }

fn execute_cognitive_query(
    scheduler: &mut DeadlineScheduler,
    query: ParsedQuery,
) -> CognitiveResult<QueryAnswer>:
    let plan = plan_query(&query, &current_store);
    match scheduler.run(plan).await:
        Ok(answer) => CognitiveResult::Complete(answer)
        Err(Timeout { partial_answer: Some(p), .. }) =>
            CognitiveResult::PartialTimeout { partial: p, layer_reached: plan.selected_layer, budget_ms: ... }
        Err(Timeout { partial_answer: None, .. }) =>
            CognitiveResult::HardTimeout { last_layer: plan.selected_layer }
```

### 101.4 Acceptance Criteria

- An L2 backward-chaining query that would require 800ms is preempted at 100ms and returns a `PartialTimeout` answer with the inference depth reached.
- 100 concurrent Interactive queries against a loaded store meet L1 latency budgets (< 10ms p99) without starvation.
- Background (L4) tasks are preempted by any incoming Interactive task within 2ms of arrival.
- All preemption events are recorded in the Experience Log as `CognitiveTimeout` events with the reason and depth reached.

---

## 102. HELIOS Knowledge — Bloom Filter Cascade for Multi-Tier Lookup

### 102.1 Motivation

The three-tier storage model (Hot → Warm → Cold, Section 57) incurs disk I/O when a query misses the hot tier. The most common case for a well-curated store is a *negative* lookup — the fact simply does not exist. Without early-rejection filters, every negative lookup traverses all three tiers at full I/O cost. A cascaded Bloom filter hierarchy eliminates the majority of these unnecessary reads.

### 102.2 Three-Level Bloom Cascade

```omni
struct BloomCascade:
    hot_filter:     BloomFilter      # Covers Hot tier units (low FPR — 0.1%)
    warm_filter:    BloomFilter      # Covers Warm tier units (medium FPR — 1%)
    cold_filter:    BloomFilter      # Covers Cold tier units (higher FPR — 5%)
    global_filter:  BloomFilter      # Covers ALL units (used for fast miss detection)

struct BloomFilter:
    bits:           BitVec
    hash_count:     u8               # Number of hash functions (k)
    expected_items: u32
    fpr:            f32              # Target false positive rate

fn lookup_with_cascade(
    cascade: &BloomCascade,
    store: &KnowledgeStore,
    key: &LookupKey,
) -> Option<InformationUnit>:
    // 1. Check global filter first — definite miss avoids ALL tier reads
    if not cascade.global_filter.may_contain(key):
        return None   // Guaranteed not present anywhere

    // 2. Check hot tier filter
    if cascade.hot_filter.may_contain(key):
        if let Some(unit) = store.hot_tier.get(key):
            return Some(unit)

    // 3. Check warm tier filter
    if cascade.warm_filter.may_contain(key):
        if let Some(unit) = store.warm_tier.get(key):
            return Some(unit)

    // 4. Check cold tier filter before expensive disk read
    if cascade.cold_filter.may_contain(key):
        return store.cold_tier.get(key)

    // All filters say "no" beyond global — cold miss path
    None
```

### 102.3 Filter Maintenance

```omni
fn update_cascade_on_insert(cascade: &mut BloomCascade, key: &LookupKey, tier: StoreTier):
    cascade.global_filter.insert(key);
    match tier:
        StoreTier::Hot  => cascade.hot_filter.insert(key)
        StoreTier::Warm => cascade.warm_filter.insert(key)
        StoreTier::Cold => cascade.cold_filter.insert(key)

// On compaction (hot → warm promotion):
fn promote_hot_to_warm(cascade: &mut BloomCascade, key: &LookupKey):
    // Counting Bloom filters support deletion via counter decrement
    cascade.hot_filter.remove(key);   // Decrement 4-bit counters
    cascade.warm_filter.insert(key);
    // Standard Bloom global_filter is append-only, rebuilt at checkpoints
```


**Filter type selection:**
*   Per-tier filters (hot, warm, cold) use **Counting Bloom Filters** with 4-bit counters per bucket, enabling O(1) deletion when units are promoted between tiers or archived. Counter overflow (rare at 4 bits = max 15) triggers a full filter rebuild.
*   The **global filter** uses a standard append-only Bloom filter (rebuilt at compaction checkpoints) since it never needs deletion — a unit present in any tier is present globally.
*   **XOR filters** (1.23× more space-efficient than Bloom, O(1) lookup) are used as an alternative for cold-tier filters that are rebuilt entirely at compaction time rather than updated incrementally.

### 102.4 Acceptance Criteria

- For a store with 1,000,000 units across all tiers, a negative lookup (key not in store) completes in < 200μs with no disk I/O, verified by hooking disk read syscalls.
- False positive rate for `global_filter` is measured empirically at < 0.15% over 1,000,000 negative test queries.
- Hot tier filter occupies < 2 MB of RAM for 1,000,000 entries at the 0.1% FPR target.
- Filter rebuilds at compaction time complete in < 1 second for stores up to 2,000,000 units.

---

## Appendix O — Implementation Breakdown (Phases MM to OO)

**Phase MM (Advanced Cryptography & Security)**
1.  Migrate TLS/Networking stack to PQC hybrid mode (classical + ML-KEM/ML-DSA) as specified in Sections 30 and 92.
2.  Implement Object-Capabilities Realm sandbox primitive in Omni's type system (Section 90) for unforgeable, linear-typed capability tokens.
3.  Implement `sync::crdt` integration with delta-state G-Counter, PN-Counter, LWW-Register, OR-Set, and RGA Sequence CRDTs for federation (Section 88).
4.  Acceptance test: Generate an ML-KEM keypair. Encapsulate a shared secret. Decapsulate and assert match. Create a Realm sandbox with a capability granting access ONLY to `/tmp/data`. Attempt to open `/etc/passwd`. Assert `RealmError::CapabilityViolation`. Create two disconnected `ORSet` CRDTs. Add "A" to node 1, "B" to node 2. Merge both. Assert both contain exactly `["A", "B"]`.

**Phase NN (Concurrency & Typing)**
1.  Implement `concurrency::stm` with software transactional `TVar` atomic blocks and best-effort HTM fast-paths for ARM TME and compatible CPUs (Section 89).
2.  Embed an SMT solver (Z3 backend wrapper) to parse, evaluate, and verify Liquid Type conditions (`{ x: T | pred }`) during the semantic check compiler phase (Section 91).
3.  Implement Omni Module System with explicit `pub`/`priv` visibility, `use` import resolution, and circular dependency detection (Section 96).
4.  Acceptance test: Create two threads modifying the same `TVar` in concurrent transactions. Assert no data races or lost updates under 1000-iteration stress test. Define `PositiveInt = {n: i32 | n > 0}`. Call a function requiring `PositiveInt` with `-1`. Assert compilation fails with SMT verification error. Introduce a circular import A→B→A. Assert compiler reports CyclicDependency error.

**Phase OO (Knowledge Infrastructure & Tooling)**
1.  Implement String Interning and Symbol Tables (Section 97) with a thread-safe `SymbolTable` and O(1) intern/resolve operations.
2.  Implement Belief Revision Protocol (Section 94) with AGM postulate-compliant retraction cascades.
3.  Implement HELIOS-Specific Static Analyzer (Section 100) with 9 check categories (H001–H009).
4.  Acceptance test: Intern 1,000,000 distinct strings. Assert memory usage < 2× the raw string bytes. Assert resolve operations complete in < 100ns. Mark a fact as Inaccurate. Assert all InferredFromRules units derived from it are queued for re-evaluation. Run static analyzer on a codebase with 3 injected anti-patterns. Assert all 3 are flagged with correct rule names.


**Phase PP (Foundational Language Safety & Store Reliability)**
1.  Implement Ownership Model and Borrow Checker enforcement in `omnic` with NLL semantics, lifetime elision, and `Pin<T>`/`Unpin` support (Section 93).
2.  Wire Query Cost Model and Cognitive Planner into the cognitive cortex, implementing the L0–L4 layer selection decision tree and `LayerBudget` enforcement (Section 95).
3.  Implement Anti-Entropy Merkle protocol for federation divergence detection and repair, integrating with §88 CRDT delta-state synchronization (Section 98).
4.  Implement Upgrade Manager 7-step hot-swap pipeline with checksum verification, signature validation, schema migration dry-run, drain, and rollback (Section 99).
5.  Integrate Deadline Scheduler into `brain/cognitive_cortex.omni`, enforcing per-layer preemption budgets with graceful degradation to partial answers (Section 101).
6.  Deploy Bloom Filter Cascade across all three storage tiers (Hot/Warm/Cold) with Counting Bloom Filters for per-tier deletion support and XOR filters for cold-tier compaction rebuilds (Section 102).
7.  Acceptance test: Compile a program using a moved value — assert `omnic` rejects it. Compile NLL-valid code (borrow ends at last use) — assert it compiles. Submit a direct-lookup OQL query on a hot-tier key — assert planner selects L0 and completes in < 1ms. Create two 500K-unit stores differing by 500 units — run anti-entropy and assert convergence in < 5s. Execute a binary upgrade with a corrupted checksum — assert rejection before state modification. Preempt an L2 query at 100ms budget — assert `PartialTimeout` result. Query a non-existent key across all three tiers — assert < 200μs with no disk I/O.

---

## Appendix P — Acceptance Test Specifications (v10.0 Additions)

**Phase MM acceptance:** Generate an ML-KEM-768 keypair. Encapsulate a 32-byte shared secret. Decapsulate and assert exact match. Sign a 256-byte message with ML-DSA. Assert verification succeeds. Flip one bit in the signature. Assert verification fails. Create a Realm sandbox with a capability for `PathBuf("/tmp/data")`. Attempt `open("/etc/passwd")`. Assert `RealmError::CapabilityViolation`. Create two disconnected `ORSet<String>` CRDTs on replica IDs 1 and 2. Add `"alpha"` on replica 1, `"beta"` on replica 2, then merge. Assert merged set contains exactly `["alpha", "beta"]` with no duplicates.

**Phase NN acceptance:** Start 10 threads each executing 500 concurrent `atomically` blocks transferring between `TVar<u64>` accounts. Assert sum of all account balances is conserved (no lost updates, no phantom writes). Define `PositiveInt = {n: i32 | n > 0}`. Write `fn halve(x: PositiveInt) -> PositiveInt { x / 2 }`. Assert the compiler reports an SMT verification error because `x/2` may not satisfy `> 0` for `x = 1`. Create a module `A` that imports module `B`, and `B` that imports `A`. Assert compiler reports `CyclicDependencyError` at the `use` statement.

**Phase OO acceptance:** Intern 500,000 unique predicate strings drawn from HELIOS knowledge store. Assert total interned memory is < 60% of the sum of raw String allocations. Assert `resolve(intern("is_a")) == resolve(intern("is_a"))` returns the same SymbolId in O(1). Mark InformationUnit #42 as `Inaccurate`. Assert all units whose `acquisition_chain` includes #42 via `InferredFromRules` are enqueued in the verification queue within 100ms. Run the static analyzer against a test module containing: (1) a call to `respond()` using a `Unverified` unit at confidence 20, (2) a web-fetch result stored without staging, (3) a plugin capability escalation. Assert all three are reported with their correct rule identifiers.


**Phase PP acceptance:** Compile `let x = InformationUnit::new(); let y = x; store(x);` — assert compiler error: use of moved value `x`. Compile `let id = store.get_id(); store.insert(unit);` — assert NLL allows this (borrow of `store` via `get_id()` ends at last use). Submit `FIND facts WHERE subject = "France"` to the Cognitive Planner with `"France"` in hot tier — assert plan selects L0, completes in < 1ms p99. Create two 500,000-unit stores with 500 divergent units — run `anti_entropy_sync` — assert both stores converge and Merkle roots match. Create an `UpgradePackage` with BLAKE3 checksum `0xABCD...` but binary content hashing to `0x1234...` — assert `ChecksumMismatch` error at Step 1. Schedule an L2 backward-chaining query requiring 800ms; set L2 budget to 100ms — assert `PartialTimeout` with inference depth reached. Insert 1M units across 3 tiers. Query for a key not in any tier — assert Bloom cascade returns `None` in < 200μs with zero `read()` syscalls on warm/cold storage files.

---


## Appendix Q — Error Code Registry

All HELIOS error types are assigned stable 4-digit numeric codes for IPC protocol interoperability (§12.4), plugin error reporting, and Experience Log `ErrorOccurred` records. Codes are grouped by subsystem:

| Code Range | Subsystem | Error Type | Description |
|------------|-----------|------------|-------------|
| E1001 | Knowledge Store | `StoreError::PageCorruption` | BLAKE3 hash mismatch on page read |
| E1002 | Knowledge Store | `StoreError::PageFull` | No space in target page for new unit |
| E1003 | Knowledge Store | `StoreError::UnitNotFound` | Requested unit ID does not exist |
| E1004 | Knowledge Store | `StoreError::VersionConflict` | MVCC write conflict detected |
| E1010 | Knowledge Store | `StoreError::VacuumFailed` | Version GC encountered locked pages |
| E2001 | Compression | `CompressError::InvalidMagic` | OmniPack magic bytes mismatch |
| E2002 | Compression | `CompressError::DecompressionFailed` | Corrupted compressed data |
| E3001 | Cryptography | `CryptoError::KeyGenerationFailed` | PQC key generation error |
| E3002 | Cryptography | `CryptoError::SignatureInvalid` | ML-DSA/SLH-DSA signature verification failed |
| E3003 | Cryptography | `CryptoError::DecapsulationFailed` | ML-KEM decapsulation mismatch |
| E3010 | Cryptography | `CryptoError::DeprecatedAlgorithm` | Classical algorithm used after Phase 2 cutoff |
| E4001 | Query Engine | `QueryError::ParseError` | OQL syntax error |
| E4002 | Query Engine | `QueryError::CognitiveTimeout` | Layer budget exceeded |
| E4003 | Query Engine | `QueryError::PlannerFailure` | No viable cognitive layer found |
| E5001 | Federation | `FederationError::PeerUnreachable` | QUIC/TCP connection failed |
| E5002 | Federation | `FederationError::MerkleRootMismatch` | Anti-entropy detected divergence |
| E5003 | Federation | `FederationError::CRDTMergeConflict` | Unexpected CRDT merge failure |
| E6001 | Plugin | `PluginError::CapabilityViolation` | Plugin accessed unauthorized resource |
| E6002 | Plugin | `PluginError::ManifestInvalid` | Plugin manifest schema validation failed |
| E7001 | Upgrade | `UpgradeError::ChecksumMismatch` | Binary checksum verification failed |
| E7002 | Upgrade | `UpgradeError::SignatureInvalid` | Upgrade package signature invalid |
| E7003 | Upgrade | `UpgradeError::MigrationDryRunFailed` | Schema migration dry-run error |
| E7004 | Upgrade | `UpgradeError::DrainTimeout` | In-flight queries did not drain in time |
| E8001 | Compiler | `CompileError::BorrowViolation` | Simultaneous &mut and & borrows |
| E8002 | Compiler | `CompileError::UseAfterMove` | Value used after ownership transfer |
| E8003 | Compiler | `CompileError::LifetimeMismatch` | Returned reference outlives borrowed data |
| E8004 | Compiler | `CompileError::CyclicDependency` | Circular module import detected |
| E8005 | Compiler | `CompileError::LiquidTypeViolation` | SMT solver disproved refinement predicate |

Error codes are stable across versions. New codes are appended; existing codes are never reassigned. Plugins use numeric codes in `PluginError::Custom(u16)` with the E6xxx range reserved for plugin-specific errors.

---


## Appendix R — Performance Baseline Hardware Profile

All performance acceptance criteria throughout this specification are measured against the following reference hardware profile. Actual performance on different hardware should be scaled proportionally.

| Component | Reference Specification |
|-----------|------------------------|
| **CPU** | x86-64 with AVX2 (or AArch64 with NEON/SVE), 8 cores / 16 threads, 3.0 GHz base clock |
| **RAM** | 16 GB DDR4-3200 (or equivalent bandwidth) |
| **Storage** | NVMe SSD, 3000 MB/s sequential read, 500K random IOPS |
| **Network** | 1 Gbit Ethernet (for federation latency targets) |
| **OS** | Linux 6.x kernel, Windows 11, or macOS 14+ |

**Key target references:**

| Target | Section | Assumed Hardware |
|--------|---------|-----------------|
| < 1ms L0 fact lookup | §95 | Hot tier in RAM, NVMe for warm/cold |
| < 10ms L1 RETE pattern match | §95 | RETE network in RAM |
| < 200μs Bloom filter negative lookup | §102 | Filter bit vectors in RAM |
| < 200ns `intern()` call | §97 | HashMap + arena in RAM |
| 2000 MB/s OmniPack decompression | §4 | AVX2 SIMD acceleration |
| < 5s anti-entropy sync (500 divergent / 500K total) | §98 | 1 Gbit LAN, NVMe storage |
| < 30s full static analysis (50K lines) | §100 | 8-core CPU, AST in RAM |

All performance acceptance criteria are expressed as p99 latencies unless otherwise noted. Deployments on significantly different hardware (e.g., Raspberry Pi, mobile ARM) should establish local baselines using the `omni bench` harness (§76) and scale acceptance thresholds proportionally.

---

*End of HELIOS Comprehensive Implementation Specification v10.0*

**Key Enhancement Summary (v9.0 → v10.0):**
- **Removals (architecture violations):** Sections 88 (FHE), 93 (MPC), 94 (Carbon-Aware Datacenter Routing), 95 (MARL/Swarm), and 97 (Learned Indexes) from the initial v10.0 draft are removed. All five assumed HELIOS to be a distributed cloud ML system, contradicting its core design as a local, deterministic, evidence-based cognitive framework with no gradient training. Section numbers have been consolidated sequentially (88–102).
- **Corrections:** Section 88 (CRDTs, formerly §89) corrected to use TCP/QUIC federation transport instead of WebRTC. Section 89 (STM/HTM, formerly §90) corrected to remove the deprecated Intel TSX reference; HTM is now described as a best-effort fast path for any compatible CPU facility. Section 90 (Capabilities, formerly §91) trimmed to define the Omni-language primitive layer, removing duplication with Sections 27 and 68. Section 92 (PQC, formerly §96) corrected from "deprecate" to "phased transition" aligned with the hybrid approach of Section 30. Duplicate Appendix N removed.
- **Structural Bug Fix:** Duplicate Appendix N was present in v9.0; removed.
- **New: Language Foundations (§93, §96):** Ownership model and borrow checker (§93) provides the formal memory-safety model required by a systems language. Module system and visibility rules (§96) formalizes imports, namespacing, and cyclic dependency detection.
- **New: Reasoning Infrastructure (§94, §95, §101):** Belief Revision Protocol (§94) applies AGM postulates to knowledge retraction cascades. Query Cost Model and Cognitive Planner (§95) enables informed layer selection before dispatching queries. Cognitive Deadline Scheduling (§101) enforces L0–L4 latency budgets with preemption and graceful degradation.
- **New: Knowledge Store Optimizations (§97, §98, §102):** String Interning/Symbol Tables (§97) reduce memory 40%+ for large stores. Anti-Entropy and Divergence Repair (§98) uses Merkle trees to detect and repair federation divergence. Bloom Filter Cascade (§102) eliminates disk I/O for negative lookups across the three storage tiers.
- **New: Operations and Tooling (§99, §100):** Upgrade Manager and Hot-Swap Protocol (§99) formalizes the 7-step zero-downtime upgrade pipeline. HELIOS-Specific Static Analyzer (§100) defines 9 lint rules (H001–H009) for HELIOS epistemic and safety contracts.
- **v10.0 Audit Fixes:** §3 OmniMAC construction upgraded from XOR combiner to BLAKE3 keyed hash. §93 extended with Non-Lexical Lifetimes (NLL) and Pin/Unpin for async soundness. §38 extended with MVCC Vacuum for version garbage collection. §92 extended with CNSA 2.0 compliance timeline. §102 upgraded from standard Bloom filters to Counting Bloom Filters with XOR filter cold-tier option. §100 extended with H008 (deprecated crypto enforcement) and H009 (resource handle Drop check). §15 extended with variance annotations for generics. §35 extended with mDNS/DNS-SD peer discovery. §34/§78 algebraic effect sections cross-referenced. §36/§26 annotated with gradient training architectural clarification. Phase PP added to Appendices O/P for 6 previously unscheduled sections. Appendix Q (Error Code Registry) and Appendix R (Performance Baseline Hardware) added.
- **v10.0 Research-Based Enhancements (2025):** §7.3 expanded with arena allocator for ephemeral reasoning temporaries and detailed slab allocator internals. §7.4 extended with Write-Ahead Log (WAL) crash recovery protocol with analysis→redo→undo replay. §9.3 RETE network annotated with incremental computation / differential dataflow performance characteristics. §12.3 cognitive tracing enhanced with OpenTelemetry-inspired span-based semantic conventions for L0–L4 layer attribution. §14.2 async model extended with structured concurrency (TaskGroup scopes with automatic cancellation propagation, modeled after JDK 25 JEP 505). §35.1 federation capabilities extended with Multipath QUIC (RFC 9369) and 0-RTT session resumption. §35.2 CRDT metadata section enhanced with delta-state streaming reference.
