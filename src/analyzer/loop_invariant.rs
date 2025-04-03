use super::types::*;
use super::loop_analysis::LoopAnalyzer;
use super::loop_reference::LoopReferenceAnalyzer;
use super::loop_state::LoopStateAnalyzer;
use std::collections::{HashMap, HashSet};

pub struct LoopInvariantAnalyzer {
    loop_analyzer: LoopAnalyzer,
    reference_analyzer: LoopReferenceAnalyzer,
    state_analyzer: LoopStateAnalyzer,
    invariants: HashMap<usize, Vec<LoopInvariant>>,
    current_loop: Option<usize>,
}

#[derive(Debug)]
struct LoopInvariant {
    condition: InvariantCondition,
    affected_refs: HashSet<String>,
    verification_points: Vec<VerificationPoint>,
    is_maintained: bool,
}

#[derive(Debug)]
enum InvariantCondition {
    ReferenceValid(String),
    NoAliasing(String, String),
    StatePreserved(String, ReferenceState),
    MutationBounded(String, usize),
    ObjectOwned(ObjectId),
    CapabilityHeld(CapId),
    Custom(String),
}

#[derive(Debug)]
struct VerificationPoint {
    location: Location,
    state: InvariantState,
    context: String,
}

#[derive(Debug)]
enum InvariantState {
    Maintained,
    Violated(String),
    Unknown,
}

impl LoopInvariantAnalyzer {
    pub fn new() -> Self {
        Self {
            loop_analyzer: LoopAnalyzer::new(),
            reference_analyzer: LoopReferenceAnalyzer::new(),
            state_analyzer: LoopStateAnalyzer::new(),
            invariants: HashMap::new(),
            current_loop: None,
        }
    }

    pub fn analyze_loop(&mut self, function: &Function) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // First analyze loop structure and references
        let loop_leaks = self.loop_analyzer.analyze_loops(function);
        let ref_leaks = self.reference_analyzer.analyze_loop(function);
        let state_leaks = self.state_analyzer.analyze_loop(function);
        
        leaks.extend(loop_leaks);
        leaks.extend(ref_leaks);
        leaks.extend(state_leaks);
        
        // Analyze loop invariants
        self.analyze_loop_invariants(function, &mut leaks);
        
        leaks
    }

    fn analyze_loop_invariants(&mut self, function: &Function, leaks: &mut Vec<ReferenceLeak>) {
        // Identify loop invariants
        self.identify_invariants(function);
        
        // Verify invariants at key points
        for (loop_id, invariants) in &mut self.invariants {
            self.current_loop = Some(*loop_id);
            
            // Check invariants at loop entry
            self.verify_entry_invariants(invariants, leaks);
            
            // Check invariants through iterations
            self.verify_iteration_invariants(invariants, leaks);
            
            // Check invariants at loop exit
            self.verify_exit_invariants(invariants, leaks);
        }
        
        self.current_loop = None;
    }

    fn identify_invariants(&mut self, function: &Function) {
        for statement in &function.body {
            match statement {
                Statement::Assignment(var, expr) => {
                    // Identify reference invariants
                    if let Some(invariant) = self.identify_reference_invariant(var, expr) {
                        self.add_invariant(invariant);
                    }
                    
                    // Identify object invariants
                    if let Some(invariant) = self.identify_object_invariant(var, expr) {
                        self.add_invariant(invariant);
                    }
                    
                    // Identify capability invariants
                    if let Some(invariant) = self.identify_capability_invariant(var, expr) {
                        self.add_invariant(invariant);
                    }
                }
                _ => {}
            }
        }
    }

    fn identify_reference_invariant(&self, var: &str, expr: &Expression) -> Option<LoopInvariant> {
        let mut affected_refs = HashSet::new();
        affected_refs.insert(var.to_string());

        match expr {
            Expression::Variable(name) => {
                Some(LoopInvariant {
                    condition: InvariantCondition::NoAliasing(
                        var.to_string(),
                        name.clone(),
                    ),
                    affected_refs,
                    verification_points: Vec::new(),
                    is_maintained: true,
                })
            }
            _ => None
        }
    }

    fn identify_object_invariant(&self, var: &str, expr: &Expression) -> Option<LoopInvariant> {
        match expr {
            Expression::FieldAccess(base, _) => {
                if let Some(obj_id) = self.get_object_id(base) {
                    let mut affected_refs = HashSet::new();
                    affected_refs.insert(var.to_string());

                    Some(LoopInvariant {
                        condition: InvariantCondition::ObjectOwned(obj_id),
                        affected_refs,
                        verification_points: Vec::new(),
                        is_maintained: true,
                    })
                } else {
                    None
                }
            }
            _ => None
        }
    }

    fn identify_capability_invariant(&self, var: &str, expr: &Expression) -> Option<LoopInvariant> {
        match expr {
            Expression::FieldAccess(base, _) => {
                if let Some(cap_id) = self.get_capability_id(base) {
                    let mut affected_refs = HashSet::new();
                    affected_refs.insert(var.to_string());

                    Some(LoopInvariant {
                        condition: InvariantCondition::CapabilityHeld(cap_id),
                        affected_refs,
                        verification_points: Vec::new(),
                        is_maintained: true,
                    })
                } else {
                    None
                }
            }
            _ => None
        }
    }

    fn verify_entry_invariants(&self, invariants: &mut Vec<LoopInvariant>, leaks: &mut Vec<ReferenceLeak>) {
        for invariant in invariants {
            let verification = self.verify_invariant_condition(&invariant.condition);
            
            match verification {
                InvariantState::Violated(reason) => {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!(
                            "Loop invariant violation at entry: {}",
                            reason
                        ),
                        severity: Severity::High,
                    });
                }
                _ => {
                    invariant.verification_points.push(VerificationPoint {
                        location: Location::default(),
                        state: verification,
                        context: "Loop entry".to_string(),
                    });
                }
            }
        }
    }

    fn verify_iteration_invariants(&self, invariants: &mut Vec<LoopInvariant>, leaks: &mut Vec<ReferenceLeak>) {
        for invariant in invariants {
            let verification = self.verify_invariant_condition(&invariant.condition);
            
            match verification {
                InvariantState::Violated(reason) => {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!(
                            "Loop invariant violation during iteration: {}",
                            reason
                        ),
                        severity: Severity::Critical,
                    });
                    invariant.is_maintained = false;
                }
                _ => {
                    invariant.verification_points.push(VerificationPoint {
                        location: Location::default(),
                        state: verification,
                        context: "Loop iteration".to_string(),
                    });
                }
            }
        }
    }

    fn verify_exit_invariants(&self, invariants: &mut Vec<LoopInvariant>, leaks: &mut Vec<ReferenceLeak>) {
        for invariant in invariants {
            if !invariant.is_maintained {
                leaks.push(ReferenceLeak {
                    location: Location::default(),
                    leaked_field: FieldId::default(),
                    context: format!(
                        "Loop invariant not maintained through all iterations"
                    ),
                    severity: Severity::High,
                });
            }
            
            let verification = self.verify_invariant_condition(&invariant.condition);
            
            match verification {
                InvariantState::Violated(reason) => {
                    leaks.push(ReferenceLeak {
                        location: Location::default(),
                        leaked_field: FieldId::default(),
                        context: format!(
                            "Loop invariant violation at exit: {}",
                            reason
                        ),
                        severity: Severity::High,
                    });
                }
                _ => {
                    invariant.verification_points.push(VerificationPoint {
                        location: Location::default(),
                        state: verification,
                        context: "Loop exit".to_string(),
                    });
                }
            }
        }
    }

    fn verify_invariant_condition(&self, condition: &InvariantCondition) -> InvariantState {
        match condition {
            InvariantCondition::ReferenceValid(var) => {
                if self.is_reference_valid(var) {
                    InvariantState::Maintained
                } else {
                    InvariantState::Violated(format!("Invalid reference to {}", var))
                }
            }
            InvariantCondition::NoAliasing(var1, var2) => {
                if self.has_aliasing(var1, var2) {
                    InvariantState::Violated(format!(
                        "References {} and {} may alias",
                        var1, var2
                    ))
                } else {
                    InvariantState::Maintained
                }
            }
            InvariantCondition::StatePreserved(var, expected_state) => {
                if self.verify_state(var, expected_state) {
                    InvariantState::Maintained
                } else {
                    InvariantState::Violated(format!(
                        "State of {} not preserved",
                        var
                    ))
                }
            }
            InvariantCondition::ObjectOwned(obj_id) => {
                if self.verify_ownership(obj_id) {
                    InvariantState::Maintained
                } else {
                    InvariantState::Violated(format!(
                        "Object ownership invariant violated"
                    ))
                }
            }
            InvariantCondition::CapabilityHeld(cap_id) => {
                if self.verify_capability(cap_id) {
                    InvariantState::Maintained
                } else {
                    InvariantState::Violated(format!(
                        "Required capability not held"
                    ))
                }
            }
            _ => InvariantState::Unknown,
        }
    }

    // Helper methods
    fn add_invariant(&mut self, invariant: LoopInvariant) {
        if let Some(loop_id) = self.current_loop {
            self.invariants
                .entry(loop_id)
                .or_default()
                .push(invariant);
        }
    }

    fn get_object_id(&self, expr: &Expression) -> Option<ObjectId> {
        None // Simplified for now
    }

    fn get_capability_id(&self, expr: &Expression) -> Option<CapId> {
        None // Simplified for now
    }

    fn is_reference_valid(&self, var: &str) -> bool {
        true // Simplified for now
    }

    fn has_aliasing(&self, var1: &str, var2: &str) -> bool {
        false // Simplified for now
    }

    fn verify_state(&self, var: &str, expected_state: &ReferenceState) -> bool {
        true // Simplified for now
    }

    fn verify_ownership(&self, obj_id: &ObjectId) -> bool {
        true // Simplified for now
    }

    fn verify_capability(&self, cap_id: &CapId) -> bool {
        true // Simplified for now
    }
} 