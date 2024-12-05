use super::types::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ObjectId {
    pub module_name: String,
    pub type_name: String,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct ObjectSafetyIssue {
    pub location: Location,
    pub object_id: String,
    pub issue_type: ObjectIssueType,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone)]
pub enum ObjectIssueType {
    UnsafeTransfer,
    InvalidSharedAccess,
    ResourceLeak,
    IncompleteCleanup,
    UnauthorizedModification,
    ImproperInitialization,
}

#[derive(Debug)]
pub struct ObjectStateTracker {
    states: HashMap<String, ObjectState>,
}

#[derive(Debug, Clone)]
pub enum ObjectState {
    Uninitialized,
    Initialized {
        owner: Option<String>,
        shared: bool,
        frozen: bool,
    },
    Transferred {
        to: String,
        guard_checked: bool,
    },
    Deleted,
}

impl ObjectStateTracker {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn verify_object_safety(&self) -> Vec<ObjectSafetyIssue> {
        let mut issues = Vec::new();
        
        for (object_id, state) in &self.states {
            match state {
                ObjectState::Uninitialized => {
                    issues.push(ObjectSafetyIssue {
                        location: Location::default(),
                        object_id: object_id.clone(),
                        issue_type: ObjectIssueType::ImproperInitialization,
                        message: "Object used before initialization".to_string(),
                        severity: Severity::High,
                    });
                }
                ObjectState::Transferred { guard_checked, .. } if !guard_checked => {
                    issues.push(ObjectSafetyIssue {
                        location: Location::default(),
                        object_id: object_id.clone(),
                        issue_type: ObjectIssueType::UnsafeTransfer,
                        message: "Object transferred without guard check".to_string(),
                        severity: Severity::High,
                    });
                }
                _ => {}
            }
        }

        issues
    }
}

impl ObjectState {
    pub fn is_shared(&self) -> bool {
        match self {
            ObjectState::Initialized { shared, .. } => *shared,
            _ => false,
        }
    }

    pub fn is_frozen(&self) -> bool {
        match self {
            ObjectState::Initialized { frozen, .. } => *frozen,
            _ => false,
        }
    }
} 