use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt;
use crate::analyzer::call_graph::CallGraph;
use crate::analyzer::object_state::ObjectSafetyIssue;
use crate::analyzer::gas_estimator::GasEstimate;

#[derive(Debug, Default)]
pub struct AnalysisResult {
    pub safety_violations: Vec<SafetyViolation>,
    pub reference_leaks: Vec<ReferenceLeak>,
    pub object_safety_issues: Vec<ObjectSafetyIssue>,
    pub gas_estimates: HashMap<String, GasEstimate>,
    pub call_graph: Option<CallGraph>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyViolation {
    pub location: Location,
    pub violation_type: ViolationType,
    pub message: String,
    pub severity: Severity,
    pub context: Option<ViolationContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationContext {
    pub affected_functions: Vec<String>,
    pub related_types: Vec<String>,
    pub suggested_fixes: Vec<String>,
    pub whitepaper_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ViolationType {
    UnsafeTransfer,
    CapabilityLeak,
    SharedObjectViolation,
    InvariantViolation,
    UnsafePublicInterface,
    UnsafeObjectDestruction,
    UnsafeCapabilityUse,
    UnsafeHotPotato,
    DosVector,
    CallStackViolation,
    ExcessiveGas,
    TypeSafetyViolation,
    ArithmeticError,
    TimestampViolation,
    OwnershipViolation,
    UnauthorizedAccess,
    IDVerificationError,
    ConsensusViolation,
    ResourceSafetyViolation,
    CapabilityViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct ReferenceLeak {
    pub location: Location,
    pub leaked_field: FieldId,
    pub context: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Default)]
pub struct FieldId {
    pub module_name: String,
    pub struct_name: String,
    pub field_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    Base(String),
    MutableReference(Box<Type>),
    Reference(Box<Type>),
    Vector(Box<Type>),
    Generic(String, Vec<Type>),
}

impl fmt::Display for ViolationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeTransfer => write!(f, "Unsafe Transfer"),
            Self::CapabilityLeak => write!(f, "Capability Leak"),
            Self::SharedObjectViolation => write!(f, "Shared Object Violation"),
            Self::InvariantViolation => write!(f, "Invariant Violation"),
            Self::UnsafePublicInterface => write!(f, "Unsafe Public Interface"),
            Self::UnsafeObjectDestruction => write!(f, "Unsafe Object Destruction"),
            Self::UnsafeCapabilityUse => write!(f, "Unsafe Capability Use"),
            Self::UnsafeHotPotato => write!(f, "Unsafe Hot Potato"),
            Self::DosVector => write!(f, "DOS Vector"),
            Self::CallStackViolation => write!(f, "Call Stack Violation"),
            Self::ExcessiveGas => write!(f, "Excessive Gas Usage"),
            Self::TypeSafetyViolation => write!(f, "Type Safety Violation"),
            Self::ArithmeticError => write!(f, "Arithmetic Error"),
            Self::TimestampViolation => write!(f, "Timestamp Violation"),
            Self::OwnershipViolation => write!(f, "Ownership Violation"),
            Self::UnauthorizedAccess => write!(f, "Unauthorized Access"),
            Self::IDVerificationError => write!(f, "ID Verification Error"),
            Self::ConsensusViolation => write!(f, "Consensus Violation"),
            Self::ResourceSafetyViolation => write!(f, "Resource Safety Violation"),
            Self::CapabilityViolation => write!(f, "Capability Violation"),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Base(name) => write!(f, "{}", name),
            Type::MutableReference(inner) => write!(f, "&mut {}", inner),
            Type::Reference(inner) => write!(f, "&{}", inner),
            Type::Vector(inner) => write!(f, "vector<{}>", inner),
            Type::Generic(name, args) => {
                write!(f, "{}<", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            }
        }
    }
}