#[cfg(test)]
mod reference_tracking_tests;
#[cfg(test)]
mod safety_violation_tests;
#[cfg(test)]
mod real_module_tests;
#[cfg(test)]
mod performance_tests;

use crate::analyzer::{
    HydraAnalyzer,
    parser::Parser,
    types::*,
    AnalyzerConfig,
};

// Common test utilities
pub fn analyze_module(source: &str) -> AnalysisResult {
    let config = AnalyzerConfig {
        strict_mode: true,
        check_transfer_safety: true,
        check_capability_safety: true,
        check_shared_objects: true,
        max_module_size: 10000,
        ignore_tests: false,
    };

    let mut analyzer = HydraAnalyzer::with_config(config);
    
    match Parser::parse_module(source) {
        Ok(module) => analyzer.analyze_module(&module),
        Err(_) => AnalysisResult::default(),
    }
}

pub fn has_critical_issues(result: &AnalysisResult) -> bool {
    result.has_critical_issues()
}

pub fn count_safety_violations(result: &AnalysisResult) -> usize {
    result.safety_violations.len()
}

// Test data utilities
pub fn create_test_module(size: usize) -> String {
    let mut source = String::from(
        "module 0x1::test {\n    struct Data has key { value: u64 }\n"
    );

    for i in 0..size/100 {
        source.push_str(&format!(
            "    public fun test_func_{0}(data: &mut Data) {{ data.value = {0}; }}\n",
            i
        ));
    }

    source.push_str("}\n");
    source
}

pub fn create_test_package(module_count: usize) -> Vec<String> {
    (0..module_count).map(|i| {
        format!(
            "module 0x1::test_{} {{\n    struct Data has key {{ value: u64 }}\n}}\n",
            i
        )
    }).collect()
}

#[cfg(test)]
mod cli_tests {
    use crate::cli::Cli;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cli_file_handling() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.move");
        
        fs::write(&test_file, "module test {}").unwrap();
        
        let cli = Cli {
            path: test_file,
            verbose: true,
        };
        
        assert!(cli.path.exists());
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("non_existent.move");
        
        let cli = Cli {
            path: non_existent.clone(),
            verbose: false,
        };
        
        assert!(!cli.path.exists());
    }
} 