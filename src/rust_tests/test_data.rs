use crate::analyzer::types::*;
use std::path::PathBuf;

pub struct TestModule {
    pub name: String,
    pub source: String,
    pub path: PathBuf,
    pub expected_leaks: Vec<ReferenceLeak>,
    pub expected_violations: Vec<SafetyViolation>,
}

impl TestModule {
    pub fn new(name: &str, source: &str) -> Self {
        Self {
            name: name.to_string(),
            source: source.to_string(),
            path: PathBuf::from(format!("test_data/{}.move", name)),
            expected_leaks: Vec::new(),
            expected_violations: Vec::new(),
        }
    }

    pub fn with_leak(mut self, leak: ReferenceLeak) -> Self {
        self.expected_leaks.push(leak);
        self
    }

    pub fn with_violation(mut self, violation: SafetyViolation) -> Self {
        self.expected_violations.push(violation);
        self
    }
}

pub fn create_test_modules() -> Vec<TestModule> {
    vec![
        // Basic reference safety tests
        create_reference_test_module(),
        
        // Object safety tests
        create_object_test_module(),
        
        // Capability tests
        create_capability_test_module(),
        
        // Real-world patterns
        create_defi_test_module(),
        create_nft_test_module(),
    ]
}

fn create_reference_test_module() -> TestModule {
    TestModule::new(
        "reference_safety",
        r#"
        module 0x1::test {
            struct Data has key { value: u64 }
            
            public fun unsafe_ref(data: &mut Data): &mut u64 {
                &mut data.value
            }

            public fun safe_ref(data: &mut Data): u64 {
                *&data.value
            }
        }
        "#,
    ).with_leak(ReferenceLeak {
        location: Location {
            file: "reference_safety.move".to_string(),
            line: 5,
            column: 16,
            context: "Function unsafe_ref".to_string(),
        },
        leaked_field: FieldId {
            module_name: "0x1::test".to_string(),
            struct_name: "Data".to_string(),
            field_name: "value".to_string(),
        },
        context: "Mutable reference escapes through return".to_string(),
        severity: Severity::Critical,
    })
}

fn create_object_test_module() -> TestModule {
    TestModule::new(
        "object_safety",
        r#"
        module 0x1::test {
            struct Token has key {
                id: ID,
                value: u64,
            }
            
            public fun unsafe_transfer(token: Token) {
                transfer::transfer(token, @recipient);
            }

            public fun safe_transfer(token: Token, recipient: address) {
                assert!(is_valid_recipient(recipient), ERROR_INVALID_RECIPIENT);
                transfer::transfer(token, recipient);
            }
        }
        "#,
    ).with_violation(SafetyViolation {
        location: Location {
            file: "object_safety.move".to_string(),
            line: 8,
            column: 16,
            context: "Function unsafe_transfer".to_string(),
        },
        violation_type: ViolationType::UnsafeTransfer,
        message: "Transfer without recipient validation".to_string(),
        severity: Severity::High,
    })
}

fn create_capability_test_module() -> TestModule {
    TestModule::new(
        "capability_safety",
        r#"
        module 0x1::test {
            struct AdminCap has key {}
            
            public fun unsafe_cap_usage(cap: &mut AdminCap): &mut AdminCap {
                cap
            }

            public(friend) fun safe_cap_usage(cap: &AdminCap) {
                // Use capability internally
            }
        }
        "#,
    ).with_violation(SafetyViolation {
        location: Location {
            file: "capability_safety.move".to_string(),
            line: 5,
            column: 16,
            context: "Function unsafe_cap_usage".to_string(),
        },
        violation_type: ViolationType::CapabilityLeak,
        message: "Capability reference exposed through return".to_string(),
        severity: Severity::Critical,
    })
}

fn create_defi_test_module() -> TestModule {
    TestModule::new(
        "defi_pool",
        r#"
        module 0x1::pool {
            struct Pool has key {
                id: ID,
                balance: Balance,
                shares: Table<address, u64>,
            }
            
            public fun deposit(pool: &mut Pool, amount: u64) {
                // Implementation
            }

            public fun withdraw(pool: &mut Pool, shares: u64): u64 {
                // Implementation
            }
        }
        "#,
    )
}

fn create_nft_test_module() -> TestModule {
    TestModule::new(
        "nft_marketplace",
        r#"
        module 0x1::marketplace {
            struct Listing has key {
                id: ID,
                token: Token,
                price: u64,
                seller: address,
            }
            
            public fun list_token(token: Token, price: u64) {
                // Implementation
            }

            public fun buy_token(listing: &mut Listing, payment: Coin<SUI>) {
                // Implementation
            }
        }
        "#,
    )
}

// Helper functions for creating test data
pub fn create_test_function(name: &str, params: &[(&str, &str)], body: &str) -> String {
    let mut func = format!("public fun {}(", name);
    
    for (i, (param_name, param_type)) in params.iter().enumerate() {
        if i > 0 {
            func.push_str(", ");
        }
        func.push_str(&format!("{}: {}", param_name, param_type));
    }
    
    func.push_str(") {\n");
    func.push_str(body);
    func.push_str("\n}");
    
    func
}

pub fn create_test_struct(name: &str, fields: &[(&str, &str)]) -> String {
    let mut strct = format!("struct {} has key {{\n", name);
    
    for (field_name, field_type) in fields {
        strct.push_str(&format!("    {}: {},\n", field_name, field_type));
    }
    
    strct.push_str("}");
    strct
} 