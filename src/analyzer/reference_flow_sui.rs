use super::types::*;
use super::parser::{Function, Statement, Expression};
use super::reference_flow::ReferenceFlowAnalyzer;
use std::collections::{HashMap, HashSet};

pub struct SuiReferenceFlowAnalyzer {
    base_analyzer: ReferenceFlowAnalyzer,
    object_states: HashMap<String, ObjectReferenceState>,
    capability_states: HashMap<String, CapabilityReferenceState>,
    shared_access_points: HashSet<Location>,
}

#[derive(Debug, Clone)]
struct ObjectReferenceState {
    object_id: ObjectId,
    reference_type: ReferenceType,
    transfer_state: TransferState,
    access_points: HashSet<Location>,
}

#[derive(Debug, Clone)]
struct CapabilityReferenceState {
    cap_id: CapId,
    permissions: HashSet<Permission>,
    delegations: Vec<CapabilityDelegation>,
}

#[derive(Debug, Clone)]
enum TransferState {
    Owned,
    Transferred(Address),
    Shared,
    Frozen,
}

#[derive(Debug)]
struct CapabilityDelegation {
    from: String,
    to: String,
    permissions: HashSet<Permission>,
    location: Location,
}

impl SuiReferenceFlowAnalyzer {
    pub fn new() -> Self {
        Self {
            base_analyzer: ReferenceFlowAnalyzer::new(),
            object_states: HashMap::new(),
            capability_states: HashMap::new(),
            shared_access_points: HashSet::new(),
        }
    }

    pub fn analyze_function(&mut self, function: &Function) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // Track object references
        self.track_object_references(function, &mut leaks);
        
        // Track capability references
        self.track_capability_references(function, &mut leaks);
        
        // Track shared object access
        self.track_shared_object_access(function, &mut leaks);
        
        // Run base reference analysis
        leaks.extend(self.base_analyzer.analyze_function(function));
        
        leaks
    }

    fn track_object_references(&mut self, function: &Function, leaks: &mut Vec<ReferenceLeak>) {
        for statement in &function.body {
            match statement {
                Statement::Assignment(var, expr) => {
                    if let Some(object_state) = self.analyze_object_expression(expr) {
                        // Check for unsafe object reference patterns
                        if self.is_unsafe_object_reference(&object_state) {
                            leaks.push(ReferenceLeak {
                                location: Location {
                                    file: String::new(),
                                    line: 0,
                                    column: 0,
                                    context: format!("Assignment to {}", var),
                                },
                                leaked_field: FieldId {
                                    module_name: object_state.object_id.module_name,
                                    struct_name: object_state.object_id.type_name,
                                    field_name: String::new(),
                                },
                                context: "Unsafe object reference pattern detected".to_string(),
                                severity: Severity::High,
                            });
                        }
                        self.object_states.insert(var.clone(), object_state);
                    }
                }
                Statement::Return(expr) => {
                    if let Some(object_state) = self.analyze_object_expression(expr) {
                        // Check for object reference leaks in return
                        if self.is_object_reference_leak(&object_state) {
                            leaks.push(ReferenceLeak {
                                location: Location {
                                    file: String::new(),
                                    line: 0,
                                    column: 0,
                                    context: "Return statement".to_string(),
                                },
                                leaked_field: FieldId {
                                    module_name: object_state.object_id.module_name,
                                    struct_name: object_state.object_id.type_name,
                                    field_name: String::new(),
                                },
                                context: "Object reference escapes through return".to_string(),
                                severity: Severity::Critical,
                            });
                        }
                    }
                }
            }
        }
    }

    fn track_capability_references(&mut self, function: &Function, leaks: &mut Vec<ReferenceLeak>) {
        for statement in &function.body {
            match statement {
                Statement::Assignment(var, expr) => {
                    if let Some(cap_state) = self.analyze_capability_expression(expr) {
                        // Check for unsafe capability patterns
                        if self.is_unsafe_capability_usage(&cap_state) {
                            leaks.push(ReferenceLeak {
                                location: Location {
                                    file: String::new(),
                                    line: 0,
                                    column: 0,
                                    context: format!("Assignment to {}", var),
                                },
                                leaked_field: FieldId {
                                    module_name: cap_state.cap_id.module_name,
                                    struct_name: cap_state.cap_id.cap_name,
                                    field_name: String::new(),
                                },
                                context: "Unsafe capability usage pattern detected".to_string(),
                                severity: Severity::Critical,
                            });
                        }
                        self.capability_states.insert(var.clone(), cap_state);
                    }
                }
                _ => {}
            }
        }
    }

    fn track_shared_object_access(&mut self, function: &Function, leaks: &mut Vec<ReferenceLeak>) {
        for statement in &function.body {
            if let Statement::Assignment(_, expr) = statement {
                if let Some(location) = self.is_shared_object_access(expr) {
                    self.shared_access_points.insert(location);
                    
                    // Check for unsafe shared object access patterns
                    if !self.is_synchronized_access(expr) {
                        leaks.push(ReferenceLeak {
                            location: location.clone(),
                            leaked_field: FieldId {
                                module_name: String::new(),
                                struct_name: String::new(),
                                field_name: String::new(),
                            },
                            context: "Unsynchronized access to shared object".to_string(),
                            severity: Severity::High,
                        });
                    }
                }
            }
        }
    }

    fn analyze_object_expression(&self, expr: &Expression) -> Option<ObjectReferenceState> {
        match expr {
            Expression::Variable(name) => {
                self.object_states.get(name).cloned()
            }
            Expression::FieldAccess(base, field) => {
                // Analyze field access on objects
                None // Simplified for now
            }
        }
    }

    fn analyze_capability_expression(&self, expr: &Expression) -> Option<CapabilityReferenceState> {
        match expr {
            Expression::Variable(name) => {
                self.capability_states.get(name).cloned()
            }
            Expression::FieldAccess(base, field) => {
                // Analyze capability field access
                None // Simplified for now
            }
        }
    }

    fn is_unsafe_object_reference(&self, state: &ObjectReferenceState) -> bool {
        matches!(state.transfer_state, TransferState::Transferred(_))
            || (matches!(state.transfer_state, TransferState::Shared)
                && matches!(state.reference_type, ReferenceType::Mutable))
    }

    fn is_object_reference_leak(&self, state: &ObjectReferenceState) -> bool {
        matches!(state.reference_type, ReferenceType::Mutable)
            && !matches!(state.transfer_state, TransferState::Owned)
    }

    fn is_unsafe_capability_usage(&self, state: &CapabilityReferenceState) -> bool {
        state.permissions.is_empty() || state.delegations.len() > 0
    }

    fn is_shared_object_access(&self, expr: &Expression) -> Option<Location> {
        // Check if expression accesses a shared object
        None // Simplified for now
    }

    fn is_synchronized_access(&self, expr: &Expression) -> bool {
        // Check if shared object access is properly synchronized
        false // Simplified for now
    }
} 