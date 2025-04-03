use crate::analyzer::types::*;
use std::fmt;
use serde::{Serialize, Deserialize};

pub mod reporter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydraError {
    pub message: String,
    pub severity: Severity,
    pub location: Location,
    pub code: ErrorCode,
    pub fixes: Vec<FixSuggestion>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ErrorKind {
    ParseError,
    TypeError,
    ReferenceError,
    SafetyViolation,
    InvariantViolation,
    CapabilityError,
    ObjectError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCode(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixSuggestion {
    pub description: String,
    pub replacement: String,
}

impl HydraError {
    pub fn new(message: String, severity: Severity) -> Self {
        Self {
            message,
            severity,
            location: Location::default(),
            code: ErrorCode("HYDRA0000".to_string()),
            fixes: Vec::new(),
        }
    }

    pub fn add_fix(&mut self, fix: FixSuggestion) {
        self.fixes.push(fix);
    }

    pub fn to_string(&self) -> String {
        let mut output = String::new();
        
        // Error code and message
        output.push_str(&format!("Error[{}]: {}\n", self.code.0, self.message));
        
        // Location
        output.push_str(&format!("  --> {}:{}:{}\n",
            self.location.file,
            self.location.line,
            self.location.column
        ));

        // Location context if available
        if !self.location.context.is_empty() {
            output.push_str(&format!("  Context: {}\n", self.location.context));
        }

        // Fixes
        if !self.fixes.is_empty() {
            output.push_str("\nSuggested fixes:\n");
            for fix in &self.fixes {
                output.push_str(&format!("  * {}\n", fix.description));
                if !fix.replacement.is_empty() {
                    output.push_str(&format!("    {}\n", fix.replacement));
                }
            }
        }

        output
    }

    pub fn from_violation(violation: SafetyViolation) -> Self {
        let mut error = Self::new(violation.message, violation.severity);
        error.location = violation.location;
        error
    }

    pub fn set_location(&mut self, location: Location) {
        self.location = location;
    }

    pub fn set_context(&mut self, context: String) {
        self.location.context = context;
    }

    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.location.context = format!(
            "{}\nRange: {}:{}", 
            context.code_snippet,
            context.highlight_range.0,
            context.highlight_range.1
        );
        self
    }

    pub fn to_diagnostic(&self) -> String {
        let mut output = String::new();
        
        // Error header
        output.push_str(&format!("Error[{}]: {}\n", self.code.0, self.message));
        
        // Location
        output.push_str(&format!("  --> {}:{}:{}\n", 
            self.location.file,
            self.location.line,
            self.location.column
        ));
        
        // Fixes
        if !self.fixes.is_empty() {
            output.push_str("\nSuggested fixes:\n");
            for fix in &self.fixes {
                output.push_str(&format!("  - {}\n", fix.description));
                if !fix.replacement.is_empty() {
                    output.push_str(&format!("    {}\n", fix.replacement));
                }
            }
        }
        
        output
    }
}

pub struct ErrorContext {
    pub code_snippet: String,
    pub highlight_range: (usize, usize),
    pub relevant_vars: Vec<String>,
}

impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Error collection
#[derive(Default)]
pub struct ErrorCollector {
    errors: Vec<HydraError>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, error: HydraError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> &[HydraError] {
        &self.errors
    }

    pub fn to_string(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Text => self.to_text(),
            OutputFormat::Json => self.to_json(),
            OutputFormat::Sarif => self.to_sarif(),
        }
    }

    fn to_text(&self) -> String {
        self.errors.iter()
            .map(|e| e.to_diagnostic())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.errors)
            .unwrap_or_else(|_| "Error serializing to JSON".to_string())
    }

    fn to_sarif(&self) -> String {
        // TODO: Implement SARIF format
        "SARIF format not yet implemented".to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,
    Json,
    Sarif,
}

#[cfg(test)]
fn error_kind_to_code(kind: &ErrorKind) -> u32 {
    match kind {
        ErrorKind::ParseError => 1,
        ErrorKind::TypeError => 2,
        ErrorKind::ReferenceError => 3,
        ErrorKind::SafetyViolation => 4,
        ErrorKind::InvariantViolation => 5,
        ErrorKind::CapabilityError => 6,
        ErrorKind::ObjectError => 7,
    }
} 