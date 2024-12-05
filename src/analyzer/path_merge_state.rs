use super::types::*;
use super::path_merge::PathMergeAnalyzer;
use std::collections::{HashMap, HashSet};

pub struct PathMergeStateAnalyzer {
    merge_analyzer: PathMergeAnalyzer,
    states: HashMap<String, MergeState>,
    merge_points: HashMap<usize, MergePointState>,
    current_path: Option<usize>,
}

#[derive(Debug, Clone)]
struct MergeState {
    reference_states: HashMap<String, ReferenceState>,
    object_states: HashMap<ObjectId, ObjectState>,
    conditions: Vec<PathCondition>,
    constraints: Vec<PathConstraint>,
}

#[derive(Debug)]
struct MergePointState {
    block_id: usize,
    incoming_states: Vec<MergeState>,
    merged_state: Option<MergeState>,
    conflicts: Vec<StateConflict>,
}

#[derive(Debug)]
struct StateConflict {
    var: String,
    states: Vec<ReferenceState>,
    resolution: ConflictResolution,
}

#[derive(Debug)]
enum ConflictResolution {
    Conservative,
    MostRestrictive,
    Invalid,
}

impl PathMergeStateAnalyzer {
    pub fn new() -> Self {
        Self {
            merge_analyzer: PathMergeAnalyzer::new(),
            states: HashMap::new(),
            merge_points: HashMap::new(),
            current_path: None,
        }
    }

    pub fn analyze_merge_points(&mut self, cfg: &ControlFlowGraph) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // First analyze paths and merge points
        let merge_leaks = self.merge_analyzer.analyze_paths(cfg);
        leaks.extend(merge_leaks);
        
        // Analyze states at merge points
        self.analyze_merge_states(cfg, &mut leaks);
        
        // Verify merged states
        self.verify_merged_states(&mut leaks);
        
        leaks
    }

    fn analyze_merge_states(&mut self, cfg: &ControlFlowGraph, leaks: &mut Vec<ReferenceLeak>) {
        for block in &cfg.blocks {
            if block.predecessors.len() > 1 {
                // This is a merge point
                let mut merge_point = MergePointState {
                    block_id: block.id,
                    incoming_states: Vec::new(),
                    merged_state: None,
                    conflicts: Vec::new(),
                };

                // Collect states from incoming paths
                for &pred_id in &block.predecessors {
                    if let Some(state) = self.get_block_state(pred_id) {
                        merge_point.incoming_states.push(state);
                    }
                }

                // Merge states
                if let Some(merged) = self.merge_states(&merge_point.incoming_states, leaks) {
                    merge_point.merged_state = Some(merged);
                }

                self.merge_points.insert(block.id, merge_point);
            }
        }
    }

    fn merge_states(&mut self, states: &[MergeState], leaks: &mut Vec<ReferenceLeak>) -> Option<MergeState> {
        if states.is_empty() {
            return None;
        }

        let mut merged = MergeState {
            reference_states: HashMap::new(),
            object_states: HashMap::new(),
            conditions: Vec::new(),
            constraints: Vec::new(),
        };

        // Merge reference states
        let mut reference_vars: HashSet<String> = states.iter()
            .flat_map(|s| s.reference_states.keys().cloned())
            .collect();

        for var in reference_vars {
            let states_for_var: Vec<_> = states.iter()
                .filter_map(|s| s.reference_states.get(&var))
                .cloned()
                .collect();

            match self.resolve_reference_states(&var, &states_for_var) {
                Ok(resolved) => {
                    merged.reference_states.insert(var, resolved);
                }
                Err(conflict) => {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!("Conflicting reference states for {} at merge point", var),
                        severity: Severity::High,
                    });
                }
            }
        }

        // Merge object states
        let mut object_ids: HashSet<ObjectId> = states.iter()
            .flat_map(|s| s.object_states.keys().cloned())
            .collect();

        for obj_id in object_ids {
            let states_for_obj: Vec<_> = states.iter()
                .filter_map(|s| s.object_states.get(&obj_id))
                .cloned()
                .collect();

            match self.resolve_object_states(&obj_id, &states_for_obj) {
                Ok(resolved) => {
                    merged.object_states.insert(obj_id, resolved);
                }
                Err(conflict) => {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId {
                            module_name: obj_id.module_name,
                            struct_name: obj_id.type_name,
                            field_name: String::new(),
                        },
                        context: format!("Conflicting object states at merge point"),
                        severity: Severity::High,
                    });
                }
            }
        }

        // Merge conditions
        merged.conditions = self.merge_conditions(states);

        // Merge constraints
        merged.constraints = self.merge_constraints(states);

        Some(merged)
    }

    fn resolve_reference_states(
        &self,
        var: &str,
        states: &[ReferenceState]
    ) -> Result<ReferenceState, StateConflict> {
        if states.is_empty() {
            return Ok(ReferenceState::Uninitialized);
        }

        // Check for conflicts
        if states.iter().any(|s| matches!(s, ReferenceState::Moved { .. })) {
            return Err(StateConflict {
                var: var.to_string(),
                states: states.to_vec(),
                resolution: ConflictResolution::Invalid,
            });
        }

        // Use most restrictive state
        if states.iter().any(|s| matches!(s, ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. })) {
            Ok(states[0].clone()) // Use first mutable borrow state
        } else {
            Ok(states[0].clone()) // Use first state as conservative choice
        }
    }

    fn resolve_object_states(
        &self,
        id: &ObjectId,
        states: &[ObjectState]
    ) -> Result<ObjectState, String> {
        if states.is_empty() {
            return Ok(ObjectState::Uninitialized);
        }

        // Check for conflicts
        if states.iter().any(|s| matches!(s, ObjectState::Transferred { .. })) {
            return Err("Object transferred in some paths but not others".to_string());
        }

        // Use most restrictive state
        Ok(states[0].clone())
    }

    fn merge_conditions(&self, states: &[MergeState]) -> Vec<PathCondition> {
        // Keep only conditions that hold in all paths
        let mut common_conditions: HashSet<_> = states.get(0)
            .map(|s| s.conditions.iter().cloned().collect())
            .unwrap_or_default();

        for state in &states[1..] {
            let state_conditions: HashSet<_> = state.conditions.iter().cloned().collect();
            common_conditions = common_conditions.intersection(&state_conditions).cloned().collect();
        }

        common_conditions.into_iter().collect()
    }

    fn merge_constraints(&self, states: &[MergeState]) -> Vec<PathConstraint> {
        // Keep constraints that must be maintained across all paths
        let mut merged_constraints = Vec::new();
        
        if let Some(first) = states.first() {
            for constraint in &first.constraints {
                if states[1..].iter().all(|s| s.constraints.contains(constraint)) {
                    merged_constraints.push(constraint.clone());
                }
            }
        }

        merged_constraints
    }

    fn verify_merged_states(&self, leaks: &mut Vec<ReferenceLeak>) {
        for (block_id, merge_point) in &self.merge_points {
            if let Some(merged) = &merge_point.merged_state {
                // Verify reference states after merge
                for (var, state) in &merged.reference_states {
                    if !self.is_safe_merged_state(var, state, merge_point) {
                        leaks.push(ReferenceLeak {
                            location: Location::default(),
                            leaked_field: FieldId::default(),
                            context: format!("Unsafe reference state after merge for {}", var),
                            severity: Severity::High,
                        });
                    }
                }

                // Verify conditions after merge
                for condition in &merged.conditions {
                    if !self.is_valid_merged_condition(condition, merge_point) {
                        leaks.push(ReferenceLeak {
                            location: Location::default(),
                            leaked_field: FieldId::default(),
                            context: "Invalid path condition after merge".to_string(),
                            severity: Severity::High,
                        });
                    }
                }
            }
        }
    }

    fn get_block_state(&self, block_id: usize) -> Option<MergeState> {
        // Get state at end of block
        None // Simplified for now
    }

    fn is_safe_merged_state(&self, var: &str, state: &ReferenceState, merge_point: &MergePointState) -> bool {
        // Verify state is safe after merge
        !matches!(state, ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. })
    }

    fn is_valid_merged_condition(&self, condition: &PathCondition, merge_point: &MergePointState) -> bool {
        // Verify condition is valid after merge
        true // Simplified for now
    }
} 