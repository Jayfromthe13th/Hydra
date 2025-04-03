use std::collections::{HashMap, HashSet};
use crate::analyzer::types::*;
use crate::analyzer::parser::{Module, Function, Statement, Struct};

#[derive(Debug, Clone)]
pub enum Property {
    Local(LocalProperty),
    Unreachable(UnreachableProperty),
    Strong(StrongProperty),
}

#[derive(Debug, Clone)]
pub struct LocalProperty {
    pub invariant: String,
    pub scope: String,
    pub condition: String,
}

#[derive(Debug, Clone)]
pub struct UnreachableProperty {
    pub resource: String,
    pub access_path: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StrongProperty {
    pub local: LocalProperty,
    pub unreachable: UnreachableProperty,
}

#[derive(Debug, Clone)]
pub enum BoundaryKind {
    TrustedToUntrusted,
    UntrustedToTrusted,
    CrossModule,
}

#[derive(Debug)]
pub struct SafetyVerifier {
    local_properties: HashMap<String, LocalProperty>,
    unreachable_properties: HashMap<String, UnreachableProperty>,
    strong_properties: HashMap<String, StrongProperty>,
    #[allow(dead_code)]
    verified_states: HashSet<String>,
}

impl SafetyVerifier {
    pub fn new() -> Self {
        Self {
            local_properties: HashMap::new(),
            unreachable_properties: HashMap::new(),
            strong_properties: HashMap::new(),
            verified_states: HashSet::new(),
        }
    }

    pub fn verify_module(&mut self, module: &Module) -> Vec<SafetyViolation> {
        let mut violations = Vec::new();

        // Verify local properties
        for function in &module.functions {
            if let Some(local_violations) = self.verify_local_properties(function) {
                violations.extend(local_violations);
            }
        }

        // Verify unreachability
        if let Some(unreachable_violations) = self.verify_unreachability(module) {
            violations.extend(unreachable_violations);
        }

        // Verify strong properties
        if let Some(strong_violations) = self.verify_strong_properties(module) {
            violations.extend(strong_violations);
        }

        // Add Sui-specific property checks
        if let Some(object_violations) = self.verify_object_properties(module) {
            violations.extend(object_violations);
        }

        violations
    }

    fn verify_local_properties(&self, function: &Function) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();

        for statement in &function.body {
            match statement {
                Statement::Call(name, _) => {
                    if let Some(property) = self.local_properties.get(name) {
                        if !self.verify_local_condition(&property.condition) {
                            violations.push(SafetyViolation {
                                location: Location::default(),
                                violation_type: ViolationType::InvariantViolation,
                                message: format!("Local property violation in function {}", function.name),
                                severity: Severity::High,
                                context: Some(ViolationContext {
                                    affected_functions: vec![function.name.clone()],
                                    related_types: vec![],
                                    suggested_fixes: vec!["Ensure local invariant holds".to_string()],
                                    whitepaper_reference: Some("Section 4.4: Local Properties".to_string()),
                                }),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn verify_unreachability(&self, module: &Module) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();

        for (resource, property) in &self.unreachable_properties {
            if self.is_resource_reachable(module, resource, &property.access_path) {
                violations.push(SafetyViolation {
                    location: Location::default(),
                    violation_type: ViolationType::ResourceSafetyViolation,
                    message: format!("Resource {} is reachable through unauthorized path", resource),
                    severity: Severity::High,
                    context: Some(ViolationContext {
                        affected_functions: vec![],
                        related_types: vec![resource.clone()],
                        suggested_fixes: vec!["Remove unauthorized access path".to_string()],
                        whitepaper_reference: Some("Section 4.4: Unreachability".to_string()),
                    }),
                });
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn verify_strong_properties(&self, module: &Module) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();

        for property in self.strong_properties.values() {
            // Verify both local and unreachable conditions
            if !self.verify_local_condition(&property.local.condition) ||
               self.is_resource_reachable(module, &property.unreachable.resource, &property.unreachable.access_path) {
                violations.push(SafetyViolation {
                    location: Location::default(),
                    violation_type: ViolationType::ResourceSafetyViolation,
                    message: "Strong property violation detected".to_string(),
                    severity: Severity::Critical,
                    context: Some(ViolationContext {
                        affected_functions: vec![],
                        related_types: vec![],
                        suggested_fixes: vec!["Ensure both local and unreachability properties hold".to_string()],
                        whitepaper_reference: Some("Section 4.4: Strong Properties".to_string()),
                    }),
                });
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn verify_local_condition(&self, condition: &str) -> bool {
        // Basic condition verification for POC
        !condition.is_empty()
    }

    fn is_resource_reachable(&self, module: &Module, resource: &str, _access_path: &[String]) -> bool {
        // Basic reachability check for POC
        for function in &module.functions {
            for statement in &function.body {
                if let Statement::Call(name, _) = statement {
                    if name.contains(resource) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn add_local_property(&mut self, name: String, property: LocalProperty) {
        self.local_properties.insert(name, property);
    }

    pub fn add_unreachable_property(&mut self, name: String, property: UnreachableProperty) {
        self.unreachable_properties.insert(name, property);
    }

    pub fn add_strong_property(&mut self, name: String, property: StrongProperty) {
        self.strong_properties.insert(name, property);
    }

    // Add Sui-specific property checks
    fn verify_object_properties(&self, module: &Module) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();

        // Check object initialization
        for struct_def in module.get_structs() {
            if struct_def.has_key_ability() {
                self.verify_uid_initialization(struct_def, &mut violations);
            }
        }

        // Check transfer safety
        for function in &module.functions {
            self.verify_transfer_patterns(function, &mut violations);
        }

        // Check shared object access
        self.verify_shared_object_access(module, &mut violations);

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn verify_uid_initialization(&self, struct_def: &Struct, violations: &mut Vec<SafetyViolation>) {
        // Check for UID field
        let has_uid = struct_def.fields.iter().any(|f| f.field_type.to_string() == "UID");
        
        if !has_uid && struct_def.has_key_ability() {
            violations.push(SafetyViolation {
                location: Location::default(),
                violation_type: ViolationType::ResourceSafetyViolation,
                message: format!("Sui object {} missing UID field", struct_def.name),
                severity: Severity::Critical,
                context: Some(ViolationContext {
                    affected_functions: vec![],
                    related_types: vec![struct_def.name.clone()],
                    suggested_fixes: vec!["Add 'id: UID' as first field".to_string()],
                    whitepaper_reference: Some("Sui Object Model".to_string()),
                }),
            });
        }
    }

    fn verify_transfer_patterns(&self, function: &Function, violations: &mut Vec<SafetyViolation>) {
        for statement in &function.body {
            if let Statement::Call(name, _args) = statement {
                if name.contains("transfer::transfer") {
                    // Check transfer preconditions
                    if !self.has_ownership_verification(function) {
                        violations.push(SafetyViolation {
                            location: Location::default(),
                            violation_type: ViolationType::UnauthorizedAccess,
                            message: "Transfer without ownership verification".to_string(),
                            severity: Severity::High,
                            context: Some(ViolationContext {
                                affected_functions: vec![function.name.clone()],
                                related_types: vec![],
                                suggested_fixes: vec!["Add ownership check before transfer".to_string()],
                                whitepaper_reference: Some("Sui Transfer Safety".to_string()),
                            }),
                        });
                    }
                }
            }
        }
    }

    fn verify_shared_object_access(&self, module: &Module, violations: &mut Vec<SafetyViolation>) {
        for function in &module.functions {
            let mut has_shared_access = false;
            let mut has_consensus = false;
            let mut has_sync = false;

            for statement in &function.body {
                if let Statement::Call(name, _) = statement {
                    if self.is_shared_object_access(name) {
                        has_shared_access = true;
                    }
                    if name.contains("consensus::verify") {
                        has_consensus = true;
                    }
                    if name.contains("sync") || name.contains("lock") {
                        has_sync = true;
                    }
                }
            }

            if has_shared_access && !(has_consensus && has_sync) {
                violations.push(SafetyViolation {
                    location: Location::default(),
                    violation_type: ViolationType::SharedObjectViolation,
                    message: format!("Shared object access without proper synchronization in function {}", 
                        function.name),
                    severity: Severity::High,
                    context: Some(ViolationContext {
                        affected_functions: vec![function.name.clone()],
                        related_types: vec![],
                        suggested_fixes: vec![
                            "Add consensus::verify call".to_string(),
                            "Add proper synchronization".to_string()
                        ],
                        whitepaper_reference: Some("Sui Shared Objects".to_string()),
                    }),
                });
            }
        }
    }

    // Sui-specific helper methods
    fn has_ownership_verification(&self, function: &Function) -> bool {
        let mut has_sender_check = false;
        let mut has_owner_access = false;
        let mut has_assertion = false;

        for statement in &function.body {
            match statement {
                Statement::Call(name, _) => {
                    if name.contains("tx_context::sender") {
                        has_sender_check = true;
                    }
                    if name.contains("owner") {
                        has_owner_access = true;
                    }
                }
                Statement::Assert(_) => {
                    has_assertion = true;
                }
                _ => {}
            }
        }

        has_sender_check && has_owner_access && has_assertion
    }

    #[allow(dead_code)]
    fn has_consensus_check(&self, function: &Function) -> bool {
        let mut has_consensus_verify = false;
        let mut has_sync = false;

        for statement in &function.body {
            if let Statement::Call(name, _) = statement {
                if name.contains("consensus::verify") {
                    has_consensus_verify = true;
                }
                if name.contains("sync") || name.contains("lock") {
                    has_sync = true;
                }
            }
        }

        has_consensus_verify && has_sync
    }

    fn is_shared_object_access(&self, call_name: &str) -> bool {
        call_name.contains("shared") || 
        call_name.contains("consensus") ||
        call_name.contains("sync")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_property_verification() {
        let mut verifier = SafetyVerifier::new();
        
        verifier.add_local_property(
            "test".to_string(),
            LocalProperty {
                invariant: "value >= 0".to_string(),
                scope: "function".to_string(),
                condition: "check".to_string(),
            }
        );

        let mut module = Module::new("test".to_string());
        let mut function = Function::new("test".to_string());
        function.add_statement(Statement::Call("test".to_string(), vec![]));
        module.add_function(function);

        let violations = verifier.verify_module(&module);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_unreachability_verification() {
        let mut verifier = SafetyVerifier::new();
        
        verifier.add_unreachable_property(
            "secret".to_string(),
            UnreachableProperty {
                resource: "secret".to_string(),
                access_path: vec!["public".to_string()],
            }
        );

        let mut module = Module::new("test".to_string());
        let mut function = Function::new("test".to_string());
        function.add_statement(Statement::Call("public::secret".to_string(), vec![]));
        module.add_function(function);

        let violations = verifier.verify_module(&module);
        assert!(!violations.is_empty());
    }
} 