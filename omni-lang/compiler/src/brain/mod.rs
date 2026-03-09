//! Brain module — Adaptive reasoning, knowledge graphs, and memory architecture.
//!
//! Provides real implementations of:
//! - `AdaptiveReasoner` — Deductive / inductive reasoning with modus ponens
//! - `KnowledgeGraph` — Weighted directed graph with Dijkstra, cycle detection, forward chaining
//! - `MemorySystem` — Short-term / long-term memory with consolidation

pub mod knowledge_graph;
pub mod adaptive_reasoning;
pub mod memory;

pub use knowledge_graph::KnowledgeGraph;
pub use adaptive_reasoning::AdaptiveReasoner;
pub use memory::MemorySystem;
