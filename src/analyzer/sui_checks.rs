use super::types::*;
use super::parser::{Function, Statement, Expression};

#[allow(dead_code)]
pub struct SuiSafetyChecker {
    violations: Vec<SafetyViolation>,
}

impl SuiSafetyChecker {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    pub fn check_transfer_safety(&mut self, function: &Function) -> Vec<SafetyViolation> {
        self.violations.clear();

        for statement in &function.body {
            if let Statement::Assignment(_, expr) = statement {
                if self.is_transfer_call(expr) && !self.has_transfer_guard(expr) {
                    self.violations.push(SafetyViolation {
                        location: Location::default(),
                        violation_type: ViolationType::UnsafeTransfer,
                        message: format!("Function {} performs transfer without validation", function.name),
                        severity: Severity::Critical,
                        context: Some(ViolationContext {
                            affected_functions: vec![function.name.clone()],
                            related_types: vec![],
                            suggested_fixes: vec!["Add transfer validation".to_string()],
                            whitepaper_reference: Some("Section 4.4: Object Safety".to_string()),
                        }),
                    });
                }
            }
        }

        std::mem::take(&mut self.violations)
    }

    pub fn check_shared_object_safety(&mut self, function: &Function) -> Vec<SafetyViolation> {
        self.violations.clear();

        for statement in &function.body {
            if let Statement::Assignment(_, expr) = statement {
                if self.is_shared_object_access(expr) && !self.has_synchronization(expr) {
                    self.violations.push(SafetyViolation {
                        location: Location::default(),
                        violation_type: ViolationType::SharedObjectViolation,
                        message: format!("Function {} accesses shared object without synchronization", function.name),
                        severity: Severity::Critical,
                        context: Some(ViolationContext {
                            affected_functions: vec![function.name.clone()],
                            related_types: vec![],
                            suggested_fixes: vec!["Add consensus synchronization".to_string()],
                            whitepaper_reference: Some("Section 4.4: Shared Objects".to_string()),
                        }),
                    });
                }
            }
        }

        std::mem::take(&mut self.violations)
    }

    fn is_transfer_call(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FieldAccess(_, field) => {
                field.contains("transfer") || 
                field.contains("move_to") ||
                field.contains("send")
            }
            _ => false,
        }
    }

    fn has_transfer_guard(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FieldAccess(_, field) => {
                field.contains("assert") && 
                (field.contains("owner") || field.contains("auth"))
            }
            _ => false,
        }
    }

    fn is_shared_object_access(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FieldAccess(_, field) => {
                field.contains("shared") || 
                field.contains("global") ||
                field.contains("state")
            }
            _ => false,
        }
    }

    fn has_synchronization(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FieldAccess(_, field) => {
                field.contains("consensus") || 
                field.contains("synchronized") ||
                field.contains("lock")
            }
            _ => false,
        }
    }
} 