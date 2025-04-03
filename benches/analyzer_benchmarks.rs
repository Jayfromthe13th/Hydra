use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hydra_analyzer::{
    analyzer::{Analysis, AnalyzerConfig},
    parser::Parser,
};

fn analyze_module_benchmark(c: &mut Criterion) {
    let source = include_str!("../src/test_beta/invariant_safety.move");
    
    c.bench_function("analyze complex module", |b| {
        b.iter(|| {
            let config = AnalyzerConfig {
                strict_mode: true,
                check_transfer_safety: true,
                check_capability_safety: true,
                check_shared_objects: true,
                max_module_size: 10000,
                ignore_tests: false,
            };

            let mut analyzer = Analysis::new();
            let module = Parser::parse_module(black_box(source)).unwrap();
            analyzer.analyze_module(&module)
        })
    });
}

fn analyze_package_benchmark(c: &mut Criterion) {
    let test_files = [
        include_str!("../src/test_beta/reference_safety.move"),
        include_str!("../src/test_beta/boundary_crossing.move"),
        include_str!("../src/test_beta/invariant_safety.move"),
        include_str!("../src/test_beta/consensus_safety.move"),
    ];

    c.bench_function("analyze full test suite", |b| {
        b.iter(|| {
            let mut analyzer = Analysis::new();

            for source in test_files.iter() {
                let module = Parser::parse_module(black_box(source)).unwrap();
                analyzer.analyze_module(&module);
            }
        })
    });
}

fn memory_usage_benchmark(c: &mut Criterion) {
    let source = include_str!("../src/test_beta/invariant_safety.move");
    
    c.bench_function("memory usage", |b| {
        b.iter(|| {
            let mut analyzer = Analysis::new();
            let module = Parser::parse_module(black_box(source)).unwrap();
            analyzer.analyze_module(&module)
        })
    });
}

fn reference_safety_benchmark(c: &mut Criterion) {
    let source = include_str!("../src/test_beta/reference_safety.move");
    
    c.bench_function("reference safety analysis", |b| {
        b.iter(|| {
            let mut analyzer = Analysis::new();
            let module = Parser::parse_module(black_box(source)).unwrap();
            analyzer.analyze_module(&module)
        })
    });
}

fn boundary_crossing_benchmark(c: &mut Criterion) {
    let source = include_str!("../src/test_beta/boundary_crossing.move");
    
    c.bench_function("boundary crossing analysis", |b| {
        b.iter(|| {
            let mut analyzer = Analysis::new();
            let module = Parser::parse_module(black_box(source)).unwrap();
            analyzer.analyze_module(&module)
        })
    });
}

fn consensus_safety_benchmark(c: &mut Criterion) {
    let source = include_str!("../src/test_beta/consensus_safety.move");
    
    c.bench_function("consensus safety analysis", |b| {
        b.iter(|| {
            let mut analyzer = Analysis::new();
            let module = Parser::parse_module(black_box(source)).unwrap();
            analyzer.analyze_module(&module)
        })
    });
}

criterion_group!(
    benches,
    analyze_module_benchmark,
    analyze_package_benchmark,
    memory_usage_benchmark,
    reference_safety_benchmark,
    boundary_crossing_benchmark,
    consensus_safety_benchmark
);
criterion_main!(benches); 