use super::types::*;
use super::path_analysis::PathAnalyzer;
use super::control_flow::{ControlFlowGraph, BasicBlock};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct SuiPathAnalyzer {
    base_analyzer: PathAnalyzer,
    object_paths: HashMap<ObjectId, Vec<ObjectPath>>,
    capability_paths: HashMap<CapId, Vec<CapabilityPath>>,
    shared_access_paths: HashMap<ObjectId, Vec<SharedAccessPath>>,
}

#[derive(Debug, Clone)]
struct ObjectPath {
    path_id: usize,
    states: Vec<ObjectPathState>,
    conditions: Vec<ObjectCondition>,
    transfers: Vec<TransferPoint>,
}

#[derive(Debug, Clone)]
struct CapabilityPath {
    path_id: usize,
    permissions: HashSet<Permission>,
    delegations: Vec<DelegationPoint>,
    checks: Vec<PermissionCheck>,
}

#[derive(Debug, Clone)]
struct SharedAccessPath {
    path_id: usize,
    access_points: Vec<AccessPoint>,
    synchronization: Vec<SyncPoint>,
}

#[derive(Debug, Clone)]
enum ObjectPathState {
    Owned {
        owner: Option<String>,
        mutable: bool,
    },
    Shared {
        synchronized: bool,
        readers: HashSet<String>,
    },
    Transferred {
        from: Option<String>,
        to: String,
        guard_checked: bool,
    },
}

#[derive(Debug, Clone)]
enum ObjectCondition {
    OwnershipCheck(String),
    TransferGuard(String),
    SharedAccess(String),
}

#[derive(Debug, Clone)]
struct TransferPoint {
    location: Location,
    from_state: ObjectPathState,
    to_state: ObjectPathState,
    guard_verified: bool,
}

#[derive(Debug, Clone)]
struct DelegationPoint {
    location: Location,
    from: String,
    to: String,
    permissions: HashSet<Permission>,
}

#[derive(Debug, Clone)]
struct PermissionCheck {
    location: Location,
    permission: Permission,
    result: bool,
}

#[derive(Debug, Clone)]
struct AccessPoint {
    location: Location,
    kind: AccessKind,
    synchronized: bool,
}

#[derive(Debug, Clone)]
struct SyncPoint {
    location: Location,
    mechanism: SyncMechanism,
    success: bool,
}

#[derive(Debug, Clone)]
enum SyncMechanism {
    Lock,
    Consensus,
    Custom(String),
}

impl SuiPathAnalyzer {
    pub fn new() -> Self {
        Self {
            base_analyzer: PathAnalyzer::new(),
            object_paths: HashMap::new(),
            capability_paths: HashMap::new(),
            shared_access_paths: HashMap::new(),
        }
    }

    pub fn analyze_paths(&mut self, cfg: &ControlFlowGraph) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // Run base path analysis
        leaks.extend(self.base_analyzer.analyze_paths(cfg));
        
        // Analyze object paths
        self.analyze_object_paths(cfg, &mut leaks);
        
        // Analyze capability paths
        self.analyze_capability_paths(cfg, &mut leaks);
        
        // Analyze shared object access paths
        self.analyze_shared_access_paths(cfg, &mut leaks);
        
        leaks
    }

    fn analyze_object_paths(&mut self, cfg: &ControlFlowGraph, leaks: &mut Vec<ReferenceLeak>) {
        for (obj_id, paths) in &mut self.object_paths {
            for path in paths {
                // Check for unsafe state transitions
                for window in path.states.windows(2) {
                    if let [state1, state2] = window {
                        if self.is_unsafe_state_transition(state1, state2) {
                            leaks.push(ReferenceLeak {
                                location: Location {
                                    file: String::new(),
                                    line: 0,
                                    column: 0,
                                    context: format!("Object path {}", path.path_id),
                                },
                                leaked_field: FieldId {
                                    module_name: obj_id.module_name.clone(),
                                    struct_name: obj_id.type_name.clone(),
                                    field_name: String::new(),
                                },
                                context: "Unsafe object state transition".to_string(),
                                severity: Severity::High,
                            });
                        }
                    }
                }

                // Check transfer safety
                for transfer in &path.transfers {
                    if !transfer.guard_verified {
                        leaks.push(ReferenceLeak {
                            location: transfer.location.clone(),
                            leaked_field: FieldId {
                                module_name: obj_id.module_name.clone(),
                                struct_name: obj_id.type_name.clone(),
                                field_name: String::new(),
                            },
                            context: "Transfer without guard verification".to_string(),
                            severity: Severity::Critical,
                        });
                    }
                }
            }
        }
    }

    fn analyze_capability_paths(&mut self, cfg: &ControlFlowGraph, leaks: &mut Vec<ReferenceLeak>) {
        for (cap_id, paths) in &mut self.capability_paths {
            for path in paths {
                // Check for unsafe delegations
                for delegation in &path.delegations {
                    if !self.is_safe_delegation(delegation, &path.permissions) {
                        leaks.push(ReferenceLeak {
                            location: delegation.location.clone(),
                            leaked_field: FieldId {
                                module_name: cap_id.module_name.clone(),
                                struct_name: cap_id.cap_name.clone(),
                                field_name: String::new(),
                            },
                            context: "Unsafe capability delegation".to_string(),
                            severity: Severity::Critical,
                        });
                    }
                }

                // Check permission consistency
                for check in &path.checks {
                    if !path.permissions.contains(&check.permission) {
                        leaks.push(ReferenceLeak {
                            location: check.location.clone(),
                            leaked_field: FieldId {
                                module_name: cap_id.module_name.clone(),
                                struct_name: cap_id.cap_name.clone(),
                                field_name: String::new(),
                            },
                            context: "Permission check without proper capability".to_string(),
                            severity: Severity::High,
                        });
                    }
                }
            }
        }
    }

    fn analyze_shared_access_paths(&mut self, cfg: &ControlFlowGraph, leaks: &mut Vec<ReferenceLeak>) {
        for (obj_id, paths) in &mut self.shared_access_paths {
            for path in paths {
                // Check synchronization
                for access in &path.access_points {
                    if !access.synchronized && matches!(access.kind, AccessKind::Write) {
                        leaks.push(ReferenceLeak {
                            location: access.location.clone(),
                            leaked_field: FieldId {
                                module_name: obj_id.module_name.clone(),
                                struct_name: obj_id.type_name.clone(),
                                field_name: String::new(),
                            },
                            context: "Unsynchronized write to shared object".to_string(),
                            severity: Severity::Critical,
                        });
                    }
                }

                // Verify sync points
                for sync in &path.synchronization {
                    if !sync.success {
                        leaks.push(ReferenceLeak {
                            location: sync.location.clone(),
                            leaked_field: FieldId {
                                module_name: obj_id.module_name.clone(),
                                struct_name: obj_id.type_name.clone(),
                                field_name: String::new(),
                            },
                            context: "Failed synchronization point".to_string(),
                            severity: Severity::High,
                        });
                    }
                }
            }
        }
    }

    fn is_unsafe_state_transition(&self, from: &ObjectPathState, to: &ObjectPathState) -> bool {
        match (from, to) {
            (ObjectPathState::Owned { .. }, ObjectPathState::Transferred { guard_checked: false, .. }) => true,
            (ObjectPathState::Shared { synchronized: false, .. }, ObjectPathState::Transferred { .. }) => true,
            _ => false,
        }
    }

    fn is_safe_delegation(&self, delegation: &DelegationPoint, allowed_permissions: &HashSet<Permission>) -> bool {
        delegation.permissions.iter().all(|p| allowed_permissions.contains(p))
    }
} 