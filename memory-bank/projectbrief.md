# OOGA BOOGA Project Brief

## Project Overview
OOGA BOOGA is a simple Alkanes token contract that implements a 1:1 exchange mechanism between two token types: OOGA and BOOGA. The contract allows users to claim OOGA tokens and then exchange them for BOOGA tokens at a fixed 1:1 rate, with the limitation that only one token can be exchanged at a time.

## Technical Requirements

### Platform
- Target Platform: Alkanes on Bitcoin
- Contract Format: WebAssembly (WASM)
- Contract Compression: gzip level 9
- Implementation Language: Rust

### Core Features
1. **Token Claiming**
   - Any user can claim OOGA tokens
   - Claiming gives exactly 1 OOGA token per claim
   - No limit on claiming

2. **Token Exchange**
   - Users can exchange 1 OOGA token for 1 BOOGA token
   - Only one token can be exchanged per transaction
   - Exchange is irreversible (BOOGA cannot be converted back to OOGA)

3. **Balance Tracking**
   - Track individual user balances for both OOGA and BOOGA tokens
   - Track total supply of both token types

### Contract Interface

#### Opcodes
The contract exposes the following operations via numeric opcodes:
- `0`: Initialize contract (sets up the initial state)
- `1`: Claim OOGA (accepts user address)
- `2`: Exchange OOGA for BOOGA (accepts user address)
- `3`: Query OOGA balance (accepts user address)
- `4`: Query BOOGA balance (accepts user address)
- `5`: Query total OOGA supply
- `6`: Query total BOOGA supply

#### Storage Structure
The contract uses these storage keys:
- `ooga_balance:{address}`: OOGA balance for a specific address
- `booga_balance:{address}`: BOOGA balance for a specific address
- `total_ooga`: Total OOGA supply
- `total_booga`: Total BOOGA supply

## Project Structure
```
ooga-booga-contract/
├── Cargo.toml         # Rust package configuration
├── .cargo/
│   └── config.toml    # Cargo configuration for WASM
├── build.rs           # Build script for Rust
├── build.sh           # Build script for WASM compression
└── src/
    └── lib.rs         # Main contract code
```

## Development Process
1. Implement the contract in Rust
2. Compile to WebAssembly (wasm32-unknown-unknown target)
3. Compress using gzip level 9
4. Deploy to Alkanes network

## Testing Approach
The contract should be tested using:
1. Rust unit tests for core functionality
2. Integration tests for complete contract flows
3. Manual testing via Alkanes simulation

## Deployment
The contract will be deployed using the Alkanes [1, 0] system call, which will assign it a unique [2, n] Alkane ID for future interaction.

# Building Alkane Contracts

This guide will walk you through how to build ALKANE contracts, explaining the core concepts and components you'll need to implement.
Contract Structure

An ALKANE contract consists of these main components:

    A struct implementing the AlkaneResponder trait
    Storage management for contract state
    Opcode handlers for different contract actions
    The required __execute export function

Basic Contract Template

Here's a minimal template for an ALKANE contract:

use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::response::CallResponse;
use metashrew_support::compat::{to_arraybuffer_layout, to_ptr};

#[derive(Default)]
pub struct MyContract(());

impl AlkaneResponder for MyContract {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context().unwrap();
        let mut inputs = context.inputs.clone();
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Your opcode handling logic here

        Ok(response)
    }
}

#[no_mangle]
pub extern "C" fn __execute() -> i32 {
    let mut response = to_arraybuffer_layout(&MyContract::default().run());
    to_ptr(&mut response) + 4
}

Storage Management

ALKANE contracts use a key-value storage system to maintain state. Here's how to implement storage:

// Define storage pointers
impl MyContract {
    pub fn some_value_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/some-value")
    }

    // Getter
    pub fn some_value(&self) -> u128 {
        self.some_value_pointer().get_value::<u128>()
    }

    // Setter
    pub fn set_some_value(&self, v: u128) {
        self.some_value_pointer().set_value::<u128>(v);
    }
}

Contract Opcodes

Opcodes are numbers that determine which action the contract should take. Here's how to implement opcode handling:

impl AlkaneResponder for MyContract {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context().unwrap();
        let mut inputs = context.inputs.clone();
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        match shift_or_err(&mut inputs)? {
            // Initialization opcode
            0 => {
                // Handle contract initialization
                let initial_value = shift_or_err(&mut inputs)?;
                self.set_some_value(initial_value);
                Ok(response)
            },

            // Custom action opcode
            1 => {
                // Handle some custom action
                let param = shift_or_err(&mut inputs)?;
                // Process the action...
                Ok(response)
            },

            // Query opcode
            2 => {
                // Return some stored value
                response.data = self.some_value().to_le_bytes().to_vec();
                Ok(response)
            },

            _ => Err(anyhow!("unrecognized opcode"))
        }
    }
}

Example: Free Mint Contract

Here's a practical example of how these components work together in a contract that allows users to mint tokens:

impl MintableAlkane {
    // Storage pointers
    pub fn minted_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/minted")
    }

    pub fn cap_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/cap")
    }

    // Storage getters/setters
    pub fn minted(&self) -> u128 {
        self.minted_pointer().get_value::<u128>()
    }

    pub fn set_cap(&self, v: u128) {
        self.cap_pointer()
            .set_value::<u128>(if v == 0 { u128::MAX } else { v })
    }
}

impl AlkaneResponder for MintableAlkane {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context().unwrap();
        let mut inputs = context.inputs.clone();
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        match shift_or_err(&mut inputs)? {
            // Initialize contract
            0 => {
                let token_units = shift_or_err(&mut inputs)?;
                self.set_value_per_mint(shift_or_err(&mut inputs)?);
                self.set_cap(shift_or_err(&mut inputs)?);
                Ok(response)
            },

            // Mint tokens
            77 => {
                response.alkanes.0.push(
                    self.mint(&context, self.value_per_mint())?
                );
                self.increment_mint()?;

                if self.minted() > self.cap() {
                    Err(anyhow!("supply has reached cap"))
                } else {
                    Ok(response)
                }
            },

            // Query total minted
            103 => {
                response.data = self.minted().to_le_bytes().to_vec();
                Ok(response)
            },

            _ => Err(anyhow!("unrecognized opcode"))
        }
    }
}

Response Handling

The CallResponse object is how your contract communicates back to the caller. It can:

    Return data via the data field
    Transfer Alkanes via the alkanes field
    Forward received Alkanes using CallResponse::forward()

Best Practices

    Always validate inputs and handle errors appropriately
    Use clear opcode numbers and document their purpose
    Implement query functions to allow reading contract state
    Use meaningful storage key names
    Keep related storage functions grouped together
    Include proper error handling for all operations

Testing

It's recommended to test your contract thoroughly before deployment. You can use the ALKANE Regtest framework to:

    Test individual opcode handlers
    Verify storage operations
    Simulate transactions
    Check error conditions

Remember to test both successful and failure scenarios for each operation your contract supports.