use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PatternDatabase {
    patterns: HashMap<PatternType, Vec<String>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum PatternType {
    ResourceLeak,
    UnsafeTransfer,
    MissingCheck,
    ExternalCallInLoop,
    UnauthorizedAccess,
}

#[derive(Debug, Clone, Copy)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

impl PatternDatabase {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Resource leak patterns
        patterns.insert(PatternType::ResourceLeak, vec![
            "init".to_string(),
            "create".to_string(),
            "store".to_string(),
        ]);

        // Unsafe transfer patterns
        patterns.insert(PatternType::UnsafeTransfer, vec![
            "transfer".to_string(),
            "send".to_string(),
            "move".to_string(),
        ]);

        // Missing check patterns
        patterns.insert(PatternType::MissingCheck, vec![
            "assert".to_string(),
            "verify".to_string(),
            "check".to_string(),
        ]);

        // External call patterns
        patterns.insert(PatternType::ExternalCallInLoop, vec![
            "while".to_string(),
            "for".to_string(),
            "loop".to_string(),
        ]);

        // Unauthorized access patterns
        patterns.insert(PatternType::UnauthorizedAccess, vec![
            "admin".to_string(),
            "owner".to_string(),
            "capability".to_string(),
        ]);

        Self { patterns }
    }

    pub fn matches_pattern(&self, pattern_type: PatternType, text: &str) -> bool {
        if let Some(patterns) = self.patterns.get(&pattern_type) {
            patterns.iter().any(|pattern| text.contains(pattern))
        } else {
            false
        }
    }

    pub fn get_risk_level(&self, pattern_type: PatternType) -> RiskLevel {
        match pattern_type {
            PatternType::ResourceLeak => RiskLevel::Critical,
            PatternType::UnsafeTransfer => RiskLevel::High,
            PatternType::MissingCheck => RiskLevel::High,
            PatternType::ExternalCallInLoop => RiskLevel::Medium,
            PatternType::UnauthorizedAccess => RiskLevel::Critical,
        }
    }
} 