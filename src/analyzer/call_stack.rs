use super::parser::{Module, Statement, Expression};
use super::types::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct CallStackViolation {
    pub function_name: String,
    pub call_path: Vec<String>,
    pub max_depth: usize,
    pub severity: Severity,
    pub message: String,
}

pub struct CallStackAnalyzer {
    max_depth: usize,
    current_path: Vec<String>,
    visited: HashSet<String>,
    call_graph: HashMap<String, Rc<Vec<String>>>,
}

impl CallStackAnalyzer {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            current_path: Vec::new(),
            visited: HashSet::new(),
            call_graph: HashMap::new(),
        }
    }

    pub fn analyze_call_stack(&mut self, module: &Module) -> Vec<CallStackViolation> {
        let mut violations = Vec::new();
        self.build_call_graph(module);

        for function in &module.functions {
            self.current_path.clear();
            self.visited.clear();
            
            self.check_call_depth(
                &function.name,
                &mut violations,
            );
        }

        violations
    }

    fn build_call_graph(&mut self, module: &Module) {
        for function in &module.functions {
            let mut calls = Vec::new();
            
            for statement in &function.body {
                if let Statement::Assignment(_, expr) = statement {
                    if let Some(called_func) = self.extract_function_call(expr) {
                        calls.push(called_func);
                    }
                }
            }
            
            self.call_graph.insert(function.name.clone(), Rc::new(calls));
        }
    }

    fn check_call_depth(
        &mut self,
        function_name: &str,
        violations: &mut Vec<CallStackViolation>,
    ) {
        if self.current_path.len() >= self.max_depth {
            violations.push(CallStackViolation {
                function_name: function_name.to_string(),
                call_path: self.current_path.clone(),
                max_depth: self.max_depth,
                severity: Severity::High,
                message: format!(
                    "Call stack depth exceeds maximum of {} in function {}",
                    self.max_depth, function_name
                ),
            });
            return;
        }

        if self.visited.contains(function_name) {
            violations.push(CallStackViolation {
                function_name: function_name.to_string(),
                call_path: self.current_path.clone(),
                max_depth: self.max_depth,
                severity: Severity::High,
                message: format!("Recursive call detected in function {}", function_name),
            });
            return;
        }

        self.visited.insert(function_name.to_string());
        self.current_path.push(function_name.to_string());

        if let Some(calls) = self.call_graph.get(function_name).cloned() {
            for called_func in calls.iter() {
                self.check_call_depth(called_func, violations);
            }
        }

        self.current_path.pop();
        self.visited.remove(function_name);
    }

    fn extract_function_call(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::FieldAccess(_, field) => Some(field.clone()),
            _ => None,
        }
    }
} 