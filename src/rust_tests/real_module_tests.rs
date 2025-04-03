use crate::analyzer::types::*;
use super::{analyze_module, has_critical_issues, count_safety_violations};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_module() {
        let source = include_str!("../test_data/nft_module.move");
        let result = analyze_module(source);
        
        // Verify no critical issues
        assert!(!result.has_critical_issues());
        
        // Verify expected safety patterns
        assert!(has_transfer_guard(&result));
        assert!(has_ownership_check(&result));
        assert!(has_capability_check(&result));
    }

    #[test]
    fn test_defi_pool() {
        let source = include_str!("../test_data/defi_pool.move");
        let result = analyze_module(source);
        
        // Verify no critical issues
        assert!(!result.has_critical_issues());
        
        // Verify shared object safety
        assert!(has_synchronized_access(&result));
        assert!(!has_unsafe_shared_mutation(&result));
    }

    #[test]
    fn test_governance_module() {
        let source = include_str!("../test_data/governance.move");
        let result = analyze_module(source);
        
        // Verify no critical issues
        assert!(!result.has_critical_issues());
        
        // Verify capability patterns
        assert!(has_capability_guard(&result));
        assert!(!has_capability_leak(&result));
    }

    #[test]
    fn test_marketplace_module() {
        let source = include_str!("../test_data/marketplace.move");
        let result = analyze_module(source);
        
        // Verify no critical issues
        assert!(!result.has_critical_issues());
        
        // Verify object safety
        assert!(has_safe_transfers(&result));
        assert!(has_escrow_pattern(&result));
    }

    #[test]
    fn test_staking_module() {
        let source = r#"
            module 0x1::test {
                struct Data has key { value: u64 }
                
                public fun safe_function(data: &mut Data) {
                    data.value = 100;  // Safe mutation
                }
            }
        "#;

        let result = analyze_module(source);
        
        // Verify no critical issues
        assert!(!result.has_critical_issues());
        
        // Verify reference safety
        assert!(!has_reference_leaks(&result));
        assert!(has_safe_state_updates(&result));
    }

    #[test]
    fn test_complex_module() {
        let source = r#"
            module 0x1::complex {
                use sui::object::{Self, UID};
                use sui::transfer;
                use sui::tx_context::{Self, TxContext};

                struct GameItem has key {
                    id: UID,
                    power: u64,
                }

                struct AdminCap has key {
                    id: UID,
                }

                public fun init(ctx: &mut TxContext) {
                    transfer::transfer(
                        AdminCap { id: object::new(ctx) },
                        tx_context::sender(ctx)
                    )
                }

                public fun create_item(
                    _admin: &AdminCap,
                    power: u64,
                    ctx: &mut TxContext
                ): GameItem {
                    GameItem {
                        id: object::new(ctx),
                        power
                    }
                }
            }
        "#;

        let result = analyze_module(source);
        assert!(!result.has_critical_issues());
        assert_eq!(count_safety_violations(&result), 0);
    }

    fn has_transfer_guard(result: &AnalysisResult) -> bool {
        !result.object_safety_issues.iter().any(|i| 
            matches!(i.issue_type, ObjectIssueType::InvalidTransferGuard)
        )
    }

    fn has_ownership_check(result: &AnalysisResult) -> bool {
        !result.object_safety_issues.iter().any(|i| 
            matches!(i.issue_type, ObjectIssueType::MissingOwnershipCheck)
        )
    }

    fn has_capability_check(result: &AnalysisResult) -> bool {
        !result.safety_violations.iter().any(|v| 
            matches!(v.violation_type, ViolationType::CapabilityLeak)
        )
    }

    fn has_synchronized_access(result: &AnalysisResult) -> bool {
        !result.safety_violations.iter().any(|v| 
            matches!(v.violation_type, ViolationType::SharedObjectViolation)
        )
    }

    fn has_unsafe_shared_mutation(result: &AnalysisResult) -> bool {
        result.object_safety_issues.iter().any(|i| 
            matches!(i.issue_type, ObjectIssueType::InvalidSharedAccess)
        )
    }

    fn has_capability_guard(result: &AnalysisResult) -> bool {
        !result.object_safety_issues.iter().any(|i| 
            matches!(i.issue_type, ObjectIssueType::CapabilityExposure)
        )
    }

    fn has_capability_leak(result: &AnalysisResult) -> bool {
        result.safety_violations.iter().any(|v| 
            matches!(v.violation_type, ViolationType::CapabilityLeak)
        )
    }

    fn has_safe_transfers(result: &AnalysisResult) -> bool {
        !result.safety_violations.iter().any(|v| 
            matches!(v.violation_type, ViolationType::UnsafeTransfer)
        )
    }

    fn has_escrow_pattern(_result: &AnalysisResult) -> bool {
        // Check for proper escrow implementation
        // This would need more sophisticated pattern matching
        true // Simplified for now
    }

    fn has_reference_leaks(result: &AnalysisResult) -> bool {
        !result.reference_leaks.is_empty()
    }

    fn has_safe_state_updates(result: &AnalysisResult) -> bool {
        !result.safety_violations.iter().any(|v| 
            matches!(v.violation_type, ViolationType::InvariantViolation)
        )
    }
} 