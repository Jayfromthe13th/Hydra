use std::path::{Path, PathBuf};
use std::fs;
use toml::Value;
use crate::analyzer::types::*;

#[derive(Debug)]
pub struct SuiPackage {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub modules: Vec<ModuleInfo>,
    pub dependencies: Vec<PackageDependency>,
}

#[derive(Debug)]
pub struct ModuleInfo {
    pub name: String,
    pub path: PathBuf,
    pub address: String,
    pub is_test: bool,
}

#[derive(Debug)]
pub struct PackageDependency {
    pub name: String,
    pub source: DependencySource,
}

#[derive(Debug)]
pub enum DependencySource {
    Git {
        url: String,
        rev: String,
    },
    Local(PathBuf),
    Published {
        address: String,
        version: String,
    },
}

pub struct PackageLoader {
    root_path: PathBuf,
}

impl PackageLoader {
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    pub fn load_package(&self) -> Result<SuiPackage, String> {
        let manifest_path = self.root_path.join("Move.toml");
        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read Move.toml: {}", e))?;

        let manifest: Value = toml::from_str(&manifest_content)
            .map_err(|e| format!("Failed to parse Move.toml: {}", e))?;

        self.parse_manifest(&manifest)
    }

    fn parse_manifest(&self, manifest: &Value) -> Result<SuiPackage, String> {
        // Parse package section
        let package = manifest.get("package")
            .ok_or("Missing [package] section")?;

        let name = package.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing package.name")?
            .to_string();

        let version = package.get("version")
            .and_then(|v| v.as_str())
            .ok_or("Missing package.version")?
            .to_string();

        // Parse modules
        let mut modules = Vec::new();
        if let Some(addresses) = manifest.get("addresses") {
            for (addr, modules_table) in addresses.as_table().unwrap() {
                if let Some(module_paths) = modules_table.get("modules").and_then(|v| v.as_array()) {
                    for path in module_paths {
                        if let Some(path_str) = path.as_str() {
                            modules.push(ModuleInfo {
                                name: Path::new(path_str)
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string(),
                                path: self.root_path.join(path_str),
                                address: addr.clone(),
                                is_test: path_str.contains("tests") || path_str.contains("test"),
                            });
                        }
                    }
                }
            }
        }

        // Parse dependencies
        let mut dependencies = Vec::new();
        if let Some(deps) = manifest.get("dependencies").and_then(|v| v.as_table()) {
            for (name, dep_value) in deps {
                dependencies.push(self.parse_dependency(name, dep_value)?);
            }
        }

        Ok(SuiPackage {
            name,
            version,
            path: self.root_path.clone(),
            modules,
            dependencies,
        })
    }

    fn parse_dependency(&self, name: &str, value: &Value) -> Result<PackageDependency, String> {
        let source = if let Some(git) = value.get("git").and_then(|v| v.as_str()) {
            let rev = value.get("rev")
                .and_then(|v| v.as_str())
                .ok_or("Git dependency missing rev")?;
            
            DependencySource::Git {
                url: git.to_string(),
                rev: rev.to_string(),
            }
        } else if let Some(local) = value.get("local").and_then(|v| v.as_str()) {
            DependencySource::Local(PathBuf::from(local))
        } else if let Some(address) = value.get("address").and_then(|v| v.as_str()) {
            let version = value.get("version")
                .and_then(|v| v.as_str())
                .ok_or("Published dependency missing version")?;
            
            DependencySource::Published {
                address: address.to_string(),
                version: version.to_string(),
            }
        } else {
            return Err(format!("Invalid dependency specification for {}", name));
        };

        Ok(PackageDependency {
            name: name.to_string(),
            source,
        })
    }
}

pub struct PackageAnalyzer {
    package: SuiPackage,
}

impl PackageAnalyzer {
    pub fn new(package: SuiPackage) -> Self {
        Self { package }
    }

    pub fn analyze(&self) -> Result<Vec<AnalysisResult>, String> {
        let mut results = Vec::new();

        // Analyze each module
        for module in &self.package.modules {
            // Skip test modules if configured
            if module.is_test {
                continue;
            }

            let source = fs::read_to_string(&module.path)
                .map_err(|e| format!("Failed to read module {}: {}", module.name, e))?;

            match crate::analyzer::parser::Parser::parse_module(&source) {
                Ok(parsed_module) => {
                    let mut analyzer = crate::analyzer::HydraAnalyzer::new();
                    results.push(analyzer.analyze_module(&parsed_module));
                }
                Err(e) => {
                    return Err(format!("Failed to parse module {}: {}", module.name, e));
                }
            }
        }

        Ok(results)
    }
} 