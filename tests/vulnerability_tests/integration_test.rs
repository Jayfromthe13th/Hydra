use hydra_analyzer::{
    HydraAnalyzer,
    Parser,
    AnalysisResult,
};

fn analyze_module(source: &str) -> AnalysisResult {
    let mut analyzer = HydraAnalyzer::new();
    let module = Parser::parse_module(source).unwrap();
    analyzer.analyze_module(&module)
}

#[test]
fn test_basic_analysis() {
    let source = r#"
        module 0x1::test {
            struct Data has key { value: u64 }
            
            public fun test_func(data: &mut Data) {
                data.value = 100;
            }
        }
    "#;

    let result = analyze_module(source);
    assert!(!result.has_critical_issues());
}

#[test]
fn test_reference_safety() {
    let source = include_str!("../src/test_beta/reference_safety.move");
    let mut analyzer = Analysis::new();
    let module = Parser::parse_module(source).unwrap();
    let result = analyzer.analyze_module(&module);
    
    // Should detect reference escapes
    assert!(result.reference_leaks.iter().any(|leak| 
        leak.context.contains("unsafe_ref")
    ));

    // Should detect nested escapes
    assert!(result.reference_leaks.iter().any(|leak|
        leak.context.contains("nested_ref_escape")
    ));

    // Should detect vector escapes
    assert!(result.reference_leaks.iter().any(|leak|
        leak.context.contains("array_ref_escape")
    ));

    // Should pass safe cases
    assert!(!result.reference_leaks.iter().any(|leak|
        leak.context.contains("safe_ref")
    ));
}

#[test]
fn test_boundary_crossing() {
    let source = include_str!("../src/test_beta/boundary_crossing.move");
    let mut analyzer = Analysis::new();
    let module = Parser::parse_module(source).unwrap();
    let result = analyzer.analyze_module(&module);

    // Should detect trust boundary violations
    assert!(result.safety_violations.iter().any(|v|
        matches!(v.violation_type, ViolationType::UnauthorizedAccess)
    ));
} 