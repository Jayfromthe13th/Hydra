use super::types::*;
use super::parser::{Module, Function, Statement, Expression, Type};
use std::collections::{HashMap, HashSet};

pub struct ModuleBoundaryChecker {
    public_interfaces: HashMap<String, InterfaceInfo>,
    exposed_references: HashSet<ExposedReference>,
    current_module: Option<String>,
}

#[derive(Debug)]
struct InterfaceInfo {
    function_name: String,
    exposed_refs: HashSet<String>,
    unsafe_params: HashSet<String>,
    ref_returns: HashSet<String>,
    invariant_violations: Vec<InvariantViolation>,
}

#[derive(Debug)]
struct ExposedReference {
    var: String,
    exposure_point: ExposurePoint,
    location: Location,
    field: Option<FieldId>,
}

#[derive(Debug)]
enum ExposurePoint {
    PublicFunction(String),
    PublicReturn(String),
    ExternalCall(String),
    FieldAccess(FieldId),
}

#[derive(Debug)]
struct InvariantViolation {
    field: FieldId,
    violation_type: ViolationType,
    location: Location,
}

impl ModuleBoundaryChecker {
    pub fn new() -> Self {
        Self {
            public_interfaces: HashMap::new(),
            exposed_references: HashSet::new(),
            current_module: None,
        }
    }

    pub fn check_module(&mut self, module: &Module) -> Vec<SafetyViolation> {
        let mut violations = Vec::new();
        self.current_module = Some(module.name.clone());

        // Check public interfaces
        for function in &module.functions {
            if function.name.starts_with("public") {
                self.check_public_interface(function, &mut violations);
            }
        }

        // Check for exposed references
        self.check_exposed_references(&mut violations);

        // Check invariant preservation
        self.check_invariant_preservation(module, &mut violations);

        self.current_module = None;
        violations
    }

    fn check_public_interface(&mut self, function: &Function, violations: &mut Vec<SafetyViolation>) {
        let mut interface_info = InterfaceInfo {
            function_name: function.name.clone(),
            exposed_refs: HashSet::new(),
            unsafe_params: HashSet::new(),
            ref_returns: HashSet::new(),
            invariant_violations: Vec::new(),
        };

        // Check parameters
        for param in &function.parameters {
            if let Type::MutableReference(_) = param.param_type {
                interface_info.unsafe_params.insert(param.name.clone());
                violations.push(SafetyViolation {
                    location: Location {
                        file: self.current_module.clone().unwrap_or_default(),
                        line: 0,
                        column: 0,
                        context: format!("Public function {}", function.name),
                    },
                    violation_type: ViolationType::UnsafePublicInterface,
                    message: format!(
                        "Public function {} accepts mutable reference parameter {}",
                        function.name, param.name
                    ),
                    severity: Severity::High,
                });
            }
        }

        // Check return type
        if let Type::MutableReference(_) = function.return_type {
            violations.push(SafetyViolation {
                location: Location {
                    file: self.current_module.clone().unwrap_or_default(),
                    line: 0,
                    column: 0,
                    context: format!("Public function {}", function.name),
                },
                violation_type: ViolationType::UnsafePublicInterface,
                message: format!(
                    "Public function {} returns mutable reference",
                    function.name
                ),
                severity: Severity::Critical,
            });
        }

        // Track exposed references in function body
        for statement in &function.body {
            self.check_statement_exposure(statement, &mut interface_info, violations);
        }

        self.public_interfaces.insert(function.name.clone(), interface_info);
    }

    fn check_statement_exposure(
        &mut self,
        statement: &Statement,
        info: &mut InterfaceInfo,
        violations: &mut Vec<SafetyViolation>
    ) {
        match statement {
            Statement::Assignment(var, expr) => {
                self.check_assignment_exposure(var, expr, info, violations);
            }
            Statement::Return(expr) => {
                self.check_return_exposure(expr, info, violations);
            }
        }
    }

    fn check_assignment_exposure(
        &mut self,
        var: &str,
        expr: &Expression,
        info: &mut InterfaceInfo,
        violations: &mut Vec<SafetyViolation>
    ) {
        if let Some(field) = self.get_exposed_field(expr) {
            info.exposed_refs.insert(var.to_string());
            
            self.exposed_references.insert(ExposedReference {
                var: var.to_string(),
                exposure_point: ExposurePoint::FieldAccess(field.clone()),
                location: Location::default(),
                field: Some(field.clone()),
            });

            violations.push(SafetyViolation {
                location: Location::default(),
                violation_type: ViolationType::ReferenceEscape,
                message: format!("Reference to protected field exposed through assignment to {}", var),
                severity: Severity::High,
            });
        }
    }

    fn check_return_exposure(
        &mut self,
        expr: &Expression,
        info: &mut InterfaceInfo,
        violations: &mut Vec<SafetyViolation>
    ) {
        if let Some(field) = self.get_exposed_field(expr) {
            info.ref_returns.insert(field.field_name.clone());
            
            violations.push(SafetyViolation {
                location: Location::default(),
                violation_type: ViolationType::ReferenceEscape,
                message: "Reference to protected field exposed through return".to_string(),
                severity: Severity::Critical,
            });
        }
    }

    fn check_exposed_references(&self, violations: &mut Vec<SafetyViolation>) {
        for exposed in &self.exposed_references {
            match &exposed.exposure_point {
                ExposurePoint::PublicFunction(func) => {
                    violations.push(SafetyViolation {
                        location: exposed.location.clone(),
                        violation_type: ViolationType::UnsafePublicInterface,
                        message: format!(
                            "Reference exposed through public function {}",
                            func
                        ),
                        severity: Severity::High,
                    });
                }
                ExposurePoint::PublicReturn(func) => {
                    violations.push(SafetyViolation {
                        location: exposed.location.clone(),
                        violation_type: ViolationType::ReferenceEscape,
                        message: format!(
                            "Reference escapes through return in public function {}",
                            func
                        ),
                        severity: Severity::Critical,
                    });
                }
                ExposurePoint::ExternalCall(func) => {
                    violations.push(SafetyViolation {
                        location: exposed.location.clone(),
                        violation_type: ViolationType::UnsafePublicInterface,
                        message: format!(
                            "Reference exposed through external call to {}",
                            func
                        ),
                        severity: Severity::High,
                    });
                }
                ExposurePoint::FieldAccess(field) => {
                    violations.push(SafetyViolation {
                        location: exposed.location.clone(),
                        violation_type: ViolationType::InvariantViolation,
                        message: format!(
                            "Protected field {} exposed through reference",
                            field.field_name
                        ),
                        severity: Severity::High,
                    });
                }
            }
        }
    }

    fn check_invariant_preservation(&self, module: &Module, violations: &mut Vec<SafetyViolation>) {
        for function in &module.functions {
            if function.name.starts_with("public") {
                if let Some(interface) = self.public_interfaces.get(&function.name) {
                    for violation in &interface.invariant_violations {
                        violations.push(SafetyViolation {
                            location: violation.location.clone(),
                            violation_type: violation.violation_type.clone(),
                            message: format!(
                                "Invariant violation for field {} in public function {}",
                                violation.field.field_name, function.name
                            ),
                            severity: Severity::Critical,
                        });
                    }
                }
            }
        }
    }

    fn get_exposed_field(&self, expr: &Expression) -> Option<FieldId> {
        match expr {
            Expression::FieldAccess(_, field) => Some(FieldId {
                module_name: self.current_module.clone().unwrap_or_default(),
                struct_name: String::new(), // Would need proper context
                field_name: field.clone(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_interface_safety() {
        let source = r#"
            module 0x1::test {
                public fun unsafe_ref(x: &mut u64): &mut u64 {
                    x
                }
            }
        "#;

        let mut checker = ModuleBoundaryChecker::new();
        let module = crate::analyzer::parser::Parser::parse_module(source).unwrap();
        let violations = checker.check_module(&module);

        assert!(violations.iter().any(|v| matches!(
            v.violation_type,
            ViolationType::UnsafePublicInterface
        )));
    }
} 