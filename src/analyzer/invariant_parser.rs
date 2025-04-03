use super::types::*;
use super::parser::{Module, Function, Statement, Expression};
use std::collections::{HashMap, HashSet};

pub struct InvariantParser {
    invariants: HashMap<String, ModuleInvariants>,
    protected_fields: HashSet<FieldId>,
    current_module: Option<String>,
}

#[derive(Debug)]
struct ModuleInvariants {
    struct_invariants: HashMap<String, Vec<StructInvariant>>,
    global_invariants: Vec<GlobalInvariant>,
    field_protections: HashMap<FieldId, Vec<InvariantProtection>>,
}

#[derive(Debug)]
struct StructInvariant {
    condition: InvariantCondition,
    affected_fields: HashSet<FieldId>,
    verification_points: Vec<VerificationPoint>,
}

#[derive(Debug)]
struct GlobalInvariant {
    condition: InvariantCondition,
    scope: InvariantScope,
    dependencies: HashSet<FieldId>,
}

#[derive(Debug)]
enum InvariantCondition {
    FieldConstraint {
        field: FieldId,
        constraint: String,
    },
    ReferenceInvariant {
        field: FieldId,
        kind: ReferenceInvariantKind,
    },
    StateInvariant {
        fields: Vec<FieldId>,
        predicate: String,
    },
    Custom(String),
}

#[derive(Debug)]
enum ReferenceInvariantKind {
    NoEscape,
    NoMutableAlias,
    LocalOnly,
    ProtectedAccess,
}

#[derive(Debug)]
enum InvariantScope {
    Module,
    Struct(String),
    Function(String),
}

#[derive(Debug)]
struct InvariantProtection {
    kind: ProtectionKind,
    condition: String,
    enforced_at: Vec<Location>,
}

#[derive(Debug)]
enum ProtectionKind {
    NoMutableReference,
    NoExternalAccess,
    GuardedAccess,
    Custom(String),
}

impl InvariantParser {
    pub fn new() -> Self {
        Self {
            invariants: HashMap::new(),
            protected_fields: HashSet::new(),
            current_module: None,
        }
    }

    pub fn parse_module(&mut self, module: &Module) -> Result<(), String> {
        self.current_module = Some(module.name.clone());
        let mut module_invariants = ModuleInvariants {
            struct_invariants: HashMap::new(),
            global_invariants: Vec::new(),
            field_protections: HashMap::new(),
        };

        // Parse struct invariants
        for struct_def in &module.structs {
            let invariants = self.parse_struct_invariants(struct_def)?;
            if !invariants.is_empty() {
                module_invariants.struct_invariants.insert(struct_def.name.clone(), invariants);
            }
        }

        // Parse global invariants
        module_invariants.global_invariants = self.parse_global_invariants(module)?;

        // Track protected fields
        self.analyze_field_protections(&module_invariants);

        // Store module invariants
        self.invariants.insert(module.name.clone(), module_invariants);
        
        Ok(())
    }

    fn parse_struct_invariants(&self, struct_def: &Struct) -> Result<Vec<StructInvariant>, String> {
        let mut invariants = Vec::new();
        
        // Parse invariant attributes
        for attr in &struct_def.attributes {
            if attr.name == "invariant" {
                let condition = self.parse_invariant_condition(&attr.value)?;
                let affected_fields = self.get_affected_fields(&condition);
                
                invariants.push(StructInvariant {
                    condition,
                    affected_fields,
                    verification_points: Vec::new(),
                });
            }
        }

        // Parse field-level invariants
        for field in &struct_def.fields {
            if let Some(field_invariant) = self.parse_field_invariant(field)? {
                invariants.push(field_invariant);
            }
        }

        Ok(invariants)
    }

    fn parse_global_invariants(&self, module: &Module) -> Result<Vec<GlobalInvariant>, String> {
        let mut invariants = Vec::new();

        // Parse module-level invariant attributes
        for attr in &module.attributes {
            if attr.name == "module_invariant" {
                let condition = self.parse_invariant_condition(&attr.value)?;
                let dependencies = self.get_affected_fields(&condition);
                
                invariants.push(GlobalInvariant {
                    condition,
                    scope: InvariantScope::Module,
                    dependencies,
                });
            }
        }

        Ok(invariants)
    }

    fn parse_invariant_condition(&self, expr: &str) -> Result<InvariantCondition, String> {
        // Parse invariant expression into structured condition
        if expr.contains("no_mutable_references") {
            let field = self.extract_field_from_expr(expr)?;
            Ok(InvariantCondition::ReferenceInvariant {
                field,
                kind: ReferenceInvariantKind::NoMutableAlias,
            })
        } else if expr.contains("local_only") {
            let field = self.extract_field_from_expr(expr)?;
            Ok(InvariantCondition::ReferenceInvariant {
                field,
                kind: ReferenceInvariantKind::LocalOnly,
            })
        } else {
            Ok(InvariantCondition::Custom(expr.to_string()))
        }
    }

    fn parse_field_invariant(&self, field: &Field) -> Result<Option<StructInvariant>, String> {
        for attr in &field.attributes {
            if attr.name == "invariant" {
                let field_id = self.get_field_id(field);
                return Ok(Some(StructInvariant {
                    condition: InvariantCondition::FieldConstraint {
                        field: field_id,
                        constraint: attr.value.clone(),
                    },
                    affected_fields: [field_id].into_iter().collect(),
                    verification_points: Vec::new(),
                }));
            }
        }
        Ok(None)
    }

    fn analyze_field_protections(&mut self, module_invariants: &ModuleInvariants) {
        // Analyze invariants to determine protected fields
        for invariants in module_invariants.struct_invariants.values() {
            for invariant in invariants {
                match &invariant.condition {
                    InvariantCondition::ReferenceInvariant { field, kind } => {
                        match kind {
                            ReferenceInvariantKind::NoMutableAlias |
                            ReferenceInvariantKind::LocalOnly |
                            ReferenceInvariantKind::ProtectedAccess => {
                                self.protected_fields.insert(field.clone());
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn get_protected_fields(&self) -> &HashSet<FieldId> {
        &self.protected_fields
    }

    pub fn verify_invariants(&self, function: &Function) -> Vec<SafetyViolation> {
        let mut violations = Vec::new();

        if let Some(module_invariants) = self.invariants.get(
            &self.current_module.clone().unwrap_or_default()
        ) {
            // Check function against relevant invariants
            for statement in &function.body {
                self.verify_statement_invariants(
                    statement,
                    module_invariants,
                    &mut violations
                );
            }
        }

        violations
    }

    fn verify_statement_invariants(
        &self,
        statement: &Statement,
        invariants: &ModuleInvariants,
        violations: &mut Vec<SafetyViolation>
    ) {
        match statement {
            Statement::Assignment(var, expr) => {
                // Check for protected field access
                if let Some(field) = self.get_accessed_field(expr) {
                    if self.protected_fields.contains(&field) {
                        violations.push(SafetyViolation {
                            location: Location::default(),
                            violation_type: ViolationType::InvariantViolation,
                            message: format!(
                                "Assignment to protected field {} violates invariant",
                                field.field_name
                            ),
                            severity: Severity::High,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    // Helper methods
    fn get_field_id(&self, field: &Field) -> FieldId {
        FieldId {
            module_name: self.current_module.clone().unwrap_or_default(),
            struct_name: String::new(), // Would need proper context
            field_name: field.name.clone(),
        }
    }

    fn extract_field_from_expr(&self, expr: &str) -> Result<FieldId, String> {
        // Parse field reference from expression
        Ok(FieldId {
            module_name: self.current_module.clone().unwrap_or_default(),
            struct_name: String::new(), // Would need proper context
            field_name: expr.to_string(),
        })
    }

    fn get_affected_fields(&self, condition: &InvariantCondition) -> HashSet<FieldId> {
        let mut fields = HashSet::new();
        match condition {
            InvariantCondition::FieldConstraint { field, .. } |
            InvariantCondition::ReferenceInvariant { field, .. } => {
                fields.insert(field.clone());
            }
            InvariantCondition::StateInvariant { fields: field_list, .. } => {
                fields.extend(field_list.iter().cloned());
            }
            _ => {}
        }
        fields
    }

    fn get_accessed_field(&self, expr: &Expression) -> Option<FieldId> {
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