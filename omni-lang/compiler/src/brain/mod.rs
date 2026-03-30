// Copyright 2024 Shreyash Jagtap
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Brain module — Adaptive reasoning, knowledge graphs, and memory architecture.
//!
//! Provides real implementations of:
//! - `AdaptiveReasoner` — Deductive / inductive reasoning with modus ponens
//! - `KnowledgeGraph` — Weighted directed graph with Dijkstra, cycle detection, forward chaining
//! - `MemorySystem` — Short-term / long-term memory with consolidation

pub mod adaptive_reasoning;
pub mod knowledge_graph;
pub mod memory;

pub use adaptive_reasoning::AdaptiveReasoner;
pub use knowledge_graph::KnowledgeGraph;
pub use memory::MemorySystem;
