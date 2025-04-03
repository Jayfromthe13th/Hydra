use crate::analyzer::types::*;
use super::analyze_module;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_reference_tracking() {
        let source = r#"
            module 0x1::test {
                struct Data has key { value: u64 }
                
                public fun unsafe_interface(data: &mut Data): &mut u64 {
                    &mut data.value  // Should detect reference leak
                }
            }
        "#;

        let result = analyze_module(source);
        assert!(has_reference_leak(&result), "Should detect reference leak");
    }

    fn has_reference_leak(result: &AnalysisResult) -> bool {
        !result.reference_leaks.is_empty()
    }
} 