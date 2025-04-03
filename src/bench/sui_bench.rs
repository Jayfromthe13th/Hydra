use super::*;
use crate::analyzer::types::*;
use std::time::{Duration, Instant};

pub struct SuiAnalysisBenchmark {
    pub module_analysis: ModuleAnalysisMetrics,
    pub object_analysis: ObjectAnalysisMetrics,
    pub capability_analysis: CapabilityAnalysisMetrics,
    pub memory_stats: MemoryStats,
}

#[derive(Debug, Default)]
pub struct ModuleAnalysisMetrics {
    pub total_modules: usize,
    pub total_functions: usize,
    pub total_structs: usize,
    pub analysis_time: Duration,
    pub avg_time_per_module: Duration,
}

#[derive(Debug, Default)]
pub struct ObjectAnalysisMetrics {
    pub total_objects: usize,
    pub shared_objects: usize,
    pub transfer_checks: usize,
    pub guard_validations: usize,
    pub analysis_time: Duration,
}

#[derive(Debug, Default)]
pub struct CapabilityAnalysisMetrics {
    pub total_capabilities: usize,
    pub capability_checks: usize,
    pub permission_validations: usize,
    pub analysis_time: Duration,
}

pub struct SuiBenchmarkCollector {
    start_time: Instant,
    module_metrics: ModuleAnalysisMetrics,
    object_metrics: ObjectAnalysisMetrics,
    capability_metrics: CapabilityAnalysisMetrics,
    memory_samples: Vec<(String, usize)>,
}

impl SuiBenchmarkCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            module_metrics: ModuleAnalysisMetrics::default(),
            object_metrics: ObjectAnalysisMetrics::default(),
            capability_metrics: CapabilityAnalysisMetrics::default(),
            memory_samples: Vec::new(),
        }
    }

    pub fn record_module_analysis(&mut self, module_count: usize, function_count: usize, struct_count: usize, duration: Duration) {
        self.module_metrics.total_modules += module_count;
        self.module_metrics.total_functions += function_count;
        self.module_metrics.total_structs += struct_count;
        self.module_metrics.analysis_time += duration;
        
        if module_count > 0 {
            self.module_metrics.avg_time_per_module = 
                self.module_metrics.analysis_time / module_count as u32;
        }
    }

    pub fn record_object_analysis(&mut self, metrics: ObjectAnalysisMetrics) {
        self.object_metrics.total_objects += metrics.total_objects;
        self.object_metrics.shared_objects += metrics.shared_objects;
        self.object_metrics.transfer_checks += metrics.transfer_checks;
        self.object_metrics.guard_validations += metrics.guard_validations;
        self.object_metrics.analysis_time += metrics.analysis_time;
    }

    pub fn record_capability_analysis(&mut self, metrics: CapabilityAnalysisMetrics) {
        self.capability_metrics.total_capabilities += metrics.total_capabilities;
        self.capability_metrics.capability_checks += metrics.capability_checks;
        self.capability_metrics.permission_validations += metrics.permission_validations;
        self.capability_metrics.analysis_time += metrics.analysis_time;
    }

    pub fn take_memory_sample(&mut self, label: &str) {
        let usage = self.get_current_memory_usage();
        self.memory_samples.push((label.to_string(), usage));
    }

    pub fn finish(self) -> SuiAnalysisBenchmark {
        let total_time = self.start_time.elapsed();
        
        // Calculate memory stats
        let mut peak_usage = 0;
        let mut total_usage = 0;
        for (_, usage) in &self.memory_samples {
            peak_usage = peak_usage.max(*usage);
            total_usage += usage;
        }

        let memory_stats = MemoryStats {
            peak_usage,
            average_usage: if !self.memory_samples.is_empty() {
                total_usage / self.memory_samples.len()
            } else {
                0
            },
            samples: self.memory_samples,
        };

        SuiAnalysisBenchmark {
            module_analysis: self.module_metrics,
            object_analysis: self.object_metrics,
            capability_analysis: self.capability_metrics,
            memory_stats,
        }
    }

    pub fn report(&self) -> String {
        let mut output = String::new();
        output.push_str("Sui Move Analysis Performance Report\n");
        output.push_str("===================================\n\n");

        // Module Analysis Metrics
        output.push_str("Module Analysis:\n");
        output.push_str(&format!("  Total Modules: {}\n", self.module_metrics.total_modules));
        output.push_str(&format!("  Total Functions: {}\n", self.module_metrics.total_functions));
        output.push_str(&format!("  Total Structs: {}\n", self.module_metrics.total_structs));
        output.push_str(&format!("  Analysis Time: {:?}\n", self.module_metrics.analysis_time));
        output.push_str(&format!("  Avg Time/Module: {:?}\n\n", self.module_metrics.avg_time_per_module));

        // Object Analysis Metrics
        output.push_str("Object Analysis:\n");
        output.push_str(&format!("  Total Objects: {}\n", self.object_metrics.total_objects));
        output.push_str(&format!("  Shared Objects: {}\n", self.object_metrics.shared_objects));
        output.push_str(&format!("  Transfer Checks: {}\n", self.object_metrics.transfer_checks));
        output.push_str(&format!("  Guard Validations: {}\n", self.object_metrics.guard_validations));
        output.push_str(&format!("  Analysis Time: {:?}\n\n", self.object_metrics.analysis_time));

        // Capability Analysis Metrics
        output.push_str("Capability Analysis:\n");
        output.push_str(&format!("  Total Capabilities: {}\n", self.capability_metrics.total_capabilities));
        output.push_str(&format!("  Capability Checks: {}\n", self.capability_metrics.capability_checks));
        output.push_str(&format!("  Permission Validations: {}\n", self.capability_metrics.permission_validations));
        output.push_str(&format!("  Analysis Time: {:?}\n\n", self.capability_metrics.analysis_time));

        // Memory Usage
        output.push_str("Memory Usage:\n");
        for (label, usage) in &self.memory_samples {
            output.push_str(&format!("  {}: {} bytes\n", label, usage));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_sui_benchmark_collection() {
        let mut collector = SuiBenchmarkCollector::new();
        
        // Simulate module analysis
        collector.record_module_analysis(
            5, // modules
            20, // functions
            10, // structs
            Duration::from_millis(100),
        );

        // Simulate object analysis
        collector.record_object_analysis(ObjectAnalysisMetrics {
            total_objects: 15,
            shared_objects: 3,
            transfer_checks: 10,
            guard_validations: 5,
            analysis_time: Duration::from_millis(50),
        });

        // Simulate capability analysis
        collector.record_capability_analysis(CapabilityAnalysisMetrics {
            total_capabilities: 8,
            capability_checks: 12,
            permission_validations: 6,
            analysis_time: Duration::from_millis(30),
        });

        let result = collector.finish();
        
        assert_eq!(result.module_analysis.total_modules, 5);
        assert_eq!(result.object_analysis.total_objects, 15);
        assert_eq!(result.capability_analysis.total_capabilities, 8);
    }

    #[test]
    fn test_memory_sampling() {
        let mut collector = SuiBenchmarkCollector::new();
        
        collector.take_memory_sample("start");
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push(i);
        }
        collector.take_memory_sample("after_allocation");

        let result = collector.finish();
        assert!(!result.memory_stats.samples.is_empty());
        assert!(result.memory_stats.peak_usage > 0);
    }
} 