pub mod analyzer;
pub mod cli;
pub mod error;

pub use crate::analyzer::parser::Parser;
pub use crate::analyzer::config::AnalyzerConfig;
pub use analyzer::types::*;
pub use analyzer::gas_estimator::GasEstimate;
pub use analyzer::call_stack::CallStackAnalyzer;
pub use analyzer::dos_detector::DosDetector;
pub use analyzer::call_graph::CallGraph;
pub use analyzer::object_state::{ObjectStateTracker, ObjectSafetyIssue};

// Re-export types needed by the CLI
pub use analyzer::types::{
    ViolationType,
    AnalysisResult,
    Severity,
    SafetyViolation,
    Location,
    ViolationContext,
};

pub use crate::analyzer::object_state::ObjectIssueType;

// Re-export HydraAnalyzer for convenience
pub use crate::analyzer::HydraAnalyzer;