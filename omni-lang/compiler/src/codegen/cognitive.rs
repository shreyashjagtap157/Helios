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

//! Cognitive Compilation Framework
//!
//! Provides intelligent, adaptive compilation strategies that learn from
//! program behavior and optimize code generation decisions over time.
//! This module implements pattern recognition, heuristic learning, and
//! adaptive optimization scheduling.

use std::collections::HashMap;

// ─── Pattern Recognition ─────────────────────────────────────────────────

/// Recognized code patterns that influence optimization decisions
#[derive(Debug, Clone, PartialEq)]
pub enum CodePattern {
    /// Tight numeric loop (e.g., `for i in 0..n { sum += arr[i]; }`)
    TightLoop { estimated_iterations: u64 },
    /// Recursive function with potential for tail-call optimization
    TailRecursion { depth_estimate: u32 },
    /// Matrix/tensor operation suitable for vectorization
    MatrixOp { dimensions: (u32, u32) },
    /// Branch-heavy control flow (e.g., match with many arms)
    BranchHeavy { branch_count: u32 },
    /// Memory-intensive pattern (frequent allocations)
    MemoryIntensive { alloc_sites: u32 },
    /// Pure function (no side effects)
    PureFunction,
    /// Hot path identified by profiling data
    HotPath { invocation_count: u64 },
    /// GPU-suitable parallel computation
    GpuCandidate { parallelism_factor: f64 },
    /// String-heavy operations
    StringProcessing,
    /// I/O bound operations
    IoBound,
}

/// Result of pattern analysis on a function
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    pub function_name: String,
    pub patterns: Vec<CodePattern>,
    pub confidence: f64,
    pub suggested_strategy: OptimizationStrategy,
}

/// Pattern recognizer that analyzes IR to identify optimization opportunities
pub struct PatternRecognizer {
    /// Historical pattern frequency data
    pattern_history: HashMap<String, Vec<CodePattern>>,
    /// Minimum confidence threshold for pattern reporting
    confidence_threshold: f64,
}

impl Default for PatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternRecognizer {
    pub fn new() -> Self {
        Self {
            pattern_history: HashMap::new(),
            confidence_threshold: 0.6,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Analyze a function's IR to detect patterns
    pub fn analyze_function(
        &mut self,
        name: &str,
        instruction_count: usize,
        has_loops: bool,
        has_recursion: bool,
        has_branches: u32,
    ) -> PatternAnalysis {
        let mut patterns = Vec::new();
        let mut confidence: f64 = 0.5;

        // Detect tight loops
        if has_loops && instruction_count < 50 {
            patterns.push(CodePattern::TightLoop {
                estimated_iterations: 1000,
            });
            confidence += 0.15;
        }

        // Detect tail recursion candidates
        if has_recursion {
            patterns.push(CodePattern::TailRecursion {
                depth_estimate: 100,
            });
            confidence += 0.1;
        }

        // Detect branch-heavy code
        if has_branches > 4 {
            patterns.push(CodePattern::BranchHeavy {
                branch_count: has_branches,
            });
            confidence += 0.1;
        }

        // Detect pure functions (heuristic: small, no calls)
        if instruction_count < 20 && !has_recursion && has_branches == 0 {
            patterns.push(CodePattern::PureFunction);
            confidence += 0.1;
        }

        // GPU candidate detection
        if has_loops && instruction_count > 100 {
            patterns.push(CodePattern::GpuCandidate {
                parallelism_factor: 0.8,
            });
            confidence += 0.05;
        }

        confidence = confidence.min(1.0);

        // Record history
        self.pattern_history
            .insert(name.to_string(), patterns.clone());

        let strategy = self.suggest_strategy(&patterns);

        PatternAnalysis {
            function_name: name.to_string(),
            patterns,
            confidence,
            suggested_strategy: strategy,
        }
    }

    /// Suggest an optimization strategy based on detected patterns
    fn suggest_strategy(&self, patterns: &[CodePattern]) -> OptimizationStrategy {
        if patterns
            .iter()
            .any(|p| matches!(p, CodePattern::GpuCandidate { .. }))
        {
            return OptimizationStrategy::GpuOffload;
        }
        if patterns
            .iter()
            .any(|p| matches!(p, CodePattern::TightLoop { .. }))
        {
            return OptimizationStrategy::AggressiveOptimize;
        }
        if patterns
            .iter()
            .any(|p| matches!(p, CodePattern::PureFunction))
        {
            return OptimizationStrategy::Inline;
        }
        if patterns
            .iter()
            .any(|p| matches!(p, CodePattern::MemoryIntensive { .. }))
        {
            return OptimizationStrategy::MemoryOptimize;
        }
        OptimizationStrategy::Standard
    }

    /// Get pattern history for a function
    pub fn get_history(&self, name: &str) -> Option<&Vec<CodePattern>> {
        self.pattern_history.get(name)
    }

    /// Clear all history
    pub fn clear_history(&mut self) {
        self.pattern_history.clear();
    }
}

// ─── Optimization Strategy ───────────────────────────────────────────────

/// High-level optimization strategy chosen by the cognitive engine
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationStrategy {
    /// Minimal optimization for fast compilation (debug builds)
    Minimal,
    /// Standard optimization (balanced speed/quality)
    Standard,
    /// Aggressive optimization (loop unrolling, vectorization, etc.)
    AggressiveOptimize,
    /// Inline the function at call sites
    Inline,
    /// Optimize memory access patterns
    MemoryOptimize,
    /// Offload computation to GPU
    GpuOffload,
    /// Profile-guided optimization
    ProfileGuided { hotness: f64 },
    /// Speculative optimization with deopt fallback
    Speculative,
}

// ─── Heuristic Engine ────────────────────────────────────────────────────

/// A learned heuristic for making compilation decisions
#[derive(Debug, Clone)]
pub struct Heuristic {
    pub name: String,
    pub weight: f64,
    pub success_count: u64,
    pub total_count: u64,
}

impl Heuristic {
    pub fn new(name: &str, initial_weight: f64) -> Self {
        Self {
            name: name.to_string(),
            weight: initial_weight,
            success_count: 0,
            total_count: 0,
        }
    }

    /// Record outcome and update weight using exponential moving average
    pub fn record_outcome(&mut self, success: bool) {
        self.total_count += 1;
        if success {
            self.success_count += 1;
        }
        let alpha = 0.1; // learning rate
        let outcome = if success { 1.0 } else { 0.0 };
        self.weight = self.weight * (1.0 - alpha) + outcome * alpha;
    }

    /// Get the success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_count == 0 {
            return 0.5; // neutral prior
        }
        self.success_count as f64 / self.total_count as f64
    }
}

/// Engine that manages multiple heuristics for compilation decisions
pub struct HeuristicEngine {
    heuristics: HashMap<String, Heuristic>,
    decision_log: Vec<Decision>,
}

/// A recorded decision for learning
#[derive(Debug, Clone)]
pub struct Decision {
    pub heuristic_name: String,
    pub chosen: bool,
    pub context: String,
    pub outcome: Option<bool>,
}

impl HeuristicEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            heuristics: HashMap::new(),
            decision_log: Vec::new(),
        };

        // Initialize default heuristics
        engine.register_heuristic("inline_small_functions", 0.8);
        engine.register_heuristic("unroll_short_loops", 0.7);
        engine.register_heuristic("vectorize_array_ops", 0.75);
        engine.register_heuristic("speculate_on_types", 0.5);
        engine.register_heuristic("gpu_offload_threshold", 0.4);
        engine.register_heuristic("tail_call_optimize", 0.9);
        engine.register_heuristic("constant_propagation", 0.85);
        engine.register_heuristic("escape_analysis", 0.6);

        engine
    }

    /// Register a new heuristic with initial weight
    pub fn register_heuristic(&mut self, name: &str, initial_weight: f64) {
        self.heuristics
            .insert(name.to_string(), Heuristic::new(name, initial_weight));
    }

    /// Query a heuristic — should we apply this optimization?
    pub fn should_apply(&self, name: &str) -> bool {
        self.heuristics
            .get(name)
            .map(|h| h.weight > 0.5)
            .unwrap_or(false)
    }

    /// Get the weight of a heuristic
    pub fn get_weight(&self, name: &str) -> f64 {
        self.heuristics.get(name).map(|h| h.weight).unwrap_or(0.0)
    }

    /// Make a decision and log it
    pub fn decide(&mut self, name: &str, context: &str) -> bool {
        let chosen = self.should_apply(name);
        self.decision_log.push(Decision {
            heuristic_name: name.to_string(),
            chosen,
            context: context.to_string(),
            outcome: None,
        });
        chosen
    }

    /// Record the outcome of a decision for learning
    pub fn record_feedback(&mut self, name: &str, success: bool) {
        if let Some(h) = self.heuristics.get_mut(name) {
            h.record_outcome(success);
        }

        // Update the most recent matching decision
        for decision in self.decision_log.iter_mut().rev() {
            if decision.heuristic_name == name && decision.outcome.is_none() {
                decision.outcome = Some(success);
                break;
            }
        }
    }

    /// Get all heuristic names
    pub fn heuristic_names(&self) -> Vec<&str> {
        self.heuristics.keys().map(|s| s.as_str()).collect()
    }

    /// Get decision count
    pub fn decision_count(&self) -> usize {
        self.decision_log.len()
    }

    /// Export heuristic state for persistence
    pub fn export_state(&self) -> Vec<(String, f64, u64, u64)> {
        self.heuristics
            .iter()
            .map(|(name, h)| (name.clone(), h.weight, h.success_count, h.total_count))
            .collect()
    }
}

// ─── Adaptive Scheduler ──────────────────────────────────────────────────

/// Compilation task with priority
#[derive(Debug, Clone)]
pub struct CompilationTask {
    pub function_name: String,
    pub priority: f64,
    pub strategy: OptimizationStrategy,
    pub estimated_cost_ms: f64,
}

/// Adaptive optimization scheduler that orders compilation work
/// based on expected benefit and available resources
pub struct AdaptiveScheduler {
    queue: Vec<CompilationTask>,
    time_budget_ms: f64,
    completed: Vec<String>,
}

impl AdaptiveScheduler {
    pub fn new(time_budget_ms: f64) -> Self {
        Self {
            queue: Vec::new(),
            time_budget_ms,
            completed: Vec::new(),
        }
    }

    /// Add a compilation task to the queue
    pub fn enqueue(&mut self, task: CompilationTask) {
        self.queue.push(task);
        // Sort by priority descending (highest priority first)
        self.queue.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Get the next task within time budget
    pub fn next_task(&mut self) -> Option<CompilationTask> {
        if self.queue.is_empty() {
            return None;
        }

        let remaining = self.remaining_budget();
        // Find the highest-priority task that fits in the budget
        let idx = self
            .queue
            .iter()
            .position(|t| t.estimated_cost_ms <= remaining)?;
        let task = self.queue.remove(idx);
        self.time_budget_ms -= task.estimated_cost_ms;
        Some(task)
    }

    /// Mark a task as completed
    pub fn complete(&mut self, function_name: &str) {
        self.completed.push(function_name.to_string());
    }

    /// Get remaining time budget
    pub fn remaining_budget(&self) -> f64 {
        self.time_budget_ms.max(0.0)
    }

    /// Get queue length
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Get completed count
    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    /// Check if there's work remaining
    pub fn has_work(&self) -> bool {
        !self.queue.is_empty() && self.remaining_budget() > 0.0
    }
}

// ─── Cognitive Compiler ──────────────────────────────────────────────────

/// The main cognitive compilation engine that ties together pattern recognition,
/// heuristic learning, and adaptive scheduling to make intelligent compilation decisions.
pub struct CognitiveCompiler {
    recognizer: PatternRecognizer,
    heuristics: HeuristicEngine,
    scheduler: AdaptiveScheduler,
    compilation_stats: CompilationStats,
}

/// Statistics about cognitive compilation
#[derive(Debug, Clone, Default)]
pub struct CompilationStats {
    pub functions_analyzed: u64,
    pub patterns_detected: u64,
    pub optimizations_applied: u64,
    pub optimizations_skipped: u64,
    pub gpu_offloads: u64,
    pub inlines: u64,
    pub total_time_ms: f64,
}

impl CognitiveCompiler {
    pub fn new(time_budget_ms: f64) -> Self {
        Self {
            recognizer: PatternRecognizer::new(),
            heuristics: HeuristicEngine::new(),
            scheduler: AdaptiveScheduler::new(time_budget_ms),
            compilation_stats: CompilationStats::default(),
        }
    }

    /// Analyze a function and schedule its compilation
    pub fn analyze_and_schedule(
        &mut self,
        name: &str,
        instruction_count: usize,
        has_loops: bool,
        has_recursion: bool,
        branch_count: u32,
    ) -> PatternAnalysis {
        let analysis = self.recognizer.analyze_function(
            name,
            instruction_count,
            has_loops,
            has_recursion,
            branch_count,
        );

        self.compilation_stats.functions_analyzed += 1;
        self.compilation_stats.patterns_detected += analysis.patterns.len() as u64;

        // Compute priority based on patterns
        let priority = self.compute_priority(&analysis);

        // Estimate compilation cost
        let cost = self.estimate_cost(instruction_count, &analysis.suggested_strategy);

        // Schedule
        self.scheduler.enqueue(CompilationTask {
            function_name: name.to_string(),
            priority,
            strategy: analysis.suggested_strategy.clone(),
            estimated_cost_ms: cost,
        });

        analysis
    }

    /// Compute compilation priority from analysis
    fn compute_priority(&self, analysis: &PatternAnalysis) -> f64 {
        let mut priority = analysis.confidence;

        for pattern in &analysis.patterns {
            match pattern {
                CodePattern::HotPath { invocation_count } => {
                    priority += (*invocation_count as f64).log2() / 10.0;
                }
                CodePattern::TightLoop { .. } => priority += 0.3,
                CodePattern::GpuCandidate { parallelism_factor } => {
                    priority += parallelism_factor * 0.5;
                }
                CodePattern::PureFunction => priority += 0.2,
                _ => {}
            }
        }

        priority.min(1.0)
    }

    /// Estimate compilation cost in milliseconds
    fn estimate_cost(&self, instruction_count: usize, strategy: &OptimizationStrategy) -> f64 {
        let base = instruction_count as f64 * 0.01; // ~0.01ms per instruction
        match strategy {
            OptimizationStrategy::Minimal => base * 0.5,
            OptimizationStrategy::Standard => base,
            OptimizationStrategy::AggressiveOptimize => base * 3.0,
            OptimizationStrategy::Inline => base * 1.5,
            OptimizationStrategy::MemoryOptimize => base * 2.0,
            OptimizationStrategy::GpuOffload => base * 5.0,
            OptimizationStrategy::ProfileGuided { .. } => base * 2.5,
            OptimizationStrategy::Speculative => base * 4.0,
        }
    }

    /// Process the next compilation task
    pub fn process_next(&mut self) -> Option<CompilationTask> {
        if let Some(task) = self.scheduler.next_task() {
            // Apply heuristic-guided decisions
            match &task.strategy {
                OptimizationStrategy::Inline => {
                    if self
                        .heuristics
                        .decide("inline_small_functions", &task.function_name)
                    {
                        self.compilation_stats.inlines += 1;
                        self.compilation_stats.optimizations_applied += 1;
                    } else {
                        self.compilation_stats.optimizations_skipped += 1;
                    }
                }
                OptimizationStrategy::GpuOffload => {
                    if self
                        .heuristics
                        .decide("gpu_offload_threshold", &task.function_name)
                    {
                        self.compilation_stats.gpu_offloads += 1;
                        self.compilation_stats.optimizations_applied += 1;
                    } else {
                        self.compilation_stats.optimizations_skipped += 1;
                    }
                }
                _ => {
                    self.compilation_stats.optimizations_applied += 1;
                }
            }

            self.scheduler.complete(&task.function_name);
            Some(task)
        } else {
            None
        }
    }

    /// Process all pending tasks
    pub fn process_all(&mut self) -> Vec<CompilationTask> {
        let mut results = Vec::new();
        while let Some(task) = self.process_next() {
            results.push(task);
        }
        results
    }

    /// Record feedback about an optimization decision
    pub fn record_feedback(&mut self, heuristic: &str, success: bool) {
        self.heuristics.record_feedback(heuristic, success);
    }

    /// Get compilation statistics
    pub fn stats(&self) -> &CompilationStats {
        &self.compilation_stats
    }

    /// Get remaining scheduler budget
    pub fn remaining_budget(&self) -> f64 {
        self.scheduler.remaining_budget()
    }

    /// Check if there's pending work
    pub fn has_pending_work(&self) -> bool {
        self.scheduler.has_work()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_recognizer_tight_loop() {
        let mut recognizer = PatternRecognizer::new();
        let analysis = recognizer.analyze_function("sum_array", 30, true, false, 1);
        assert!(analysis
            .patterns
            .iter()
            .any(|p| matches!(p, CodePattern::TightLoop { .. })));
        assert_eq!(
            analysis.suggested_strategy,
            OptimizationStrategy::AggressiveOptimize
        );
    }

    #[test]
    fn test_pattern_recognizer_pure_function() {
        let mut recognizer = PatternRecognizer::new();
        let analysis = recognizer.analyze_function("add", 5, false, false, 0);
        assert!(analysis
            .patterns
            .iter()
            .any(|p| matches!(p, CodePattern::PureFunction)));
        assert_eq!(analysis.suggested_strategy, OptimizationStrategy::Inline);
    }

    #[test]
    fn test_pattern_recognizer_gpu_candidate() {
        let mut recognizer = PatternRecognizer::new();
        let analysis = recognizer.analyze_function("matrix_mul", 200, true, false, 2);
        assert!(analysis
            .patterns
            .iter()
            .any(|p| matches!(p, CodePattern::GpuCandidate { .. })));
        assert_eq!(
            analysis.suggested_strategy,
            OptimizationStrategy::GpuOffload
        );
    }

    #[test]
    fn test_pattern_recognizer_branch_heavy() {
        let mut recognizer = PatternRecognizer::new();
        let analysis = recognizer.analyze_function("dispatch", 60, false, false, 10);
        assert!(analysis
            .patterns
            .iter()
            .any(|p| matches!(p, CodePattern::BranchHeavy { branch_count: 10 })));
    }

    #[test]
    fn test_pattern_recognizer_tail_recursion() {
        let mut recognizer = PatternRecognizer::new();
        let analysis = recognizer.analyze_function("factorial", 15, false, true, 1);
        assert!(analysis
            .patterns
            .iter()
            .any(|p| matches!(p, CodePattern::TailRecursion { .. })));
    }

    #[test]
    fn test_pattern_recognizer_history() {
        let mut recognizer = PatternRecognizer::new();
        recognizer.analyze_function("foo", 10, false, false, 0);
        assert!(recognizer.get_history("foo").is_some());
        assert!(recognizer.get_history("bar").is_none());
        recognizer.clear_history();
        assert!(recognizer.get_history("foo").is_none());
    }

    #[test]
    fn test_pattern_recognizer_confidence_threshold() {
        let recognizer = PatternRecognizer::new().with_threshold(0.9);
        assert!((recognizer.confidence_threshold - 0.9).abs() < f64::EPSILON);

        // Clamp to [0, 1]
        let recognizer2 = PatternRecognizer::new().with_threshold(1.5);
        assert!((recognizer2.confidence_threshold - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_heuristic_learning() {
        let mut h = Heuristic::new("test", 0.5);
        assert!((h.success_rate() - 0.5).abs() < f64::EPSILON); // neutral prior

        // Record successes
        for _ in 0..10 {
            h.record_outcome(true);
        }
        assert!(h.weight > 0.5);
        assert!((h.success_rate() - 1.0).abs() < f64::EPSILON);

        // Record failures
        for _ in 0..10 {
            h.record_outcome(false);
        }
        assert!(h.success_rate() < 1.0);
    }

    #[test]
    fn test_heuristic_engine_defaults() {
        let engine = HeuristicEngine::new();
        assert!(engine.should_apply("inline_small_functions")); // weight 0.8
        assert!(engine.should_apply("unroll_short_loops")); // weight 0.7
        assert!(engine.should_apply("vectorize_array_ops")); // weight 0.75
        assert!(!engine.should_apply("nonexistent"));
    }

    #[test]
    fn test_heuristic_engine_decide_and_feedback() {
        let mut engine = HeuristicEngine::new();

        let decision = engine.decide("inline_small_functions", "test_func");
        assert!(decision); // initial weight 0.8 > 0.5

        engine.record_feedback("inline_small_functions", true);
        assert_eq!(engine.decision_count(), 1);
    }

    #[test]
    fn test_heuristic_engine_register_custom() {
        let mut engine = HeuristicEngine::new();
        engine.register_heuristic("custom_opt", 0.3);
        assert!(!engine.should_apply("custom_opt")); // 0.3 < 0.5
        assert!((engine.get_weight("custom_opt") - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_heuristic_engine_export_state() {
        let engine = HeuristicEngine::new();
        let state = engine.export_state();
        assert!(state.len() >= 8); // 8 default heuristics
    }

    #[test]
    fn test_adaptive_scheduler_basic() {
        let mut scheduler = AdaptiveScheduler::new(100.0);

        scheduler.enqueue(CompilationTask {
            function_name: "low".to_string(),
            priority: 0.3,
            strategy: OptimizationStrategy::Minimal,
            estimated_cost_ms: 10.0,
        });

        scheduler.enqueue(CompilationTask {
            function_name: "high".to_string(),
            priority: 0.9,
            strategy: OptimizationStrategy::AggressiveOptimize,
            estimated_cost_ms: 20.0,
        });

        assert_eq!(scheduler.queue_len(), 2);

        let task = scheduler.next_task().unwrap();
        assert_eq!(task.function_name, "high"); // highest priority first
        assert!((scheduler.remaining_budget() - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_adaptive_scheduler_budget_limit() {
        let mut scheduler = AdaptiveScheduler::new(5.0);

        scheduler.enqueue(CompilationTask {
            function_name: "expensive".to_string(),
            priority: 0.9,
            strategy: OptimizationStrategy::AggressiveOptimize,
            estimated_cost_ms: 100.0,
        });

        // Task is too expensive for budget
        assert!(scheduler.next_task().is_none());
    }

    #[test]
    fn test_adaptive_scheduler_complete() {
        let mut scheduler = AdaptiveScheduler::new(100.0);
        scheduler.complete("test_fn");
        assert_eq!(scheduler.completed_count(), 1);
    }

    #[test]
    fn test_cognitive_compiler_full_pipeline() {
        let mut compiler = CognitiveCompiler::new(1000.0);

        // Analyze several functions
        compiler.analyze_and_schedule("tight_loop", 30, true, false, 1);
        compiler.analyze_and_schedule("pure_add", 5, false, false, 0);
        compiler.analyze_and_schedule("big_compute", 200, true, false, 2);
        compiler.analyze_and_schedule("recursive", 15, false, true, 1);

        assert_eq!(compiler.stats().functions_analyzed, 4);
        assert!(compiler.stats().patterns_detected > 0);

        // Process all tasks
        let tasks = compiler.process_all();
        assert!(!tasks.is_empty());
        assert!(compiler.stats().optimizations_applied > 0);
    }

    #[test]
    fn test_cognitive_compiler_feedback_loop() {
        let mut compiler = CognitiveCompiler::new(1000.0);

        compiler.analyze_and_schedule("test_fn", 10, false, false, 0);
        compiler.process_all();

        // Record feedback
        compiler.record_feedback("inline_small_functions", true);
        compiler.record_feedback("inline_small_functions", true);
        compiler.record_feedback("inline_small_functions", false);
    }

    #[test]
    fn test_cognitive_compiler_budget_tracking() {
        let mut compiler = CognitiveCompiler::new(50.0);

        // Schedule a small function
        compiler.analyze_and_schedule("small", 10, false, false, 0);
        let initial_budget = compiler.remaining_budget();

        compiler.process_all();
        assert!(compiler.remaining_budget() < initial_budget);
    }

    #[test]
    fn test_cognitive_compiler_no_pending_after_drain() {
        let mut compiler = CognitiveCompiler::new(10000.0);
        compiler.analyze_and_schedule("fn1", 20, false, false, 0);
        compiler.analyze_and_schedule("fn2", 30, true, false, 0);
        compiler.process_all();
        assert!(!compiler.has_pending_work());
    }
}
