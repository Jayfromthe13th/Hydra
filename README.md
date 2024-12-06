# Hydra: Static Analyzer for Sui Move

![hydra logo](https://github.com/user-attachments/assets/90f1abcc-be1f-423f-b726-651ff86c99a5)


Hydra is a sophisticated static analysis tool designed specifically for Sui Move smart contracts. It performs deep analysis of Move code to detect potential security vulnerabilities, reference safety issues, and violations of Sui-specific safety patterns.

[![Build Status](https://github.com/jayfromthe13th/hydra/workflows/CI/badge.svg)](https://github.com/jayfromthe13th/hydra/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Key Features

### 1. Reference Safety Analysis
- **Escape Detection**: Identifies references that may escape their intended scope
- **Alias Tracking**: Detects potential aliasing of mutable references
- **Path-sensitive Analysis**: Analyzes all possible execution paths for reference safety
- **Cross-module Reference Flow**: Tracks references across module boundaries

### 2. Object Safety Verification
- **Transfer Safety**: Verifies proper object transfer patterns
- **Ownership Verification**: Ensures ownership rules are maintained
- **Shared Object Access**: Validates consensus requirements for shared objects
- **Hot Potato Checks**: Detects unsafe temporary object patterns

### 3. Capability Analysis
- **Permission Tracking**: Verifies proper capability usage and permissions
- **Delegation Safety**: Analyzes capability delegation patterns
- **Privilege Escalation**: Detects potential privilege escalation vectors
- **Cross-module Capability Flow**: Tracks capability usage across modules

### 4. Advanced Analysis Features
- **Path-sensitive Analysis**: Considers all possible execution paths
- **State-tracking**: Maintains object and reference states through control flow
- **Loop Analysis**: Verifies invariants and safety in loops
- **Boundary Analysis**: Checks trust boundary crossings

## Quick Start

```bash
# Install Hydra
cargo install hydra-analyzer

# Analyze a single file
hydra analyze path/to/module.move

# Analyze with detailed output
hydra analyze --verbose path/to/module.move

# Generate SARIF report
hydra analyze --format sarif path/to/module.move
```

## Understanding Hydra's Analysis

### Reference Safety
Hydra performs sophisticated reference analysis to ensure Move's reference safety rules are maintained:

### Object Safety
Hydra verifies proper object handling patterns:

### Capability Patterns
Hydra ensures proper capability usage:

## Configuration

Create `hydra.toml` in your project root:

```toml
[hydra]
strict = false
ignore_tests = true
max_module_size = 10000

[checks]
transfer_safety = true
capability_safety = true
shared_objects = true

[output]
format = "text"
verbose = false
show_fixes = true
```

## Integration

### GitHub Actions
```yaml
name: Hydra Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: hydra-analyzer/action@v1
        with:
          paths: "sources/**/*.move"
```

### Pre-commit Hook
```bash
#!/bin/sh
hydra analyze --strict $(git diff --cached --name-only | grep ".move$")
```

## Roadmap

### Phase 1 (Current)
- [x] Reference Safety
  - Reference lifetime verification
  - Mutable reference uniqueness
  - Reference escape analysis
  - Cross-module reference tracking

- [x] Move Type System Safety
  - Resource type safety
  - Ability constraints
  - Generic type bounds
  - Struct field access control

### Phase 2 (In Progress) 
- [ ] Global Storage Safety
  - Resource persistence guarantees
  - Key-value store integrity
  - Global state invariants
  - Cross-function state consistency

- [ ] Module Publishing Safety
  - Module isolation
  - Friend visibility
  - Capability delegation
  - Module upgrade safety

### Phase 3 (Planned)
- [ ] Advanced Safety Features
  - Consensus safety patterns
  - Economic safety analysis
  - Cross-chain interaction safety
  - Formal correctness proofs

## Technical Details

### Analysis Pipeline

1. **Parsing**: Convert Move source into AST
2. **Control Flow Analysis**: Build control flow graph
3. **Reference Analysis**: Track reference states and flows
4. **Path Analysis**: Analyze all execution paths
5. **State Tracking**: Monitor object and capability states
6. **Pattern Matching**: Detect unsafe patterns
7. **Report Generation**: Generate detailed findings

### Analysis Types

#### 1. Static Analysis
- Control flow analysis
- Data flow analysis
- Reference tracking
- State transition analysis
- Pattern matching

#### 2. Semantic Analysis
- Type checking
- Ownership verification
- Capability tracking
- Invariant verification

#### 3. Safety Verification
- Reference safety
- Object safety
- Capability safety
- Consensus safety


### Development Setup
```bash
# Clone repository
git clone https://github.com/jayfromthe13th/hydra
cd hydra

# Build
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## License

This project is licensed under the MIT License
