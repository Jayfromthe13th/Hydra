use crate::analyzer::call_graph::*;
use colored::*;
use std::collections::HashMap;

pub struct CallGraphOutput {
    analyzer: CallGraphAnalyzer,
}

impl CallGraphOutput {
    pub fn new(analyzer: CallGraphAnalyzer) -> Self {
        Self { analyzer }
    }

    pub fn generate_output(&self, format: &str, object_filter: Option<&str>, shared_only: bool) -> String {
        match format {
            "text" => self.generate_text_output(object_filter, shared_only),
            "dot" => self.generate_dot_output(object_filter, shared_only),
            "json" => self.generate_json_output(object_filter, shared_only),
            _ => "Unsupported output format".to_string(),
        }
    }

    fn generate_text_output(&self, object_filter: Option<&str>, shared_only: bool) -> String {
        let mut output = String::new();
        output.push_str("\nCall Graph Analysis\n");
        output.push_str("═══════════════════\n\n");

        // Objects Section
        output.push_str(&"Objects".blue().bold().to_string());
        output.push_str("\n-------\n");

        for (name, node) in &self.analyzer.objects {
            if shared_only && !node.is_shared {
                continue;
            }
            if let Some(filter) = object_filter {
                if !name.contains(filter) {
                    continue;
                }
            }

            // Object header
            let shared_indicator = if node.is_shared { "[SHARED] ".red().bold() } else { "".normal() };
            output.push_str(&format!("\n{}{}\n", shared_indicator, name.green()));

            // Functions that operate on this object
            if !node.functions.is_empty() {
                output.push_str("  Functions:\n");
                for func in &node.functions {
                    output.push_str(&format!("    → {}\n", func));
                }
            }

            // Called by
            if !node.called_by.is_empty() {
                output.push_str("  Called by:\n");
                for caller in &node.called_by {
                    output.push_str(&format!("    ← {}\n", caller));
                }
            }

            // References
            if !node.references.is_empty() {
                output.push_str("  Referenced by:\n");
                for reference in &node.references {
                    output.push_str(&format!("    ⟶ {}\n", reference));
                }
            }

            // Shared location
            if node.is_shared {
                if let Some(location) = &node.shared_at {
                    output.push_str(&format!("  Shared at: {}:{}:{}\n", 
                        location.file,
                        location.line,
                        location.column
                    ));
                }
            }
        }

        // Function Calls Section
        output.push_str("\nFunction Calls\n");
        output.push_str("-------------\n");
        for (caller, callees) in &self.analyzer.function_calls {
            if !callees.is_empty() {
                output.push_str(&format!("\n{}\n", caller.yellow()));
                for callee in callees {
                    output.push_str(&format!("  → {}\n", callee));
                }
            }
        }

        output
    }

    fn generate_dot_output(&self, object_filter: Option<&str>, shared_only: bool) -> String {
        let mut output = String::from("digraph call_graph {\n");
        output.push_str("  node [shape=box];\n");

        // Add nodes
        for (name, node) in &self.analyzer.objects {
            if shared_only && !node.is_shared {
                continue;
            }
            if let Some(filter) = object_filter {
                if !name.contains(filter) {
                    continue;
                }
            }

            let color = if node.is_shared { "red" } else { "black" };
            output.push_str(&format!("  \"{}\" [color={}];\n", name, color));
        }

        // Add edges
        for (caller, callees) in &self.analyzer.function_calls {
            for callee in callees {
                output.push_str(&format!("  \"{}\" -> \"{}\";\n", caller, callee));
            }
        }

        output.push_str("}\n");
        output
    }

    fn generate_json_output(&self, object_filter: Option<&str>, shared_only: bool) -> String {
        let mut map = HashMap::new();
        map.insert("objects", &self.analyzer.objects);
        map.insert("shared_objects", &self.analyzer.shared_objects);
        map.insert("function_calls", &self.analyzer.function_calls);
        serde_json::to_string_pretty(&map).unwrap_or_else(|_| "Error generating JSON".to_string())
    }
} 