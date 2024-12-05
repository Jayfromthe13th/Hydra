use crate::analyzer::parser::{Module, Function, Statement, Struct};
use crate::analyzer::types::*;
use crate::analyzer::escape_analysis::{EscapeAnalyzer, EscapePoint, SuiObjectKind};
use crate::analyzer::reference_flow::ReferenceFlowAnalyzer;
use crate::analyzer::path_analysis::PathAnalyzer;
use crate::analyzer::object_state::ObjectStateTracker;
use crate::analyzer::capability_checks::CapabilityChecker;
use crate::analyzer::invariant_checks::InvariantChecker;
use crate::analyzer::sui_checks::SuiSafetyChecker;
use crate::analyzer::invariant_tracking::{InvariantTracker, InvariantInfo};
use crate::analyzer::trace_semantics::{TraceAnalyzer, SecurityEvent, BoundaryKind};
use crate::analyzer::safety_verifier::{SafetyVerifier, LocalProperty, UnreachableProperty, StrongProperty};
use std::collections::{HashMap, HashSet, BTreeMap};

#[derive(Debug)]
pub struct ModuleInfo {
    pub is_trusted: bool,
    pub dependencies: HashSet<String>,
    pub external_calls: HashSet<String>,
    pub public_functions: HashSet<String>,
}

#[allow(dead_code)]
pub struct Analysis {
    reference_analyzer: ReferenceFlowAnalyzer,
    path_analyzer: PathAnalyzer,
    object_tracker: ObjectStateTracker,
    capability_checker: CapabilityChecker,
    invariant_checker: InvariantChecker,
    sui_checker: SuiSafetyChecker,
    escape_analyzer: EscapeAnalyzer,
    current_module: Option<String>,
    analyzed_functions: HashMap<String, Vec<EscapePoint>>,
    module_info: BTreeMap<String, ModuleInfo>,
    trusted_modules: HashSet<String>,
    invariant_tracker: InvariantTracker,
    trace_analyzer: TraceAnalyzer,
    safety_verifier: SafetyVerifier,
}

impl Analysis {
    pub fn new() -> Self {
        Self {
            reference_analyzer: ReferenceFlowAnalyzer::new(),
            path_analyzer: PathAnalyzer::new(),
            object_tracker: ObjectStateTracker::new(),
            capability_checker: CapabilityChecker::new(),
            invariant_checker: InvariantChecker::new(),
            sui_checker: SuiSafetyChecker::new(),
            escape_analyzer: EscapeAnalyzer::new(),
            current_module: None,
            analyzed_functions: HashMap::new(),
            module_info: BTreeMap::new(),
            trusted_modules: HashSet::new(),
            invariant_tracker: InvariantTracker::new(),
            trace_analyzer: TraceAnalyzer::new(),
            safety_verifier: SafetyVerifier::new(),
        }
    }

    pub fn analyze_module(&mut self, module: &Module) -> AnalysisResult {
        let mut result = AnalysisResult::default();
        self.current_module = Some(module.name.clone());

        // Add safety properties based on module analysis
        self.add_safety_properties(module);

        // Verify safety properties
        let safety_violations = self.safety_verifier.verify_module(module);
        result.safety_violations.extend(safety_violations);

        // Parse and track invariants
        if let Err(e) = self.invariant_tracker.parse_invariants(module) {
            result.safety_violations.push(SafetyViolation {
                location: Location::default(),
                violation_type: ViolationType::InvariantViolation,
                message: format!("Failed to parse invariants: {}", e),
                severity: Severity::High,
                context: None,
            });
            return result;
        }

        // Track module dependencies and build module info
        let mut module_info = ModuleInfo {
            is_trusted: self.trusted_modules.contains(&module.name),
            dependencies: HashSet::new(),
            external_calls: HashSet::new(),
            public_functions: HashSet::new(),
        };

        // Analyze imports and dependencies
        for import in &module.imports {
            module_info.dependencies.insert(import.module_name.clone());
        }

        // Track public functions
        for function in &module.functions {
            if function.is_public {
                module_info.public_functions.insert(function.name.clone());
            }
        }

        // Analyze cross-module calls
        for function in &module.functions {
            for statement in &function.body {
                if let Statement::Call(name, _) = statement {
                    if !name.starts_with("Self::") {
                        module_info.external_calls.insert(name.clone());
                    }
                }
            }
        }

        // Store module info
        self.module_info.insert(module.name.clone(), module_info);

        // Verify module isolation
        if let Some(violations) = self.verify_module_isolation(module) {
            result.safety_violations.extend(violations);
        }

        // Perform other analyses
        for function in &module.functions {
            let ref_leaks = self.reference_analyzer.analyze_function(function);
            result.reference_leaks.extend(ref_leaks);

            let path_leaks = self.path_analyzer.analyze_paths(function);
            result.reference_leaks.extend(path_leaks);

            let cap_violations = self.capability_checker.check_capability_safety(function);
            result.safety_violations.extend(cap_violations);

            let inv_violations = self.invariant_checker.check_invariants(function);
            result.safety_violations.extend(inv_violations);
        }

        // Track global state for invariants
        for function in &module.functions {
            for statement in &function.body {
                self.invariant_tracker.track_global_state(statement);
            }
        }

        // Analyze security events and traces
        let security_events = self.trace_analyzer.analyze_module(module);
        for event in security_events {
            match event {
                SecurityEvent::CallToTrusted { caller, callee, context } => {
                    result.safety_violations.push(SafetyViolation {
                        location: Location::default(),
                        violation_type: ViolationType::UnauthorizedAccess,
                        message: format!("Untrusted module {} calling trusted module {}", caller, callee),
                        severity: Severity::High,
                        context: Some(ViolationContext {
                            affected_functions: vec![],
                            related_types: vec![],
                            suggested_fixes: vec!["Remove direct call to trusted module".to_string()],
                            whitepaper_reference: Some(context),
                        }),
                    });
                }
                SecurityEvent::BoundaryCrossing { from_module, to_module, kind } => {
                    // Track boundary crossings in analysis result
                    if let Some(violations) = self.verify_boundary_crossing(&from_module, &to_module, &kind) {
                        result.safety_violations.extend(violations);
                    }
                }
                _ => {}
            }
        }

        // Add Sui-specific object analysis
        self.analyze_sui_objects(module, &mut result);

        result
    }

    fn verify_module_isolation(&self, module: &Module) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();
        let module_info = self.module_info.get(&module.name)?;

        // Check for calls from untrusted to trusted modules
        if !module_info.is_trusted {
            for call in &module_info.external_calls {
                let called_module = call.split("::").next()?;
                if self.trusted_modules.contains(called_module) {
                    violations.push(SafetyViolation {
                        location: Location::default(),
                        violation_type: ViolationType::UnauthorizedAccess,
                        message: format!("Untrusted module {} calling trusted module {}", 
                            module.name, called_module),
                        severity: Severity::High,
                        context: Some(ViolationContext {
                            affected_functions: vec![],
                            related_types: vec![],
                            suggested_fixes: vec!["Remove direct call to trusted module".to_string()],
                            whitepaper_reference: Some("Section 4.4: Module Isolation".to_string()),
                        }),
                    });
                }
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn verify_boundary_crossing(
        &self,
        from: &str,
        to: &str,
        kind: &BoundaryKind
    ) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();

        match kind {
            BoundaryKind::UntrustedToTrusted => {
                // Check for unauthorized access to trusted modules
                if !self.trusted_modules.contains(from) && self.trusted_modules.contains(to) {
                    violations.push(SafetyViolation {
                        location: Location::default(),
                        violation_type: ViolationType::UnauthorizedAccess,
                        message: format!("Unauthorized access from untrusted module {} to trusted module {}", from, to),
                        severity: Severity::High,
                        context: Some(ViolationContext {
                            affected_functions: vec![],
                            related_types: vec![],
                            suggested_fixes: vec!["Add proper capability verification".to_string()],
                            whitepaper_reference: Some("Section 4.4: Trust Boundaries".to_string()),
                        }),
                    });
                }
            }
            BoundaryKind::TrustedToUntrusted => {
                // Check for potential leaks to untrusted modules
                if self.trusted_modules.contains(from) && !self.trusted_modules.contains(to) {
                    violations.push(SafetyViolation {
                        location: Location::default(),
                        violation_type: ViolationType::ResourceSafetyViolation,
                        message: format!("Potential resource leak from trusted module {} to untrusted module {}", from, to),
                        severity: Severity::High,
                        context: Some(ViolationContext {
                            affected_functions: vec![],
                            related_types: vec![],
                            suggested_fixes: vec!["Add proper encapsulation".to_string()],
                            whitepaper_reference: Some("Section 4.4: Resource Safety".to_string()),
                        }),
                    });
                }
            }
            BoundaryKind::CrossModule => {
                // Check for proper module isolation
                if let (Some(from_info), Some(_to_info)) = (
                    self.module_info.get(from),
                    self.module_info.get(to)
                ) {
                    if !from_info.dependencies.contains(to) {
                        violations.push(SafetyViolation {
                            location: Location::default(),
                            violation_type: ViolationType::UnauthorizedAccess,
                            message: format!("Undeclared cross-module call from {} to {}", from, to),
                            severity: Severity::Medium,
                            context: Some(ViolationContext {
                                affected_functions: vec![],
                                related_types: vec![],
                                suggested_fixes: vec!["Add proper module dependency".to_string()],
                                whitepaper_reference: Some("Section 4.4: Module Isolation".to_string()),
                            }),
                        });
                    }
                }
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    pub fn mark_trusted_module(&mut self, module_name: String) {
        self.trusted_modules.insert(module_name);
    }

    pub fn get_module_info(&self, module_name: &str) -> Option<&ModuleInfo> {
        self.module_info.get(module_name)
    }

    pub fn get_cross_module_calls(&self, module_name: &str) -> HashSet<String> {
        self.module_info
            .get(module_name)
            .map(|info| info.external_calls.clone())
            .unwrap_or_default()
    }

    // Helper methods for escape analysis results
    pub fn get_escape_points(&self, function_name: &str) -> Option<&Vec<EscapePoint>> {
        self.analyzed_functions.get(function_name)
    }

    pub fn has_reference_leaks(&self, function_name: &str) -> bool {
        self.get_escape_points(function_name)
            .map(|points| !points.is_empty())
            .unwrap_or(false)
    }

    // Helper methods for invariant tracking
    pub fn get_field_invariants(&self, field_name: &str) -> Option<&InvariantInfo> {
        self.invariant_tracker.get_field_invariants(field_name)
    }

    pub fn get_struct_invariants(&self, struct_name: &str) -> Option<&InvariantInfo> {
        self.invariant_tracker.get_struct_invariants(struct_name)
    }

    pub fn get_global_invariants(&self, global_name: &str) -> Option<&InvariantInfo> {
        self.invariant_tracker.get_global_invariants(global_name)
    }

    pub fn has_invariant(&self, name: &str) -> bool {
        self.invariant_tracker.has_invariant(name)
    }

    // Helper methods for trace analysis
    pub fn get_security_events(&self) -> &[SecurityEvent] {
        self.trace_analyzer.get_security_events()
    }

    pub fn get_call_chain(&self, module_name: &str) -> Option<&Vec<String>> {
        self.trace_analyzer.get_call_chain(module_name)
    }

    fn add_safety_properties(&mut self, module: &Module) {
        // Add local properties for invariants
        for function in &module.functions {
            if function.has_assertions {
                self.safety_verifier.add_local_property(
                    function.name.clone(),
                    LocalProperty {
                        invariant: "assertion".to_string(),
                        scope: "function".to_string(),
                        condition: "assert".to_string(),
                    }
                );
            }
        }

        // Add unreachability properties for private resources
        for field in module.get_fields() {
            if !field.is_public() {
                self.safety_verifier.add_unreachable_property(
                    field.name.clone(),
                    UnreachableProperty {
                        resource: field.name.clone(),
                        access_path: vec!["public".to_string()],
                    }
                );
            }
        }

        // Add strong properties for critical resources
        for field in module.get_fields() {
            if field.has_invariant() {
                self.safety_verifier.add_strong_property(
                    field.name.clone(),
                    StrongProperty {
                        local: LocalProperty {
                            invariant: field.get_invariant().unwrap_or_default(),
                            scope: "field".to_string(),
                            condition: "check".to_string(),
                        },
                        unreachable: UnreachableProperty {
                            resource: field.name.clone(),
                            access_path: vec!["unauthorized".to_string()],
                        },
                    }
                );
            }
        }
    }

    // Helper methods for safety verification
    pub fn get_safety_violations(&mut self, module_name: &str) -> Vec<SafetyViolation> {
        if let Some(_module) = self.get_module_info(module_name) {
            self.safety_verifier.verify_module(&Module::new(module_name.to_string()))
        } else {
            Vec::new()
        }
    }

    fn analyze_sui_objects(&mut self, module: &Module, result: &mut AnalysisResult) {
        // Track object definitions
        for struct_def in module.get_structs() {
            if struct_def.has_key_ability() {
                self.escape_analyzer.track_object_creation(
                    &struct_def.name,
                    Self::determine_object_kind(struct_def)
                );
            }
        }

        // Analyze object usage
        for function in &module.functions {
            // Check transfer safety
            if let Some(violations) = self.verify_transfer_safety(function) {
                result.safety_violations.extend(violations);
            }

            // Check capability usage
            if let Some(violations) = self.verify_capability_usage(function) {
                result.safety_violations.extend(violations);
            }

            // Check shared object access
            if let Some(violations) = self.verify_shared_object_access(function) {
                result.safety_violations.extend(violations);
            }
        }
    }

    fn determine_object_kind(struct_def: &Struct) -> SuiObjectKind {
        if struct_def.has_key_ability() {
            if struct_def.abilities.contains(&"shared".to_string()) {
                SuiObjectKind::Shared
            } else if struct_def.abilities.contains(&"immutable".to_string()) {
                SuiObjectKind::Immutable
            } else {
                SuiObjectKind::Owned
            }
        } else {
            SuiObjectKind::Dynamic
        }
    }

    fn verify_transfer_safety(&mut self, function: &Function) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();
        
        for statement in &function.body {
            if let Statement::Call(name, _) = statement {
                if name.contains("transfer::transfer") {
                    if let Err(e) = self.escape_analyzer.verify_transfer_safety(name) {
                        violations.push(SafetyViolation {
                            location: Location::default(),
                            violation_type: ViolationType::UnauthorizedAccess,
                            message: e,
                            severity: Severity::High,
                            context: Some(ViolationContext {
                                affected_functions: vec![function.name.clone()],
                                related_types: vec![],
                                suggested_fixes: vec!["Verify ownership before transfer".to_string()],
                                whitepaper_reference: Some("Sui Transfer Safety".to_string()),
                            }),
                        });
                    }
                }
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn verify_capability_usage(&mut self, function: &Function) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();

        for statement in &function.body {
            if let Statement::Call(name, _) = statement {
                if self.is_capability_usage(name) {
                    if let Err(e) = self.escape_analyzer.check_capability_usage(name) {
                        violations.push(SafetyViolation {
                            location: Location::default(),
                            violation_type: ViolationType::CapabilityViolation,
                            message: e,
                            severity: Severity::High,
                            context: Some(ViolationContext {
                                affected_functions: vec![function.name.clone()],
                                related_types: vec![],
                                suggested_fixes: vec!["Verify capability before use".to_string()],
                                whitepaper_reference: Some("Sui Capabilities".to_string()),
                            }),
                        });
                    }
                }
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn verify_shared_object_access(&self, function: &Function) -> Option<Vec<SafetyViolation>> {
        let mut violations = Vec::new();

        for statement in &function.body {
            if let Statement::Call(name, _) = statement {
                if self.is_shared_object_access(name) {
                    violations.push(SafetyViolation {
                        location: Location::default(),
                        violation_type: ViolationType::SharedObjectViolation,
                        message: "Shared object access requires consensus".to_string(),
                        severity: Severity::High,
                        context: Some(ViolationContext {
                            affected_functions: vec![function.name.clone()],
                            related_types: vec![],
                            suggested_fixes: vec!["Add consensus verification".to_string()],
                            whitepaper_reference: Some("Sui Shared Objects".to_string()),
                        }),
                    });
                }
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    fn is_capability_usage(&self, name: &str) -> bool {
        name.contains("_cap") || name.contains("capability")
    }

    fn is_shared_object_access(&self, name: &str) -> bool {
        name.contains("shared") || name.contains("consensus")
    }
} 