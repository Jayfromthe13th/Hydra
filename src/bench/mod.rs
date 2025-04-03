use std::time::{Duration, Instant};
use std::collections::HashMap;
use crate::analyzer::types::*;

#[derive(Debug, Default)]
pub struct BenchmarkResults {
    pub total_time: Duration,
    pub module_times: HashMap<String, Duration>,
    pub phase_times: HashMap<String, Duration>,
    pub memory_stats: MemoryStats,
    pub analysis_stats: AnalysisStats,
}

#[derive(Debug, Default)]
pub struct MemoryStats {
    pub peak_usage: usize,
    pub average_usage: usize,
    pub samples: Vec<(String, usize)>,
}

pub struct Benchmark {
    start_time: Instant,
    checkpoints: Vec<(String, Instant)>,
    memory_samples: Vec<(String, usize)>,
    current_phase: Option<String>,
}

impl Benchmark {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            checkpoints: Vec::new(),
            memory_samples: Vec::new(),
            current_phase: None,
        }
    }

    pub fn start_phase(&mut self, name: &str) {
        if let Some(current) = &self.current_phase {
            self.checkpoints.push((current.clone(), Instant::now()));
        }
        self.current_phase = Some(name.to_string());
        self.take_memory_sample(name);
    }

    pub fn end_phase(&mut self) {
        if let Some(name) = &self.current_phase {
            self.checkpoints.push((name.clone(), Instant::now()));
            self.take_memory_sample(&format!("{}_end", name));
        }
        self.current_phase = None;
    }

    pub fn take_memory_sample(&mut self, label: &str) {
        let usage = self.get_current_memory_usage();
        self.memory_samples.push((label.to_string(), usage));
    }

    pub fn finish(self) -> BenchmarkResults {
        let total_time = self.start_time.elapsed();
        let mut phase_times = HashMap::new();
        
        // Calculate phase times
        let mut last_time = self.start_time;
        for (name, time) in &self.checkpoints {
            let duration = time.duration_since(last_time);
            phase_times.insert(name.clone(), duration);
            last_time = *time;
        }

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

        BenchmarkResults {
            total_time,
            module_times: HashMap::new(),
            phase_times,
            memory_stats,
            analysis_stats: AnalysisStats::default(),
        }
    }

    fn get_current_memory_usage(&self) -> usize {
        // This is a simplified implementation
        // In a real implementation, we would use platform-specific APIs
        // to get actual memory usage
        #[cfg(target_os = "linux")]
        {
            use std::fs::File;
            use std::io::Read;
            if let Ok(mut file) = File::open("/proc/self/statm") {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Some(pages) = contents.split_whitespace().next() {
                        if let Ok(pages) = pages.parse::<usize>() {
                            return pages * 4096; // Convert pages to bytes
                        }
                    }
                }
            }
        }
        0
    }
}

pub struct PerformanceMonitor {
    benchmarks: HashMap<String, Benchmark>,
    current_benchmark: Option<String>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            benchmarks: HashMap::new(),
            current_benchmark: None,
        }
    }

    pub fn start_benchmark(&mut self, name: &str) {
        let benchmark = Benchmark::new();
        self.benchmarks.insert(name.to_string(), benchmark);
        self.current_benchmark = Some(name.to_string());
    }

    pub fn start_phase(&mut self, name: &str) {
        if let Some(benchmark_name) = &self.current_benchmark {
            if let Some(benchmark) = self.benchmarks.get_mut(benchmark_name) {
                benchmark.start_phase(name);
            }
        }
    }

    pub fn end_phase(&mut self) {
        if let Some(benchmark_name) = &self.current_benchmark {
            if let Some(benchmark) = self.benchmarks.get_mut(benchmark_name) {
                benchmark.end_phase();
            }
        }
    }

    pub fn take_memory_sample(&mut self, label: &str) {
        if let Some(benchmark_name) = &self.current_benchmark {
            if let Some(benchmark) = self.benchmarks.get_mut(benchmark_name) {
                benchmark.take_memory_sample(label);
            }
        }
    }

    pub fn end_benchmark(&mut self) -> Option<BenchmarkResults> {
        if let Some(benchmark_name) = self.current_benchmark.take() {
            if let Some(benchmark) = self.benchmarks.remove(&benchmark_name) {
                return Some(benchmark.finish());
            }
        }
        None
    }

    pub fn report(&self) -> String {
        let mut output = String::new();
        output.push_str("Performance Report\n");
        output.push_str("=================\n\n");

        for (name, benchmark) in &self.benchmarks {
            output.push_str(&format!("Benchmark: {}\n", name));
            output.push_str("-----------------\n");
            
            // Add benchmark-specific metrics
            if let Some(current_phase) = &benchmark.current_phase {
                output.push_str(&format!("Current phase: {}\n", current_phase));
            }

            // Add memory stats
            output.push_str("\nMemory Usage:\n");
            for (label, usage) in &benchmark.memory_samples {
                output.push_str(&format!("  {}: {} bytes\n", label, usage));
            }

            output.push_str("\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_benchmark_phases() {
        let mut benchmark = Benchmark::new();
        
        benchmark.start_phase("phase1");
        thread::sleep(Duration::from_millis(10));
        benchmark.end_phase();

        benchmark.start_phase("phase2");
        thread::sleep(Duration::from_millis(10));
        benchmark.end_phase();

        let results = benchmark.finish();
        assert!(results.total_time >= Duration::from_millis(20));
        assert!(!results.phase_times.is_empty());
    }

    #[test]
    fn test_memory_sampling() {
        let mut benchmark = Benchmark::new();
        
        benchmark.take_memory_sample("start");
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push(i);
        }
        benchmark.take_memory_sample("after_allocation");

        let results = benchmark.finish();
        assert!(results.memory_stats.peak_usage > 0);
        assert_eq!(results.memory_stats.samples.len(), 2);
    }
} 