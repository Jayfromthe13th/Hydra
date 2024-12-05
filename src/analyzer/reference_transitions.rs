use super::types::*;
use super::state_transitions::{ReferenceState, BorrowKind};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ReferenceTransitionAnalyzer {
    states: HashMap<String, ReferenceState>,
    transitions: Vec<TransitionEvent>,
    valid_transitions: HashSet<(ReferenceState, ReferenceState)>,
    borrow_stack: Vec<BorrowInfo>,
}

#[derive(Debug)]
struct TransitionEvent {
    var: String,
    from: ReferenceState,
    to: ReferenceState,
    location: Location,
    cause: TransitionCause,
}

#[derive(Debug)]
struct BorrowInfo {
    var: String,
    kind: BorrowKind,
    location: Location,
    active_references: HashSet<String>,
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

impl ReferenceTransitionAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            states: HashMap::new(),
            transitions: Vec::new(),
            valid_transitions: HashSet::new(),
            borrow_stack: Vec::new(),
        };
        analyzer.initialize_valid_transitions();
        analyzer
    }

    fn initialize_valid_transitions(&mut self) {
        use ReferenceState::*;
        use BorrowKind::*;

        // Basic transitions
        self.add_valid_transition(
            Uninitialized,
            Initialized(AbstractValue::NonRef)
        );

        // Reference transitions
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

    pub fn analyze_statement(&mut self, statement: &Statement, location: Location) -> Result<(), String> {
        match statement {
            Statement::Assignment(var, expr) => {
                self.analyze_assignment(var, expr, location)?;
            }
            Statement::Return(expr) => {
                self.analyze_return(expr, location)?;
            }
        }
        Ok(())
    }

    fn analyze_assignment(&mut self, var: &str, expr: &Expression, location: Location) -> Result<(), String> {
        let new_state = self.analyze_expression(expr)?;
        
        // Check for valid transition
        let current_state = self.states.get(var)
            .cloned()
            .unwrap_or(ReferenceState::Uninitialized);

        if !self.is_valid_transition(&current_state, &new_state) {
            return Err(format!(
                "Invalid reference state transition for {} from {:?} to {:?}",
                var, current_state, new_state
            ));
        }

        // Record transition
        self.record_transition(var, current_state, new_state.clone(), location, TransitionCause::Assignment);

        // Update state
        self.states.insert(var.to_string(), new_state);
        Ok(())
    }

    fn analyze_expression(&self, expr: &Expression) -> Result<ReferenceState, String> {
        match expr {
            Expression::Variable(name) => {
                Ok(self.states.get(name)
                    .cloned()
                    .unwrap_or(ReferenceState::Uninitialized))
            }
            Expression::FieldAccess(base, field) => {
                let base_state = self.analyze_expression(base)?;
                self.analyze_field_access(base_state, field)
            }
        }
    }

    fn analyze_field_access(&self, base_state: ReferenceState, field: &str) -> Result<ReferenceState, String> {
        match base_state {
            ReferenceState::Borrowed(BorrowKind::MutableWrite) => {
                // Check for field access on mutable reference
                Ok(ReferenceState::Borrowed(BorrowKind::MutableWrite))
            }
            ReferenceState::Borrowed(BorrowKind::TransferGuarded) => {
                // Check for transfer guard conditions
                if field.contains("transfer") || field.contains("guard") {
                    Ok(ReferenceState::Borrowed(BorrowKind::TransferGuarded))
                } else {
                    Err("Invalid field access on transfer-guarded reference".to_string())
                }
            }
            _ => Ok(ReferenceState::Initialized(AbstractValue::NonRef))
        }
    }

    fn analyze_return(&mut self, expr: &Expression, location: Location) -> Result<(), String> {
        let return_state = self.analyze_expression(expr)?;
        
        // Check for reference leaks in return
        match &return_state {
            ReferenceState::Borrowed(BorrowKind::MutableWrite) => {
                Err("Returning mutable reference may cause reference leak".to_string())
            }
            ReferenceState::Borrowed(BorrowKind::TransferGuarded) => {
                Err("Returning transfer-guarded reference is unsafe".to_string())
            }
            _ => Ok(())
        }
    }

    fn record_transition(
        &mut self,
        var: &str,
        from: ReferenceState,
        to: ReferenceState,
        location: Location,
        cause: TransitionCause,
    ) {
        self.transitions.push(TransitionEvent {
            var: var.to_string(),
            from,
            to,
            location,
            cause,
        });
    }

    pub fn start_borrow(&mut self, var: &str, kind: BorrowKind, location: Location) {
        self.borrow_stack.push(BorrowInfo {
            var: var.to_string(),
            kind,
            location,
            active_references: HashSet::new(),
        });
    }

    pub fn end_borrow(&mut self, var: &str) -> Result<(), String> {
        if let Some(borrow) = self.borrow_stack.last() {
            if borrow.var != var {
                return Err(format!(
                    "Mismatched borrow end: expected {}, got {}",
                    borrow.var, var
                ));
            }
            self.borrow_stack.pop();
            Ok(())
        } else {
            Err("No active borrow to end".to_string())
        }
    }

    pub fn check_safety(&self) -> Vec<SafetyViolation> {
        let mut violations = Vec::new();

        // Check for unclosed borrows
        for borrow in &self.borrow_stack {
            violations.push(SafetyViolation {
                location: borrow.location.clone(),
                violation_type: ViolationType::ReferenceEscape,
                message: format!("Unclosed borrow of {}", borrow.var),
                severity: Severity::High,
            });
        }

        // Check transition history for unsafe patterns
        for event in &self.transitions {
            match (&event.from, &event.to) {
                (ReferenceState::Borrowed(BorrowKind::MutableWrite), ReferenceState::Moved) => {
                    violations.push(SafetyViolation {
                        location: event.location.clone(),
                        violation_type: ViolationType::ReferenceEscape,
                        message: format!("Moving {} while borrowed", event.var),
                        severity: Severity::Critical,
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