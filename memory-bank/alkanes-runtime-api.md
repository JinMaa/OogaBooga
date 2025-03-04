# Alkanes Runtime API Documentation

This document provides an overview of the Alkanes Runtime API based on the analysis of the alkanes-rs repository.

## Core Components

### AlkaneResponder Trait

The `AlkaneResponder` trait is the central component of the Alkanes Runtime API. It defines the interface for Alkanes contracts and provides methods for interacting with the Alkanes platform.

```rust
pub trait AlkaneResponder {
    // Get the execution context
    fn context(&self) -> Result<Context>;
    
    // Get the current block data
    fn block(&self) -> Vec<u8>;
    
    // Initialize the contract
    fn initialize(&self) -> &Self;
    
    // Get the current transaction data
    fn transaction(&self) -> Vec<u8>;
    
    // Load data from storage
    fn load(&self, k: Vec<u8>) -> Vec<u8>;
    
    // Store data in storage
    fn store(&self, k: Vec<u8>, v: Vec<u8>);
    
    // Get the balance of an Alkane
    fn balance(&self, who: &AlkaneId, what: &AlkaneId) -> u128;
    
    // Get the current sequence number
    fn sequence(&self) -> u128;
    
    // Get the remaining fuel
    fn fuel(&self) -> u64;
    
    // Get the current block height
    fn height(&self) -> u64;
    
    // Make an external call
    fn extcall<T: Extcall>(
        &self,
        cellpack: &Cellpack,
        outgoing_alkanes: &AlkaneTransferParcel,
        fuel: u64,
    ) -> Result<CallResponse>;
    
    // Make a regular call
    fn call(
        &self,
        cellpack: &Cellpack,
        outgoing_alkanes: &AlkaneTransferParcel,
        fuel: u64,
    ) -> Result<CallResponse>;
    
    // Make a delegate call
    fn delegatecall(
        &self,
        cellpack: &Cellpack,
        outgoing_alkanes: &AlkaneTransferParcel,
        fuel: u64,
    ) -> Result<CallResponse>;
    
    // Make a static call
    fn staticcall(
        &self,
        cellpack: &Cellpack,
        outgoing_alkanes: &AlkaneTransferParcel,
        fuel: u64,
    ) -> Result<CallResponse>;
    
    // Run the contract
    fn run(&self) -> Vec<u8>;
    
    // Run the contract and forward Alkanes
    fn run_and_forward(&self) -> Vec<u8>;
    
    // Execute the contract (must be implemented by the contract)
    fn execute(&self) -> Result<CallResponse>;
}
```

### StoragePointer

The `StoragePointer` struct is used to interact with the Alkanes storage system. It provides a type-safe way to read and write values to storage.

```rust
#[derive(Debug, Clone, Default)]
pub struct StoragePointer(pub Arc<Vec<u8>>);

impl KeyValuePointer for StoragePointer {
    fn wrap(word: &Vec<u8>) -> StoragePointer;
    fn unwrap(&self) -> Arc<Vec<u8>>;
    fn inherits(&mut self, _v: &Self);
    fn set(&mut self, v: Arc<Vec<u8>>);
    fn get(&self) -> Arc<Vec<u8>>;
}
```

In our OOGA BOOGA contract, we use the `StoragePointer` to define storage keys for token balances and total supplies:

```rust
pub fn ooga_balance_pointer(&self, address: &str) -> StoragePointer {
    StoragePointer::from_keyword(&format!("/ooga-balance/{}", address))
}

pub fn booga_balance_pointer(&self, address: &str) -> StoragePointer {
    StoragePointer::from_keyword(&format!("/booga-balance/{}", address))
}

pub fn total_ooga_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/total-ooga")
}

pub fn total_booga_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/total-booga")
}
```

### CallResponse

The `CallResponse` struct represents the response from a contract execution. It contains the Alkanes to transfer and the data to return.

```rust
#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallResponse {
    pub alkanes: AlkaneTransferParcel,
    pub data: Vec<u8>,
}

impl CallResponse {
    // Parse a CallResponse from a cursor
    pub fn parse(cursor: &mut std::io::Cursor<Vec<u8>>) -> Result<CallResponse>;
    
    // Serialize a CallResponse to bytes
    pub fn serialize(&self) -> Vec<u8>;
    
    // Create a CallResponse that forwards incoming Alkanes
    pub fn forward(incoming_alkanes: &AlkaneTransferParcel) -> CallResponse;
}
```

### Utility Functions

The Alkanes Support crate provides several utility functions for working with contract inputs:

```rust
// Shift a value from a vector
pub fn shift<T>(v: &mut Vec<T>) -> Option<T>;

// Shift a value from a vector or return an error
pub fn shift_or_err(v: &mut Vec<u128>) -> Result<u128>;

// Shift an AlkaneId from a vector
pub fn shift_id(v: &mut Vec<u128>) -> Option<AlkaneId>;

// Shift an AlkaneId from a vector or return an error
pub fn shift_id_or_err(v: &mut Vec<u128>) -> Result<AlkaneId>;

// Shift a u64 from a vector
pub fn shift_as_long(v: &mut Vec<u128>) -> Option<u64>;

// Shift a u64 from a vector or return an error
pub fn shift_as_long_or_err(v: &mut Vec<u128>) -> Result<u64>;

// Check for overflow and return an error if it occurs
pub fn overflow_error<T>(v: Option<T>) -> Result<T>;

// Shift a bytes32 from a vector
pub fn shift_bytes32(v: &mut Vec<u128>) -> Option<Vec<u8>>;

// Shift a bytes32 from a vector or return an error
pub fn shift_bytes32_or_err(v: &mut Vec<u128>) -> Result<Vec<u8>>;
```

## Contract Implementation Pattern

Alkanes contracts follow a specific implementation pattern:

1. Define a struct that will implement the `AlkaneResponder` trait:
   ```rust
   #[derive(Default)]
   pub struct OogaBoogaContract(());
   ```

2. Implement storage-related methods:
   ```rust
   impl OogaBoogaContract {
       // Storage pointers
       pub fn ooga_balance_pointer(&self, address: &str) -> StoragePointer { ... }
       
       // Getters
       pub fn ooga_balance_of(&self, address: &str) -> u128 { ... }
       
       // Setters
       pub fn set_ooga_balance(&self, address: &str, amount: u128) { ... }
       
       // Business logic
       fn claim_ooga(&self, address: &str) -> Result<()> { ... }
   }
   ```

3. Implement the `AlkaneResponder` trait:
   ```rust
   impl AlkaneResponder for OogaBoogaContract {
       fn execute(&self) -> Result<CallResponse> {
           let context = self.context().unwrap();
           let mut inputs = context.inputs.clone();
           let mut response = CallResponse::forward(&context.incoming_alkanes);
           
           match shift_or_err(&mut inputs)? {
               // Handle different opcodes
               0 => { /* Initialize contract */ },
               1 => { /* Claim OOGA */ },
               // ...
               _ => Err(anyhow!("unrecognized opcode"))
           }
       }
   }
   ```

4. Export the contract entry point:
   ```rust
   #[no_mangle]
   pub extern "C" fn __execute() -> i32 {
       let mut response = to_arraybuffer_layout(&OogaBoogaContract::default().run());
       to_ptr(&mut response) + 4
   }
   ```

   Alternatively, use the `declare_alkane!` macro:
   ```rust
   declare_alkane!(OogaBoogaContract);
   ```

## Testing Alkanes Contracts

Testing Alkanes contracts requires mocking the Alkanes runtime environment. This includes:

1. Mocking the `StoragePointer` to use an in-memory storage system
2. Mocking the `Context` to provide test inputs
3. Mocking the `CallResponse` to capture outputs
4. Implementing test helpers to execute contract opcodes

Our OOGA BOOGA contract tests use this approach to test the contract functionality without requiring the actual Alkanes runtime.
