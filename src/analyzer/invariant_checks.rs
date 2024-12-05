use super::types::*;
use super::parser::Function;

pub struct InvariantChecker {
    violations: Vec<SafetyViolation>,
}

impl InvariantChecker {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    pub fn check_invariants(&mut self, _function: &Function) -> Vec<SafetyViolation> {
        self.violations.clear();
        std::mem::take(&mut self.violations)
    }
} 