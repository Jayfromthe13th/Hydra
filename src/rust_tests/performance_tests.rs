use crate::analyzer::HydraAnalyzer;
use crate::analyzer::parser::Parser;
use std::time::Instant;
use super::{analyze_module, create_test_module, create_test_package, has_critical_issues};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_module_performance() {
        let large_module = create_test_module(10000);
        
        let start = Instant::now();
        let result = analyze_module(&large_module);
        let duration = start.elapsed();

        println!("Large module analysis took: {:?}", duration);
        assert!(duration.as_secs() < 5, "Analysis took too long");
        assert!(!has_critical_issues(&result));
    }

    #[test]
    fn test_multiple_modules_performance() {
        let modules = create_test_package(100);
        let start = Instant::now();
        
        for module in &modules {
            let _result = analyze_module(module);
        }
        
        let duration = start.elapsed();
        println!("Multiple modules analysis took: {:?}", duration);
        assert!(duration.as_secs() < 10, "Analysis took too long");
    }

    #[test]
    fn test_incremental_analysis_performance() {
        let mut analyzer = HydraAnalyzer::new();
        let source = include_str!("../test_data/module_with_changes.move");

        // First analysis
        let start = Instant::now();
        let module = Parser::parse_module(source).expect("Failed to parse module");
        let _result1 = analyzer.analyze_module(&module);
        let first_time = start.elapsed();

        // Second analysis (should be faster due to caching)
        let start = Instant::now();
        let _result2 = analyzer.analyze_module(&module);
        let second_time = start.elapsed();

        println!("First analysis: {:?}", first_time);
        println!("Second analysis: {:?}", second_time);

        assert!(
            second_time < first_time,
            "Incremental analysis not faster: {:?} vs {:?}",
            second_time,
            first_time
        );
    }
}