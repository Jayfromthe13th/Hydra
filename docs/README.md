# Hydra: Static Safety Analyzer for Sui Move

Hydra is a proof-of-concept static analyzer designed to detect safety issues in Sui Move smart contracts, with a focus on reference safety and object capability patterns.

## Core Features

### 1. Reference Safety Analysis (Ξimm)
- Three-value abstract domain tracking:
  - `NonRef`: Non-reference values
  - `OkRef`: Safe references
  - `InvRef`: References to invariant-protected state
- Reference leak detection
- Basic path tracking
- Escape point detection

### 2. Object Safety Verification
- Basic object lifecycle tracking
- Transfer safety analysis
- Capability-based access control
- Shared object consensus checks

### 3. Module Analysis
- Basic boundary detection
- Function-level analysis
- Initial isolation checks
- Cross-module call tracking

## Getting Started

### Installation
```bash
# Clone repository
git clone https://github.com/your-org/hydra
cd hydra

# Build
cargo build

# Run tests
cargo test
```

### Running Analysis
```rust
use hydra_analyzer::{
    analyzer::Analysis,
    parser::Parser,
};

fn main() {
    let source = include_str!("path/to/module.move");
    let mut analyzer = Analysis::new();
    let module = Parser::parse_module(source).unwrap();
    let result = analyzer.analyze_module(&module);
    
    // Check results
    if !result.reference_leaks.is_empty() {
        println!("Found reference leaks!");
    }
}
```

## Test Suite

### Reference Safety Tests
```move
// src/test_beta/reference_safety.move
module test::reference_safety {
    struct Data has key {
        id: UID,
        value: u64
    }

    // Should detect - reference escapes
    public fun unsafe_ref(data: &mut Data): &mut u64 {
        &mut data.value
    }

    // Should pass - scoped reference
    public fun safe_ref(data: &mut Data) {
        let value = &mut data.value;
        *value = 100;
    }
}
```

### Boundary Tests
```move
// src/test_beta/boundary_crossing.move
module trusted::core {
    // Should detect - untrusted access
    public fun leak_data(data: SecretData) {
        untrusted::module::receive(data)
    }
}
```

## Current Limitations

### Analysis Scope
- Function-level analysis only
- Limited cross-module analysis
- Basic pattern detection
- No symbolic execution

### Features
- No Move Prover integration
- Limited invariant checking
- Basic object tracking
- Simple capability model

## Benchmarks

Run benchmarks:
```bash
cargo bench
```

Current benchmarks:
- Module analysis performance
- Package analysis throughput
- Memory usage tracking
- Core analysis timing

## Development Status

### ✅ Implemented
- Basic reference tracking
- Escape analysis (Ξimm)
- Path analysis
- Initial safety verification
- Test framework

### 🚧 In Progress
- Full path sensitivity
- Complex invariant tracking
- Cross-module analysis
- Performance optimization

### ❌ Planned
- Move Prover integration
- Custom safety properties
- Automated fixes
- IDE integration

## Contributing

1. Fork repository
2. Create feature branch
3. Add tests
4. Submit pull request

## License

MIT License

## Acknowledgments

Based on research in:
- Sui Move's object model and resource safety
  - Sui's object-capability model
  - Sui Move's transfer safety and consensus mechanisms
  - Static analysis for capability-based systems