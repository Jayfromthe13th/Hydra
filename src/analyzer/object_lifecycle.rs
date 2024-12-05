use super::types::*;
use super::object_state::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ObjectLifecycleTracker {
    objects: HashMap<ObjectId, ObjectLifecycle>,
    current_function: Option<String>,
    violations: Vec<ObjectSafetyIssue>,
}

#[derive(Debug, Clone)]
pub struct ObjectLifecycle {
    state: ObjectState,
    history: Vec<ObjectEvent>,
    access_points: HashSet<Location>,
    capabilities: HashSet<CapId>,
}

#[derive(Debug, Clone)]
pub enum ObjectEvent {
    Created {
        location: Location,
        has_id: bool,
        has_owner: bool,
    },
    Transferred {
        location: Location,
        from: Option<Address>,
        to: Address,
        guard_checked: bool,
    },
    SharedAccess {
        location: Location,
        access_type: SharedAccessType,
        synchronized: bool,
    },
    CapabilityCheck {
        location: Location,
        capability: CapId,
        result: bool,
    },
}

impl ObjectLifecycleTracker {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            current_function: None,
            violations: Vec::new(),
        }
    }

    pub fn track_object(&mut self, id: ObjectId, location: Location) {
        let lifecycle = ObjectLifecycle {
            state: ObjectState::Uninitialized,
            history: Vec::new(),
            access_points: HashSet::new(),
            capabilities: HashSet::new(),
        };
        self.objects.insert(id, lifecycle);
    }

    pub fn record_creation(&mut self, id: &ObjectId, has_id: bool, has_owner: bool, location: Location) {
        if let Some(lifecycle) = self.objects.get_mut(id) {
            lifecycle.history.push(ObjectEvent::Created {
                location: location.clone(),
                has_id,
                has_owner,
            });

            // Check for unsafe construction
            if !has_id || !has_owner {
                self.violations.push(ObjectSafetyIssue {
                    location,
                    object_id: id.clone(),
                    issue_type: ObjectIssueType::UnsafeObjectConstruction,
                    message: "Object created without proper ID or owner".to_string(),
                    severity: Severity::High,
                });
            }

            lifecycle.state = ObjectState::Initialized {
                owner: None,
                shared: false,
                frozen: false,
            };
        }
    }

    pub fn record_transfer(&mut self, id: &ObjectId, from: Option<Address>, to: Address, guard_checked: bool, location: Location) {
        if let Some(lifecycle) = self.objects.get_mut(id) {
            // Check for unsafe transfers
            if !guard_checked {
                self.violations.push(ObjectSafetyIssue {
                    location: location.clone(),
                    object_id: id.clone(),
                    issue_type: ObjectIssueType::UnsafeTransfer,
                    message: "Object transferred without guard check".to_string(),
                    severity: Severity::Critical,
                });
            }

            lifecycle.history.push(ObjectEvent::Transferred {
                location,
                from,
                to: to.clone(),
                guard_checked,
            });

            lifecycle.state = ObjectState::Transferred {
                to,
                guard_checked,
            };
        }
    }

    pub fn record_shared_access(&mut self, id: &ObjectId, access_type: SharedAccessType, synchronized: bool, location: Location) {
        if let Some(lifecycle) = self.objects.get_mut(id) {
            // Check for unsafe shared access
            if !synchronized && matches!(access_type, SharedAccessType::Write) {
                self.violations.push(ObjectSafetyIssue {
                    location: location.clone(),
                    object_id: id.clone(),
                    issue_type: ObjectIssueType::InvalidSharedAccess,
                    message: "Unsynchronized write to shared object".to_string(),
                    severity: Severity::Critical,
                });
            }

            lifecycle.history.push(ObjectEvent::SharedAccess {
                location,
                access_type,
                synchronized,
            });

            lifecycle.access_points.insert(location);
        }
    }

    pub fn verify_capability(&mut self, id: &ObjectId, cap: &CapId, location: Location) -> bool {
        if let Some(lifecycle) = self.objects.get_mut(id) {
            let has_capability = lifecycle.capabilities.contains(cap);
            
            lifecycle.history.push(ObjectEvent::CapabilityCheck {
                location: location.clone(),
                capability: cap.clone(),
                result: has_capability,
            });

            if !has_capability {
                self.violations.push(ObjectSafetyIssue {
                    location,
                    object_id: id.clone(),
                    issue_type: ObjectIssueType::CapabilityExposure,
                    message: format!("Missing required capability: {}", cap.cap_name),
                    severity: Severity::High,
                });
            }

            has_capability
        } else {
            false
        }
    }

    pub fn get_violations(&self) -> &[ObjectSafetyIssue] {
        &self.violations
    }

    pub fn analyze_lifecycle(&self, id: &ObjectId) -> Vec<ObjectSafetyIssue> {
        let mut issues = Vec::new();
        
        if let Some(lifecycle) = self.objects.get(id) {
            // Check for proper initialization
            if let Some(ObjectEvent::Created { has_id, has_owner, location }) = lifecycle.history.first() {
                if !has_id || !has_owner {
                    issues.push(ObjectSafetyIssue {
                        location: location.clone(),
                        object_id: id.clone(),
                        issue_type: ObjectIssueType::UnsafeObjectConstruction,
                        message: "Object not properly initialized".to_string(),
                        severity: Severity::High,
                    });
                }
            }

            // Check transfer patterns
            for (i, event) in lifecycle.history.iter().enumerate() {
                if let ObjectEvent::Transferred { guard_checked, location, .. } = event {
                    if !guard_checked {
                        issues.push(ObjectSafetyIssue {
                            location: location.clone(),
                            object_id: id.clone(),
                            issue_type: ObjectIssueType::InvalidTransferGuard,
                            message: "Transfer without proper guard check".to_string(),
                            severity: Severity::High,
                        });
                    }

                    // Check for transfers after shared access
                    if lifecycle.history[..i].iter().any(|e| matches!(e, ObjectEvent::SharedAccess { .. })) {
                        issues.push(ObjectSafetyIssue {
                            location: location.clone(),
                            object_id: id.clone(),
                            issue_type: ObjectIssueType::InvalidSharedAccess,
                            message: "Transfer after shared access".to_string(),
                            severity: Severity::Critical,
                        });
                    }
                }
            }
        }

        issues
    }
} 