use super::parser::{Statement, Expression, Function};
use super::types::*;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum DosVectorType {
    ExternalCallInLoop,
    UnboundedVectorOperation,
    NestedLoopWithExternal,
    RecursiveExternalCall,
    MultipleExternalCalls,
    ConditionalExternalCall,
    IndirectRecursion,
    DynamicLoopBound,
    DeepNesting,
}

pub struct DosDetector {
    violations: Vec<SafetyViolation>,
    current_loops: Vec<LoopType>,
    external_calls: HashSet<String>,
    loop_depth: usize,
    external_calls_in_current_loop: usize,
}

#[derive(Debug, Clone)]
pub enum LoopType {
    While,
    For,
    VectorIteration,
}

impl LoopType {
    pub fn is_unbounded(&self) -> bool {
        matches!(self, LoopType::While | LoopType::VectorIteration)
    }

    pub fn from_expr(expr: &Expression) -> Option<Self> {
        match expr {
            Expression::Variable(name) if name.contains("while") => Some(LoopType::While),
            Expression::Variable(name) if name.contains("for") => Some(LoopType::For),
            Expression::Call(name, _) if name.contains("vector") => Some(LoopType::VectorIteration),
            _ => None,
        }
    }
}

impl DosDetector {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            current_loops: Vec::new(),
            external_calls: HashSet::new(),
            loop_depth: 0,
            external_calls_in_current_loop: 0,
        }
    }

    pub fn check_dos_vectors(&mut self, function: &Function) -> Vec<SafetyViolation> {
        self.violations.clear();
        self.current_loops.clear();
        self.external_calls.clear();
        self.loop_depth = 0;
        self.external_calls_in_current_loop = 0;

        for statement in &function.body {
            match statement {
                Statement::Loop(_) => {
                    self.loop_depth += 1;
                }
                Statement::Call(name, _) => {
                    if !name.starts_with("Self::") {
                        self.external_calls.insert(name.clone());
                        if self.loop_depth > 0 {
                            self.external_calls_in_current_loop += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        std::mem::take(&mut self.violations)
    }
} 