use super::parser::{Statement, Expression, Function};
use super::types::*;
use super::path_analysis::ReferenceState;
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
pub struct ReferenceFlowAnalyzer {
    references: HashMap<String, ReferenceState>,
    #[allow(dead_code)]
    active_borrows: HashSet<String>,
}

impl ReferenceFlowAnalyzer {
    pub fn new() -> Self {
        Self {
            references: HashMap::new(),
            active_borrows: HashSet::new(),
        }
    }

    pub fn analyze_function(&mut self, function: &Function) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        for statement in &function.body {
            self.analyze_statement(statement, &mut leaks);
        }
        
        leaks
    }

    fn analyze_statement(&mut self, statement: &Statement, leaks: &mut Vec<ReferenceLeak>) {
        match statement {
            Statement::Assignment(_, expr) => {
                self.analyze_expression(expr, leaks);
            }
            Statement::Return(expr) => {
                self.analyze_expression(expr, leaks);
            }
            Statement::Loop(expr) => {
                self.analyze_expression(expr, leaks);
            }
            Statement::Call(name, args) => {
                for arg in args {
                    self.analyze_expression(arg, leaks);
                }
                self.track_function_call(name);
            }
            Statement::Assert(expr) => {
                self.analyze_expression(expr, leaks);
            }
            Statement::ExternalCall(_) | Statement::InternalCall(_) => {}
            Statement::BorrowField(_) | Statement::BorrowGlobal(_) | Statement::BorrowLocal(_) => {
                // Handle borrow statements
            },
        }
    }

    fn analyze_expression(&mut self, expr: &Expression, leaks: &mut Vec<ReferenceLeak>) {
        match expr {
            Expression::Variable(name) => {
                if self.is_reference(name) {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!("Reference {} may leak", name),
                        severity: Severity::High,
                    });
                }
            }
            Expression::FieldAccess(base, field) => {
                self.analyze_expression(base, leaks);
                self.track_field_access(field);
            }
            Expression::Call(name, args) => {
                for arg in args {
                    self.analyze_expression(arg, leaks);
                }
                self.track_function_call(name);
            }
            Expression::Value(_) => {}
        }
    }

    pub fn is_reference(&self, name: &str) -> bool {
        self.references.contains_key(name)
    }

    #[allow(dead_code)]
    fn track_reference(&mut self, name: &str) {
        self.references.insert(name.to_string(), ReferenceState::Valid);
    }

    fn track_field_access(&mut self, field: &str) {
        if !self.references.contains_key(field) {
            self.references.insert(field.to_string(), ReferenceState::Valid);
        }
    }

    fn track_function_call(&mut self, _name: &str) {
        // Track function calls that might affect references
    }
} 