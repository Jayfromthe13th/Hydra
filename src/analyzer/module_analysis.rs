use std::collections::{HashMap, HashSet, BTreeMap};
use crate::analyzer::types::*;
use crate::analyzer::parser::{Module, Statement};

#[derive(Debug, Clone)]
pub struct ModuleDependency {
    pub name: String,
    pub is_direct: bool,
    pub path: Vec<String>,
}

#[derive(Debug)]
pub struct ModuleAnalyzer {
    dependency_graph: BTreeMap<String, HashSet<ModuleDependency>>,
    trusted_boundaries: HashMap<String, bool>,
    cross_module_calls: HashMap<String, HashSet<String>>,
    module_isolation: HashMap<String, bool>,
}

impl ModuleAnalyzer {
    pub fn new() -> Self {
        Self {
            dependency_graph: BTreeMap::new(),
            trusted_boundaries: HashMap::new(),
            cross_module_calls: HashMap::new(),
            module_isolation: HashMap::new(),
        }
    }

    pub fn analyze_module(&mut self, module: &Module) -> Result<Vec<SafetyViolation>, String> {
        let mut violations = Vec::new();

        // Build dependency graph
        self.analyze_dependencies(module);

        // Track cross-module calls
        self.analyze_cross_module_calls(module);

        // Verify trusted boundaries
        if let Some(boundary_violations) = self.verify_trusted_boundaries(module) {
            violations.extend(boundary_violations);
        }

        // Verify module isolation
        if let Some(isolation_violations) = self.verify_module_isolation(module) {
            violations.extend(isolation_violations);
        }

        Ok(violations)
    }

    fn analyze_dependencies(&mut self, module: &Module) {
        let mut deps = HashSet::new();

        // Direct dependencies from imports
        for import in &module.imports {
            deps.insert(ModuleDependency {
                name: import.module_name.clone(),
                is_direct: true,
                path: vec![module.name.clone(), import.module_name.clone()],
            });
        }

        // Indirect dependencies from function calls
        for function in &module.functions {
            for statement in &function.body {
                if let Statement::Call(name, _) = statement {
                    if let Some(module_name) = name.split("::").next() {
                        if module_name != "Self" {
                            deps.insert(ModuleDependency {
                                name: module_name.to_string(),
                                is_direct: false,
                                path: vec![module.name.clone(), module_name.to_string()],
                            });
                        }
                    }
                }
            }
        }

        self.dependency_graph.insert(module.name.clone(), deps);
    }

    fn analyze_cross_module_calls(&mut self, module: &Module) {
        let mut calls = HashSet::new();

        for function in &module.functions {
            for statement in &function.body {
                if let Statement::Call(name, _) = statement {
                    if !name.starts_with("Self::") {
                        calls.insert(name.clone());
                    }
                }
            }
        }

        self.cross_module_calls.insert(module.name.clone(), calls);
    }

    fn verify_trusted_boundaries(&self, module: &Module) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();
        let is_trusted = self.trusted_boundaries.get(&module.name).copied().unwrap_or(false);

        if !is_trusted {
            // Check calls to trusted modules
            if let Some(calls) = self.cross_module_calls.get(&module.name) {
                for call in calls {
                    if let Some(called_module) = call.split("::").next() {
                        if self.trusted_boundaries.get(called_module).copied().unwrap_or(false) {
                            violations.push(SafetyViolation {
                                location: Location::default(),
                                violation_type: ViolationType::UnauthorizedAccess,
                                message: format!("Untrusted module {} calling trusted module {}", 
                                    module.name, called_module),
                                severity: Severity::High,
                                context: Some(ViolationContext {
                                    affected_functions: vec![],
                                    related_types: vec![],
                                    suggested_fixes: vec!["Remove direct call to trusted module".to_string()],
                                    whitepaper_reference: Some("Section 4.4: Module Isolation".to_string()),
                                }),
                            });
                        }
                    }
                }
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn verify_module_isolation(&self, module: &Module) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();

        // Check for circular dependencies
        if let Some(deps) = self.dependency_graph.get(&module.name) {
            let mut visited = HashSet::new();
            visited.insert(module.name.clone());

            for dep in deps {
                if self.has_circular_dependency(&dep.name, &mut visited) {
                    violations.push(SafetyViolation {
                        location: Location::default(),
                        violation_type: ViolationType::ResourceSafetyViolation,
                        message: format!("Circular dependency detected: {}", 
                            visited.iter().collect::<Vec<_>>().join(" -> ")),
                        severity: Severity::High,
                        context: Some(ViolationContext {
                            affected_functions: vec![],
                            related_types: vec![],
                            suggested_fixes: vec!["Break circular dependency".to_string()],
                            whitepaper_reference: Some("Section 4.4: Module Isolation".to_string()),
                        }),
                    });
                }
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn has_circular_dependency(&self, module_name: &str, visited: &mut HashSet<String>) -> bool {
        if !visited.insert(module_name.to_string()) {
            return true;
        }

        if let Some(deps) = self.dependency_graph.get(module_name) {
            for dep in deps {
                if self.has_circular_dependency(&dep.name, visited) {
                    return true;
                }
            }
        }

        visited.remove(module_name);
        false
    }

    pub fn mark_trusted_module(&mut self, module_name: String) {
        self.trusted_boundaries.insert(module_name, true);
    }

    pub fn get_module_dependencies(&self, module_name: &str) -> Option<&HashSet<ModuleDependency>> {
        self.dependency_graph.get(module_name)
    }

    pub fn get_cross_module_calls(&self, module_name: &str) -> Option<&HashSet<String>> {
        self.cross_module_calls.get(module_name)
    }
} 