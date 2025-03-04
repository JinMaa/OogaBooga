# OOGA BOOGA Technical Context

## Technology Stack

The OOGA BOOGA contract is built using the following technologies:

| Component | Technology | Version |
|-----------|------------|---------|
| Implementation Language | Rust | Latest stable |
| Target Platform | Alkanes on Bitcoin | - |
| Compilation Target | WebAssembly (wasm32-unknown-unknown) | - |
| Compression | gzip | Level 9 |
| Error Handling | anyhow | 1.0 |

## Dependencies

### Core Dependencies

```toml
[dependencies]
alkanes_runtime = "0.1.0"    # Core Alkanes runtime functionality
alkanes_support = "0.1.0"    # Support utilities for Alkanes contracts
metashrew_support = "0.1.0"  # Metashrew platform support
anyhow = "1.0"               # Error handling utilities
```

### Dependency Purposes

1. **alkanes_runtime**
   - Provides the `AlkaneResponder` trait
   - Handles contract execution context
   - Manages contract lifecycle

2. **alkanes_support**
   - Provides the `CallResponse` type
   - Handles response formatting
   - Manages Alkane transfers

3. **metashrew_support**
   - Provides memory layout utilities
   - Handles WebAssembly integration
   - Manages pointer conversions

4. **anyhow**
   - Simplifies error handling
   - Provides the `Result` and `anyhow!` macros
   - Enables error context and propagation

## Build Configuration

### Cargo Configuration

```toml
[package]
name = "ooga-booga-contract"
version = "0.1.0"
edition = "2025"

[lib]
crate-type = ["cdylib"]  # Compile as a dynamic library for WASM

[profile.release]
opt-level = 's'          # Optimize for size
lto = true               # Enable link-time optimization
codegen-units = 1        # Reduce parallel code generation units
panic = 'abort'          # Remove panic unwinding to reduce binary size
strip = true             # Strip symbols from binary
```

### Build Script (build.rs)

The build script configures Rust compiler options for WebAssembly output:

```rust
fn main() {
    // Tell Cargo to only rebuild if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
    
    // Set optimization flags for WASM output
    println!("cargo:rustc-link-arg=-zstack-size=32768");
    println!("cargo:rustc-link-arg=--import-memory");
}
```

### Build Shell Script (build.sh)

The build shell script handles the compilation and compression process:

```bash
#!/bin/bash

# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Create a gzipped version with maximum compression
gzip -9 -c target/wasm32-unknown-unknown/release/ooga_booga_contract.wasm > contract.wasm.gz
```

## Alkanes Platform

### Overview

Alkanes is a smart contract platform for Bitcoin that:
- Uses WebAssembly for contract execution
- Provides a key-value storage system
- Supports token operations
- Enables contract-to-contract communication

### Storage System

The Alkanes platform provides a key-value storage system:
- Keys are strings (typically formatted as paths)
- Values can be any serializable type
- Storage is persistent across contract calls
- Access is managed through storage pointers

### Contract Interface

Alkanes contracts expose functionality through numeric opcodes:
- Opcodes are integers that determine the operation
- Parameters are passed as a sequence of values
- Responses can include data and Alkanes transfers

## Development Environment

### Required Tools

1. **Rust Toolchain**
   - Latest stable Rust compiler
   - Cargo package manager
   - rustup for toolchain management

2. **WebAssembly Target**
   - wasm32-unknown-unknown target
   - wasm-opt (optional for additional optimization)

3. **Compression Tools**
   - gzip for final binary compression

### Development Workflow

1. Implement contract functionality in Rust
2. Compile to WebAssembly using cargo
3. Compress the WASM binary using gzip
4. Deploy to the Alkanes platform

## Testing Approach

### Unit Testing

Unit tests can be written using Rust's standard testing framework:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_ooga() {
        // Test implementation
    }
}
```

### Integration Testing

Integration tests would involve:
- Setting up a test environment
- Initializing the contract
- Executing various operations
- Verifying the contract state

### Deployment Testing

Before final deployment:
- Test on an Alkanes testnet
- Verify all opcodes function correctly
- Check storage operations
- Validate error handling

## Performance Considerations

1. **Binary Size**
   - WebAssembly binary size affects deployment cost
   - Optimization for size is prioritized (opt-level = 's')
   - Final gzip compression reduces size further

2. **Execution Efficiency**
   - Minimize storage operations
   - Use efficient data structures
   - Avoid unnecessary computations

3. **Memory Usage**
   - Minimize dynamic allocations
   - Reuse memory where possible
   - Be mindful of stack usage (stack-size=32768)
