use super::types::*;
use serde::Serialize;
use std::collections::HashMap;

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
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
    context_region: Option<SarifRegion>,
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
    #[serde(rename = "endLine")]
    end_line: u32,
    #[serde(rename = "endColumn")]
    end_column: u32,
    snippet: Option<SarifMessage>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
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

#[derive(Serialize)]
struct SarifRuleProperties {
    precision: String,
    #[serde(rename = "securitySeverity")]
    security_severity: String,
    tags: Vec<String>,
}

impl SarifReport {
    pub fn new(analysis_results: &[AnalysisResult]) -> Self {
        let mut report = Self {
            schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: Vec::new(),
        };

        let run = SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "Hydra".to_string(),
                    information_uri: "https://github.com/your-org/hydra".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    rules: get_default_rules(),
                },
            },
            results: convert_results(analysis_results),
            artifacts: Vec::new(),
            column_kind: "utf16CodeUnits".to_string(),
        };

        report.runs.push(run);
        report
    }

    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn convert_results(analysis_results: &[AnalysisResult]) -> Vec<SarifResult> {
    let mut sarif_results = Vec::new();

    for result in analysis_results {
        // Convert safety violations
        for violation in &result.safety_violations {
            sarif_results.push(convert_violation(violation));
        }

        // Convert reference leaks
        for leak in &result.reference_leaks {
            sarif_results.push(convert_leak(leak));
        }

        // Convert object safety issues
        for issue in &result.object_safety_issues {
            sarif_results.push(convert_object_issue(issue));
        }
    }

    sarif_results
}

fn convert_violation(violation: &SafetyViolation) -> SarifResult {
    SarifResult {
        rule_id: format!("HYDRA{:04}", get_rule_id(&violation.violation_type)),
        level: severity_to_sarif_level(&violation.severity),
        message: SarifMessage {
            text: violation.message.clone(),
        },
        locations: vec![convert_location(&violation.location)],
        fixes: None, // Could add fix suggestions here
    }
}

fn convert_leak(leak: &ReferenceLeak) -> SarifResult {
    SarifResult {
        rule_id: "HYDRA0001".to_string(),
        level: severity_to_sarif_level(&leak.severity),
        message: SarifMessage {
            text: leak.context.clone(),
        },
        locations: vec![convert_location(&leak.location)],
        fixes: None,
    }
}

fn convert_object_issue(issue: &ObjectSafetyIssue) -> SarifResult {
    SarifResult {
        rule_id: format!("HYDRA{:04}", get_object_rule_id(&issue.issue_type)),
        level: severity_to_sarif_level(&issue.severity),
        message: SarifMessage {
            text: issue.message.clone(),
        },
        locations: vec![convert_location(&issue.location)],
        fixes: None,
    }
}

fn convert_location(location: &Location) -> SarifLocation {
    SarifLocation {
        physical_location: SarifPhysicalLocation {
            artifact_location: SarifArtifactLocation {
                uri: location.file.clone(),
            },
            region: SarifRegion {
                start_line: location.line,
                start_column: location.column,
                end_line: location.line,
                end_column: location.column + 1,
                snippet: Some(SarifMessage {
                    text: location.context.clone(),
                }),
            },
            context_region: None,
        },
    }
}

fn severity_to_sarif_level(severity: &Severity) -> String {
    match severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
        Severity::Info => "none",
    }.to_string()
}

fn get_rule_id(violation_type: &ViolationType) -> u32 {
    match violation_type {
        ViolationType::ReferenceEscape => 1001,
        ViolationType::InvariantViolation => 1002,
        ViolationType::UnsafePublicInterface => 1003,
        ViolationType::UnsafeTransfer => 1004,
        ViolationType::CapabilityLeak => 1005,
        ViolationType::SharedObjectViolation => 1006,
    }
}

fn get_object_rule_id(issue_type: &ObjectIssueType) -> u32 {
    match issue_type {
        ObjectIssueType::UnsafeTransfer => 2001,
        ObjectIssueType::MissingOwnershipCheck => 2002,
        ObjectIssueType::InvalidSharedAccess => 2003,
        ObjectIssueType::CapabilityExposure => 2004,
        ObjectIssueType::UnsafeObjectConstruction => 2005,
        ObjectIssueType::InvalidTransferGuard => 2006,
    }
}

fn get_default_rules() -> Vec<SarifRule> {
    vec![
        SarifRule {
            id: "HYDRA1001".to_string(),
            name: "reference-escape".to_string(),
            short_description: SarifMessage {
                text: "Reference escape detected".to_string(),
            },
            full_description: SarifMessage {
                text: "A reference to protected data may escape its intended scope".to_string(),
            },
            help: SarifMessage {
                text: "Consider using a copy instead of a reference".to_string(),
            },
            properties: SarifRuleProperties {
                precision: "high".to_string(),
                security_severity: "8.0".to_string(),
                tags: vec!["security".to_string(), "sui-move".to_string()],
            },
        },
        // Add more rules...
    ]
} 