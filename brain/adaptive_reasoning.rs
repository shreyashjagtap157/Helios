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

    // Individual reasoning implementations (simplified)

    fn deductive_reasoning(&self, premises: &[String]) -> String {
        format!("Deductive conclusion from: {}", premises.join(", "))
    }

    fn inductive_reasoning(&self, premises: &[String]) -> String {
        format!("Inductive pattern from: {}", premises.join(", "))
    }

    fn abductive_reasoning(&self, premises: &[String]) -> String {
        format!("Best explanation for: {}", premises.join(", "))
    }

    fn analogical_reasoning(&self, premises: &[String]) -> String {
        format!("By analogy with: {}", premises.join(", "))
    }

    fn causal_reasoning(&self, premises: &[String]) -> String {
        format!("Causal chain: {}", premises.join(" -> "))
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
