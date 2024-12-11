use std::fs;
use colored::*;
use hydra_analyzer::analyzer::HydraAnalyzer;

fn print_banner() {
    println!(r#"
██╗  ██╗██╗   ██╗██████╗ ██████╗  █████╗ 
██║  ██║╚██╗ ██╔╝██╔══██╗██╔══██╗██╔══██╗
███████║ ╚████╔╝ ██║  ██║██████╔╝███████║
██╔══██║  ╚██╔╝  ██║  ██║██╔══██╗██╔══██║
██║  ██║   ██║   ██████╔╝██║  ██║██║  ██║
╚═╝  ╚═╝   ╚═╝   ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
"#);
}

fn main() {
    print_banner();
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: hydra analyze <file_or_directory>");
        return;
    }

    let path = &args[2];
    let analyzer = HydraAnalyzer::new();

    // If path is a directory, analyze all .move files in it
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.is_dir() {
            let entries = fs::read_dir(path).expect("Failed to read directory");
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "move") {
                        analyze_file(&analyzer, &path.to_string_lossy());
                    }
                }
            }
        } else {
            analyze_file(&analyzer, path);
        }
    }
}

fn analyze_file(analyzer: &HydraAnalyzer, file_path: &str) {
    match analyzer.analyze_file(file_path) {
        Ok(violations) => {
            if violations.is_empty() {
                println!("\n✅ No vulnerabilities detected in {}", file_path);
            } else {
                println!("\n⚠️  Vulnerabilities detected in {}:", file_path);
                for violation in violations {
                    println!("\n[{}] {}", 
                        violation.severity.to_string().red().bold(), 
                        violation.message
                    );
                    println!("Location: {}:{}:{}", violation.location.file, 
                        violation.location.line, violation.location.column);
                    if let Some(context) = violation.context {
                        println!("Suggested fix: {}", context.suggested_fixes[0]);
                    }
                    println!(); // Add blank line between vulnerabilities
                }
            }
        }
        Err(e) => println!("Error analyzing file {}: {}", file_path, e),
    }
}
  