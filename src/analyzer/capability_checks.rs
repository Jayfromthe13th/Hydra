use super::parser::{Function, Statement};
use super::types::*;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct CapabilityChecker {
    #[allow(dead_code)]
    capabilities: HashMap<String, Vec<String>>,
    violations: Vec<SafetyViolation>,
    #[allow(dead_code)]
    current_module: Option<String>,
}

impl CapabilityChecker {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            violations: Vec::new(),
            current_module: None,
        }
    }

    pub fn check_capability_safety(&mut self, function: &Function) -> Vec<SafetyViolation> {
        self.violations.clear();
        
        for statement in &function.body {
            if let Statement::Call(name, _) = statement {
                if name.contains("transfer") || name.contains("modify") {
                    self.violations.push(SafetyViolation {
                        location: Location::default(),
                        violation_type: ViolationType::CapabilityLeak,
                        message: "Missing capability check".to_string(),
                        severity: Severity::High,
                        context: Some(ViolationContext {
                            affected_functions: vec![function.name.clone()],
                            related_types: vec![],
                            suggested_fixes: vec!["Add capability verification".to_string()],
                            whitepaper_reference: Some("Section 2.1.1: Capability Safety".to_string()),
                        }),
                    });
                }
            }
        }
        
        std::mem::take(&mut self.violations)
    }
} 