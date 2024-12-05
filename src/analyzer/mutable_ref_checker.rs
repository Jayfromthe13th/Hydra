use super::types::*;
use super::parser::{Module, Function, Statement, Expression, Type};
use std::collections::{HashMap, HashSet};

pub struct MutableRefChecker {
    mutable_refs: HashMap<String, MutableRefInfo>,
    tracked_fields: HashSet<FieldId>,
    current_module: Option<String>,
}

#[derive(Debug)]
struct MutableRefInfo {
    definition: Location,
    current_state: RefState,
    access_points: Vec<AccessPoint>,
    escapes: bool,
}

#[derive(Debug)]
enum RefState {
    Valid,
    Escaped {
        through: EscapePoint,
        location: Location,
    },
    Invalid {
        reason: String,
        location: Location,
    },
}

#[derive(Debug)]
struct AccessPoint {
    location: Location,
    kind: AccessKind,
    context: String,
}

#[derive(Debug)]
enum AccessKind {
    Read,
    Write,
    Return,
    FieldAccess(FieldId),
    MethodCall(String),
}

#[derive(Debug)]
enum EscapePoint {
    Return,
    PublicInterface,
    FieldStore,
    GlobalStorage,
}

impl MutableRefChecker {
    pub fn new() -> Self {
        Self {
            mutable_refs: HashMap::new(),
            tracked_fields: HashSet::new(),
            current_module: None,
        }
    }

    pub fn check_module(&mut self, module: &Module) -> Vec<SafetyViolation> {
        let mut violations = Vec::new();
        self.current_module = Some(module.name.clone());

        // Track mutable references in each function
        for function in &module.functions {
            violations.extend(self.check_function(function));
        }

        self.current_module = None;
        violations
    }

    fn check_function(&mut self, function: &Function) -> Vec<SafetyViolation> {
        let mut violations = Vec::new();

        // Track mutable reference parameters
        self.track_mutable_parameters(function, &mut violations);

        // Analyze function body
        for statement in &function.body {
            self.check_statement(statement, function, &mut violations);
        }

        // Check for reference escapes
        self.check_reference_escapes(function, &mut violations);

        violations
    }

    fn track_mutable_parameters(&mut self, function: &Function, violations: &mut Vec<SafetyViolation>) {
        for param in &function.parameters {
            if let Type::MutableReference(_) = param.param_type {
                // Track mutable reference parameter
                self.mutable_refs.insert(param.name.clone(), MutableRefInfo {
                    definition: Location {
                        file: self.current_module.clone().unwrap_or_default(),
                        line: 0,
                        column: 0,
                        context: format!("Parameter in function {}", function.name),
                    },
                    current_state: RefState::Valid,
                    access_points: Vec::new(),
                    escapes: false,
                });

                // Check if public function exposes mutable reference
                if function.name.starts_with("public") {
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
        }
    }

    fn check_statement(&mut self, statement: &Statement, function: &Function, violations: &mut Vec<SafetyViolation>) {
        match statement {
            Statement::Assignment(var, expr) => {
                self.check_assignment(var, expr, violations);
            }
            Statement::Return(expr) => {
                self.check_return(expr, function, violations);
            }
        }
    }

    fn check_assignment(&mut self, var: &str, expr: &Expression, violations: &mut Vec<SafetyViolation>) {
        // Check for mutable reference assignments
        if let Some(field_id) = self.get_assigned_field(expr) {
            if self.tracked_fields.contains(&field_id) {
                violations.push(SafetyViolation {
                    location: Location::default(),
                    violation_type: ViolationType::InvariantViolation,
                    message: format!(
                        "Assignment to invariant-protected field {} through reference",
                        field_id.field_name
                    ),
                    severity: Severity::Critical,
                });
            }
        }

        // Track reference state
        if let Some(ref_info) = self.mutable_refs.get_mut(var) {
            ref_info.access_points.push(AccessPoint {
                location: Location::default(),
                kind: AccessKind::Write,
                context: "Assignment".to_string(),
            });
        }
    }

    fn check_return(&mut self, expr: &Expression, function: &Function, violations: &mut Vec<SafetyViolation>) {
        // Check for returning mutable references
        if let Some(ref_var) = self.get_returned_reference(expr) {
            if let Some(ref_info) = self.mutable_refs.get_mut(&ref_var) {
                ref_info.escapes = true;
                ref_info.current_state = RefState::Escaped {
                    through: EscapePoint::Return,
                    location: Location::default(),
                };

                violations.push(SafetyViolation {
                    location: Location::default(),
                    violation_type: ViolationType::ReferenceEscape,
                    message: format!(
                        "Mutable reference to {} escapes through return in function {}",
                        ref_var, function.name
                    ),
                    severity: Severity::Critical,
                });
            }
        }
    }

    fn check_reference_escapes(&self, function: &Function, violations: &mut Vec<SafetyViolation>) {
        for (var, ref_info) in &self.mutable_refs {
            if ref_info.escapes {
                violations.push(SafetyViolation {
                    location: ref_info.definition.clone(),
                    violation_type: ViolationType::ReferenceEscape,
                    message: format!(
                        "Mutable reference {} escapes its scope in function {}",
                        var, function.name
                    ),
                    severity: Severity::High,
                });
            }
        }
    }

    fn get_assigned_field(&self, expr: &Expression) -> Option<FieldId> {
        match expr {
            Expression::FieldAccess(_, field) => Some(FieldId {
                module_name: self.current_module.clone().unwrap_or_default(),
                struct_name: String::new(), // Would need proper context
                field_name: field.clone(),
            }),
            _ => None,
        }
    }

    fn get_returned_reference(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Variable(name) => {
                if self.mutable_refs.contains_key(name) {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
} 