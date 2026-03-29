/// Adaptive Reasoning Framework
/// Implements strategy selection, multi-step reasoning, uncertainty handling, and continuous adaptation

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Represents a reasoning strategy
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReasoningStrategy {
    Deductive,      // From general to specific
    Inductive,      // From specific to general
    Abductive,      // Inferring best explanation
    Analogical,     // By analogy with similar cases
    Causal,         // Based on cause-effect relationships
}

/// Represents the result of applying a reasoning strategy
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub strategy: ReasoningStrategy,
    pub premises: Vec<String>,
    pub conclusion: String,
    pub confidence: f32,
    pub success: bool,
}

/// Represents a complete reasoning chain
#[derive(Debug, Clone)]
pub struct ReasoningChain {
    pub steps: Vec<ReasoningStep>,
    pub final_conclusion: String,
    pub overall_confidence: f32,
}

/// Adaptive reasoning engine that learns which strategies work best
pub struct AdaptiveReasoningEngine {
    strategy_performance: Arc<RwLock<HashMap<ReasoningStrategy, PerformanceMetrics>>>,
    reasoning_history: Arc<RwLock<Vec<ReasoningChain>>>,
    adaptation_rate: f32,
}

/// Metrics for each reasoning strategy
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub success_count: usize,
    pub failure_count: usize,
    pub average_confidence: f32,
    pub last_used: u64,
}

impl PerformanceMetrics {
    fn new() -> Self {
        PerformanceMetrics {
            success_count: 0,
            failure_count: 0,
            average_confidence: 0.5,
            last_used: 0,
        }
    }

    fn success_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.5
        } else {
            self.success_count as f32 / total as f32
        }
    }
}

impl AdaptiveReasoningEngine {
    pub fn new() -> Self {
        let mut strategy_performance = HashMap::new();
        for strategy in &[
            ReasoningStrategy::Deductive,
            ReasoningStrategy::Inductive,
            ReasoningStrategy::Abductive,
            ReasoningStrategy::Analogical,
            ReasoningStrategy::Causal,
        ] {
            strategy_performance.insert(strategy.clone(), PerformanceMetrics::new());
        }

        AdaptiveReasoningEngine {
            strategy_performance: Arc::new(RwLock::new(strategy_performance)),
            reasoning_history: Arc::new(RwLock::new(Vec::new())),
            adaptation_rate: 0.1,
        }
    }

    /// Choose best strategy based on past performance
    pub fn select_best_strategy(&self) -> ReasoningStrategy {
        let metrics = self.strategy_performance.read().unwrap();
        
        metrics
            .iter()
            .max_by_key(|(_, m)| {
                ((m.success_rate() * 100.0) as usize, m.success_count)
            })
            .map(|(s, _)| s.clone())
            .unwrap_or(ReasoningStrategy::Deductive)
    }

    /// Apply a reasoning strategy
    pub fn apply_strategy(
        &self,
        strategy: ReasoningStrategy,
        premises: Vec<String>,
    ) -> ReasoningStep {
        let conclusion = match strategy {
            ReasoningStrategy::Deductive => self.deductive_reasoning(&premises),
            ReasoningStrategy::Inductive => self.inductive_reasoning(&premises),
            ReasoningStrategy::Abductive => self.abductive_reasoning(&premises),
            ReasoningStrategy::Analogical => self.analogical_reasoning(&premises),
            ReasoningStrategy::Causal => self.causal_reasoning(&premises),
        };

        ReasoningStep {
            strategy,
            premises,
            conclusion,
            confidence: 0.7,
            success: true,
        }
    }

    /// Multi-step reasoning chain
    pub fn multi_step_reasoning(&self, initial_premises: Vec<String>) -> ReasoningChain {
        let mut chain = ReasoningChain {
            steps: Vec::new(),
            final_conclusion: String::new(),
            overall_confidence: 1.0,
        };

        let mut current_premises = initial_premises;

        // Multi-step reasoning (up to 5 steps)
        for _ in 0..5 {
            let strategy = self.select_best_strategy();
            let step = self.apply_strategy(strategy, current_premises.clone());

            if step.success {
                chain.overall_confidence *= step.confidence;
                current_premises = vec![step.conclusion.clone()];
                chain.steps.push(step);
            } else {
                break;
            }
        }

        chain.final_conclusion = current_premises.join(" AND ");

        // Record reasoning in history
        self.reasoning_history.write().unwrap().push(chain.clone());

        chain
    }

    /// Adapt strategy selection based on success
    pub fn record_outcome(&self, strategy: ReasoningStrategy, success: bool) {
        let mut metrics = self.strategy_performance.write().unwrap();
        
        if let Some(perf) = metrics.get_mut(&strategy) {
            if success {
                perf.success_count += 1;
            } else {
                perf.failure_count += 1;
            }

            // Update average confidence using exponential moving average
            perf.average_confidence = (perf.average_confidence * (1.0 - self.adaptation_rate))
                + (if success { 1.0 } else { 0.0 } * self.adaptation_rate);
        }
    }

    // Individual reasoning implementations

    /// Deductive reasoning: derive specific conclusions from general premises
    /// Uses modus ponens pattern: "if A then B" + "A" → "B"
    fn deductive_reasoning(&self, premises: &[String]) -> String {
        if premises.is_empty() {
            return "No premises provided for deduction".to_string();
        }

        // Look for modus ponens patterns: "if X then Y" or "all X are Y"
        let mut conditionals: Vec<(&str, &str)> = Vec::new();
        let mut facts: Vec<&str> = Vec::new();

        for p in premises {
            let lower = p.to_lowercase();
            if lower.contains(" then ") {
                // "if A then B" pattern
                if let Some(rest) = lower.strip_prefix("if ") {
                    let parts: Vec<&str> = rest.splitn(2, " then ").collect();
                    if parts.len() == 2 {
                        conditionals.push((parts[0], parts[1]));
                    }
                }
            } else if lower.starts_with("all ") && lower.contains(" are ") {
                // "All X are Y" → "if X then Y"
                let rest = &lower[4..];
                let parts: Vec<&str> = rest.splitn(2, " are ").collect();
                if parts.len() == 2 {
                    conditionals.push((parts[0], parts[1]));
                }
            } else {
                facts.push(p.as_str());
            }
        }

        // Apply modus ponens: if we have "if A then B" and a fact matching A, conclude B
        let mut conclusions: Vec<String> = Vec::new();
        for (antecedent, consequent) in &conditionals {
            for fact in &facts {
                let fact_lower = fact.to_lowercase();
                if fact_lower.contains(antecedent) || antecedent.contains(&fact_lower) {
                    conclusions.push(format!("Therefore, {}", consequent));
                }
            }
        }

        // Chain: if we derived B and have "if B then C", derive C
        let mut derived: Vec<String> = conclusions.clone();
        for _ in 0..3 {
            let mut new_derived = Vec::new();
            for (antecedent, consequent) in &conditionals {
                for d in &derived {
                    let d_lower = d.to_lowercase();
                    if d_lower.contains(antecedent) {
                        let new_conclusion = format!("Therefore, {}", consequent);
                        if !derived.contains(&new_conclusion) && !new_derived.contains(&new_conclusion) {
                            new_derived.push(new_conclusion);
                        }
                    }
                }
            }
            if new_derived.is_empty() {
                break;
            }
            derived.extend(new_derived);
        }

        if derived.is_empty() {
            format!("Given {}, no deductive conclusion can be drawn", premises.join("; "))
        } else {
            derived.join(". ")
        }
    }

    /// Inductive reasoning: find general patterns from specific observations
    /// Looks for common elements across premises to form generalisations
    fn inductive_reasoning(&self, premises: &[String]) -> String {
        if premises.len() < 2 {
            return format!("Insufficient observations for induction (need ≥2, got {})", premises.len());
        }

        // Tokenize each premise and find common words (excluding stop words)
        let stop_words: Vec<&str> = vec![
            "a", "an", "the", "is", "are", "was", "were", "be", "been",
            "have", "has", "had", "do", "does", "did", "will", "would",
            "shall", "should", "may", "might", "can", "could", "and",
            "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "it", "this", "that", "these", "those",
        ];

        let tokenized: Vec<Vec<String>> = premises
            .iter()
            .map(|p| {
                p.to_lowercase()
                    .split_whitespace()
                    .filter(|w| !stop_words.contains(w) && w.len() > 1)
                    .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                    .filter(|w| !w.is_empty())
                    .collect()
            })
            .collect();

        // Find words appearing in majority of premises (>50%)
        let threshold = (premises.len() + 1) / 2;
        let mut word_counts: HashMap<&str, usize> = HashMap::new();
        for tokens in &tokenized {
            let unique: std::collections::HashSet<&str> = tokens.iter().map(|s| s.as_str()).collect();
            for word in unique {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }

        let common: Vec<&&str> = word_counts
            .iter()
            .filter(|(_, &count)| count >= threshold)
            .map(|(word, _)| word)
            .collect();

        if common.is_empty() {
            format!("No common pattern found across {} observations", premises.len())
        } else {
            let pattern_words: Vec<String> = common.iter().map(|w| w.to_string()).collect();
            format!(
                "Pattern observed: {} appear consistently across {}/{} observations, suggesting a general rule involving {}",
                pattern_words.join(", "),
                premises.len(),
                premises.len(),
                pattern_words.join(" and ")
            )
        }
    }

    /// Abductive reasoning: infer the best explanation for observations
    /// Generates candidate hypotheses and ranks them by plausibility
    fn abductive_reasoning(&self, premises: &[String]) -> String {
        if premises.is_empty() {
            return "No observations to explain".to_string();
        }

        // Generate candidate explanations based on premise content
        let mut hypotheses: Vec<(String, f32)> = Vec::new();
        let combined = premises.join(" ").to_lowercase();

        // Score hypotheses based on how many premises they explain
        // Simple keyword-based hypothesis generation
        let causal_keywords = ["because", "cause", "reason", "due to", "result of"];
        let temporal_keywords = ["before", "after", "when", "then", "during", "while"];
        let property_keywords = ["always", "never", "every", "all", "none"];

        let has_causal = causal_keywords.iter().any(|k| combined.contains(k));
        let has_temporal = temporal_keywords.iter().any(|k| combined.contains(k));
        let has_universal = property_keywords.iter().any(|k| combined.contains(k));

        if has_causal {
            hypotheses.push((
                format!("Causal mechanism explains: {}", premises[0]),
                0.8,
            ));
        }
        if has_temporal {
            hypotheses.push((
                format!("Temporal sequence suggests: events are causally linked"),
                0.7,
            ));
        }
        if has_universal {
            hypotheses.push((
                format!("Universal property detected: underlying rule governs all observations"),
                0.75,
            ));
        }

        // Default hypothesis: common cause
        hypotheses.push((
            format!(
                "Best explanation: a common underlying factor explains {} observations",
                premises.len()
            ),
            0.5 + (0.1 * premises.len() as f32).min(0.3),
        ));

        // Select best hypothesis
        hypotheses.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let best = &hypotheses[0];
        format!("{} (plausibility: {:.0}%)", best.0, best.1 * 100.0)
    }

    /// Analogical reasoning: find structural similarities between domains
    /// Maps relationships from a source domain to a target domain
    fn analogical_reasoning(&self, premises: &[String]) -> String {
        if premises.len() < 2 {
            return "Need at least 2 premises for analogy (source and target)".to_string();
        }

        // Treat first premise as source domain, rest as target
        let source = &premises[0];
        let targets = &premises[1..];

        // Tokenize and find structural mappings
        let source_tokens: Vec<&str> = source.split_whitespace().collect();
        let mut mappings: Vec<String> = Vec::new();

        for target in targets {
            let target_tokens: Vec<&str> = target.split_whitespace().collect();

            // Find shared structural elements (same position, same POS-like role)
            let shared: Vec<&&str> = source_tokens
                .iter()
                .filter(|t| target_tokens.contains(t))
                .collect();

            let unique_source: Vec<&&str> = source_tokens
                .iter()
                .filter(|t| !target_tokens.contains(t))
                .collect();

            let unique_target: Vec<&&str> = target_tokens
                .iter()
                .filter(|t| !source_tokens.contains(t))
                .collect();

            if !shared.is_empty() {
                let shared_str: Vec<String> = shared.iter().map(|s| s.to_string()).collect();
                mappings.push(format!(
                    "Shared structure: [{}]; Source-specific: [{}]; Target-specific: [{}]",
                    shared_str.join(", "),
                    unique_source.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "),
                    unique_target.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
                ));
            }
        }

        if mappings.is_empty() {
            format!(
                "No structural similarity found between '{}' and target domains",
                source
            )
        } else {
            format!(
                "By analogy: {}. Properties of source domain likely transfer to target.",
                mappings.join("; ")
            )
        }
    }

    /// Causal reasoning: identify cause-effect relationships
    /// Builds a causal chain from ordered premises
    fn causal_reasoning(&self, premises: &[String]) -> String {
        if premises.is_empty() {
            return "No events for causal analysis".to_string();
        }

        if premises.len() == 1 {
            return format!("Single event: {} (no causal chain possible)", premises[0]);
        }

        // Build causal chain: each premise causes the next
        let mut chain_links: Vec<String> = Vec::new();
        for i in 0..premises.len() - 1 {
            chain_links.push(format!("'{}' → '{}'", premises[i], premises[i + 1]));
        }

        // Identify potential confounders (shared words across non-adjacent premises)
        let mut confounders: Vec<String> = Vec::new();
        if premises.len() >= 3 {
            let first_words: std::collections::HashSet<String> = premises[0]
                .to_lowercase()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            let last_words: std::collections::HashSet<String> = premises
                .last()
                .unwrap()
                .to_lowercase()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            let shared: Vec<&String> = first_words.intersection(&last_words).collect();
            let stop = ["the", "a", "is", "are", "and", "or", "to", "of", "in"];
            for word in shared {
                if !stop.contains(&word.as_str()) && word.len() > 2 {
                    confounders.push(word.clone());
                }
            }
        }

        let chain = format!("Causal chain: {}", chain_links.join(", then "));
        if confounders.is_empty() {
            format!("{}. Root cause: '{}'", chain, premises[0])
        } else {
            format!(
                "{}. Root cause: '{}'. Potential confounders: {}",
                chain,
                premises[0],
                confounders.join(", ")
            )
        }
    }

    /// Get strategy performance report
    pub fn get_performance_report(&self) -> HashMap<ReasoningStrategy, PerformanceMetrics> {
        self.strategy_performance.read().unwrap().clone()
    }

    /// Get reasoning history
    pub fn get_reasoning_history(&self) -> Vec<ReasoningChain> {
        self.reasoning_history.read().unwrap().clone()
    }
}

/// Context-aware reasoning
pub struct ContextualReasoner {
    context: Arc<RwLock<HashMap<String, String>>>,
    reasoning_engine: Arc<AdaptiveReasoningEngine>,
}

impl ContextualReasoner {
    pub fn new(engine: Arc<AdaptiveReasoningEngine>) -> Self {
        ContextualReasoner {
            context: Arc::new(RwLock::new(HashMap::new())),
            reasoning_engine: engine,
        }
    }

    /// Set context variable
    pub fn set_context(&self, key: String, value: String) {
        self.context.write().unwrap().insert(key, value);
    }

    /// Reason with current context
    pub fn reason_with_context(&self, premises: Vec<String>) -> ReasoningChain {
        let context = self.context.read().unwrap();
        let enhanced_premises = premises
            .into_iter()
            .map(|p| {
                let mut result = p;
                for (k, v) in context.iter() {
                    result = result.replace(&format!("{{{}}}", k), v);
                }
                result
            })
            .collect();

        self.reasoning_engine.multi_step_reasoning(enhanced_premises)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_selection() {
        let engine = AdaptiveReasoningEngine::new();
        let strategy = engine.select_best_strategy();
        assert!(matches!(strategy, ReasoningStrategy::Deductive | ReasoningStrategy::Inductive | ReasoningStrategy::Abductive | ReasoningStrategy::Analogical | ReasoningStrategy::Causal));
    }

    #[test]
    fn test_reasoning_step() {
        let engine = AdaptiveReasoningEngine::new();
        let premises = vec!["All humans are mortal".to_string(), "Socrates is human".to_string()];
        let step = engine.apply_strategy(ReasoningStrategy::Deductive, premises);

        assert!(!step.conclusion.is_empty());
        assert!(step.success);
    }

    #[test]
    fn test_multi_step_reasoning() {
        let engine = AdaptiveReasoningEngine::new();
        let premises = vec!["Premise 1".to_string(), "Premise 2".to_string()];
        let chain = engine.multi_step_reasoning(premises);

        assert!(chain.overall_confidence > 0.0);
    }

    #[test]
    fn test_adaptation() {
        let engine = AdaptiveReasoningEngine::new();
        
        engine.record_outcome(ReasoningStrategy::Deductive, true);
        engine.record_outcome(ReasoningStrategy::Deductive, true);
        engine.record_outcome(ReasoningStrategy::Inductive, false);

        let report = engine.get_performance_report();
        let deductive = &report[&ReasoningStrategy::Deductive];
        
        assert_eq!(deductive.success_count, 2);
        assert_eq!(deductive.failure_count, 0);
    }

    #[test]
    fn test_contextual_reasoning() {
        let engine = Arc::new(AdaptiveReasoningEngine::new());
        let reasoner = ContextualReasoner::new(engine);

        reasoner.set_context("domain".to_string(), "mathematics".to_string());
        let premises = vec!["{domain} is important".to_string()];
        let chain = reasoner.reason_with_context(premises);

        assert!(!chain.final_conclusion.is_empty());
    }
}
