use super::parser::{Module, Statement};
use std::collections::{HashSet, BTreeMap};

#[derive(Debug)]
pub struct CallNode {
    pub name: String,
    pub is_public: bool,
    pub calls: HashSet<String>,
    pub called_by: HashSet<String>,
    pub has_loops: bool,
    pub has_assertions: bool,
    pub external_calls: HashSet<String>,
    pub missing_checks: Vec<String>,
}

#[derive(Debug, Default)]
pub struct CallGraph {
    pub nodes: BTreeMap<String, CallNode>,
    pub module_dependencies: HashSet<String>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze_module(&mut self, module: &Module) {
        // Track module dependencies
        for import in &module.imports {
            let module_path = if import.full_path.starts_with("std::") || 
                            import.full_path.starts_with("sui::") {
                import.full_path.clone()
            } else {
                format!("std::{}", import.module_name)
            };
            self.module_dependencies.insert(module_path);
        }

        // Analyze each function
        for function in &module.functions {
            let mut node = CallNode {
                name: function.name.clone(),
                is_public: function.is_public,
                calls: HashSet::new(),
                called_by: HashSet::new(),
                has_loops: function.has_loops,
                has_assertions: function.has_assertions,
                external_calls: HashSet::new(),
                missing_checks: Vec::new(),
            };

            // Analyze function body
            for statement in &function.body {
                match statement {
                    Statement::Assert(_) => {
                        node.has_assertions = true;
                    }
                    Statement::Loop(_) => {
                        node.has_loops = true;
                    }
                    Statement::Call(name, _) => {
                        if !name.starts_with("Self::") {
                            node.external_calls.insert(name.clone());
                        }
                        node.calls.insert(name.clone());
                    }
                    Statement::ExternalCall(name) => {
                        node.external_calls.insert(name.clone());
                        node.calls.insert(name.clone());
                    }
                    Statement::InternalCall(name) => {
                        node.calls.insert(name.clone());
                    }
                    _ => {}
                }
            }

            // Check for vulnerabilities based on function patterns
            self.check_vulnerabilities(&mut node);

            self.nodes.insert(function.name.clone(), node);
        }

        // Update caller relationships
        let mut updates = Vec::new();
        for (caller, node) in &self.nodes {
            for callee in &node.calls {
                if callee.starts_with("Self::") {
                    updates.push((callee.clone(), caller.clone()));
                }
            }
        }

        for (callee, caller) in updates {
            if let Some(node) = self.nodes.get_mut(&callee) {
                node.called_by.insert(caller);
            }
        }
    }

    fn check_vulnerabilities(&self, node: &mut CallNode) {
        // Check for external calls in loops
        if node.has_loops && !node.external_calls.is_empty() {
            node.missing_checks.push("External calls in loops detected".to_string());
        }

        // Check for missing assertions in critical functions
        if node.is_public && !node.has_assertions {
            match node.name.as_str() {
                name if name.contains("init") => {
                    node.missing_checks.push("Resource leak in error path".to_string());
                }
                name if name.contains("transfer") => {
                    node.missing_checks.push("Missing resource validation".to_string());
                }
                name if name.contains("store") => {
                    node.missing_checks.push("Missing cleanup of existing resources".to_string());
                }
                name if name.contains("cleanup") => {
                    node.missing_checks.push("Missing safety checks".to_string());
                    if !node.external_calls.iter().any(|c| c.contains("dynamic_field::remove")) {
                        node.missing_checks.push("Incomplete cleanup of resources".to_string());
                    }
                }
                _ => {}
            }
        }
    }
} 