use crate::analyzer::types::*;
use colored::Colorize;
use std::fmt::Write;

pub struct ErrorReporter {
    errors: Vec<AnalysisError>,
}

#[derive(Debug)]
pub struct AnalysisError {
    pub severity: Severity,
    pub location: Location,
    pub message: String,
    pub suggested_fix: Option<String>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: AnalysisError) {
        self.errors.push(error);
    }

    pub fn report(&self) -> String {
        let mut output = String::new();
        
        if self.errors.is_empty() {
            writeln!(output, "{}", "No issues found".green().bold()).unwrap();
            return output;
        }

        for error in &self.errors {
            let header = self.format_error_header(error);
            writeln!(output, "{}", header).unwrap();
            writeln!(output, "{}", error.message.bold()).unwrap();
            
            if let Some(fix) = &error.suggested_fix {
                writeln!(output, "Suggested fix: {}", fix.blue()).unwrap();
            }
            writeln!(output).unwrap();
        }

        output
    }

    fn format_error_header(&self, error: &AnalysisError) -> String {
        let location = format!("{}:{}:{}", 
            error.location.file,
            error.location.line,
            error.location.column
        );
        
        match error.severity {
            Severity::Critical => format!("Critical Error at {}", location).red().bold(),
            Severity::High => format!("Error at {}", location).yellow().bold(),
            _ => format!("Warning at {}", location).normal(),
        }.to_string()
    }
} 