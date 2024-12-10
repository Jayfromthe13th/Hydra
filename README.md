# Hydra: Static Analyzer for Sui Move

![hydra logo](https://github.com/user-attachments/assets/90f1abcc-be1f-423f-b726-651ff86c99a5)


Hydra is a proof of concept static analysis tool designed specifically for Sui Move smart contracts. It performs deep analysis of Move code to detect potential security vulnerabilities, reference safety issues, and violations of Sui-specific safety patterns.

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





## License

This project is licensed under the MIT License
