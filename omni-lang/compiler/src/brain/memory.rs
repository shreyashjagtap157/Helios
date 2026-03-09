//! Memory Architecture — short-term / long-term memory with consolidation.
//!
//! Supports importance-based promotion from short-term to long-term memory.

/// A memory item stored in the system.
#[derive(Debug, Clone)]
pub struct MemoryItem {
    pub content: String,
    pub importance: f64,
    pub access_count: usize,
}

/// Result of a memory consolidation pass.
#[derive(Debug, Clone)]
pub struct ConsolidationResult {
    pub promoted_to_long_term: usize,
    pub dropped: usize,
}

/// Short-term / long-term memory system with consolidation.
pub struct MemorySystem {
    short_term: Vec<MemoryItem>,
    long_term: Vec<MemoryItem>,
    consolidation_threshold: f64,
}

impl MemorySystem {
    pub fn new() -> Self {
        MemorySystem {
            short_term: Vec::new(),
            long_term: Vec::new(),
            consolidation_threshold: 0.5,
        }
    }

    /// Add a fact to short-term memory with given importance.
    pub fn add_short_term(&mut self, content: String, importance: f64) {
        self.short_term.push(MemoryItem {
            content,
            importance,
            access_count: 0,
        });
    }

    /// Consolidate memory: promote high-importance items to long-term, drop low-importance ones.
    pub fn consolidate(&mut self) -> ConsolidationResult {
        let mut promoted = 0;
        let mut dropped = 0;

        let items = std::mem::take(&mut self.short_term);
        for item in items {
            if item.importance >= self.consolidation_threshold {
                self.long_term.push(item);
                promoted += 1;
            } else {
                dropped += 1;
            }
        }

        ConsolidationResult {
            promoted_to_long_term: promoted,
            dropped,
        }
    }

    /// Retrieve all long-term memories.
    pub fn long_term_memories(&self) -> &[MemoryItem] {
        &self.long_term
    }

    /// Retrieve all short-term memories.
    pub fn short_term_memories(&self) -> &[MemoryItem] {
        &self.short_term
    }

    /// Query long-term memory for items containing the given keyword.
    pub fn query(&self, keyword: &str) -> Vec<&MemoryItem> {
        self.long_term
            .iter()
            .filter(|item| item.content.contains(keyword))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_consolidation() {
        let mut memory = MemorySystem::new();
        // Add 5 facts with high importance
        for i in 0..5 {
            memory.add_short_term(format!("fact_{}", i), 0.9);
        }
        // Add 2 facts with low importance
        memory.add_short_term("noise_1".to_string(), 0.1);
        memory.add_short_term("noise_2".to_string(), 0.1);

        let consolidated = memory.consolidate();
        assert_eq!(consolidated.promoted_to_long_term, 5);
        assert_eq!(consolidated.dropped, 2);
    }

    #[test]
    fn test_short_term_cleared_after_consolidation() {
        let mut memory = MemorySystem::new();
        memory.add_short_term("test".to_string(), 0.8);
        memory.consolidate();
        assert!(memory.short_term_memories().is_empty());
        assert_eq!(memory.long_term_memories().len(), 1);
    }

    #[test]
    fn test_query_long_term() {
        let mut memory = MemorySystem::new();
        memory.add_short_term("Rust is fast".to_string(), 0.9);
        memory.add_short_term("Python is slow".to_string(), 0.8);
        memory.add_short_term("noise".to_string(), 0.1);
        memory.consolidate();

        let results = memory.query("fast");
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("Rust"));
    }
}
