use super::types::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct StateTransitionAnalyzer {
    states: HashMap<String, ReferenceState>,
    transitions: Vec<StateTransition>,
    valid_transitions: HashSet<(ReferenceState, ReferenceState)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReferenceState {
    Uninitialized,
    Initialized(AbstractValue),
    Borrowed(BorrowKind),
    Moved,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BorrowKind {
    SharedRead,
    MutableWrite,
    TransferGuarded,
    CapabilityProtected,
}

#[derive(Debug)]
struct StateTransition {
    from: ReferenceState,
    to: ReferenceState,
    location: Location,
    cause: TransitionCause,
}

#[derive(Debug)]
enum TransitionCause {
    Assignment,
    BorrowStart,
    BorrowEnd,
    Move,
    Transfer,
    CapabilityUse,
}

impl StateTransitionAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            states: HashMap::new(),
            transitions: Vec::new(),
            valid_transitions: HashSet::new(),
        };
        analyzer.initialize_valid_transitions();
        analyzer
    }

    fn initialize_valid_transitions(&mut self) {
        // Initialize all valid state transitions
        use ReferenceState::*;
        use BorrowKind::*;

        // Basic transitions
        self.add_valid_transition(
            Uninitialized,
            Initialized(AbstractValue::NonRef)
        );

        // Borrow transitions
        self.add_valid_transition(
            Initialized(AbstractValue::NonRef),
            Borrowed(SharedRead)
        );
        self.add_valid_transition(
            Initialized(AbstractValue::NonRef),
            Borrowed(MutableWrite)
        );

        // Object-specific transitions
        self.add_valid_transition(
            Initialized(AbstractValue::ObjectRef(ObjectId::default())),
            Borrowed(TransferGuarded)
        );

        // Capability-specific transitions
        self.add_valid_transition(
            Initialized(AbstractValue::CapabilityRef(CapId::default())),
            Borrowed(CapabilityProtected)
        );
    }

    pub fn analyze_transition(
        &mut self,
        var: &str,
        new_state: ReferenceState,
        location: Location,
        cause: TransitionCause,
    ) -> Result<(), StateTransitionError> {
        let current_state = self.states.get(var)
            .cloned()
            .unwrap_or(ReferenceState::Uninitialized);

        if !self.is_valid_transition(&current_state, &new_state) {
            return Err(StateTransitionError::InvalidTransition {
                var: var.to_string(),
                from: current_state,
                to: new_state,
                location,
            });
        }

        // Record transition
        self.transitions.push(StateTransition {
            from: current_state,
            to: new_state.clone(),
            location,
            cause,
        });

        // Update state
        self.states.insert(var.to_string(), new_state);
        Ok(())
    }

    pub fn check_safety(&self) -> Vec<SafetyViolation> {
        let mut violations = Vec::new();

        // Check for unsafe transitions
        for transition in &self.transitions {
            match (&transition.from, &transition.to) {
                (ReferenceState::Borrowed(BorrowKind::MutableWrite), ReferenceState::Moved) => {
                    violations.push(SafetyViolation {
                        location: transition.location.clone(),
                        violation_type: ViolationType::ReferenceEscape,
                        message: "Mutable reference moved while borrowed".to_string(),
                        severity: Severity::Critical,
                    });
                }
                (ReferenceState::Borrowed(BorrowKind::TransferGuarded), ReferenceState::Released) => {
                    violations.push(SafetyViolation {
                        location: transition.location.clone(),
                        violation_type: ViolationType::UnsafeTransfer,
                        message: "Object released without transfer guard check".to_string(),
                        severity: Severity::High,
                    });
                }
                // Add more unsafe patterns
                _ => {}
            }
        }

        violations
    }

    fn is_valid_transition(&self, from: &ReferenceState, to: &ReferenceState) -> bool {
        self.valid_transitions.contains(&(from.clone(), to.clone()))
    }

    fn add_valid_transition(&mut self, from: ReferenceState, to: ReferenceState) {
        self.valid_transitions.insert((from, to));
    }
}

#[derive(Debug)]
pub enum StateTransitionError {
    InvalidTransition {
        var: String,
        from: ReferenceState,
        to: ReferenceState,
        location: Location,
    },
    // Add more error types
}

// Default implementations for ObjectId and CapId
impl Default for ObjectId {
    fn default() -> Self {
        Self {
            module_name: String::new(),
            type_name: String::new(),
            is_shared: false,
        }
    }
}

impl Default for CapId {
    fn default() -> Self {
        Self {
            module_name: String::new(),
            cap_name: String::new(),
            permissions: Vec::new(),
        }
    }
} 