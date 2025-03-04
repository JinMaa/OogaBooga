use crate::OogaBoogaContract;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;

// Thread-local storage for testing to avoid deadlocks
thread_local! {
    pub static MOCK_STORAGE: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
    pub static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

// Mock implementation of AlkaneResponder trait for testing
pub trait AlkaneResponder {
    fn execute(&self) -> Result<CallResponse>;
    fn context(&self) -> Result<Context>;
    fn run(&self) -> Result<CallResponse>;
}

// Mock implementation of StoragePointer for testing
#[derive(Debug, Clone)]
pub struct StoragePointer {
    key: String,
}

impl StoragePointer {
    pub fn from_keyword(key: &str) -> Self {
        StoragePointer {
            key: key.to_string(),
        }
    }

    pub fn get_value<T: From<u128>>(&self) -> T {
        let result = MOCK_STORAGE.with(|storage| {
            let storage = storage.borrow();
            if let Some(value) = storage.get(&self.key) {
                if value.len() >= 16 {
                    let mut bytes = [0u8; 16];
                    bytes.copy_from_slice(&value[0..16]);
                    let value = u128::from_le_bytes(bytes);
                    T::from(value)
                } else {
                    T::from(0)
                }
            } else {
                T::from(0)
            }
        });
        result
    }

    pub fn set_value<T: Into<u128>>(&self, value: T) {
        MOCK_STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            let value: u128 = value.into();
            storage.insert(self.key.clone(), value.to_le_bytes().to_vec());
        });
    }
}

// Mock implementation of shift_or_err for testing
pub fn shift<T>(v: &mut Vec<T>) -> Option<T> {
    if v.is_empty() {
        None
    } else {
        Some(v.remove(0))
    }
}

pub fn shift_or_err<T>(v: &mut Vec<T>) -> Result<T> {
    shift(v)
        .ok_or_else(|| anyhow!("expected value in list but list is exhausted"))
}

// Mock implementation of Context for testing
#[derive(Clone, Debug)]
pub struct Context {
    pub inputs: Vec<String>,
    pub incoming_alkanes: Vec<u8>,
}

// Mock implementation of CallResponse for testing
#[derive(Debug)]
pub struct CallResponse {
    pub data: Vec<u8>,
    pub alkanes: (Vec<u8>, Vec<u8>),
}

impl CallResponse {
    pub fn forward(_incoming_alkanes: &Vec<u8>) -> Self {
        CallResponse {
            data: Vec::new(),
            alkanes: (Vec::new(), Vec::new()),
        }
    }
}

// Implement AlkaneResponder for OogaBoogaContract in test mode
impl AlkaneResponder for OogaBoogaContract {
    fn context(&self) -> Result<Context> {
        CONTEXT.with(|ctx| {
            if let Some(context) = &*ctx.borrow() {
                Ok(context.clone())
            } else {
                Err(anyhow!("No context available"))
            }
        })
    }

    fn execute(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut inputs = context.inputs.clone();
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Get the opcode from the first input
        let opcode_str = shift_or_err(&mut inputs)?;
        let opcode: u8 = opcode_str.parse().map_err(|_| anyhow!("invalid opcode format"))?;

        match opcode {
            // Initialize contract - opcode 0
            0 => {
                self.set_total_ooga(0);
                self.set_total_booga(0);
                Ok(response)
            },

            // Claim OOGA - opcode 1
            1 => {
                let address = shift_or_err(&mut inputs)?;
                self.claim_ooga(&address)?;
                Ok(response)
            },

            // Exchange OOGA for BOOGA - opcode 2
            2 => {
                let address = shift_or_err(&mut inputs)?;
                self.exchange_ooga_for_booga(&address)?;
                Ok(response)
            },

            // Query OOGA balance - opcode 3
            3 => {
                let address = shift_or_err(&mut inputs)?;
                response.data = self.ooga_balance_of(&address).to_le_bytes().to_vec();
                Ok(response)
            },

            // Query BOOGA balance - opcode 4
            4 => {
                let address = shift_or_err(&mut inputs)?;
                response.data = self.booga_balance_of(&address).to_le_bytes().to_vec();
                Ok(response)
            },

            // Query total OOGA supply - opcode 5
            5 => {
                response.data = self.total_ooga().to_le_bytes().to_vec();
                Ok(response)
            },

            // Query total BOOGA supply - opcode 6
            6 => {
                response.data = self.total_booga().to_le_bytes().to_vec();
                Ok(response)
            },

            _ => Err(anyhow!("unrecognized opcode"))
        }
    }
    
    fn run(&self) -> Result<CallResponse> {
        self.execute()
    }
}

// Test harness for executing contract operations
pub struct TestHarness {
    pub contract: OogaBoogaContract,
}

impl TestHarness {
    pub fn new() -> Self {
        let contract = OogaBoogaContract::default();
        // Reset storage
        MOCK_STORAGE.with(|storage| {
            storage.borrow_mut().clear();
        });
        Self { contract }
    }
    
    pub fn execute(&self, opcode: u8, inputs: Vec<String>) -> Result<CallResponse> {
        // Create proper context with inputs
        let mut all_inputs = vec![opcode.to_string()];
        all_inputs.extend(inputs);
        
        // Set up context
        CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = Some(Context {
                inputs: all_inputs,
                incoming_alkanes: Vec::new(),
            });
        });
        
        // Execute contract
        self.contract.execute()
    }
}

// Helper function to extract u128 from response data
pub fn extract_u128(response: &CallResponse) -> u128 {
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&response.data[0..16]);
    u128::from_le_bytes(bytes)
}
