use crate::analyzer::types::*;
use super::analyze_module;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsafe_transfer() {
        let source = r#"
            module 0x1::unsafe_transfer {
                struct Token has key { value: u64 }
                
                public fun unsafe_transfer(token: Token) {
                    transfer::transfer(token, @recipient);  // Missing recipient validation
                }
            }
        "#;

        let result = analyze_module(source);
        assert!(has_safety_violation(&result, ViolationType::UnsafeTransfer), 
            "Should detect unsafe transfer");
    }

    #[test]
    fn test_capability_leak() {
        let source = r#"
            module 0x1::unsafe_cap {
                struct AdminCap has key {}
                
                public fun unsafe_cap(cap: &mut AdminCap): &mut AdminCap {
                    cap  // Leaking capability reference
                }
            }
        "#;

        let result = analyze_module(source);
        assert!(has_safety_violation(&result, ViolationType::CapabilityLeak),
            "Should detect capability leak");
    }

    #[test]
    fn test_shared_object_violation() {
        let source = r#"
            module 0x1::unsafe_shared {
                struct SharedCounter has key { value: u64 }
                
                public fun unsafe_shared(counter: &mut SharedCounter) {
                    counter.value = counter.value + 1;  // Missing synchronization
                }
            }
        "#;

        let result = analyze_module(source);
        assert!(has_safety_violation(&result, ViolationType::SharedObjectViolation),
            "Should detect shared object violation");
    }

    #[test]
    fn test_invariant_violation() {
        let source = r#"
            module 0x1::unsafe_update {
                struct Data has key {
                    value: u64,
                    min_value: u64,
                }
                
                public fun unsafe_update(data: &mut Data, new_value: u64) {
                    data.value = new_value;  // Missing invariant check
                }
            }
        "#;

        let result = analyze_module(source);
        assert!(has_safety_violation(&result, ViolationType::InvariantViolation),
            "Should detect invariant violation");
    }

    #[test]
    fn test_unsafe_public_interface() {
        let source = r#"
            module 0x1::unsafe_interface {
                struct Data has key { value: u64 }
                
                public fun unsafe_interface(data: &mut Data): &mut u64 {
                    &mut data.value  // Exposing mutable reference
                }
            }
        "#;

        let result = analyze_module(source);
        assert!(has_safety_violation(&result, ViolationType::UnsafePublicInterface),
            "Should detect unsafe public interface");
    }

    #[test]
    fn test_multiple_violations() {
        let source = r#"
            module 0x1::unsafe_multi {
                struct Token has key { value: u64 }
                struct AdminCap has key {}
                
                public fun unsafe_multi(token: Token, cap: &mut AdminCap): &mut AdminCap {
                    transfer::transfer(token, @recipient);  // Unsafe transfer
                    cap  // Capability leak
                }
            }
        "#;

        let result = analyze_module(source);
        let violations = count_unique_violations(&result);
        assert_eq!(violations, 2, "Should detect both unsafe transfer and capability leak");
        assert!(has_safety_violation(&result, ViolationType::UnsafeTransfer),
            "Should detect unsafe transfer");
        assert!(has_safety_violation(&result, ViolationType::CapabilityLeak),
            "Should detect capability leak");
    }

    #[test]
    fn test_safe_patterns() {
        let source = r#"
            module 0x1::test {
                struct Token has key { value: u64 }
                
                public fun safe_patterns(token: Token, recipient: address) {
                    assert!(is_valid_recipient(recipient), ERROR_INVALID_RECIPIENT);
                    transfer::transfer(token, recipient);
                }
            }
        "#;

        let result = analyze_module(source);
        assert!(!has_safety_violations(&result),
            "Should not detect any violations in safe code");
    }

    fn count_unique_violations(result: &AnalysisResult) -> usize {
        use std::collections::HashSet;
        result.safety_violations.iter()
            .map(|v| &v.violation_type)
            .collect::<HashSet<_>>()
            .len()
    }

    fn has_safety_violation(result: &AnalysisResult, violation_type: ViolationType) -> bool {
        result.safety_violations.iter()
            .any(|v| v.violation_type == violation_type)
    }

    fn has_safety_violations(result: &AnalysisResult) -> bool {
        !result.safety_violations.is_empty()
    }

    fn count_safety_violations(result: &AnalysisResult) -> usize {
        result.safety_violations.len()
    }
} 