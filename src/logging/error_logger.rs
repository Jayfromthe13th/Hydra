use crate::analyzer::types::*;
use colored::*;
use std::collections::HashMap;
use chrono::Local;

pub struct ErrorLogger {
    errors: Vec<LoggedError>,
    stats: ErrorStats,
    config: LogConfig,
}

#[derive(Debug)]
struct LoggedError {
    timestamp: String,
    level: ErrorLevel,
    message: String,
    location: Location,
    context: String,
    fixes: Vec<String>,
}

#[derive(Debug)]
struct ErrorStats {
    total_errors: usize,
    by_severity: HashMap<Severity, usize>,
    by_type: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct LogConfig {
    show_timestamps: bool,
    colored_output: bool,
    verbose: bool,
    min_severity: Severity,
}

#[derive(Debug)]
enum ErrorLevel {
    Error,
    Warning,
    Info,
}

impl ErrorLogger {
    pub fn new(config: LogConfig) -> Self {
        Self {
            errors: Vec::new(),
            stats: ErrorStats::new(),
            config,
        }
    }

    pub fn log_violation(&mut self, violation: &SafetyViolation) {
        let level = match violation.severity {
            Severity::Critical | Severity::High => ErrorLevel::Error,
            Severity::Medium => ErrorLevel::Warning,
            _ => ErrorLevel::Info,
        };

        let error = LoggedError {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            level,
            message: violation.message.clone(),
            location: violation.location.clone(),
            context: format!("Safety violation: {:?}", violation.violation_type),
            fixes: self.get_violation_fixes(violation),
        };

        self.add_error(error, violation.severity.clone());
    }

    pub fn log_leak(&mut self, leak: &ReferenceLeak) {
        let error = LoggedError {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            level: ErrorLevel::Error,
            message: leak.context.clone(),
            location: leak.location.clone(),
            context: format!("Reference leak in field: {}", leak.leaked_field.field_name),
            fixes: self.get_leak_fixes(leak),
        };

        self.add_error(error, leak.severity.clone());
    }

    pub fn format_report(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.format_header());
        output.push_str("\n\n");

        // Group errors by severity
        for severity in [Severity::Critical, Severity::High, Severity::Medium, Severity::Low] {
            let severity_errors: Vec<_> = self.errors.iter()
                .filter(|e| self.get_severity(&e.level) == severity)
                .collect();

            if !severity_errors.is_empty() {
                output.push_str(&self.format_severity_section(&severity, &severity_errors));
                output.push_str("\n");
            }
        }

        // Statistics
        if self.config.verbose {
            output.push_str("\n");
            output.push_str(&self.format_statistics());
        }

        output
    }

    fn format_header(&self) -> String {
        let total = self.stats.total_errors;
        let critical = self.stats.by_severity.get(&Severity::Critical).unwrap_or(&0);
        
        let header = format!("Found {} issues ({} critical)", total, critical);
        
        if self.config.colored_output {
            if *critical > 0 {
                header.red().bold().to_string()
            } else if total > 0 {
                header.yellow().bold().to_string()
            } else {
                "No issues found".green().bold().to_string()
            }
        } else {
            header
        }
    }

    fn format_severity_section(&self, severity: &Severity, errors: &[&LoggedError]) -> String {
        let mut output = String::new();

        // Section header
        let header = format!("{:?} Severity Issues", severity);
        output.push_str(&if self.config.colored_output {
            match severity {
                Severity::Critical => header.red().bold().to_string(),
                Severity::High => header.yellow().bold().to_string(),
                _ => header.normal().to_string(),
            }
        } else {
            header
        });
        output.push_str("\n");

        // Error details
        for error in errors {
            output.push_str(&self.format_error(error));
            output.push_str("\n");
        }

        output
    }

    fn format_error(&self, error: &LoggedError) -> String {
        let mut output = String::new();

        // Timestamp
        if self.config.show_timestamps {
            output.push_str(&format!("[{}] ", error.timestamp));
        }

        // Location and message
        output.push_str(&format!("  → {}: {}\n",
            error.location.file,
            if self.config.colored_output {
                error.message.bold().to_string()
            } else {
                error.message.clone()
            }
        ));

        // Context
        if !error.context.is_empty() {
            output.push_str(&format!("    {}\n", error.context));
        }

        // Source location
        output.push_str(&format!("    at {}:{}:{}\n",
            error.location.file,
            error.location.line,
            error.location.column
        ));

        // Fixes
        if !error.fixes.is_empty() {
            output.push_str("    Suggested fixes:\n");
            for fix in &error.fixes {
                output.push_str(&format!("      - {}\n", 
                    if self.config.colored_output {
                        fix.blue().to_string()
                    } else {
                        fix.clone()
                    }
                ));
            }
        }

        output
    }

    fn format_statistics(&self) -> String {
        let mut output = String::new();
        
        output.push_str("Analysis Statistics\n");
        output.push_str("-------------------\n");
        output.push_str(&format!("Total issues: {}\n", self.stats.total_errors));
        
        output.push_str("\nBy severity:\n");
        for (severity, count) in &self.stats.by_severity {
            output.push_str(&format!("  {:?}: {}\n", severity, count));
        }

        output.push_str("\nBy type:\n");
        for (type_name, count) in &self.stats.by_type {
            output.push_str(&format!("  {}: {}\n", type_name, count));
        }

        output
    }

    fn add_error(&mut self, error: LoggedError, severity: Severity) {
        // Update statistics
        self.stats.total_errors += 1;
        *self.stats.by_severity.entry(severity).or_insert(0) += 1;
        *self.stats.by_type.entry(error.context.clone()).or_insert(0) += 1;

        // Add error if it meets severity threshold
        if severity as i32 <= self.config.min_severity as i32 {
            self.errors.push(error);
        }
    }

    fn get_severity(&self, level: &ErrorLevel) -> Severity {
        match level {
            ErrorLevel::Error => Severity::Critical,
            ErrorLevel::Warning => Severity::Medium,
            ErrorLevel::Info => Severity::Low,
        }
    }

    fn get_violation_fixes(&self, violation: &SafetyViolation) -> Vec<String> {
        match violation.violation_type {
            ViolationType::ReferenceEscape => 
                vec!["Consider returning a copy instead of a reference".to_string()],
            ViolationType::InvariantViolation =>
                vec!["Ensure invariant conditions are maintained".to_string()],
            ViolationType::UnsafePublicInterface =>
                vec!["Make the function private or remove mutable reference parameters".to_string()],
            ViolationType::UnsafeTransfer =>
                vec!["Add proper ownership verification before transfer".to_string()],
            ViolationType::CapabilityLeak =>
                vec!["Restrict capability access to internal module functions".to_string()],
            ViolationType::SharedObjectViolation =>
                vec!["Implement proper synchronization for shared object access".to_string()],
        }
    }

    fn get_leak_fixes(&self, leak: &ReferenceLeak) -> Vec<String> {
        vec![
            format!("Consider using a copy instead of a reference to {}", leak.leaked_field.field_name),
            "Ensure reference does not escape its scope".to_string(),
        ]
    }
}

impl ErrorStats {
    fn new() -> Self {
        Self {
            total_errors: 0,
            by_severity: HashMap::new(),
            by_type: HashMap::new(),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            show_timestamps: true,
            colored_output: true,
            verbose: false,
            min_severity: Severity::Low,
        }
    }
} 