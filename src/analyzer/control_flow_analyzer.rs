use super::types::*;
use super::loop_analysis::LoopAnalyzer;
use super::loop_reference::LoopReferenceAnalyzer;
use super::loop_state::LoopStateAnalyzer;
use super::loop_context::LoopContextAnalyzer;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct ControlFlowAnalyzer {
    loop_analyzer: LoopAnalyzer,
    reference_analyzer: LoopReferenceAnalyzer,
    state_analyzer: LoopStateAnalyzer,
    context_analyzer: LoopContextAnalyzer,
    cfg: ControlFlowGraph,
    visited_blocks: HashSet<usize>,
    block_states: HashMap<usize, BlockState>,
}

#[derive(Debug, Clone)]
struct BlockState {
    reference_states: HashMap<String, ReferenceState>,
    object_states: HashMap<ObjectId, ObjectState>,
    conditions: Vec<PathCondition>,
    mutations: Vec<Mutation>,
}

#[derive(Debug, Clone)]
enum Mutation {
    Assignment(String, AbstractValue),
    FieldAccess(FieldId),
    Transfer(ObjectId),
}

impl ControlFlowAnalyzer {
    pub fn new(cfg: ControlFlowGraph) -> Self {
        Self {
            loop_analyzer: LoopAnalyzer::new(),
            reference_analyzer: LoopReferenceAnalyzer::new(),
            state_analyzer: LoopStateAnalyzer::new(),
            context_analyzer: LoopContextAnalyzer::new(),
            cfg,
            visited_blocks: HashSet::new(),
            block_states: HashMap::new(),
        }
    }

    pub fn analyze_control_flow(&mut self) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // First analyze loops
        leaks.extend(self.analyze_loops());
        
        // Then analyze paths through CFG
        leaks.extend(self.analyze_paths());
        
        // Finally verify state consistency
        leaks.extend(self.verify_states());
        
        leaks
    }

    fn analyze_loops(&mut self) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // Detect and analyze loops
        let loop_leaks = self.loop_analyzer.analyze_loops(&self.cfg);
        leaks.extend(loop_leaks);
        
        // Analyze references in loops
        let ref_leaks = self.reference_analyzer.analyze_loop(&self.cfg);
        leaks.extend(ref_leaks);
        
        // Analyze states in loops
        let state_leaks = self.state_analyzer.analyze_loop(&self.cfg);
        leaks.extend(state_leaks);
        
        // Analyze contexts in loops
        let context_leaks = self.context_analyzer.analyze_loop(&self.cfg);
        leaks.extend(context_leaks);
        
        leaks
    }

    fn analyze_paths(&mut self) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        let mut work_list = VecDeque::new();
        
        // Start from entry block
        work_list.push_back((self.cfg.entry_block, BlockState::new()));
        
        while let Some((block_id, mut state)) = work_list.pop_front() {
            // Skip if already visited with same state
            if !self.should_visit_block(block_id, &state) {
                continue;
            }
            
            // Mark as visited
            self.visited_blocks.insert(block_id);
            self.block_states.insert(block_id, state.clone());
            
            // Analyze block
            let block = &self.cfg.blocks[block_id];
            self.analyze_block(block, &mut state, &mut leaks);
            
            // Add successors to worklist
            for &succ_id in &block.successors {
                work_list.push_back((succ_id, state.clone()));
            }
        }
        
        leaks
    }

    fn analyze_block(&mut self, block: &BasicBlock, state: &mut BlockState, leaks: &mut Vec<ReferenceLeak>) {
        // Analyze each statement in block
        for statement in &block.statements {
            self.analyze_statement(statement, state, leaks);
        }
        
        // Check state consistency at block boundaries
        self.check_block_boundaries(block, state, leaks);
    }

    fn analyze_statement(&mut self, statement: &Statement, state: &mut BlockState, leaks: &mut Vec<ReferenceLeak>) {
        match statement {
            Statement::Assignment(var, expr) => {
                self.analyze_assignment(var, expr, state, leaks);
            }
            Statement::Return(expr) => {
                self.analyze_return(expr, state, leaks);
            }
        }
    }

    fn analyze_assignment(&mut self, var: &str, expr: &Expression, state: &mut BlockState, leaks: &mut Vec<ReferenceLeak>) {
        let new_value = self.evaluate_expression(expr, state);
        
        // Check for reference safety
        if let AbstractValue::InvRef(field) = &new_value {
            if self.is_unsafe_assignment(var, expr, state) {
                leaks.push(ReferenceLeak {
                    location: Location::default(),
                    leaked_field: field.clone(),
                    context: format!("Unsafe reference assignment to {}", var),
                    severity: Severity::High,
                });
            }
        }
        
        // Update state
        state.reference_states.insert(var.to_string(), ReferenceState::Initialized {
            value: new_value,
            mutable: true,
        });
        
        // Record mutation
        state.mutations.push(Mutation::Assignment(
            var.to_string(),
            new_value,
        ));
    }

    fn analyze_return(&mut self, expr: &Expression, state: &mut BlockState, leaks: &mut Vec<ReferenceLeak>) {
        let return_value = self.evaluate_expression(expr, state);
        
        // Check for reference leaks
        if let AbstractValue::InvRef(field) = return_value {
            leaks.push(ReferenceLeak {
                location: Location::default(),
                leaked_field: field,
                context: "Reference escapes through return".to_string(),
                severity: Severity::Critical,
            });
        }
    }

    fn evaluate_expression(&self, expr: &Expression, state: &BlockState) -> AbstractValue {
        match expr {
            Expression::Variable(name) => {
                state.reference_states.get(name)
                    .map(|s| match s {
                        ReferenceState::Initialized { value, .. } => value.clone(),
                        _ => AbstractValue::NonRef,
                    })
                    .unwrap_or(AbstractValue::NonRef)
            }
            Expression::FieldAccess(base, field) => {
                let base_value = self.evaluate_expression(base, state);
                match base_value {
                    AbstractValue::ObjectRef(obj_id) => {
                        AbstractValue::InvRef(FieldId {
                            module_name: obj_id.module_name,
                            struct_name: obj_id.type_name,
                            field_name: field.clone(),
                        })
                    }
                    _ => AbstractValue::NonRef,
                }
            }
        }
    }

    fn is_unsafe_assignment(&self, var: &str, expr: &Expression, state: &BlockState) -> bool {
        // Check various unsafe patterns
        match expr {
            Expression::Variable(name) => {
                // Check if source is already moved or borrowed
                if let Some(source_state) = state.reference_states.get(name) {
                    matches!(source_state,
                        ReferenceState::Moved { .. } |
                        ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. }
                    )
                } else {
                    false
                }
            }
            Expression::FieldAccess(_, _) => {
                // Check if field access is safe
                false // Simplified for now
            }
        }
    }

    fn check_block_boundaries(&self, block: &BasicBlock, state: &BlockState, leaks: &mut Vec<ReferenceLeak>) {
        // Check reference state consistency at block boundaries
        for (var, ref_state) in &state.reference_states {
            if self.is_unsafe_at_boundary(var, ref_state, block) {
                leaks.push(ReferenceLeak {
                    location: Location::default(),
                    leaked_field: FieldId::default(),
                    context: format!("Unsafe reference state at block boundary for {}", var),
                    severity: Severity::High,
                });
            }
        }
    }

    fn is_unsafe_at_boundary(&self, var: &str, state: &ReferenceState, block: &BasicBlock) -> bool {
        // Check if reference state is safe at block boundary
        matches!(state,
            ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. } |
            ReferenceState::Moved { .. }
        )
    }

    fn should_visit_block(&self, block_id: usize, state: &BlockState) -> bool {
        if let Some(existing_state) = self.block_states.get(&block_id) {
            // Visit if state has changed
            state != existing_state
        } else {
            // Visit if not seen before
            true
        }
    }

    fn verify_states(&self) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // Verify reference state consistency across all paths
        for (block_id, state) in &self.block_states {
            for (var, ref_state) in &state.reference_states {
                if self.is_inconsistent_state(var, ref_state, block_id) {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!(
                            "Inconsistent reference state for {} in block {}",
                            var, block_id
                        ),
                        severity: Severity::High,
                    });
                }
            }
        }
        
        leaks
    }

    fn is_inconsistent_state(&self, var: &str, state: &ReferenceState, block_id: usize) -> bool {
        // Check if state is consistent with all paths leading to this block
        if let Some(block) = self.cfg.blocks.get(block_id) {
            for &pred_id in &block.predecessors {
                if let Some(pred_state) = self.block_states.get(&pred_id) {
                    if let Some(pred_ref_state) = pred_state.reference_states.get(var) {
                        if !self.is_valid_transition(pred_ref_state, state) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_valid_transition(&self, from: &ReferenceState, to: &ReferenceState) -> bool {
        // Check if state transition is valid
        match (from, to) {
            (ReferenceState::Borrowed { .. }, ReferenceState::Moved { .. }) => false,
            _ => true,
        }
    }
}

impl BlockState {
    fn new() -> Self {
        Self {
            reference_states: HashMap::new(),
            object_states: HashMap::new(),
            conditions: Vec::new(),
            mutations: Vec::new(),
        }
    }
} 