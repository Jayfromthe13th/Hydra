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