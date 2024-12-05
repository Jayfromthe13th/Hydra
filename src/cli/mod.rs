use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "hydra", about = "Sui Move Static Analyzer")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    /// Run security analysis
    Analyze {
        /// Path to the Move source file
        #[clap(value_parser)]
        path: PathBuf,
        
        /// Enable verbose output
        #[clap(short, long)]
        verbose: bool,
        
        /// Exit with error code on critical issues
        #[clap(long)]
        fail_on_critical: bool,
    },
    
    /// Generate call graph
    CallGraph {
        /// Path to the Move source file
        #[clap(value_parser)]
        path: PathBuf,
        
        /// Filter by object type
        #[clap(short, long)]
        object: Option<String>,
        
        /// Show only shared objects
        #[clap(short, long)]
        shared_only: bool,
        
        /// Output format (text, dot)
        #[clap(short, long, default_value = "text")]
        format: String,
    }
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
} 