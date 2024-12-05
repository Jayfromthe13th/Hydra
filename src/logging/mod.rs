use std::sync::Mutex;
use chrono::Local;
use colored::*;
use std::collections::HashMap;
use crate::analyzer::types::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

#[derive(Clone)]
pub struct LogConfig {
    pub verbose: bool,
    pub show_timestamps: bool,
    pub colored_output: bool,
    pub log_level: LogLevel,
}

pub struct Logger {
    entries: Vec<LogEntry>,
    stats: AnalysisStats,
    config: LogConfig,
}

#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    message: String,
    context: Option<String>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            stats: AnalysisStats::default(),
            config: LogConfig::default(),
        }
    }

    pub fn configure(config: LogConfig) {
        let mut logger = LOGGER.lock().unwrap();
        logger.config = config;
    }

    pub fn log_violation(violation: &SafetyViolation) {
        let mut logger = LOGGER.lock().unwrap();
        let level = match violation.severity {
            Severity::Critical | Severity::High => LogLevel::Error,
            Severity::Medium => LogLevel::Warning,
            _ => LogLevel::Info,
        };

        logger.log(level, &violation.message, Some(&violation.location.context));
    }

    pub fn log_object_issue(issue: &ObjectSafetyIssue) {
        let mut logger = LOGGER.lock().unwrap();
        let level = match issue.severity {
            Severity::Critical | Severity::High => LogLevel::Error,
            Severity::Medium => LogLevel::Warning,
            _ => LogLevel::Info,
        };

        logger.log(level, &issue.message, Some(&issue.location.context));
    }

    fn log(&mut self, level: LogLevel, message: &str, context: Option<&str>) {
        if level as i32 <= self.config.log_level as i32 {
            let entry = LogEntry {
                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                level,
                message: message.to_string(),
                context: context.map(String::from),
            };
            self.entries.push(entry);
        }
    }

    pub fn format_report(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("\nHydra Analysis Report\n{}\n\n", 
            "=".repeat(20)));

        // Group entries by level
        let mut entries_by_level: HashMap<LogLevel, Vec<&LogEntry>> = HashMap::new();
        for entry in &self.entries {
            entries_by_level.entry(entry.level).or_default().push(entry);
        }

        // Format entries by severity
        for level in [LogLevel::Error, LogLevel::Warning, LogLevel::Info, LogLevel::Debug] {
            if let Some(entries) = entries_by_level.get(&level) {
                let header = format!("{:?} Level Messages", level);
                output.push_str(&format!("{}\n{}\n\n", header, "-".repeat(header.len())));

                for entry in entries {
                    output.push_str(&self.format_entry(entry));
                    output.push('\n');
                }
            }
        }

        // Statistics
        if self.config.verbose {
            output.push_str("\nAnalysis Statistics\n");
            output.push_str("------------------\n");
            output.push_str(&format!("Total issues: {}\n", self.entries.len()));
            output.push_str(&format!("Analysis time: {}ms\n", self.stats.total_time_ms));
            output.push_str(&format!("Modules analyzed: {}\n", self.stats.modules_analyzed));
            output.push_str(&format!("Memory usage: {} KB\n", self.stats.memory_usage_kb));
        }

        output
    }

    fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = String::new();

        if self.config.show_timestamps {
            output.push_str(&format!("[{}] ", entry.timestamp));
        }

        let level_str = match entry.level {
            LogLevel::Error => "ERROR".red(),
            LogLevel::Warning => "WARN".yellow(),
            LogLevel::Info => "INFO".blue(),
            LogLevel::Debug => "DEBUG".normal(),
        };

        output.push_str(&format!("{}: {}", level_str, entry.message));

        if let Some(context) = &entry.context {
            output.push_str(&format!("\n  Context: {}", context));
        }

        output
    }

    pub fn update_stats(&mut self, stats: AnalysisStats) {
        self.stats = stats;
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            show_timestamps: true,
            colored_output: true,
            log_level: LogLevel::Info,
        }
    }
}

// Public interface
pub fn configure(config: LogConfig) {
    Logger::configure(config);
}

pub fn log_violation(violation: &SafetyViolation) {
    Logger::log_violation(violation);
}

pub fn log_object_issue(issue: &ObjectSafetyIssue) {
    Logger::log_object_issue(issue);
}

pub fn get_report() -> String {
    LOGGER.lock().unwrap().format_report()
}

pub fn update_stats(stats: AnalysisStats) {
    LOGGER.lock().unwrap().update_stats(stats);
} 