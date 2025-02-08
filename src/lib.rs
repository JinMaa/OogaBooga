use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::response::CallResponse;
use metashrew_support::compat::{to_arraybuffer_layout, to_ptr};
use anyhow::{Result, anyhow};

#[derive(Default)]
pub struct OogaBoogaContract(());

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
}

// Contract logic implementation
impl AlkaneResponder for OogaBoogaContract {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context().unwrap();
        let mut inputs = context.inputs.clone();
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        match shift_or_err(&mut inputs)? {
            // Initialize contract - opcode 0
            0 => {
                self.set_total_ooga(0);
                self.set_total_booga(0);
                Ok(response)
            },

            // Claim OOGA - opcode 1
            1 => {
                let address = shift_or_err::<String>(&mut inputs)?;
                self.claim_ooga(&address)?;
                Ok(response)
            },

            // Exchange OOGA for BOOGA - opcode 2
            2 => {
                let address = shift_or_err::<String>(&mut inputs)?;
                self.exchange_ooga_for_booga(&address)?;
                Ok(response)
            },

            // Query OOGA balance - opcode 3
            3 => {
                let address = shift_or_err::<String>(&mut inputs)?;
                response.data = self.ooga_balance_of(&address).to_le_bytes().to_vec();
                Ok(response)
            },

            // Query BOOGA balance - opcode 4
            4 => {
                let address = shift_or_err::<String>(&mut inputs)?;
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

#[no_mangle]
pub extern "C" fn __execute() -> i32 {
    let mut response = to_arraybuffer_layout(&OogaBoogaContract::default().run());
    to_ptr(&mut response) + 4
}