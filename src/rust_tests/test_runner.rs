use crate::analyzer::{HydraAnalyzer, AnalyzerConfig};
use crate::analyzer::types::*;
use std::path::PathBuf;
use std::fs;

pub struct TestRunner {
    analyzer: HydraAnalyzer,
    test_files: Vec<PathBuf>,
    results: Vec<TestResult>,
}

#[derive(Debug)]
pub struct TestResult {
    pub file: String,
    pub success: bool,
    pub leaks: Vec<ReferenceLeak>,
    pub violations: Vec<SafetyViolation>,
    pub error: Option<String>,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            analyzer: HydraAnalyzer::new(),
            test_files: Vec::new(),
            results: Vec::new(),
        }
    }

    pub fn add_test_file<P: Into<PathBuf>>(&mut self, path: P) {
        self.test_files.push(path.into());
    }

    pub fn run_tests(&mut self) -> Vec<TestResult> {
        for file in &self.test_files {
            let result = self.run_single_test(file);
            self.results.push(result);
        }
        self.results.clone()
    }

    fn run_single_test(&self, file: &PathBuf) -> TestResult {
        match fs::read_to_string(file) {
            Ok(source) => {
                match crate::analyzer::parser::Parser::parse_module(&source) {
                    Ok(module) => {
                        let result = self.analyzer.analyze_module(&module);
                        TestResult {
                            file: file.display().to_string(),
                            success: !result.has_critical_issues(),
                            leaks: result.reference_leaks,
                            violations: result.safety_violations,
                            error: None,
                        }
                    }
                    Err(e) => TestResult {
                        file: file.display().to_string(),
                        success: false,
                        leaks: Vec::new(),
                        violations: Vec::new(),
                        error: Some(format!("Parse error: {}", e)),
                    }
                }
            }
            Err(e) => TestResult {
                file: file.display().to_string(),
                success: false,
                leaks: Vec::new(),
                violations: Vec::new(),
                error: Some(format!("File error: {}", e)),
            }
        }
    }

    pub fn print_results(&self) {
        println!("\nTest Results");
        println!("============");
        
        let mut passed = 0;
        let mut failed = 0;

        for result in &self.results {
            if result.success {
                println!("✅ PASS: {}", result.file);
                passed += 1;
            } else {
                println!("❌ FAIL: {}", result.file);
                failed += 1;

                if let Some(error) = &result.error {
                    println!("   Error: {}", error);
                }

                for leak in &result.leaks {
                    println!("   Leak: {} at {:?}", leak.context, leak.location);
                }

                for violation in &result.violations {
                    println!("   Violation: {} at {:?}", violation.message, violation.location);
                }
            }
        }

        println!("\nSummary");
        println!("=======");
        println!("Total:  {}", self.results.len());
        println!("Passed: {}", passed);
        println!("Failed: {}", failed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_analysis() {
        let mut runner = TestRunner::new();
        
        // Add test files
        runner.add_test_file("src/tests/test_data/nft_module.move");
        runner.add_test_file("src/tests/test_data/defi_pool.move");
        
        // Run tests
        let results = runner.run_tests();
        
        // Print results
        runner.print_results();
        
        // Verify results
        assert!(results.iter().any(|r| r.success));
    }
} 