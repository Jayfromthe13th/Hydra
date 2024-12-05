use std::path::{Path, PathBuf};
use std::fs;
use toml::Value;
use crate::analyzer::HydraAnalyzer;
use crate::analyzer::types::AnalysisResult;

pub struct BuildSystem {
    package_path: PathBuf,
    manifest: PackageManifest,
    analyzer: HydraAnalyzer,
}

#[derive(Debug)]
pub struct PackageManifest {
    name: String,
    version: String,
    dependencies: Vec<Dependency>,
    sources: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct Dependency {
    name: String,
    git: Option<String>,
    rev: Option<String>,
    local: Option<PathBuf>,
}

impl BuildSystem {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let package_path = path.as_ref().to_path_buf();
        let manifest = Self::load_manifest(&package_path)?;
        
        Ok(Self {
            package_path,
            manifest,
            analyzer: HydraAnalyzer::new(),
        })
    }

    pub fn analyze_package(&mut self) -> Result<Vec<AnalysisResult>, String> {
        let mut results = Vec::new();

        // Set package path for analyzer context
        self.analyzer.set_package_path(
            self.package_path.to_string_lossy().to_string()
        );

        // Analyze each source file
        for source_path in &self.manifest.sources {
            let full_path = self.package_path.join(source_path);
            if let Ok(source) = fs::read_to_string(&full_path) {
                match crate::analyzer::parser::Parser::parse_module(&source) {
                    Ok(module) => {
                        results.push(self.analyzer.analyze_module(&module));
                    }
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", source_path.display(), e);
                    }
                }
            }
        }

        Ok(results)
    }

    fn load_manifest(path: &Path) -> Result<PackageManifest, String> {
        let manifest_path = path.join("Move.toml");
        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read Move.toml: {}", e))?;

        let value: Value = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse Move.toml: {}", e))?;

        Self::parse_manifest(value)
    }

    fn parse_manifest(value: Value) -> Result<PackageManifest, String> {
        let package = value.get("package")
            .ok_or("Missing [package] section")?;

        let name = package.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing package.name")?
            .to_string();

        let version = package.get("version")
            .and_then(|v| v.as_str())
            .ok_or("Missing package.version")?
            .to_string();

        let mut dependencies = Vec::new();
        if let Some(deps) = value.get("dependencies").and_then(|v| v.as_table()) {
            for (dep_name, dep_value) in deps {
                dependencies.push(Self::parse_dependency(dep_name, dep_value)?);
            }
        }

        let mut sources = Vec::new();
        if let Some(addresses) = value.get("addresses").and_then(|v| v.as_table()) {
            for (_, addr_value) in addresses {
                if let Some(modules) = addr_value.get("modules").and_then(|v| v.as_array()) {
                    for module in modules {
                        if let Some(path) = module.as_str() {
                            sources.push(PathBuf::from(path));
                        }
                    }
                }
            }
        }

        Ok(PackageManifest {
            name,
            version,
            dependencies,
            sources,
        })
    }

    fn parse_dependency(name: &str, value: &Value) -> Result<Dependency, String> {
        let git = value.get("git")
            .and_then(|v| v.as_str())
            .map(String::from);

        let rev = value.get("rev")
            .and_then(|v| v.as_str())
            .map(String::from);

        let local = value.get("local")
            .and_then(|v| v.as_str())
            .map(|p| PathBuf::from(p));

        Ok(Dependency {
            name: name.to_string(),
            git,
            rev,
            local,
        })
    }
}

// Add tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_manifest() {
        let manifest_content = r#"
            [package]
            name = "example"
            version = "0.1.0"

            [dependencies]
            Sui = { git = "https://github.com/MystenLabs/sui.git", rev = "main" }

            [addresses]
            example = { modules = ["sources/example.move"] }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("Move.toml");
        fs::write(&manifest_path, manifest_content).unwrap();

        let build_system = BuildSystem::new(temp_dir.path()).unwrap();
        assert_eq!(build_system.manifest.name, "example");
        assert_eq!(build_system.manifest.version, "0.1.0");
        assert!(!build_system.manifest.dependencies.is_empty());
        assert!(!build_system.manifest.sources.is_empty());
    }
} 