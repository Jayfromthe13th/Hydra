# Hydra: Static Safety Analyzer for Sui Move

Hydra is a static analyzer designed to detect safety issues in Sui Move smart contracts, with a focus on reference safety and object capability patterns.

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

