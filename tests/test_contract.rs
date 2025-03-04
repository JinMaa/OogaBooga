use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::cell::RefCell;

// Thread-local storage for testing to avoid deadlocks
thread_local! {
    static MOCK_STORAGE: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
    static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

// Mock implementation of Context for testing
#[derive(Clone, Debug)]
struct Context {
    pub inputs: Vec<String>,
    pub incoming_alkanes: Vec<u8>,
}

// Mock implementation of CallResponse for testing
#[derive(Debug)]
struct CallResponse {
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

// Mock implementation of StoragePointer for testing
#[derive(Debug, Clone)]
struct StoragePointer {
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
fn shift<T>(v: &mut Vec<T>) -> Option<T> {
    if v.is_empty() {
        None
    } else {
        Some(v.remove(0))
    }
}

fn shift_or_err<T>(v: &mut Vec<T>) -> Result<T> {
    shift(v)
        .ok_or_else(|| anyhow!("expected value in list but list is exhausted"))
}

// OogaBoogaContract implementation for testing
#[derive(Default)]
struct OogaBoogaContract(());

// Storage implementation
impl OogaBoogaContract {
    // Storage pointers
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

    // Getters
    pub fn ooga_balance_of(&self, address: &str) -> u128 {
        self.ooga_balance_pointer(address).get_value::<u128>()
    }

    pub fn booga_balance_of(&self, address: &str) -> u128 {
        self.booga_balance_pointer(address).get_value::<u128>()
    }

    pub fn total_ooga(&self) -> u128 {
        self.total_ooga_pointer().get_value::<u128>()
    }

    pub fn total_booga(&self) -> u128 {
        self.total_booga_pointer().get_value::<u128>()
    }

    // Setters
    pub fn set_ooga_balance(&self, address: &str, amount: u128) {
        self.ooga_balance_pointer(address).set_value::<u128>(amount);
    }

    pub fn set_booga_balance(&self, address: &str, amount: u128) {
        self.booga_balance_pointer(address).set_value::<u128>(amount);
    }

    pub fn set_total_ooga(&self, amount: u128) {
        self.total_ooga_pointer().set_value::<u128>(amount);
    }

    pub fn set_total_booga(&self, amount: u128) {
        self.total_booga_pointer().set_value::<u128>(amount);
    }

    // Token operations
    fn claim_ooga(&self, address: &str) -> Result<()> {
        let current_balance = self.ooga_balance_of(address);
        let new_balance = current_balance.checked_add(1)
            .ok_or_else(|| anyhow!("balance overflow"))?;
        
        let total_ooga = self.total_ooga();
        self.set_total_ooga(total_ooga + 1);
        self.set_ooga_balance(address, new_balance);
        
        Ok(())
    }

    fn exchange_ooga_for_booga(&self, address: &str) -> Result<()> {
        let ooga_balance = self.ooga_balance_of(address);
        if ooga_balance < 1 {
            return Err(anyhow!("insufficient OOGA balance"));
        }

        let booga_balance = self.booga_balance_of(address);
        
        // Exchange exactly 1 OOGA for 1 BOOGA
        self.set_ooga_balance(address, ooga_balance - 1);
        self.set_booga_balance(address, booga_balance + 1);
        
        let total_ooga = self.total_ooga();
        let total_booga = self.total_booga();
        self.set_total_ooga(total_ooga - 1);
        self.set_total_booga(total_booga + 1);

        Ok(())
    }

    pub fn context(&self) -> Result<Context> {
        CONTEXT.with(|ctx| {
            if let Some(context) = &*ctx.borrow() {
                Ok(context.clone())
            } else {
                Err(anyhow!("No context available"))
            }
        })
    }

    pub fn execute(&self) -> Result<CallResponse> {
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
}

// Test harness for executing contract operations
struct TestHarness {
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
fn extract_u128(response: &CallResponse) -> u128 {
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&response.data[0..16]);
    u128::from_le_bytes(bytes)
}


#[cfg(test)]
mod integration_tests {
    use super::*;

    // Helper function to create a test address
    fn test_address() -> String {
        "integration_test_user".to_string()
    }

    #[test]
    fn test_complete_flow() {
        let harness = TestHarness::new();
        let address = test_address();
        
        // Initialize contract
        let result = harness.execute(0, vec![]);
        assert!(result.is_ok());
        
        // Claim multiple OOGA tokens
        for _ in 0..5 {
            let result = harness.execute(1, vec![address.clone()]);
            assert!(result.is_ok());
        }
        
        // Verify OOGA balance
        let result = harness.execute(3, vec![address.clone()]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 5);
        }
        
        // Exchange some OOGA for BOOGA
        for _ in 0..3 {
            let result = harness.execute(2, vec![address.clone()]);
            assert!(result.is_ok());
        }
        
        // Verify final balances
        let result = harness.execute(3, vec![address.clone()]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 2); // 5 - 3 = 2 OOGA remaining
        }
        
        let result = harness.execute(4, vec![address.clone()]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 3); // 3 BOOGA received
        }
        
        // Verify total supplies
        let result = harness.execute(5, vec![]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 2); // 2 OOGA total supply
        }
        
        let result = harness.execute(6, vec![]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 3); // 3 BOOGA total supply
        }
    }

    #[test]
    fn test_multi_user_interaction() {
        let harness = TestHarness::new();
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        let charlie = "charlie".to_string();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Alice claims 3 OOGA
        for _ in 0..3 {
            let _ = harness.execute(1, vec![alice.clone()]);
        }
        
        // Bob claims 2 OOGA
        for _ in 0..2 {
            let _ = harness.execute(1, vec![bob.clone()]);
        }
        
        // Charlie claims 1 OOGA
        let _ = harness.execute(1, vec![charlie.clone()]);
        
        // Verify total OOGA supply
        let result = harness.execute(5, vec![]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 6); // 3 + 2 + 1 = 6 OOGA total
        }
        
        // Alice exchanges 2 OOGA for BOOGA
        for _ in 0..2 {
            let _ = harness.execute(2, vec![alice.clone()]);
        }
        
        // Bob exchanges 1 OOGA for BOOGA
        let _ = harness.execute(2, vec![bob.clone()]);
        
        // Verify individual balances
        let result = harness.execute(3, vec![alice.clone()]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 1); // 3 - 2 = 1 OOGA remaining
        }
        
        let result = harness.execute(4, vec![alice.clone()]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 2); // 2 BOOGA received
        }
        
        let result = harness.execute(3, vec![bob.clone()]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 1); // 2 - 1 = 1 OOGA remaining
        }
        
        let result = harness.execute(4, vec![bob.clone()]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 1); // 1 BOOGA received
        }
        
        // Verify total supplies after exchanges
        let result = harness.execute(5, vec![]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 3); // 6 - 3 = 3 OOGA total
        }
        
        let result = harness.execute(6, vec![]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 3); // 3 BOOGA total
        }
    }

    #[test]
    fn test_error_handling() {
        let harness = TestHarness::new();
        let address = test_address();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Try to exchange without claiming first (should fail)
        let result = harness.execute(2, vec![address.clone()]);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("insufficient OOGA balance"));
        }
        
        // Claim one OOGA
        let _ = harness.execute(1, vec![address.clone()]);
        
        // Exchange it for BOOGA (should succeed)
        let result = harness.execute(2, vec![address.clone()]);
        assert!(result.is_ok());
        
        // Try to exchange again (should fail)
        let result = harness.execute(2, vec![address.clone()]);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("insufficient OOGA balance"));
        }
        
        // Try invalid opcode
        let result = harness.execute(99, vec![]);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("unrecognized opcode"));
        }
    }
    
    #[test]
    fn test_balance_overflow() {
        let harness = TestHarness::new();
        let address = test_address();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Set balance to max u128 - 1
        harness.contract.set_ooga_balance(&address, u128::MAX - 1);
        
        // Claim OOGA (should succeed)
        let result = harness.execute(1, vec![address.clone()]);
        assert!(result.is_ok());
        assert_eq!(harness.contract.ooga_balance_of(&address), u128::MAX);
        
        // Claim OOGA again (should fail with overflow)
        let result = harness.execute(1, vec![address.clone()]);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("balance overflow"));
        }
    }
}
