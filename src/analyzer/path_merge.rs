use super::types::*;
use super::control_flow::{ControlFlowGraph, BasicBlock};
use super::path_conditions::{PathCondition, PathConditionAnalyzer};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct PathMergeAnalyzer {
    path_states: HashMap<usize, PathState>,
    merge_points: HashMap<usize, MergePoint>,
    condition_analyzer: PathConditionAnalyzer,
    current_path: Option<usize>,
}

#[derive(Debug, Clone)]
struct PathState {
    reference_states: HashMap<String, ReferenceState>,
    object_states: HashMap<ObjectId, ObjectState>,
    conditions: Vec<PathCondition>,
    constraints: Vec<PathConstraint>,
}

#[derive(Debug)]
struct MergePoint {
    block_id: usize,
    incoming_paths: Vec<usize>,
    merged_state: PathState,
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

#[derive(Debug)]
struct PathConstraint {
    condition: PathCondition,
    required_state: ReferenceState,
    location: Location,
}

impl PathMergeAnalyzer {
    pub fn new() -> Self {
        Self {
            path_states: HashMap::new(),
            merge_points: HashMap::new(),
            condition_analyzer: PathConditionAnalyzer::new(),
            current_path: None,
        }
    }

    pub fn analyze_paths(&mut self, cfg: &ControlFlowGraph) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // First identify merge points
        self.identify_merge_points(cfg);
        
        // Analyze paths to merge points
        self.analyze_path_flows(cfg, &mut leaks);
        
        // Verify merged states
        self.verify_merged_states(&mut leaks);
        
        leaks
    }

    fn identify_merge_points(&mut self, cfg: &ControlFlowGraph) {
        for (block_id, block) in cfg.blocks.iter().enumerate() {
            if block.predecessors.len() > 1 {
                // This is a merge point
                self.merge_points.insert(block_id, MergePoint {
                    block_id,
                    incoming_paths: block.predecessors.iter().cloned().collect(),
                    merged_state: PathState::new(),
                    conflicts: Vec::new(),
                });
            }
        }
    }

    fn analyze_path_flows(&mut self, cfg: &ControlFlowGraph, leaks: &mut Vec<ReferenceLeak>) {
        let mut work_list = VecDeque::new();
        let mut visited = HashSet::new();
        
        // Start from entry block
        work_list.push_back((cfg.entry_block, PathState::new()));
        
        while let Some((block_id, mut state)) = work_list.pop_front() {
            // Skip if we've seen this exact state at this block
            let state_key = self.get_state_key(block_id, &state);
            if !visited.insert(state_key) {
                continue;
            }

            // Analyze block with current state
            self.analyze_block_flow(&cfg.blocks[block_id], &mut state, leaks);
            
            // Store state for this path
            self.path_states.insert(block_id, state.clone());
            
            // If this is a merge point, handle merging
            if let Some(merge_point) = self.merge_points.get_mut(&block_id) {
                self.handle_merge_point(merge_point, &state, leaks);
            }

            // Add successors to worklist
            for &succ_id in &cfg.blocks[block_id].successors {
                let next_state = if self.merge_points.contains_key(&succ_id) {
                    // Use merged state for merge points
                    self.merge_points[&succ_id].merged_state.clone()
                } else {
                    state.clone()
                };
                work_list.push_back((succ_id, next_state));
            }
        }
    }

    fn analyze_block_flow(&mut self, block: &BasicBlock, state: &mut PathState, leaks: &mut Vec<ReferenceLeak>) {
        for statement in &block.statements {
            match statement {
                Statement::Assignment(var, expr) => {
                    self.analyze_assignment_flow(var, expr, state, leaks);
                }
                Statement::Return(expr) => {
                    self.analyze_return_flow(expr, state, leaks);
                }
            }
        }
    }

    fn analyze_assignment_flow(
        &mut self,
        var: &str,
        expr: &Expression,
        state: &mut PathState,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        let new_state = self.evaluate_expression_flow(expr, state);
        
        // Check for state conflicts
        if let Some(current_state) = state.reference_states.get(var) {
            if !self.is_compatible_state(current_state, &new_state) {
                leaks.push(ReferenceLeak {
                    location: Location::default(),
                    leaked_field: FieldId::default(),
                    context: format!("Incompatible state transition for {} in path", var),
                    severity: Severity::High,
                });
            }
        }
        
        // Update state
        state.reference_states.insert(var.to_string(), new_state);
    }

    fn analyze_return_flow(
        &mut self,
        expr: &Expression,
        state: &PathState,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        let return_state = self.evaluate_expression_flow(expr, state);
        
        // Check for reference leaks in return paths
        if let ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. } = return_state {
            leaks.push(ReferenceLeak {
                location: Location::default(),
                leaked_field: FieldId::default(),
                context: "Mutable reference may escape through return path".to_string(),
                severity: Severity::Critical,
            });
        }
    }

    fn handle_merge_point(
        &mut self,
        merge_point: &mut MergePoint,
        incoming_state: &PathState,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        // Collect all states for each reference
        let mut reference_states: HashMap<String, Vec<ReferenceState>> = HashMap::new();
        
        for var in incoming_state.reference_states.keys() {
            let states: Vec<_> = merge_point.incoming_paths.iter()
                .filter_map(|path_id| self.path_states.get(path_id))
                .filter_map(|state| state.reference_states.get(var))
                .cloned()
                .collect();
            
            if !states.is_empty() {
                reference_states.insert(var.clone(), states);
            }
        }

        // Resolve conflicts and merge states
        for (var, states) in reference_states {
            match self.resolve_state_conflict(&var, &states) {
                ConflictResolution::Conservative => {
                    // Use most conservative state
                    if let Some(merged) = self.get_conservative_state(&states) {
                        merge_point.merged_state.reference_states.insert(var.clone(), merged);
                    }
                }
                ConflictResolution::MostRestrictive => {
                    // Use most restrictive state
                    if let Some(merged) = self.get_restrictive_state(&states) {
                        merge_point.merged_state.reference_states.insert(var.clone(), merged);
                    }
                }
                ConflictResolution::Invalid => {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!("Invalid state merge for {} at path merge point", var),
                        severity: Severity::Critical,
                    });
                }
            }
        }
    }

    fn verify_merged_states(&self, leaks: &mut Vec<ReferenceLeak>) {
        for merge_point in self.merge_points.values() {
            // Verify reference states after merge
            for (var, state) in &merge_point.merged_state.reference_states {
                if !self.is_safe_merged_state(var, state, merge_point) {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!("Unsafe reference state after path merge for {}", var),
                        severity: Severity::High,
                    });
                }
            }

            // Verify conditions after merge
            for condition in &merge_point.merged_state.conditions {
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

    // Helper methods
    fn evaluate_expression_flow(&self, expr: &Expression, state: &PathState) -> ReferenceState {
        match expr {
            Expression::Variable(name) => {
                state.reference_states.get(name)
                    .cloned()
                    .unwrap_or(ReferenceState::Uninitialized)
            }
            Expression::FieldAccess(base, _) => {
                // Analyze field access in flow context
                self.evaluate_expression_flow(base, state)
            }
        }
    }

    fn is_compatible_state(&self, state1: &ReferenceState, state2: &ReferenceState) -> bool {
        match (state1, state2) {
            (ReferenceState::Borrowed { .. }, ReferenceState::Moved { .. }) => false,
            _ => true,
        }
    }

    fn resolve_state_conflict(&self, var: &str, states: &[ReferenceState]) -> ConflictResolution {
        // Determine how to resolve conflicting states
        if states.iter().any(|s| matches!(s, ReferenceState::Moved { .. })) {
            ConflictResolution::Invalid
        } else if states.iter().any(|s| matches!(s, ReferenceState::Borrowed { .. })) {
            ConflictResolution::MostRestrictive
        } else {
            ConflictResolution::Conservative
        }
    }

    fn get_conservative_state(&self, states: &[ReferenceState]) -> Option<ReferenceState> {
        // Get most conservative state (least restrictive)
        states.first().cloned()
    }

    fn get_restrictive_state(&self, states: &[ReferenceState]) -> Option<ReferenceState> {
        // Get most restrictive state
        states.iter()
            .find(|s| matches!(s, ReferenceState::Borrowed { .. }))
            .cloned()
    }

    fn is_safe_merged_state(&self, var: &str, state: &ReferenceState, merge_point: &MergePoint) -> bool {
        // Verify state is safe after merge
        !matches!(state, ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. })
    }

    fn is_valid_merged_condition(&self, condition: &PathCondition, merge_point: &MergePoint) -> bool {
        // Verify condition is valid after merge
        true // Simplified for now
    }

    fn get_state_key(&self, block_id: usize, state: &PathState) -> String {
        // Create unique key for state at block
        format!("{}:{:?}", block_id, state.reference_states.keys().collect::<Vec<_>>())
    }
}

impl PathState {
    fn new() -> Self {
        Self {
            reference_states: HashMap::new(),
            object_states: HashMap::new(),
            conditions: Vec::new(),
            constraints: Vec::new(),
        }
    }
} 