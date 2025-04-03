# Hydra User Guide

## Overview

Hydra is a static analyzer for Sui Move smart contracts that helps detect reference safety issues, invariant violations, and Sui-specific security patterns.

## Installation

```bash
cargo install hydra-analyzer
```

## Quick Start

### Basic Analysis
```bash
# Analyze a single file
hydra analyze path/to/module.move

# Analyze with detailed output
hydra analyze --verbose path/to/module.move

# Show fix suggestions
hydra analyze --fixes path/to/module.move
```

### Output Formats
```bash
# Default text output
hydra analyze module.move

# JSON output
hydra analyze --format json module.move

# SARIF output (for IDE/tool integration)
hydra analyze --format sarif module.move
```

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

## Safety Checks

### 1. Reference Safety
- Mutable reference leaks
- Reference escape detection
- Cross-module reference tracking
- Field reference safety

Example:
```move
// UNSAFE: Reference leak
public fun unsafe_ref(obj: &mut Object): &mut u64 {
    &mut obj.value
}

// SAFE: Return copy
public fun safe_ref(obj: &mut Object): u64 {
    *&obj.value
}
```

### 2. Object Safety
- Transfer function safety
- Ownership verification
- Shared object access patterns
- Object construction safety
- Transfer guard validation

Example:
```move
// UNSAFE: Missing checks
public fun unsafe_transfer(token: Token) {
    transfer::transfer(token, @recipient);
}

// SAFE: With checks
public fun safe_transfer(token: Token, recipient: address) {
    assert!(is_valid_recipient(recipient), ERROR_INVALID_RECIPIENT);
    transfer::transfer(token, recipient);
}
```

### 3. Capability Safety
- Capability exposure checks
- Permission validation
- Privilege escalation detection
- Capability usage patterns

Example:
```move
// UNSAFE: Capability leak
public fun unsafe_cap_usage(cap: &mut AdminCap): &mut AdminCap {
    cap
}

// SAFE: Internal usage
public(friend) fun safe_cap_usage(cap: &AdminCap) {
    // Use capability internally
}
```

## Error Messages

### Severity Levels
- **Critical**: Must be fixed immediately
- **High**: Should be fixed before deployment
- **Medium**: Should be reviewed
- **Low**: Best practice violations
- **Info**: Suggestions for improvement

### Common Error Types
1. **Reference Safety**
   - `ReferenceEscape`: Mutable reference leak
   - `InvariantViolation`: Breaking object invariants
   - `UnsafePublicInterface`: Exposing internal references

2. **Object Safety**
   - `UnsafeTransfer`: Missing ownership or recipient validation
   - `InvalidSharedAccess`: Improper shared object access pattern
   - `UnsafeObjectConstruction`: Improper object initialization

3. **Capability Safety**
   - `CapabilityLeak`: Exposing capabilities
   - `InvalidCapabilityUsage`: Improper capability pattern
   - `UnauthorizedAccess`: Missing capability checks

## Best Practices

### 1. Reference Handling
- Avoid returning mutable references from public functions
- Use accessor functions instead of exposing references
- Validate all reference parameters
- Maintain object invariants

### 2. Object Safety
- Always validate recipients in transfer functions
- Use transfer guards for custom transfer logic
- Implement proper shared object synchronization
- Initialize all object fields properly

### 3. Capability Usage
- Keep capabilities internal to modules
- Use friend functions for cross-module capability usage
- Implement proper capability checks
- Follow the principle of least privilege

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

## Command Line Options
```
hydra analyze [OPTIONS] <INPUT>

OPTIONS:
    -f, --format <FORMAT>    Output format: text, json, sarif [default: text]
    -v, --verbose           Enable verbose output
    --strict               Enable strict mode
    --fixes                Show suggested fixes
    --ignore-tests        Skip test modules
    --check <CHECKS>       Specific checks to run [possible values: transfer, capability, shared]
```

## Performance Considerations
- Analysis typically takes < 100ms per module
- Memory usage < 500MB for most projects
- Supports packages up to 10k LOC
- Use `--ignore-tests` for faster analysis

## Troubleshooting

### Common Issues
1. **False Positives**
   - Use `// hydra-ignore: reason` to suppress warnings
   - Configure check levels in `hydra.toml`
   - Report false positives on GitHub

2. **Performance Issues**
   - Enable incremental analysis
   - Use `--ignore-tests`
   - Split large modules

3. **Integration Issues**
   - Check SARIF output format
   - Verify GitHub Actions setup
   - Check IDE plugin configuration

## Support

- GitHub Issues: [Report bugs](https://github.com/your-org/hydra/issues)