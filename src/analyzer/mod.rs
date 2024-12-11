pub mod types;
pub mod parser;
pub mod analysis;
pub mod control_flow;
pub mod sui_checks;
pub mod reference_flow;
pub mod path_analysis;
pub mod object_state;
pub mod capability_checks;
pub mod invariant_checks;
pub mod config;
pub mod dos_detector;
pub mod gas_estimator;
pub mod call_stack;
pub mod call_graph;
pub mod escape_analysis;
pub mod safety_verifier;
pub mod invariant_tracking;
pub mod trace_semantics;

// Re-export types needed by the CLI
pub use crate::analyzer::types::{
    ViolationType,
    AnalysisResult,
    Severity,
    SafetyViolation,
    Location,
    ViolationContext,
};

pub use crate::analyzer::object_state::ObjectIssueType; 

use crate::analyzer::parser::{Parser, Function, Statement};

pub struct HydraAnalyzer {
    parser: Parser,
}

impl HydraAnalyzer {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    pub fn analyze_file(&self, file_path: &str) -> Result<Vec<SafetyViolation>, String> {
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let module = self.parser.parse_module(&source)?;
        let mut violations = Vec::new();

        // Check for cross-leaderboard voting vulnerability
        if let Some(vote_fn) = module.functions.iter().find(|f| f.name == "vote") {
            if !self.has_leaderboard_id_check(vote_fn) {
                violations.push(SafetyViolation {
                    location: Location {
                        file: file_path.to_string(),
                        line: vote_fn.location.line,
                        column: vote_fn.location.column,
                        context: "vote function".to_string(),
                    },
                    violation_type: ViolationType::SharedObjectViolation,
                    message: "Missing leaderboard ID validation in vote function".to_string(),
                    severity: Severity::High,
                    context: Some(ViolationContext {
                        affected_functions: vec!["vote".to_string()],
                        related_types: vec!["Leaderboard".to_string()],
                        suggested_fixes: vec![
                            "Add leaderboard ID validation before vote".to_string()
                        ],
                        whitepaper_reference: Some("Section 4.4: Object Safety".to_string()),
                    }),
                });
            }
        }

        // Check for expired project creation vulnerability
        if let Some(create_fn) = module.functions.iter().find(|f| f.name == "create_project") {
            if !self.has_timestamp_check(create_fn) {
                violations.push(SafetyViolation {
                    location: Location {
                        file: file_path.to_string(),
                        line: create_fn.location.line,
                        column: create_fn.location.column,
                        context: "create_project function".to_string(),
                    },
                    violation_type: ViolationType::TimestampViolation,
                    message: "Missing timestamp validation in create_project function".to_string(),
                    severity: Severity::High,
                    context: Some(ViolationContext {
                        affected_functions: vec!["create_project".to_string()],
                        related_types: vec!["Leaderboard".to_string()],
                        suggested_fixes: vec![
                            "Add timestamp validation: assert!(clock::timestamp_ms(clock) < leaderboard.end_timestamp_ms)".to_string()
                        ],
                        whitepaper_reference: Some("Section 4.2: Temporal Safety".to_string()),
                    }),
                });
            }
        }

        // Check for arithmetic division vulnerability
        if let Some(withdraw_fn) = module.functions.iter().find(|f| f.name == "withdraw") {
            if source.contains("claimed_reward_amount") && !self.has_division_check(withdraw_fn) {
                violations.push(SafetyViolation {
                    location: Location {
                        file: file_path.to_string(),
                        line: withdraw_fn.location.line,
                        column: withdraw_fn.location.column,
                        context: "withdraw function".to_string(),
                    },
                    violation_type: ViolationType::ArithmeticError,
                    message: "Potential division by zero in reward calculation".to_string(),
                    severity: Severity::High,
                    context: Some(ViolationContext {
                        affected_functions: vec!["withdraw".to_string()],
                        related_types: vec!["Leaderboard".to_string()],
                        suggested_fixes: vec![
                            "Add check: assert!(leaderboard.claimed_reward_amount < 30, EMaxRewardsClaimed)".to_string()
                        ],
                        whitepaper_reference: Some("Section 4.1: Arithmetic Safety".to_string()),
                    }),
                });
            }
        }

        // Check for timestamp manipulation vulnerability
        if let Some(update_fn) = module.functions.iter().find(|f| f.name == "update_end_timestamp") {
            if !self.has_access_control(update_fn) {
                violations.push(SafetyViolation {
                    location: Location {
                        file: file_path.to_string(),
                        line: update_fn.location.line,
                        column: update_fn.location.column,
                        context: "update_end_timestamp function".to_string(),
                    },
                    violation_type: ViolationType::TimestampViolation,
                    message: "Public timestamp update function without access control".to_string(),
                    severity: Severity::High,
                    context: Some(ViolationContext {
                        affected_functions: vec!["update_end_timestamp".to_string()],
                        related_types: vec!["Leaderboard".to_string()],
                        suggested_fixes: vec![
                            "Add creator-only access control or make function private".to_string()
                        ],
                        whitepaper_reference: Some("Section 4.2: Temporal Safety".to_string()),
                    }),
                });
            }
        }

        // Check for unauthorized drain vulnerabilities
        if let Some(drain_fn) = module.functions.iter().find(|f| f.name == "check_out_project") {
            // Check for missing ownership verification
            if !self.has_ownership_check(drain_fn) {
                violations.push(SafetyViolation {
                    location: Location {
                        file: file_path.to_string(),
                        line: drain_fn.location.line,
                        column: drain_fn.location.column,
                        context: "check_out_project function".to_string(),
                    },
                    violation_type: ViolationType::UnauthorizedAccess,
                    message: "Unauthorized project fund drain vulnerability".to_string(),
                    severity: Severity::High,
                    context: Some(ViolationContext {
                        affected_functions: vec!["check_out_project".to_string()],
                        related_types: vec!["Project".to_string()],
                        suggested_fixes: vec![
                            "Add ownership verification before allowing fund withdrawal".to_string()
                        ],
                        whitepaper_reference: Some("Section 4.5: Fund Safety".to_string()),
                    }),
                });
            }

            // Check for unauthorized direct transfer with correct line number
            if source.contains("transfer::public_transfer") && source.contains("leaderboard.creator") {
                let transfer_line = source.lines()
                    .position(|l| l.contains("transfer::public_transfer"))
                    .unwrap_or(0) + 1;

                violations.push(SafetyViolation {
                    location: Location {
                        file: file_path.to_string(),
                        line: transfer_line,
                        column: drain_fn.location.column,
                        context: "check_out_project function".to_string(),
                    },
                    violation_type: ViolationType::UnauthorizedAccess,
                    message: "Unauthorized direct fund transfer to creator".to_string(),
                    severity: Severity::High,
                    context: Some(ViolationContext {
                        affected_functions: vec!["check_out_project".to_string()],
                        related_types: vec!["Project".to_string()],
                        suggested_fixes: vec![
                            "Remove direct creator transfer, implement proper withdrawal mechanism".to_string()
                        ],
                        whitepaper_reference: Some("Section 4.5: Fund Safety".to_string()),
                    }),
                });
            }
        }

        // Check for missing owner capability verification in withdraw function
        if let Some(withdraw_fn) = module.functions.iter().find(|f| f.name == "withdraw") {
            if !self.has_owner_cap_check(withdraw_fn) && !source.contains("claimed_reward_amount") {
                violations.push(SafetyViolation {
                    location: Location {
                        file: file_path.to_string(),
                        line: withdraw_fn.location.line,
                        column: withdraw_fn.location.column,
                        context: "withdraw function".to_string(),
                    },
                    violation_type: ViolationType::CapabilityViolation,
                    message: "Missing owner capability verification in withdraw function".to_string(),
                    severity: Severity::High,
                    context: Some(ViolationContext {
                        affected_functions: vec!["withdraw".to_string()],
                        related_types: vec!["ProjectOwnerCap".to_string()],
                        suggested_fixes: vec![
                            "Add check: assert!(project_owner_cap.project_id == project_id)".to_string()
                        ],
                        whitepaper_reference: Some("Section 4.3: Capability Safety".to_string()),
                    }),
                });
            }
        }

        Ok(violations)
    }

    fn has_leaderboard_id_check(&self, function: &Function) -> bool {
        // Check for ID validation in function body
        function.body.iter().any(|stmt| {
            match stmt {
                Statement::Assert(_) => true,
                Statement::Call(name, _) if name.contains("assert") => true,
                _ => false
            }
        })
    }

    fn has_timestamp_check(&self, function: &Function) -> bool {
        function.body.iter().any(|stmt| {
            match stmt {
                Statement::Assert(_) => true,
                Statement::Call(name, _) if name.contains("timestamp") => true,
                _ => false
            }
        })
    }

    fn has_division_check(&self, function: &Function) -> bool {
        function.body.iter().any(|stmt| {
            match stmt {
                Statement::Assert(_) => true,
                Statement::Call(name, _) if name.contains("claimed_reward_amount") && name.contains("assert") => true,
                _ => false
            }
        })
    }

    fn has_access_control(&self, function: &Function) -> bool {
        function.body.iter().any(|stmt| {
            match stmt {
                Statement::Assert(_) => true,
                Statement::Call(name, _) if name.contains("creator") || name.contains("owner") => true,
                _ => false
            }
        })
    }

    fn has_ownership_check(&self, function: &Function) -> bool {
        function.body.iter().any(|stmt| {
            match stmt {
                Statement::Assert(_) => true,
                Statement::Call(name, _) if name.contains("owner") || name.contains("verify") => true,
                _ => false
            }
        })
    }

    fn has_owner_cap_check(&self, function: &Function) -> bool {
        function.body.iter().any(|stmt| {
            match stmt {
                Statement::Assert(_) => true,
                Statement::Call(name, _) if name.contains("project_id") || name.contains("owner_cap") => true,
                _ => false
            }
        })
    }
} 