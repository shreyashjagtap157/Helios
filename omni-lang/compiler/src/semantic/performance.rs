//! Performance profiling and metrics collection
//!
//! Provides:
//! - Timing collection for semantic phases
//! - Memory usage tracking
//! - Cache hit/miss statistics
//! - Compilation metrics

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics for a single operation
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub name: String,
    pub duration: Duration,
    pub allocated: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl OperationMetrics {
    pub fn new(name: String) -> Self {
        Self {
            name,
            duration: Duration::ZERO,
            allocated: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let total = (self.cache_hits + self.cache_misses) as f64;
        if total == 0.0 {
            0.0
        } else {
            self.cache_hits as f64 / total
        }
    }
}

/// Profiler for tracking semantic analysis performance
pub struct SemanticProfiler {
    /// Map operation name to metrics
    operations: HashMap<String, OperationMetrics>,
    /// Stack of currently running operations
    stack: Vec<(String, Instant)>,
}

impl Default for SemanticProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticProfiler {
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
            stack: Vec::new(),
        }
    }

    /// Start profiling an operation
    pub fn start_operation(&mut self, name: &str) {
        self.stack.push((name.to_string(), Instant::now()));
    }

    /// End profiling and record metrics
    pub fn end_operation(&mut self) -> Option<Duration> {
        if let Some((name, start)) = self.stack.pop() {
            let duration = start.elapsed();

            let entry = self
                .operations
                .entry(name.clone())
                .or_insert_with(|| OperationMetrics::new(name));
            entry.duration += duration;

            Some(duration)
        } else {
            None
        }
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self, operation: &str) {
        if let Some(metrics) = self.operations.get_mut(operation) {
            metrics.cache_hits += 1;
        }
    }

    /// Record cache miss
    pub fn record_cache_miss(&mut self, operation: &str) {
        if let Some(metrics) = self.operations.get_mut(operation) {
            metrics.cache_misses += 1;
        }
    }

    /// Get metrics for an operation
    pub fn get_metrics(&self, operation: &str) -> Option<&OperationMetrics> {
        self.operations.get(operation)
    }

    /// Get all metrics
    pub fn all_metrics(&self) -> Vec<&OperationMetrics> {
        self.operations.values().collect()
    }

    /// Clear profiling data
    pub fn clear(&mut self) {
        self.operations.clear();
        self.stack.clear();
    }

    /// Generate performance report
    pub fn report(&self) -> String {
        let mut report = String::from("=== Semantic Analysis Performance Report ===\n\n");

        let mut metrics: Vec<_> = self.operations.values().collect();
        metrics.sort_by_key(|m| std::cmp::Reverse(m.duration));

        for metric in metrics {
            report.push_str(&format!(
                "{}: {:.3}ms (cache hit rate: {:.1}%)\n",
                metric.name,
                metric.duration.as_secs_f64() * 1000.0,
                metric.hit_rate() * 100.0
            ));
        }

        let total: Duration = self.operations.values().map(|m| m.duration).sum();
        report.push_str(&format!(
            "\nTotal time: {:.3}ms\n",
            total.as_secs_f64() * 1000.0
        ));

        report
    }
}

/// Memory profiler for tracking allocation
#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub operation: String,
    pub allocated_bytes: usize,
    pub deallocated_bytes: usize,
    pub peak_usage: usize,
}

impl MemoryMetrics {
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            allocated_bytes: 0,
            deallocated_bytes: 0,
            peak_usage: 0,
        }
    }

    pub fn current_usage(&self) -> usize {
        self.allocated_bytes - self.deallocated_bytes
    }
}

/// Compilation metrics aggregator
pub struct CompilationMetrics {
    /// Metrics per semantic phase
    phase_metrics: HashMap<String, PhaseMetrics>,
    /// Total compilation time
    start_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct PhaseMetrics {
    pub phase: String,
    pub duration: Duration,
    pub errors: usize,
    pub warnings: usize,
}

impl Default for CompilationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilationMetrics {
    pub fn new() -> Self {
        Self {
            phase_metrics: HashMap::new(),
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn end(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    pub fn record_phase(
        &mut self,
        phase: &str,
        duration: Duration,
        errors: usize,
        warnings: usize,
    ) {
        self.phase_metrics.insert(
            phase.to_string(),
            PhaseMetrics {
                phase: phase.to_string(),
                duration,
                errors,
                warnings,
            },
        );
    }

    pub fn get_phase(&self, phase: &str) -> Option<&PhaseMetrics> {
        self.phase_metrics.get(phase)
    }

    pub fn critical_path(&self) -> Duration {
        self.phase_metrics
            .values()
            .map(|p| p.duration)
            .max()
            .unwrap_or(Duration::ZERO)
    }

    pub fn total_duration(&self) -> Duration {
        self.phase_metrics.values().map(|p| p.duration).sum()
    }

    pub fn format_report(&self) -> String {
        let mut report = String::from("=== Compilation Metrics ===\n\n");

        let mut phases: Vec<_> = self.phase_metrics.values().collect();
        phases.sort_by_key(|p| std::cmp::Reverse(p.duration));

        for phase in phases {
            report.push_str(&format!(
                "{}: {:.3}ms (errors: {}, warnings: {})\n",
                phase.phase,
                phase.duration.as_secs_f64() * 1000.0,
                phase.errors,
                phase.warnings
            ));
        }

        if let Some(total) = self.end() {
            report.push_str(&format!(
                "\nTotal compilation time: {:.3}ms\n",
                total.as_secs_f64() * 1000.0
            ));
        }

        report.push_str(&format!(
            "Critical path: {:.3}ms\n",
            self.critical_path().as_secs_f64() * 1000.0
        ));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_operation_profiler() {
        let mut profiler = SemanticProfiler::new();
        profiler.start_operation("test_op");
        thread::sleep(Duration::from_millis(10));
        let duration = profiler.end_operation();
        assert!(duration.is_some());
    }

    #[test]
    fn test_cache_tracking() {
        let mut profiler = SemanticProfiler::new();
        profiler.operations.insert(
            "test".to_string(),
            OperationMetrics::new("test".to_string()),
        );
        profiler.record_cache_hit("test");
        profiler.record_cache_hit("test");
        profiler.record_cache_miss("test");

        let metrics = profiler.get_metrics("test").unwrap();
        assert_eq!(metrics.cache_hits, 2);
        assert_eq!(metrics.cache_misses, 1);
        assert!(metrics.hit_rate() > 0.6);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let mut metrics = OperationMetrics::new("test".to_string());
        metrics.cache_hits = 8;
        metrics.cache_misses = 2;
        assert_eq!(metrics.hit_rate(), 0.8);
    }

    #[test]
    fn test_compilation_metrics() {
        let mut metrics = CompilationMetrics::new();
        metrics.start();

        let phase_duration = Duration::from_millis(50);
        metrics.record_phase("type_check", phase_duration, 0, 2);
        metrics.record_phase("borrow_check", phase_duration, 1, 0);

        assert_eq!(metrics.phase_metrics.len(), 2);
        assert!(metrics.end().is_some());
    }

    #[test]
    fn test_profiler_report() {
        let mut profiler = SemanticProfiler::new();
        profiler.operations.insert(
            "parse".to_string(),
            OperationMetrics::new("parse".to_string()),
        );
        let report = profiler.report();
        assert!(report.contains("parse"));
        assert!(report.contains("ms"));
    }
}
