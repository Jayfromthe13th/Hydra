use crate::analyzer::{HydraAnalyzer, AnalyzerConfig};
use crate::analyzer::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze_code(source: &str) -> AnalysisResult {
        let config = AnalyzerConfig {
            strict_mode: true,
            check_transfer_safety: true,
            check_capability_safety: true,
            check_shared_objects: true,
            max_module_size: 10000,
            ignore_tests: false,
        };

        let mut analyzer = HydraAnalyzer::with_config(config);
        match crate::analyzer::parser::Parser::parse_module(source) {
            Ok(module) => analyzer.analyze_module(&module),
            Err(_) => AnalysisResult::default(),
        }
    }

    #[test]
    fn test_basic_reference_leak() {
        let source = r#"
            module 0x1::test {
                struct Data has key { value: u64 }
                
                public fun leak_reference(data: &mut Data): &mut u64 {
                    &mut data.value
                }
            }
        "#;

        let result = analyze_code(source);
        assert!(!result.reference_leaks.is_empty());
        assert_eq!(result.reference_leaks[0].severity, Severity::Critical);
    }

    #[test]
    fn test_safe_reference_usage() {
        let source = r#"
            module 0x1::test {
                struct Data has key { value: u64 }
                
                public fun safe_function(data: &mut Data): u64 {
                    *&data.value
                }
            }
        "#;

        let result = analyze_code(source);
        assert!(result.reference_leaks.is_empty());
    }

    #[test]
    fn test_nested_reference_leak() {
        let source = r#"
            module 0x1::test {
                struct Inner has store { value: u64 }
                struct Outer has key { inner: Inner }
                
                public fun leak_nested(data: &mut Outer): &mut u64 {
                    &mut data.inner.value
                }
            }
        "#;

        let result = analyze_code(source);
        assert!(!result.reference_leaks.is_empty());
    }

    #[test]
    fn test_reference_in_loop() {
        let source = r#"
            module 0x1::test {
                struct Data has key { value: u64 }
                
                public fun loop_reference(data: &mut Data) {
                    let i = 0;
                    let ref = &mut data.value;
                    while (i < 10) {
                        *ref = i;
                        i = i + 1;
                    }
                }
            }
        "#;

        let result = analyze_code(source);
        assert!(result.reference_leaks.is_empty());
    }

    #[test]
    fn test_reference_escape_through_struct() {
        let source = r#"
            module 0x1::test {
                struct Data has key { value: u64 }
                struct Holder { ref: &mut u64 }
                
                public fun create_holder(data: &mut Data): Holder {
                    Holder { ref: &mut data.value }
                }
            }
        "#;

        let result = analyze_code(source);
        assert!(!result.reference_leaks.is_empty());
        assert_eq!(result.reference_leaks[0].severity, Severity::Critical);
    }

    #[test]
    fn test_multiple_mutable_references() {
        let source = r#"
            module 0x1::test {
                struct Data has key { value: u64 }
                
                public fun multiple_refs(data: &mut Data) {
                    let ref1 = &mut data.value;
                    let ref2 = &mut data.value;  // Should detect this as unsafe
                    *ref1 = 1;
                    *ref2 = 2;
                }
            }
        "#;

        let result = analyze_code(source);
        assert!(!result.safety_violations.is_empty());
    }
} 