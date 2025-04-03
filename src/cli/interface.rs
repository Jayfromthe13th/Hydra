use crate::analyzer::{HydraAnalyzer, AnalyzerConfig, types::*};
use crate::error::ErrorCollector;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use colored::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "hydra", about = "Sui Move Static Analyzer")]
pub struct HydraCli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Analyze Sui Move code for safety violations
    #[structopt(name = "analyze")]
    Analyze {
        /// Input Move file or directory
        #[structopt(parse(from_os_str))]
        input: PathBuf,

        /// Output format (text, json, sarif)
        #[structopt(short, long, default_value = "text")]
        format: String,

        /// Enable strict mode
        #[structopt(long)]
        strict: bool,

        /// Verbose output
        #[structopt(short, long)]
        verbose: bool,

        /// Show suggested fixes
        #[structopt(short, long)]
        fixes: bool,
    },
}

pub struct CliRunner {
    analyzer: HydraAnalyzer,
    error_collector: ErrorCollector,
    config: AnalyzerConfig,
}

impl CliRunner {
    pub fn new() -> Self {
        Self {
            analyzer: HydraAnalyzer::new(),
            error_collector: ErrorCollector::new(),
            config: AnalyzerConfig::default(),
        }
    }

    pub fn run(&mut self, cli: HydraCli) -> Result<(), String> {
        match cli.cmd {
            Command::Analyze { input, format, strict, verbose, fixes } => {
                self.config.strict_mode = strict;
                self.analyzer = HydraAnalyzer::with_config(self.config.clone());
                
                // Process input
                let results = self.process_input(&input)?;
                
                // Format and output results
                self.output_results(results, &format, verbose, fixes)?;
                
                Ok(())
            }
        }
    }

    fn process_input(&mut self, input: &Path) -> Result<Vec<AnalysisResult>, String> {
        let mut results = Vec::new();

        if input.is_file() {
            // Process single file
            let source = std::fs::read_to_string(input)
                .map_err(|e| format!("Failed to read input file: {}", e))?;

            match crate::analyzer::parser::Parser::parse_module(&source) {
                Ok(module) => {
                    results.push(self.analyzer.analyze_module(&module));
                }
                Err(e) => {
                    return Err(format!("Failed to parse module: {}", e));
                }
            }
        } else if input.is_dir() {
            // Process directory
            for entry in std::fs::read_dir(input)
                .map_err(|e| format!("Failed to read directory: {}", e))? {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.extension().map_or(false, |ext| ext == "move") {
                    results.extend(self.process_input(&path)?);
                }
            }
        } else {
            return Err("Input path does not exist".to_string());
        }

        Ok(results)
    }

    fn output_results(
        &self,
        results: Vec<AnalysisResult>,
        format: &str,
        verbose: bool,
        show_fixes: bool,
    ) -> Result<(), String> {
        match format {
            "text" => self.output_text(results, verbose, show_fixes),
            "json" => self.output_json(results),
            "sarif" => self.output_sarif(results),
            _ => Err(format!("Unsupported output format: {}", format)),
        }
    }

    fn output_text(
        &self,
        results: Vec<AnalysisResult>,
        verbose: bool,
        show_fixes: bool,
    ) -> Result<(), String> {
        let mut has_issues = false;

        for result in results {
            // Print safety violations
            for violation in result.safety_violations {
                has_issues = true;
                println!("{}", "Safety Violation:".red().bold());
                println!("  {}", violation.message);
                
                if verbose {
                    println!("  Location: {}:{}:{}", 
                        violation.location.file,
                        violation.location.line,
                        violation.location.column
                    );
                    println!("  Severity: {:?}", violation.severity);
                }
                
                if show_fixes {
                    if let Some(fix) = self.suggest_fix(&violation) {
                        println!("  Suggested fix: {}", fix.blue());
                    }
                }
                println!();
            }

            // Print reference leaks
            for leak in result.reference_leaks {
                has_issues = true;
                println!("{}", "Reference Leak:".yellow().bold());
                println!("  {}", leak.context);
                
                if verbose {
                    println!("  Location: {}:{}:{}", 
                        leak.location.file,
                        leak.location.line,
                        leak.location.column
                    );
                    println!("  Field: {}", leak.leaked_field.field_name);
                    println!("  Severity: {:?}", leak.severity);
                }
                
                if show_fixes {
                    if let Some(fix) = self.suggest_leak_fix(&leak) {
                        println!("  Suggested fix: {}", fix.blue());
                    }
                }
                println!();
            }
        }

        if !has_issues {
            println!("{}", "✓ No issues found".green());
        }

        Ok(())
    }

    fn output_json(&self, results: Vec<AnalysisResult>) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| format!("Failed to serialize results: {}", e))?;
        println!("{}", json);
        Ok(())
    }

    fn output_sarif(&self, results: Vec<AnalysisResult>) -> Result<(), String> {
        let sarif_report = crate::sarif::SarifReport::new(&results);
        let json = sarif_report.to_string()
            .map_err(|e| format!("Failed to generate SARIF report: {}", e))?;
        println!("{}", json);
        Ok(())
    }

    fn suggest_fix(&self, violation: &SafetyViolation) -> Option<String> {
        match violation.violation_type {
            ViolationType::ReferenceEscape => 
                Some("Consider returning a copy instead of a reference".to_string()),
            ViolationType::InvariantViolation =>
                Some("Ensure invariant conditions are maintained".to_string()),
            ViolationType::UnsafePublicInterface =>
                Some("Make the function private or remove mutable reference parameters".to_string()),
            ViolationType::UnsafeTransfer =>
                Some("Add proper ownership verification before transfer".to_string()),
            ViolationType::CapabilityLeak =>
                Some("Restrict capability access to internal module functions".to_string()),
            ViolationType::SharedObjectViolation =>
                Some("Implement proper synchronization for shared object access".to_string()),
        }
    }

    fn suggest_leak_fix(&self, leak: &ReferenceLeak) -> Option<String> {
        Some(format!(
            "Consider using a copy or owned value instead of a reference to {}",
            leak.leaked_field.field_name
        ))
    }
} 