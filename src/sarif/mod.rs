use crate::analyzer::types::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct SarifReport {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
    artifacts: Vec<SarifArtifact>,
    #[serde(rename = "columnKind")]
    column_kind: String,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: String,
    #[serde(rename = "informationUri")]
    information_uri: String,
    version: String,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
struct SarifRule {
    id: String,
    name: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
    #[serde(rename = "fullDescription")]
    full_description: SarifMessage,
    help: SarifMessage,
    properties: SarifRuleProperties,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    fixes: Option<Vec<SarifFix>>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifArtifact {
    uri: String,
    description: Option<SarifMessage>,
}

impl SarifReport {
    pub fn new(results: &[AnalysisResult]) -> Self {
        Self {
            schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: vec![create_run(results)],
        }
    }

    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn create_run(results: &[AnalysisResult]) -> SarifRun {
    SarifRun {
        tool: SarifTool {
            driver: SarifDriver {
                name: "Hydra".to_string(),
                information_uri: "https://github.com/your-org/hydra".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                rules: create_rules(),
            },
        },
        results: create_results(results),
        artifacts: Vec::new(),
        column_kind: "utf16CodeUnits".to_string(),
    }
}

fn create_rules() -> Vec<SarifRule> {
    vec![
        SarifRule {
            id: "HYDRA001".to_string(),
            name: "reference-leak".to_string(),
            short_description: SarifMessage {
                text: "Reference leak detected".to_string(),
            },
            full_description: SarifMessage {
                text: "A reference escapes its intended scope".to_string(),
            },
            help: SarifMessage {
                text: "Consider using a copy instead of a reference".to_string(),
            },
            properties: SarifRuleProperties {
                precision: "high".to_string(),
                security_severity: "8.0".to_string(),
                tags: vec!["security".to_string()],
            },
        },
    ]
}

fn create_results(analysis_results: &[AnalysisResult]) -> Vec<SarifResult> {
    let mut results = Vec::new();
    
    for result in analysis_results {
        // Convert safety violations
        for violation in &result.safety_violations {
            results.push(SarifResult {
                rule_id: format!("HYDRA{:03}", violation_type_to_rule_id(&violation.violation_type)),
                level: severity_to_sarif_level(&violation.severity),
                message: SarifMessage {
                    text: violation.message.clone(),
                },
                locations: vec![create_location(&violation.location)],
                fixes: None,
            });
        }

        // Convert reference leaks
        for leak in &result.reference_leaks {
            results.push(SarifResult {
                rule_id: "HYDRA001".to_string(),
                level: severity_to_sarif_level(&leak.severity),
                message: SarifMessage {
                    text: leak.context.clone(),
                },
                locations: vec![create_location(&leak.location)],
                fixes: None,
            });
        }
    }

    results
}

fn severity_to_sarif_level(severity: &Severity) -> String {
    match severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
        Severity::Info => "none",
    }.to_string()
}

fn violation_type_to_rule_id(violation_type: &ViolationType) -> u32 {
    match violation_type {
        ViolationType::ReferenceEscape => 001,
        ViolationType::InvariantViolation => 002,
        ViolationType::UnsafePublicInterface => 003,
        ViolationType::UnsafeTransfer => 004,
        ViolationType::CapabilityLeak => 005,
        ViolationType::SharedObjectViolation => 006,
    }
}

fn create_location(_location: &Location) -> SarifLocation {
    // Implementation details...
    unimplemented!()
}

#[derive(Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: u32,
    #[serde(rename = "startColumn")]
    start_column: u32,
}

#[derive(Serialize)]
struct SarifRuleProperties {
    precision: String,
    #[serde(rename = "securitySeverity")]
    security_severity: String,
    tags: Vec<String>,
}

#[derive(Serialize)]
struct SarifFix {
    description: SarifMessage,
    #[serde(rename = "artifactChanges")]
    artifact_changes: Vec<SarifArtifactChange>,
}

#[derive(Serialize)]
struct SarifArtifactChange {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    replacements: Vec<SarifReplacement>,
}

#[derive(Serialize)]
struct SarifReplacement {
    #[serde(rename = "deletedRegion")]
    deleted_region: SarifRegion,
    #[serde(rename = "insertedContent")]
    inserted_content: SarifInsertedContent,
}

#[derive(Serialize)]
struct SarifInsertedContent {
    text: String,
} 