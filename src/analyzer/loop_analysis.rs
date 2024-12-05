use super::types::*;
use super::control_flow::{ControlFlowGraph, BasicBlock};
use super::reference_transitions::ReferenceState;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct LoopAnalyzer {
    loops: Vec<LoopInfo>,
    reference_states: HashMap<String, LoopReferenceState>,
    invariants: Vec<LoopInvariant>,
    current_loop: Option<usize>,
}

#[derive(Debug)]
struct LoopInfo {
    id: usize,
    header: usize,
    body_blocks: HashSet<usize>,
    back_edges: Vec<(usize, usize)>,
    exit_blocks: HashSet<usize>,
    reference_mutations: Vec<ReferenceMutation>,
}

#[derive(Debug, Clone)]
struct LoopReferenceState {
    initial_state: ReferenceState,
    per_iteration_states: Vec<ReferenceState>,
    mutations: Vec<ReferenceMutation>,
    escapes: bool,
}

#[derive(Debug)]
struct LoopInvariant {
    condition: InvariantCondition,
    reference_vars: HashSet<String>,
    verified: bool,
}

#[derive(Debug, Clone)]
struct ReferenceMutation {
    var: String,
    from_state: ReferenceState,
    to_state: ReferenceState,
    location: Location,
    iteration: Option<usize>,
}

#[derive(Debug)]
enum InvariantCondition {
    NoEscape(String),
    StatePreserved(String),
    MutationBounded(String, usize),
    Custom(String),
}

impl LoopAnalyzer {
    pub fn new() -> Self {
        Self {
            loops: Vec::new(),
            reference_states: HashMap::new(),
            invariants: Vec::new(),
            current_loop: None,
        }
    }

    pub fn analyze_loops(&mut self, cfg: &ControlFlowGraph) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();

        // Detect loops in CFG
        self.detect_loops(cfg);

        // Analyze each loop
        for loop_info in &mut self.loops {
            self.current_loop = Some(loop_info.id);
            
            // Track reference states through iterations
            self.track_loop_references(loop_info, cfg, &mut leaks);
            
            // Verify loop invariants
            self.verify_loop_invariants(loop_info, &mut leaks);
            
            // Check for reference escapes
            self.check_loop_escapes(loop_info, &mut leaks);
        }

        self.current_loop = None;
        leaks
    }

    fn detect_loops(&mut self, cfg: &ControlFlowGraph) {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut loop_id = 0;

        // Use Tarjan's algorithm to find strongly connected components (loops)
        fn dfs(
            node: usize,
            cfg: &ControlFlowGraph,
            visited: &mut HashSet<usize>,
            stack: &mut Vec<usize>,
            loops: &mut Vec<LoopInfo>,
            loop_id: &mut usize,
        ) {
            visited.insert(node);
            stack.push(node);

            for &succ in &cfg.blocks[node].successors {
                if stack.contains(&succ) {
                    // Found a loop
                    let loop_start = stack.iter().position(|&x| x == succ).unwrap();
                    let loop_blocks: HashSet<_> = stack[loop_start..].iter().cloned().collect();
                    
                    let back_edges = vec![(node, succ)];
                    let exit_blocks = cfg.blocks[node].successors
                        .iter()
                        .filter(|&&b| !loop_blocks.contains(&b))
                        .cloned()
                        .collect();

                    loops.push(LoopInfo {
                        id: *loop_id,
                        header: succ,
                        body_blocks: loop_blocks,
                        back_edges,
                        exit_blocks,
                        reference_mutations: Vec::new(),
                    });
                    *loop_id += 1;
                } else if !visited.contains(&succ) {
                    dfs(succ, cfg, visited, stack, loops, loop_id);
                }
            }

            stack.pop();
        }

        dfs(cfg.entry_block, cfg, &mut visited, &mut stack, &mut self.loops, &mut loop_id);
    }

    fn track_loop_references(
        &mut self,
        loop_info: &mut LoopInfo,
        cfg: &ControlFlowGraph,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        // Track reference states through first few iterations
        const MAX_ITERATIONS: usize = 3;
        
        for iteration in 0..MAX_ITERATIONS {
            let mut current_states = self.reference_states.clone();
            
            // Analyze loop body
            for &block_id in &loop_info.body_blocks {
                let block = &cfg.blocks[block_id];
                self.analyze_block_references(block, iteration, &mut current_states, loop_info, leaks);
            }

            // Check for state changes
            for (var, state) in current_states {
                if let Some(prev_state) = self.reference_states.get(&var) {
                    if state.per_iteration_states.len() > prev_state.per_iteration_states.len() {
                        // Record mutation
                        loop_info.reference_mutations.push(ReferenceMutation {
                            var: var.clone(),
                            from_state: prev_state.initial_state.clone(),
                            to_state: state.initial_state.clone(),
                            location: Location::default(), // Would need proper location
                            iteration: Some(iteration),
                        });
                    }
                }
                self.reference_states.insert(var, state);
            }
        }
    }

    fn analyze_block_references(
        &self,
        block: &BasicBlock,
        iteration: usize,
        states: &mut HashMap<String, LoopReferenceState>,
        loop_info: &LoopInfo,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        for statement in &block.statements {
            match statement {
                Statement::Assignment(var, expr) => {
                    // Track reference state changes
                    if let Some(state) = states.get_mut(var) {
                        let new_state = self.evaluate_expression_in_loop(expr, states);
                        
                        // Check for unsafe mutations
                        if self.is_unsafe_loop_mutation(&state.initial_state, &new_state, iteration) {
                            leaks.push(ReferenceLeak {
                                location: Location::default(), // Would need proper location
                                leaked_field: FieldId::default(), // Would need proper field
                                context: format!(
                                    "Unsafe reference mutation in loop iteration {}",
                                    iteration
                                ),
                                severity: Severity::High,
                            });
                        }

                        state.per_iteration_states.push(new_state);
                    }
                }
                Statement::Return(expr) => {
                    // Check for references escaping through loop returns
                    if self.can_escape_through_loop_return(expr, states) {
                        leaks.push(ReferenceLeak {
                            location: Location::default(),
                            leaked_field: FieldId::default(),
                            context: "Reference may escape through loop return".to_string(),
                            severity: Severity::Critical,
                        });
                    }
                }
            }
        }
    }

    fn verify_loop_invariants(&mut self, loop_info: &LoopInfo, leaks: &mut Vec<ReferenceLeak>) {
        for invariant in &mut self.invariants {
            let mut verified = true;

            // Check invariant holds through all iterations
            for var in &invariant.reference_vars {
                if let Some(state) = self.reference_states.get(var) {
                    if !self.verify_invariant_for_state(&invariant.condition, state) {
                        verified = false;
                        leaks.push(ReferenceLeak {
                            location: Location::default(),
                            leaked_field: FieldId::default(),
                            context: format!(
                                "Loop invariant violation for reference {}",
                                var
                            ),
                            severity: Severity::High,
                        });
                    }
                }
            }

            invariant.verified = verified;
        }
    }

    fn check_loop_escapes(&self, loop_info: &LoopInfo, leaks: &mut Vec<ReferenceLeak>) {
        // Check for references escaping through loop exits
        for &exit_block in &loop_info.exit_blocks {
            for (var, state) in &self.reference_states {
                if state.escapes && self.can_reach_exit(exit_block, loop_info) {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!(
                            "Reference {} may escape through loop exit",
                            var
                        ),
                        severity: Severity::High,
                    });
                }
            }
        }
    }

    fn evaluate_expression_in_loop(
        &self,
        expr: &Expression,
        states: &HashMap<String, LoopReferenceState>
    ) -> ReferenceState {
        match expr {
            Expression::Variable(name) => {
                states.get(name)
                    .map(|s| s.initial_state.clone())
                    .unwrap_or(ReferenceState::Uninitialized)
            }
            Expression::FieldAccess(base, _) => {
                // Analyze field access in loop context
                self.evaluate_expression_in_loop(base, states)
            }
        }
    }

    fn is_unsafe_loop_mutation(
        &self,
        initial_state: &ReferenceState,
        new_state: &ReferenceState,
        iteration: usize
    ) -> bool {
        match (initial_state, new_state) {
            (ReferenceState::Borrowed { .. }, ReferenceState::Moved { .. }) => true,
            _ => false
        }
    }

    fn can_escape_through_loop_return(
        &self,
        expr: &Expression,
        states: &HashMap<String, LoopReferenceState>
    ) -> bool {
        match expr {
            Expression::Variable(name) => {
                states.get(name)
                    .map(|s| s.escapes)
                    .unwrap_or(false)
            }
            _ => false
        }
    }

    fn verify_invariant_for_state(
        &self,
        condition: &InvariantCondition,
        state: &LoopReferenceState
    ) -> bool {
        match condition {
            InvariantCondition::NoEscape(_) => !state.escapes,
            InvariantCondition::StatePreserved(_) => {
                state.per_iteration_states.iter()
                    .all(|s| s == &state.initial_state)
            }
            InvariantCondition::MutationBounded(_, bound) => {
                state.mutations.len() <= *bound
            }
            InvariantCondition::Custom(_) => true // Custom verification
        }
    }

    fn can_reach_exit(&self, exit_block: usize, loop_info: &LoopInfo) -> bool {
        // Check if exit block is reachable from loop body
        loop_info.body_blocks.contains(&exit_block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_detection() {
        // Create a simple CFG with a loop
        let mut cfg = ControlFlowGraph::new();
        // Add test implementation
    }

    #[test]
    fn test_reference_tracking_in_loop() {
        let mut analyzer = LoopAnalyzer::new();
        // Add test implementation
    }
} 