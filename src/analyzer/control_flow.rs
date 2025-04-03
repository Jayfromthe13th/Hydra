use super::parser::{Statement, Expression};
use super::path_analysis::ReferenceState;
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
pub struct ControlFlowAnalyzer {
    current_block: usize,
    blocks: Vec<Block>,
    reference_states: HashMap<String, ReferenceState>,
    loop_count: usize,
    external_calls: HashSet<String>,
    variables: HashSet<String>,
    fields: HashSet<String>,
    function_calls: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub successors: Vec<usize>,
    pub predecessors: Vec<usize>,
}

impl ControlFlowAnalyzer {
    pub fn new() -> Self {
        Self {
            current_block: 0,
            blocks: Vec::new(),
            reference_states: HashMap::new(),
            loop_count: 0,
            external_calls: HashSet::new(),
            variables: HashSet::new(),
            fields: HashSet::new(),
            function_calls: HashSet::new(),
        }
    }

    #[allow(dead_code)]
    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Assignment(_, expr) => {
                self.analyze_expression(expr);
            }
            Statement::Return(expr) => {
                self.analyze_expression(expr);
            }
            Statement::Loop(expr) => {
                self.analyze_expression(expr);
                self.loop_count += 1;
            }
            Statement::Call(name, args) => {
                for arg in args {
                    self.analyze_expression(arg);
                }
                if !name.starts_with("Self::") {
                    self.external_calls.insert(name.clone());
                }
            }
            Statement::Assert(expr) => {
                self.analyze_expression(expr);
            }
            Statement::ExternalCall(name) => {
                self.external_calls.insert(name.clone());
            }
            Statement::InternalCall(_) => {
                // Internal calls are tracked separately
            }
            Statement::BorrowField(_) | Statement::BorrowGlobal(_) | Statement::BorrowLocal(_) => {
                // Handle borrow statements
            }
        }
    }

    #[allow(dead_code)]
    fn analyze_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Variable(name) => {
                self.variables.insert(name.clone());
            }
            Expression::FieldAccess(base, field) => {
                self.analyze_expression(base);
                self.fields.insert(field.clone());
            }
            Expression::Call(name, args) => {
                for arg in args {
                    self.analyze_expression(arg);
                }
                self.function_calls.insert(name.clone());
            }
            Expression::Value(_) => {
                // Simple values don't need tracking
            }
        }
    }
} 