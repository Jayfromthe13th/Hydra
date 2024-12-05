use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydraConfig {
    pub strict_mode: bool,
    pub ignore_tests: bool,
    pub max_module_size: usize,
    pub output_format: OutputFormat,
    pub checks: CheckConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConfig {
    pub transfer_safety: bool,
    pub capability_safety: bool,
    pub shared_objects: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Sarif,
}

impl Default for HydraConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            ignore_tests: false,
            max_module_size: 10000,
            output_format: OutputFormat::Text,
            checks: CheckConfig {
                transfer_safety: true,
                capability_safety: true,
                shared_objects: true,
            },
        }
    }
}

impl HydraConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))
    }
} 