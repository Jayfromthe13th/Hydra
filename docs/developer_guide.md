# Hydra Developer Guide

## Architecture Overview

### Core Components

1. **Reference Analysis (Ξimm)**
   - Three-value abstract domain (NonRef, OkRef, InvRef)
   - Reference state tracking
   - Path-sensitive analysis
   - Escape detection

2. **Safety Check System**
   - Object capability verification
   - Shared object consensus
   - Transfer safety
   - Dynamic field safety

3. **Test Framework**
   - Reference safety tests
   - Boundary crossing tests
   - Invariant safety tests
   - Consensus safety tests

## Implementation Details

### 1. Reference Analysis

The core reference analysis is implemented through:

```rust
// Reference state tracking
impl EscapeAnalyzer {
    pub fn analyze_function(&mut self, function: &Function) -> Result<(), String> {
        // Track reference states
        for statement in &function.body {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }
}

// Path analysis
impl PathAnalyzer {
    pub fn analyze_paths(&mut self, function: &Function) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        for statement in &function.body {
            self.analyze_statement(statement, &mut leaks);
        }
        leaks
    }
}
```

### 2. Safety Checks

Current safety checks include:

```rust
// Object capability checks
impl CapabilityChecker {
    pub fn check_capability_safety(&mut self, function: &Function) -> Vec<SafetyViolation> {
        // Verify capability usage
    }
}

// Shared object checks
impl SafetyVerifier {
    fn verify_shared_object_access(&self, module: &Module) -> Option<Vec<SafetyViolation>> {
        // Verify consensus requirements
    }
}
```

### 3. Testing Framework

Test cases are organized by safety category:

```rust
#[test]
fn test_reference_safety() {
    let source = include_str!("../src/test_beta/reference_safety.move");
    let mut analyzer = Analysis::new();
    let module = Parser::parse_module(source).unwrap();
    let result = analyzer.analyze_module(&module);
    
    // Verify reference safety
    assert!(result.reference_leaks.iter().any(|leak| 
        leak.context.contains("unsafe_ref")
    ));
}
```

## Current Limitations

### 1. Analysis Scope
- Basic path sensitivity
- Limited cross-module analysis
- Simple invariant tracking

### 2. Performance
- No caching yet
- Basic memory optimization
- Limited parallelization

### 3. Features
- No Move Prover integration
- Limited custom rules
- No automated fixes

## Contributing

### Setting Up Development Environment
```bash
# Clone repository
git clone https://github.com/your-org/hydra
cd hydra

# Build
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test category
cargo test test_reference_safety
cargo test test_boundary_crossing
cargo test test_consensus_safety

# Run with output
cargo test -- --nocapture
```

### Benchmarking
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench reference_safety
```

## Test Files

### 1. Reference Safety Tests
```move
// src/test_beta/reference_safety.move
module test::reference_safety {
    struct Data has key {
        id: UID,
        value: u64
    }

    // Test cases for reference safety
    public fun unsafe_ref(data: &mut Data): &mut u64 {
        &mut data.value  // Should detect
    }
}
```

### 2. Boundary Tests
```move
// src/test_beta/boundary_crossing.move
module trusted::core {
    // Test cases for boundary crossing
    public fun leak_data(data: SecretData) {
        untrusted::module::receive(data)  // Should detect
    }
}
```

### 3. Consensus Tests
```move
// src/test_beta/consensus_safety.move
module test::consensus_safety {
    // Test cases for consensus safety
    public fun increment(counter: &mut SharedCounter) {
        // Missing consensus::verify()  // Should detect
        counter.value = counter.value + 1;
    }
}
```

## Next Steps

1. Complete core analysis features
2. Add more test cases
3. Implement caching
4. Add performance optimizations
5. Improve documentation