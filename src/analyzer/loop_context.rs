use super::types::*;
use super::loop_analysis::LoopAnalyzer;
use super::loop_state::LoopStateAnalyzer;
use super::loop_reference::LoopReferenceAnalyzer;
use std::collections::{HashMap, HashSet};

pub struct LoopContextAnalyzer {
    loop_analyzer: LoopAnalyzer,
    state_analyzer: LoopStateAnalyzer,
    reference_analyzer: LoopReferenceAnalyzer,
    contexts: HashMap<usize, LoopContext>,
    current_loop: Option<usize>,
}

#[derive(Debug)]
struct LoopContext {
    id: usize,
    references: HashMap<String, ReferenceContext>,
    invariants: Vec<LoopInvariant>,
    iterations: Vec<IterationContext>,
    escapes: HashSet<String>,
}

#[derive(Debug)]
struct ReferenceContext {
    initial_state: ReferenceState,
    per_iteration_states: Vec<ReferenceState>,
    mutations: Vec<ReferenceMutation>,
    access_paths: Vec<AccessPath>,
}

#[derive(Debug)]
struct IterationContext {
    iteration: usize,
    reference_states: HashMap<String, ReferenceState>,
    borrows: HashSet<BorrowInfo>,
    mutations: Vec<ReferenceMutation>,
}

#[derive(Debug)]
struct LoopInvariant {
    condition: InvariantCondition,
    affected_refs: HashSet<String>,
    verified: bool,
}

#[derive(Debug)]
enum InvariantCondition {
    NoEscape(String),
    StatePreserved(String),
    MutationBounded(String, usize),
    Custom(String),
}

impl LoopContextAnalyzer {
    pub fn new() -> Self {
        Self {
            loop_analyzer: LoopAnalyzer::new(),
            state_analyzer: LoopStateAnalyzer::new(),
            reference_analyzer: LoopReferenceAnalyzer::new(),
            contexts: HashMap::new(),
            current_loop: None,
        }
    }

    pub fn analyze_function(&mut self, function: &Function) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // First analyze loop structure
        let loop_leaks = self.loop_analyzer.analyze_loops(function);
        leaks.extend(loop_leaks);
        
        // Track reference states through loops
        let state_leaks = self.state_analyzer.analyze_loop(function);
        leaks.extend(state_leaks);
        
        // Analyze reference patterns in loops
        let ref_leaks = self.reference_analyzer.analyze_function(function);
        leaks.extend(ref_leaks);
        
        // Analyze each loop context
        for (loop_id, context) in &mut self.contexts {
            self.current_loop = Some(*loop_id);
            
            // Verify loop invariants
            self.verify_loop_invariants(context, &mut leaks);
            
            // Check reference safety across iterations
            self.check_iteration_safety(context, &mut leaks);
            
            // Check for escaping references
            self.check_loop_escapes(context, &mut leaks);
        }
        
        self.current_loop = None;
        leaks
    }

    fn verify_loop_invariants(&self, context: &LoopContext, leaks: &mut Vec<ReferenceLeak>) {
        for invariant in &context.invariants {
            if !invariant.verified {
                for var in &invariant.affected_refs {
                    if let Some(ref_context) = context.references.get(var) {
                        match &invariant.condition {
                            InvariantCondition::NoEscape(var) => {
                                if context.escapes.contains(var) {
                                    leaks.push(ReferenceLeak {
                                        location: Location::default(),
                                        leaked_field: FieldId::default(),
                                        context: format!("Loop invariant violation: {} escapes", var),
                                        severity: Severity::High,
                                    });
                                }
                            }
                            InvariantCondition::StatePreserved(var) => {
                                if !self.has_consistent_states(ref_context) {
                                    leaks.push(ReferenceLeak {
                                        location: Location::default(),
                                        leaked_field: FieldId::default(),
                                        context: format!("Loop invariant violation: {} state not preserved", var),
                                        severity: Severity::High,
                                    });
                                }
                            }
                            InvariantCondition::MutationBounded(var, bound) => {
                                if ref_context.mutations.len() > *bound {
                                    leaks.push(ReferenceLeak {
                                        location: Location::default(),
                                        leaked_field: FieldId::default(),
                                        context: format!("Loop invariant violation: {} mutations exceed bound", var),
                                        severity: Severity::High,
                                    });
                                }
                            }
                            InvariantCondition::Custom(_) => {}
                        }
                    }
                }
            }
        }
    }

    fn check_iteration_safety(&self, context: &LoopContext, leaks: &mut Vec<ReferenceLeak>) {
        for iteration in &context.iterations {
            // Check for unsafe mutations across iterations
            for mutation in &iteration.mutations {
                if self.is_unsafe_mutation(mutation, context) {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!(
                            "Unsafe reference mutation in loop iteration {}",
                            iteration.iteration
                        ),
                        severity: Severity::High,
                    });
                }
            }

            // Check for unsafe borrows
            for borrow in &iteration.borrows {
                if self.is_unsafe_cross_iteration_borrow(borrow, context) {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: "Unsafe cross-iteration borrow".to_string(),
                        severity: Severity::High,
                    });
                }
            }
        }
    }

    fn check_loop_escapes(&self, context: &LoopContext, leaks: &mut Vec<ReferenceLeak>) {
        for var in &context.escapes {
            if let Some(ref_context) = context.references.get(var) {
                leaks.push(ReferenceLeak {
                    location: Location::default(),
                    leaked_field: FieldId::default(),
                    context: format!("Reference {} escapes loop", var),
                    severity: Severity::High,
                });
            }
        }
    }

    fn has_consistent_states(&self, context: &ReferenceContext) -> bool {
        if context.per_iteration_states.len() < 2 {
            return true;
        }
        let first = &context.per_iteration_states[0];
        context.per_iteration_states.iter().all(|s| s == first)
    }

    fn is_unsafe_mutation(&self, mutation: &ReferenceMutation, context: &LoopContext) -> bool {
        // Check if mutation is safe given loop context
        match (&mutation.from_state, &mutation.to_state) {
            (ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. }, _) => true,
            _ => false,
        }
    }

    fn is_unsafe_cross_iteration_borrow(&self, borrow: &BorrowInfo, context: &LoopContext) -> bool {
        matches!(borrow.kind, BorrowKind::MutableWrite)
    }
} 