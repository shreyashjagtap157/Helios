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

//! Knowledge Graph — weighted directed graph with real algorithms.
//!
//! Supports: Dijkstra shortest path, cycle detection, forward-chaining rule inference.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// A weighted directed knowledge graph.
#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    /// Adjacency list: node → [(neighbor, weight)]
    adjacency: HashMap<String, Vec<(String, f64)>>,
    /// Fact store: (predicate, subject) pairs
    facts: HashSet<(String, String)>,
    /// Rules: (premises, conclusion) — premises are (predicate, variable) pairs
    rules: Vec<(Vec<(String, String)>, (String, String))>,
}

/// Helper for Dijkstra's priority queue (min-heap via Reverse ordering).
#[derive(Debug, Clone, PartialEq)]
struct DijkstraState {
    cost: f64,
    node: String,
}

impl Eq for DijkstraState {}

impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeGraph {
    /// Create an empty knowledge graph.
    pub fn new() -> Self {
        KnowledgeGraph {
            adjacency: HashMap::new(),
            facts: HashSet::new(),
            rules: Vec::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, name: &str) {
        self.adjacency
            .entry(name.to_string())
            .or_insert_with(Vec::new);
    }

    /// Add a weighted directed edge from `from` to `to`.
    pub fn add_edge(&mut self, from: &str, to: &str, weight: f64) {
        self.add_node(from);
        self.add_node(to);
        self.adjacency
            .get_mut(from)
            .unwrap()
            .push((to.to_string(), weight));
    }

    /// Find the shortest path from `start` to `end` using Dijkstra's algorithm.
    /// Returns `Some(path)` with the list of node names, or `None` if unreachable.
    pub fn shortest_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        if !self.adjacency.contains_key(start) || !self.adjacency.contains_key(end) {
            return None;
        }

        let mut dist: HashMap<String, f64> = HashMap::new();
        let mut prev: HashMap<String, String> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(start.to_string(), 0.0);
        heap.push(DijkstraState {
            cost: 0.0,
            node: start.to_string(),
        });

        while let Some(DijkstraState { cost, node }) = heap.pop() {
            if node == end {
                // Reconstruct path
                let mut path = vec![end.to_string()];
                let mut current = end.to_string();
                while let Some(predecessor) = prev.get(&current) {
                    path.push(predecessor.clone());
                    current = predecessor.clone();
                }
                path.reverse();
                return Some(path);
            }

            if let Some(&best) = dist.get(&node) {
                if cost > best {
                    continue; // skip stale entry
                }
            }

            if let Some(neighbors) = self.adjacency.get(&node) {
                for (neighbor, weight) in neighbors {
                    let new_cost = cost + weight;
                    let is_better = match dist.get(neighbor) {
                        Some(&existing) => new_cost < existing,
                        None => true,
                    };
                    if is_better {
                        dist.insert(neighbor.clone(), new_cost);
                        prev.insert(neighbor.clone(), node.clone());
                        heap.push(DijkstraState {
                            cost: new_cost,
                            node: neighbor.clone(),
                        });
                    }
                }
            }
        }

        None // unreachable
    }

    /// Detect whether the graph contains any cycle (DFS-based).
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in self.adjacency.keys() {
            if !visited.contains(node) {
                if self.dfs_cycle(node, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        false
    }

    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = self.adjacency.get(node) {
            for (neighbor, _) in neighbors {
                if !visited.contains(neighbor) {
                    if self.dfs_cycle(neighbor, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Add a fact (predicate, subject) to the knowledge base.
    pub fn add_fact(&mut self, predicate: &str, subject: &str) {
        self.facts
            .insert((predicate.to_string(), subject.to_string()));
    }

    /// Check if a fact exists.
    pub fn has_fact(&self, predicate: &str, subject: &str) -> bool {
        self.facts
            .contains(&(predicate.to_string(), subject.to_string()))
    }

    /// Add a forward-chaining rule.
    /// premises: list of (predicate, variable) pairs — variables start with "?"
    /// conclusion: (predicate, variable) pair
    pub fn add_rule(&mut self, premises: Vec<(&str, &str)>, conclusion: (&str, &str)) {
        let premises_owned: Vec<(String, String)> = premises
            .into_iter()
            .map(|(p, v)| (p.to_string(), v.to_string()))
            .collect();
        let conclusion_owned = (conclusion.0.to_string(), conclusion.1.to_string());
        self.rules.push((premises_owned, conclusion_owned));
    }

    /// Run forward chaining for up to `max_iterations` rounds.
    /// Returns a list of newly derived facts as "predicate(subject)" strings.
    pub fn forward_chain(&mut self, max_iterations: usize) -> Vec<String> {
        let mut derived = Vec::new();

        for _ in 0..max_iterations {
            let mut new_facts = Vec::new();

            for (premises, conclusion) in &self.rules {
                // Collect all variables used in premises
                let variables: HashSet<&str> = premises
                    .iter()
                    .filter(|(_, v)| v.starts_with('?'))
                    .map(|(_, v)| v.as_str())
                    .collect();

                // For each variable, find all possible bindings
                if variables.len() == 1 {
                    let var_name = variables.into_iter().next().unwrap();
                    // Find all subjects that match ALL premises for this variable
                    let all_subjects: HashSet<&str> =
                        self.facts.iter().map(|(_, s)| s.as_str()).collect();

                    for subject in &all_subjects {
                        let all_match = premises.iter().all(|(pred, v)| {
                            if v.starts_with('?') {
                                self.facts.contains(&(pred.clone(), subject.to_string()))
                            } else {
                                self.facts.contains(&(pred.clone(), v.clone()))
                            }
                        });

                        if all_match {
                            let conc_subject = if conclusion.1.starts_with('?') {
                                subject.to_string()
                            } else {
                                conclusion.1.clone()
                            };
                            let fact = (conclusion.0.clone(), conc_subject.clone());
                            if !self.facts.contains(&fact) {
                                new_facts.push(fact);
                            }
                        }
                    }
                }
            }

            if new_facts.is_empty() {
                break; // fixed point reached
            }

            for fact in new_facts {
                let repr = format!("{}({})", fact.0, fact.1);
                derived.push(repr);
                self.facts.insert(fact);
            }
        }

        derived
    }

    /// Return the number of nodes.
    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Return all node names.
    pub fn nodes(&self) -> Vec<String> {
        self.adjacency.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dijkstra_shortest_path() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node("A");
        graph.add_node("B");
        graph.add_node("C");
        graph.add_node("D");
        // A→B=1, A→C=4, B→C=2, B→D=5, C→D=1
        graph.add_edge("A", "B", 1.0);
        graph.add_edge("A", "C", 4.0);
        graph.add_edge("B", "C", 2.0);
        graph.add_edge("B", "D", 5.0);
        graph.add_edge("C", "D", 1.0);

        let path = graph.shortest_path("A", "D");
        // Shortest: A→B(1)→C(2)→D(1) = total 4
        assert_eq!(path.unwrap(), vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_dijkstra_no_path() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node("A");
        graph.add_node("B");
        // No edge A→B
        assert!(graph.shortest_path("A", "B").is_none());
    }

    #[test]
    fn test_dijkstra_direct_path() {
        let mut graph = KnowledgeGraph::new();
        graph.add_edge("A", "B", 10.0);
        graph.add_edge("A", "C", 3.0);
        graph.add_edge("C", "B", 2.0);
        // Direct A→B=10, indirect A→C→B=5
        let path = graph.shortest_path("A", "B").unwrap();
        assert_eq!(path, vec!["A", "C", "B"]);
    }

    #[test]
    fn test_cycle_detection_cyclic() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node("X");
        graph.add_node("Y");
        graph.add_node("Z");
        graph.add_edge("X", "Y", 1.0);
        graph.add_edge("Y", "Z", 1.0);
        graph.add_edge("Z", "X", 1.0); // cycle
        assert!(graph.has_cycle());
    }

    #[test]
    fn test_cycle_detection_acyclic() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node("A");
        graph.add_node("B");
        graph.add_node("C");
        graph.add_edge("A", "B", 1.0);
        graph.add_edge("B", "C", 1.0);
        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_forward_chaining_derives_facts() {
        let mut graph = KnowledgeGraph::new();
        // Facts: bird(Tweety), has_wings(Tweety)
        graph.add_fact("bird", "Tweety");
        graph.add_fact("has_wings", "Tweety");
        // Rule: bird(X) ∧ has_wings(X) → can_fly(X)
        graph.add_rule(vec![("bird", "?X"), ("has_wings", "?X")], ("can_fly", "?X"));
        let derived = graph.forward_chain(10);
        assert!(
            derived.contains(&"can_fly(Tweety)".to_string()),
            "Forward chaining must derive can_fly(Tweety); got: {:?}",
            derived
        );
    }

    #[test]
    fn test_forward_chaining_no_match() {
        let mut graph = KnowledgeGraph::new();
        graph.add_fact("bird", "Tweety");
        // Rule requires has_wings too, but Tweety doesn't have it
        graph.add_rule(vec![("bird", "?X"), ("has_wings", "?X")], ("can_fly", "?X"));
        let derived = graph.forward_chain(10);
        assert!(
            derived.is_empty(),
            "No derivation should occur without all premises"
        );
    }

    #[test]
    fn test_forward_chaining_multi_step() {
        let mut graph = KnowledgeGraph::new();
        graph.add_fact("animal", "Fido");
        graph.add_fact("domesticated", "Fido");
        // Rule 1: animal(X) ∧ domesticated(X) → pet(X)
        graph.add_rule(
            vec![("animal", "?X"), ("domesticated", "?X")],
            ("pet", "?X"),
        );
        // Rule 2: pet(X) → needs_care(X)  (requires pet to be derived first)
        graph.add_rule(vec![("pet", "?X")], ("needs_care", "?X"));
        let derived = graph.forward_chain(10);
        assert!(derived.contains(&"pet(Fido)".to_string()));
        assert!(derived.contains(&"needs_care(Fido)".to_string()));
    }
}
