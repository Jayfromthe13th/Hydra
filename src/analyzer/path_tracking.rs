use super::types::*;
use super::reference_transitions::{ReferenceState, BorrowKind};
use super::control_flow::{ControlFlowGraph, BasicBlock};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct PathTracker {
    paths: Vec<ReferencePath>,
    current_path: Option<usize>,
    reference_states: HashMap<String, PathState>,
    visited_paths: HashSet<PathKey>,
}

#[derive(Debug, Clone)]
struct ReferencePath {
    blocks: Vec<usize>,
    conditions: Vec<PathCondition>,
    state_transitions: Vec<StateTransition>,
    active_borrows: HashSet<BorrowInfo>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct PathKey {
    blocks: Vec<usize>,
    conditions: Vec<PathCondition>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum PathCondition {
    BorrowActive(String),
    ReferenceValid(String),
    ObjectOwned(ObjectId),
    CapabilityHeld(CapId),
}

#[derive(Debug, Clone)]
struct PathState {
    current_state: ReferenceState,
    conditions: Vec<PathCondition>,
    transitions: Vec<StateTransition>,
}

#[derive(Debug, Clone)]
struct StateTransition {
    from: ReferenceState,
    to: ReferenceState,
    location: Location,
    cause: TransitionCause,
}

#[derive(Debug, Clone)]
struct BorrowInfo {
    var: String,
    kind: BorrowKind,
    location: Location,
    path_id: usize,
}

impl PathTracker {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            current_path: None,
            reference_states: HashMap::new(),
            visited_paths: HashSet::new(),
        }
    }

    pub fn analyze_paths(&mut self, cfg: &ControlFlowGraph) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        let mut work_list = VecDeque::new();

        // Start with entry block
        let initial_path = ReferencePath::new(cfg.entry_block);
        work_list.push_back((cfg.entry_block, initial_path));

        while let Some((block_id, mut path)) = work_list.pop_front() {
            let path_key = path.get_key();
            if self.visited_paths.contains(&path_key) {
                continue;
            }
            self.visited_paths.insert(path_key);

            // Analyze current path
            self.current_path = Some(self.paths.len());
            self.analyze_block_in_path(&cfg.blocks[block_id], &mut path, &mut leaks);
            self.paths.push(path.clone());

            // Add successor paths to worklist
            for &succ_id in &cfg.blocks[block_id].successors {
                let mut new_path = path.clone();
                new_path.blocks.push(succ_id);
                work_list.push_back((succ_id, new_path));
            }
        }

        leaks
    }

    fn analyze_block_in_path(
        &mut self,
        block: &BasicBlock,
        path: &mut ReferencePath,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        // Analyze each statement in the block
        for statement in &block.statements {
            self.analyze_statement_in_path(statement, path, leaks);
        }

        // Check for path-specific violations
        self.check_path_safety(path, leaks);
    }

    fn analyze_statement_in_path(
        &mut self,
        statement: &Statement,
        path: &mut ReferencePath,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        match statement {
            Statement::Assignment(var, expr) => {
                self.analyze_assignment_in_path(var, expr, path, leaks);
            }
            Statement::Return(expr) => {
                self.analyze_return_in_path(expr, path, leaks);
            }
        }
    }

    fn analyze_assignment_in_path(
        &mut self,
        var: &str,
        expr: &Expression,
        path: &mut ReferencePath,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        let new_state = self.analyze_expression_in_path(expr, path);
        
        // Check for valid state transition in this path
        if let Some(state) = self.reference_states.get(var) {
            let transition = StateTransition {
                from: state.current_state.clone(),
                to: new_state.clone(),
                location: Location {
                    file: String::new(),
                    line: 0,
                    column: 0,
                    context: format!("Assignment to {}", var),
                },
                cause: TransitionCause::Assignment,
            };

            // Check if transition is safe in current path context
            if !self.is_safe_transition_in_path(&transition, path) {
                leaks.push(ReferenceLeak {
                    location: transition.location.clone(),
                    leaked_field: FieldId {
                        module_name: String::new(),
                        struct_name: String::new(),
                        field_name: var.to_string(),
                    },
                    context: format!("Unsafe reference state transition in path"),
                    severity: Severity::High,
                });
            }

            path.state_transitions.push(transition);
        }

        // Update state
        self.reference_states.insert(var.to_string(), PathState {
            current_state: new_state,
            conditions: path.conditions.clone(),
            transitions: Vec::new(),
        });
    }

    fn analyze_expression_in_path(
        &self,
        expr: &Expression,
        path: &ReferencePath
    ) -> ReferenceState {
        match expr {
            Expression::Variable(name) => {
                self.reference_states.get(name)
                    .map(|s| s.current_state.clone())
                    .unwrap_or(ReferenceState::Uninitialized)
            }
            Expression::FieldAccess(base, field) => {
                let base_state = self.analyze_expression_in_path(base, path);
                self.analyze_field_access_in_path(base_state, field, path)
            }
        }
    }

    fn analyze_field_access_in_path(
        &self,
        base_state: ReferenceState,
        field: &str,
        path: &ReferencePath
    ) -> ReferenceState {
        match base_state {
            ReferenceState::Borrowed(BorrowKind::MutableWrite) => {
                // Check if mutable access is safe in this path
                if path.conditions.iter().any(|c| matches!(c, PathCondition::BorrowActive(_))) {
                    ReferenceState::Borrowed(BorrowKind::MutableWrite)
                } else {
                    ReferenceState::Uninitialized
                }
            }
            _ => ReferenceState::Uninitialized
        }
    }

    fn analyze_return_in_path(
        &self,
        expr: &Expression,
        path: &ReferencePath,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        let return_state = self.analyze_expression_in_path(expr, path);
        
        // Check for reference leaks in return paths
        match return_state {
            ReferenceState::Borrowed(kind) => {
                if !self.is_safe_return_in_path(&kind, path) {
                    leaks.push(ReferenceLeak {
                        location: Location {
                            file: String::new(),
                            line: 0,
                            column: 0,
                            context: "Return statement".to_string(),
                        },
                        leaked_field: FieldId {
                            module_name: String::new(),
                            struct_name: String::new(),
                            field_name: String::new(),
                        },
                        context: "Reference may escape through return path".to_string(),
                        severity: Severity::Critical,
                    });
                }
            }
            _ => {}
        }
    }

    fn is_safe_transition_in_path(
        &self,
        transition: &StateTransition,
        path: &ReferencePath
    ) -> bool {
        // Check if transition is safe given path conditions
        match (&transition.from, &transition.to) {
            (ReferenceState::Borrowed(kind), _) => {
                path.conditions.iter().any(|c| {
                    matches!(c, PathCondition::BorrowActive(_))
                })
            }
            _ => true
        }
    }

    fn is_safe_return_in_path(
        &self,
        kind: &BorrowKind,
        path: &ReferencePath
    ) -> bool {
        match kind {
            BorrowKind::MutableWrite => false,
            BorrowKind::SharedRead => true,
            _ => path.conditions.iter().any(|c| {
                matches!(c, PathCondition::ReferenceValid(_))
            })
        }
    }

    fn check_path_safety(
        &self,
        path: &ReferencePath,
        leaks: &mut Vec<ReferenceLeak>
    ) {
        // Check for active borrows at path end
        for borrow in &path.active_borrows {
            leaks.push(ReferenceLeak {
                location: borrow.location.clone(),
                leaked_field: FieldId {
                    module_name: String::new(),
                    struct_name: String::new(),
                    field_name: borrow.var.clone(),
                },
                context: format!("Borrow remains active at end of path"),
                severity: Severity::High,
            });
        }
    }
}

impl ReferencePath {
    fn new(entry_block: usize) -> Self {
        Self {
            blocks: vec![entry_block],
            conditions: Vec::new(),
            state_transitions: Vec::new(),
            active_borrows: HashSet::new(),
        }
    }

    fn get_key(&self) -> PathKey {
        PathKey {
            blocks: self.blocks.clone(),
            conditions: self.conditions.clone(),
        }
    }
} 