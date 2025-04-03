use super::types::*;
use super::reference_path::ReferencePathAnalyzer;
use std::collections::{HashMap, HashSet};

pub struct ReferenceStateAnalyzer {
    states: HashMap<String, ReferenceStateInfo>,
    transitions: Vec<StateTransitionEvent>,
    active_scopes: Vec<ScopeInfo>,
    path_analyzer: ReferencePathAnalyzer,
}

#[derive(Debug, Clone)]
struct ReferenceStateInfo {
    current_state: ReferenceState,
    definition_point: Location,
    last_use: Option<Location>,
    aliases: HashSet<String>,
    constraints: Vec<ReferenceConstraint>,
}

#[derive(Debug, Clone)]
enum ReferenceState {
    Uninitialized,
    Initialized {
        value: AbstractValue,
        mutable: bool,
    },
    Borrowed {
        kind: BorrowKind,
        source: String,
    },
    Moved {
        destination: String,
        location: Location,
    },
    Invalid {
        reason: String,
    },
}

#[derive(Debug)]
struct StateTransitionEvent {
    var: String,
    from: ReferenceState,
    to: ReferenceState,
    location: Location,
    cause: TransitionCause,
}

#[derive(Debug)]
struct ScopeInfo {
    level: usize,
    variables: HashSet<String>,
    borrows: HashSet<String>,
}

#[derive(Debug, Clone)]
enum ReferenceConstraint {
    MustBeValid,
    NoAliasing(String),
    NoMutation,
    OwnershipRequired,
    Custom(String),
}

impl ReferenceStateAnalyzer {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            transitions: Vec::new(),
            active_scopes: vec![ScopeInfo::new(0)],
            path_analyzer: ReferencePathAnalyzer::new(),
        }
    }

    pub fn analyze_function(&mut self, function: &Function) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // Initialize parameter states
        self.initialize_parameters(function);
        
        // Enter function scope
        self.enter_scope();
        
        // Analyze statements
        for statement in &function.body {
            self.analyze_statement(statement, &mut leaks);
        }
        
        // Check for unclosed references
        self.check_unclosed_references(&mut leaks);
        
        // Exit function scope
        self.exit_scope(&mut leaks);
        
        leaks
    }

    fn initialize_parameters(&mut self, function: &Function) {
        for param in &function.parameters {
            let state = match &param.param_type {
                Type::MutableReference(_) => ReferenceState::Borrowed {
                    kind: BorrowKind::MutableWrite,
                    source: "parameter".to_string(),
                },
                Type::Reference(_) => ReferenceState::Borrowed {
                    kind: BorrowKind::SharedRead,
                    source: "parameter".to_string(),
                },
                _ => ReferenceState::Initialized {
                    value: AbstractValue::NonRef,
                    mutable: false,
                },
            };

            self.states.insert(param.name.clone(), ReferenceStateInfo {
                current_state: state,
                definition_point: Location::default(), // Would need proper location
                last_use: None,
                aliases: HashSet::new(),
                constraints: Vec::new(),
            });
        }
    }

    fn analyze_statement(&mut self, statement: &Statement, leaks: &mut Vec<ReferenceLeak>) {
        match statement {
            Statement::Assignment(var, expr) => {
                self.analyze_assignment(var, expr, leaks);
            }
            Statement::Return(expr) => {
                self.analyze_return(expr, leaks);
            }
        }
    }

    fn analyze_assignment(&mut self, var: &str, expr: &Expression, leaks: &mut Vec<ReferenceLeak>) {
        let value = self.evaluate_expression(expr);
        
        // Check for reference leaks through assignment
        if let AbstractValue::InvRef(field) = &value {
            if self.can_escape_through_assignment(var) {
                leaks.push(ReferenceLeak {
                    location: Location::default(), // Would need proper location
                    leaked_field: field.clone(),
                    context: format!("Reference may escape through assignment to {}", var),
                    severity: Severity::High,
                });
            }
        }

        // Update state
        let new_state = ReferenceState::Initialized {
            value,
            mutable: true, // Would need proper mutability analysis
        };

        self.transition_state(var, new_state, Location::default(), TransitionCause::Assignment);
    }

    fn analyze_return(&mut self, expr: &Expression, leaks: &mut Vec<ReferenceLeak>) {
        let value = self.evaluate_expression(expr);
        
        // Check for reference leaks through return
        if let AbstractValue::InvRef(field) = value {
            leaks.push(ReferenceLeak {
                location: Location::default(),
                leaked_field: field,
                context: "Reference escapes through return".to_string(),
                severity: Severity::Critical,
            });
        }
    }

    fn evaluate_expression(&self, expr: &Expression) -> AbstractValue {
        match expr {
            Expression::Variable(name) => {
                if let Some(state) = self.states.get(name) {
                    match &state.current_state {
                        ReferenceState::Initialized { value, .. } => value.clone(),
                        _ => AbstractValue::NonRef,
                    }
                } else {
                    AbstractValue::NonRef
                }
            }
            Expression::FieldAccess(base, field) => {
                let base_value = self.evaluate_expression(base);
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

    fn transition_state(
        &mut self,
        var: &str,
        new_state: ReferenceState,
        location: Location,
        cause: TransitionCause,
    ) {
        if let Some(state_info) = self.states.get_mut(var) {
            let old_state = std::mem::replace(&mut state_info.current_state, new_state.clone());
            
            self.transitions.push(StateTransitionEvent {
                var: var.to_string(),
                from: old_state,
                to: new_state,
                location,
                cause,
            });
        }
    }

    fn enter_scope(&mut self) {
        let new_level = self.active_scopes.last().map_or(0, |s| s.level + 1);
        self.active_scopes.push(ScopeInfo::new(new_level));
    }

    fn exit_scope(&mut self, leaks: &mut Vec<ReferenceLeak>) {
        if let Some(scope) = self.active_scopes.pop() {
            // Check for references that escape their scope
            for var in scope.variables {
                if let Some(state) = self.states.get(&var) {
                    if self.reference_escapes_scope(&var, state) {
                        if let ReferenceState::Initialized { value: AbstractValue::InvRef(field), .. } = &state.current_state {
                            leaks.push(ReferenceLeak {
                                location: Location::default(),
                                leaked_field: field.clone(),
                                context: format!("Reference {} escapes its scope", var),
                                severity: Severity::High,
                            });
                        }
                    }
                }
            }
        }
    }

    fn check_unclosed_references(&self, leaks: &mut Vec<ReferenceLeak>) {
        for (var, state) in &self.states {
            if let ReferenceState::Borrowed { .. } = state.current_state {
                if let Some(field) = self.get_borrowed_field(var) {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: field,
                        context: format!("Unclosed borrow of {}", var),
                        severity: Severity::High,
                    });
                }
            }
        }
    }

    fn can_escape_through_assignment(&self, var: &str) -> bool {
        // Check if variable can escape current scope
        self.active_scopes.iter().any(|scope| !scope.variables.contains(var))
    }

    fn reference_escapes_scope(&self, var: &str, state: &ReferenceStateInfo) -> bool {
        matches!(state.current_state,
            ReferenceState::Borrowed { .. } | 
            ReferenceState::Initialized { value: AbstractValue::InvRef(_), .. }
        )
    }

    fn get_borrowed_field(&self, var: &str) -> Option<FieldId> {
        self.states.get(var).and_then(|state| {
            match &state.current_state {
                ReferenceState::Initialized { value: AbstractValue::InvRef(field), .. } => Some(field.clone()),
                _ => None,
            }
        })
    }
}

impl ScopeInfo {
    fn new(level: usize) -> Self {
        Self {
            level,
            variables: HashSet::new(),
            borrows: HashSet::new(),
        }
    }
} 