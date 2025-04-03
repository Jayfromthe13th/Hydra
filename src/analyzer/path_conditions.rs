use super::types::*;
use super::path_tracking::PathTracker;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct PathConditionAnalyzer {
    conditions: Vec<PathCondition>,
    active_conditions: HashSet<PathCondition>,
    condition_dependencies: HashMap<String, HashSet<PathCondition>>,
    path_tracker: PathTracker,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum PathCondition {
    // Reference conditions
    ReferenceValid(String),
    ReferenceMutable(String),
    ReferenceShared(String),
    
    // Object conditions
    ObjectOwned {
        id: ObjectId,
        owner: Option<String>,
    },
    ObjectShared {
        id: ObjectId,
        synchronized: bool,
    },
    ObjectFrozen(ObjectId),
    
    // Capability conditions
    CapabilityHeld {
        id: CapId,
        permissions: HashSet<Permission>,
    },
    CapabilityDelegated {
        id: CapId,
        from: String,
        to: String,
    },
    
    // Transfer conditions
    TransferGuarded {
        id: ObjectId,
        guard_checked: bool,
    },
    TransferAuthorized {
        id: ObjectId,
        from: Option<String>,
        to: String,
    },
    
    // Custom conditions
    Custom(String),
}

impl PathConditionAnalyzer {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            active_conditions: HashSet::new(),
            condition_dependencies: HashMap::new(),
            path_tracker: PathTracker::new(),
        }
    }

    pub fn analyze_path_conditions(&mut self, cfg: &ControlFlowGraph) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // Get path analysis from tracker
        let paths = self.path_tracker.analyze_paths(cfg);
        
        // Analyze conditions for each path
        for path in paths {
            self.analyze_path(&path, &mut leaks);
        }
        
        leaks
    }

    fn analyze_path(&mut self, path: &ReferencePath, leaks: &mut Vec<ReferenceLeak>) {
        // Reset active conditions for new path
        self.active_conditions.clear();
        
        // Analyze conditions in path order
        for condition in &path.conditions {
            self.add_condition(condition.clone());
            
            // Check for condition violations
            if let Some(violation) = self.check_condition_violation(condition) {
                leaks.push(violation);
            }
        }

        // Check state transitions under current conditions
        for transition in &path.state_transitions {
            if let Some(violation) = self.check_transition_safety(transition) {
                leaks.push(violation);
            }
        }
    }

    fn add_condition(&mut self, condition: PathCondition) {
        // Add condition and update dependencies
        self.active_conditions.insert(condition.clone());
        
        match &condition {
            PathCondition::ReferenceValid(var) |
            PathCondition::ReferenceMutable(var) |
            PathCondition::ReferenceShared(var) => {
                self.add_reference_dependency(var, condition);
            }
            PathCondition::ObjectOwned { id, .. } |
            PathCondition::ObjectShared { id, .. } |
            PathCondition::ObjectFrozen(id) => {
                self.add_object_dependency(id, condition);
            }
            PathCondition::CapabilityHeld { id, .. } |
            PathCondition::CapabilityDelegated { id, .. } => {
                self.add_capability_dependency(id, condition);
            }
            PathCondition::TransferGuarded { id, .. } |
            PathCondition::TransferAuthorized { id, .. } => {
                self.add_transfer_dependency(id, condition);
            }
            PathCondition::Custom(_) => {}
        }
    }

    fn check_condition_violation(&self, condition: &PathCondition) -> Option<ReferenceLeak> {
        match condition {
            PathCondition::ReferenceValid(var) => {
                if !self.is_reference_valid(var) {
                    Some(ReferenceLeak {
                        location: Location::default(), // Would need proper location
                        leaked_field: FieldId::default(), // Would need proper field
                        context: format!("Invalid reference to {}", var),
                        severity: Severity::High,
                    })
                } else {
                    None
                }
            }
            PathCondition::ObjectOwned { id, owner } => {
                if !self.verify_ownership(id, owner) {
                    Some(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId {
                            module_name: id.module_name.clone(),
                            struct_name: id.type_name.clone(),
                            field_name: String::new(),
                        },
                        context: "Invalid object ownership".to_string(),
                        severity: Severity::Critical,
                    })
                } else {
                    None
                }
            }
            PathCondition::TransferGuarded { id, guard_checked } => {
                if !guard_checked {
                    Some(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId {
                            module_name: id.module_name.clone(),
                            struct_name: id.type_name.clone(),
                            field_name: String::new(),
                        },
                        context: "Transfer without guard check".to_string(),
                        severity: Severity::Critical,
                    })
                } else {
                    None
                }
            }
            // Add more condition checks
            _ => None
        }
    }

    fn check_transition_safety(&self, transition: &StateTransition) -> Option<ReferenceLeak> {
        match (&transition.from, &transition.to) {
            (ReferenceState::Borrowed(_), ReferenceState::Moved) => {
                Some(ReferenceLeak {
                    location: transition.location.clone(),
                    leaked_field: FieldId::default(), // Would need proper field
                    context: "Moving borrowed reference".to_string(),
                    severity: Severity::Critical,
                })
            }
            // Add more transition checks
            _ => None
        }
    }

    // Helper methods
    fn add_reference_dependency(&mut self, var: &str, condition: PathCondition) {
        self.condition_dependencies
            .entry(var.to_string())
            .or_default()
            .insert(condition);
    }

    fn add_object_dependency(&mut self, id: &ObjectId, condition: PathCondition) {
        self.condition_dependencies
            .entry(id.type_name.clone())
            .or_default()
            .insert(condition);
    }

    fn add_capability_dependency(&mut self, id: &CapId, condition: PathCondition) {
        self.condition_dependencies
            .entry(id.cap_name.clone())
            .or_default()
            .insert(condition);
    }

    fn add_transfer_dependency(&mut self, id: &ObjectId, condition: PathCondition) {
        self.condition_dependencies
            .entry(format!("transfer_{}", id.type_name))
            .or_default()
            .insert(condition);
    }

    fn is_reference_valid(&self, var: &str) -> bool {
        self.active_conditions.iter().any(|c| {
            matches!(c, PathCondition::ReferenceValid(v) if v == var)
        })
    }

    fn verify_ownership(&self, id: &ObjectId, expected_owner: &Option<String>) -> bool {
        self.active_conditions.iter().any(|c| {
            matches!(c, PathCondition::ObjectOwned { id: obj_id, owner }
                if obj_id == id && owner == expected_owner)
        })
    }
} 