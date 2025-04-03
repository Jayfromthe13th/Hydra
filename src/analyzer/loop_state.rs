use super::types::*;
use super::loop_analysis::LoopAnalyzer;
use super::loop_reference::LoopReferenceAnalyzer;
use std::collections::{HashMap, HashSet};

pub struct LoopStateAnalyzer {
    reference_analyzer: LoopReferenceAnalyzer,
    loop_analyzer: LoopAnalyzer,
    loop_states: HashMap<usize, LoopState>,
    reference_states: HashMap<String, ReferenceLoopState>,
    current_loop: Option<usize>,
}

#[derive(Debug)]
struct LoopState {
    id: usize,
    iteration_states: Vec<IterationState>,
    reference_mutations: Vec<ReferenceMutation>,
    invariants: Vec<LoopInvariant>,
    escapes: HashSet<String>,
}

#[derive(Debug)]
struct IterationState {
    iteration: usize,
    reference_states: HashMap<String, ReferenceState>,
    borrows: HashSet<BorrowInfo>,
    mutations: Vec<ReferenceMutation>,
}

#[derive(Debug, Clone)]
struct ReferenceLoopState {
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

impl LoopStateAnalyzer {
    pub fn new() -> Self {
        Self {
            reference_analyzer: LoopReferenceAnalyzer::new(),
            loop_analyzer: LoopAnalyzer::new(),
            loop_states: HashMap::new(),
            reference_states: HashMap::new(),
            current_loop: None,
        }
    }

    pub fn analyze_loop(&mut self, function: &Function) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();

        // First analyze loop structure
        let loop_leaks = self.loop_analyzer.analyze_loops(function);
        leaks.extend(loop_leaks);

        // Track references through iterations
        self.track_loop_references(function, &mut leaks);

        // Verify reference safety across iterations
        self.verify_iteration_safety(&mut leaks);

        // Check for escaping references
        self.check_loop_escapes(&mut leaks);

        leaks
    }

    fn track_loop_references(&mut self, function: &Function, leaks: &mut Vec<ReferenceLeak>) {
        // Track through a fixed number of iterations to detect patterns
        const MAX_ITERATIONS: usize = 3;

        for iteration in 0..MAX_ITERATIONS {
            let mut iteration_state = IterationState {
                iteration,
                reference_states: HashMap::new(),
                borrows: HashSet::new(),
                mutations: Vec::new(),
            };

            // Analyze statements in loop body
            for statement in &function.body {
                self.analyze_statement(statement, &mut iteration_state, leaks);
            }

            // Record iteration state
            if let Some(loop_id) = self.current_loop {
                let loop_state = self.loop_states.entry(loop_id).or_insert_with(|| LoopState {
                    id: loop_id,
                    iteration_states: Vec::new(),
                    reference_mutations: Vec::new(),
                    invariants: Vec::new(),
                    escapes: HashSet::new(),
                });

                loop_state.iteration_states.push(iteration_state);

                // Check for stabilization of reference patterns
                if self.has_stable_pattern(loop_id) {
                    break;
                }
            }
        }
    }

    fn analyze_statement(
        &mut self,
        statement: &Statement,
        iteration_state: &mut IterationState,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        match statement {
            Statement::Assignment(var, expr) => {
                self.analyze_assignment(var, expr, iteration_state, leaks);
            }
            Statement::Return(expr) => {
                self.analyze_return(expr, iteration_state, leaks);
            }
        }
    }

    fn analyze_assignment(
        &mut self,
        var: &str,
        expr: &Expression,
        iteration_state: &mut IterationState,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        let new_state = self.evaluate_expression(expr, iteration_state);

        // Check for reference mutations across iterations
        if let Some(prev_state) = iteration_state.reference_states.get(var) {
            let mutation = ReferenceMutation {
                var: var.to_string(),
                from_state: prev_state.clone(),
                to_state: new_state.clone(),
                location: Location::default(), // Would need proper location
                iteration: iteration_state.iteration,
            };

            // Check if mutation is safe
            if !self.is_safe_mutation(&mutation) {
                leaks.push(ReferenceLeak {
                    location: Location::default(),
                    leaked_field: FieldId::default(), // Would need proper field
                    context: format!(
                        "Unsafe reference mutation in loop iteration {}",
                        iteration_state.iteration
                    ),
                    severity: Severity::High,
                });
            }

            iteration_state.mutations.push(mutation);
        }

        // Update state
        iteration_state.reference_states.insert(var.to_string(), new_state);
    }

    fn analyze_return(
        &mut self,
        expr: &Expression,
        iteration_state: &mut IterationState,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        let return_state = self.evaluate_expression(expr, iteration_state);

        // Check for references escaping through return in loop
        if self.can_escape_through_return(&return_state) {
            leaks.push(ReferenceLeak {
                location: Location::default(),
                leaked_field: FieldId::default(),
                context: "Reference may escape through loop return".to_string(),
                severity: Severity::Critical,
            });
        }
    }

    fn evaluate_expression(
        &self,
        expr: &Expression,
        iteration_state: &IterationState
    ) -> ReferenceState {
        match expr {
            Expression::Variable(name) => {
                iteration_state.reference_states
                    .get(name)
                    .cloned()
                    .unwrap_or(ReferenceState::Uninitialized)
            }
            Expression::FieldAccess(base, _) => {
                // Analyze field access in loop context
                self.evaluate_expression(base, iteration_state)
            }
        }
    }

    fn verify_iteration_safety(&self, leaks: &mut Vec<ReferenceLeak>) {
        // Check reference state consistency across iterations
        for (var, states) in &self.reference_states {
            if !self.has_consistent_states(states) {
                leaks.push(ReferenceLeak {
                    location: Location::default(),
                    leaked_field: FieldId::default(),
                    context: format!(
                        "Reference {} has inconsistent states across loop iterations",
                        var
                    ),
                    severity: Severity::High,
                });
            }
        }

        // Verify borrow safety across iterations
        for loop_state in self.loop_states.values() {
            for iteration_state in &loop_state.iteration_states {
                for borrow in &iteration_state.borrows {
                    if self.is_unsafe_cross_iteration_borrow(borrow) {
                        leaks.push(ReferenceLeak {
                            location: Location::default(),
                            leaked_field: FieldId::default(),
                            context: format!(
                                "Unsafe cross-iteration borrow in loop iteration {}",
                                iteration_state.iteration
                            ),
                            severity: Severity::High,
                        });
                    }
                }
            }
        }
    }

    fn check_loop_escapes(&self, leaks: &mut Vec<ReferenceLeak>) {
        for (loop_id, loop_state) in &self.loop_states {
            for var in &loop_state.escapes {
                leaks.push(ReferenceLeak {
                    location: Location::default(),
                    leaked_field: FieldId::default(),
                    context: format!("Reference {} may escape loop {}", var, loop_id),
                    severity: Severity::High,
                });
            }
        }
    }

    fn has_stable_pattern(&self, loop_id: usize) -> bool {
        if let Some(loop_state) = self.loop_states.get(&loop_id) {
            if loop_state.iteration_states.len() < 2 {
                return false;
            }

            let last = loop_state.iteration_states.last().unwrap();
            let prev = &loop_state.iteration_states[loop_state.iteration_states.len() - 2];

            // Compare states
            for (var, state) in &last.reference_states {
                if let Some(prev_state) = prev.reference_states.get(var) {
                    if state != prev_state {
                        return false;
                    }
                }
            }

            true
        } else {
            false
        }
    }

    fn is_safe_mutation(&self, mutation: &ReferenceMutation) -> bool {
        match (&mutation.from_state, &mutation.to_state) {
            (ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. }, _) => false,
            _ => true,
        }
    }

    fn can_escape_through_return(&self, state: &ReferenceState) -> bool {
        matches!(state,
            ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. } |
            ReferenceState::Moved { .. }
        )
    }

    fn has_consistent_states(&self, states: &ReferenceLoopState) -> bool {
        if states.per_iteration_states.len() < 2 {
            return true;
        }

        let first = &states.per_iteration_states[0];
        states.per_iteration_states.iter().all(|s| s == first)
    }

    fn is_unsafe_cross_iteration_borrow(&self, borrow: &BorrowInfo) -> bool {
        matches!(borrow.kind, BorrowKind::MutableWrite)
    }
} 