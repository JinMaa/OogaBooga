use crate::test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test address
    fn test_address() -> String {
        "test_user_123".to_string()
    }

    // Helper function to extract u128 from response data
    fn extract_u128(response: &CallResponse) -> u128 {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&response.data[0..16]);
        u128::from_le_bytes(bytes)
    }

    #[test]
    fn test_initialization() {
        let harness = TestHarness::new();
        
        // Execute initialization opcode
        let result = harness.execute(0, vec![]);
        assert!(result.is_ok());
        
        // Verify total supplies are set to 0
        assert_eq!(harness.contract.total_ooga(), 0);
        assert_eq!(harness.contract.total_booga(), 0);
    }

    #[test]
    fn test_claim_ooga() {
        let harness = TestHarness::new();
        let address = test_address();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Initial balance should be 0
        assert_eq!(harness.contract.ooga_balance_of(&address), 0);
        
        // Claim OOGA
        let result = harness.execute(1, vec![address.clone()]);
        assert!(result.is_ok());
        
        // Balance should be 1
        assert_eq!(harness.contract.ooga_balance_of(&address), 1);
        assert_eq!(harness.contract.total_ooga(), 1);
        
        // Claim again
        let result = harness.execute(1, vec![address.clone()]);
        assert!(result.is_ok());
        
        // Balance should be 2
        assert_eq!(harness.contract.ooga_balance_of(&address), 2);
        assert_eq!(harness.contract.total_ooga(), 2);
    }

    #[test]
    fn test_exchange_ooga_for_booga() {
        let harness = TestHarness::new();
        let address = test_address();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Claim OOGA
        let _ = harness.execute(1, vec![address.clone()]);
        
        // Initial balances
        assert_eq!(harness.contract.ooga_balance_of(&address), 1);
        assert_eq!(harness.contract.booga_balance_of(&address), 0);
        assert_eq!(harness.contract.total_ooga(), 1);
        assert_eq!(harness.contract.total_booga(), 0);
        
        // Exchange OOGA for BOOGA
        let result = harness.execute(2, vec![address.clone()]);
        assert!(result.is_ok());
        
        // Final balances
        assert_eq!(harness.contract.ooga_balance_of(&address), 0);
        assert_eq!(harness.contract.booga_balance_of(&address), 1);
        assert_eq!(harness.contract.total_ooga(), 0);
        assert_eq!(harness.contract.total_booga(), 1);
    }

    #[test]
    fn test_exchange_with_insufficient_balance() {
        let harness = TestHarness::new();
        let address = test_address();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Try to exchange without claiming first
        let result = harness.execute(2, vec![address.clone()]);
        assert!(result.is_err());
        
        // Error should be about insufficient balance
        if let Err(e) = result {
            assert!(e.to_string().contains("insufficient OOGA balance"));
        }
    }

    #[test]
    fn test_balance_queries() {
        let harness = TestHarness::new();
        let address = test_address();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Claim OOGA
        let _ = harness.execute(1, vec![address.clone()]);
        
        // Query OOGA balance
        let result = harness.execute(3, vec![address.clone()]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 1);
        }
        
        // Exchange OOGA for BOOGA
        let _ = harness.execute(2, vec![address.clone()]);
        
        // Query BOOGA balance
        let result = harness.execute(4, vec![address.clone()]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 1);
        }
    }

    #[test]
    fn test_total_supply_queries() {
        let harness = TestHarness::new();
        let address = test_address();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Claim OOGA
        let _ = harness.execute(1, vec![address.clone()]);
        
        // Query total OOGA supply
        let result = harness.execute(5, vec![]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 1);
        }
        
        // Exchange OOGA for BOOGA
        let _ = harness.execute(2, vec![address.clone()]);
        
        // Query total BOOGA supply
        let result = harness.execute(6, vec![]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 1);
        }
        
        // Query total OOGA supply (should be 0 after exchange)
        let result = harness.execute(5, vec![]);
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(extract_u128(&response), 0);
        }
    }

    #[test]
    fn test_invalid_opcode() {
        let harness = TestHarness::new();
        
        // Try an invalid opcode
        let result = harness.execute(99, vec![]);
        assert!(result.is_err());
        
        // Error should be about unrecognized opcode
        if let Err(e) = result {
            assert!(e.to_string().contains("unrecognized opcode"));
        }
    }

    // New test for multiple users
    #[test]
    fn test_multiple_users() {
        let harness = TestHarness::new();
        let address1 = "user1".to_string();
        let address2 = "user2".to_string();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // User 1 claims OOGA
        let _ = harness.execute(1, vec![address1.clone()]);
        
        // User 2 claims OOGA
        let _ = harness.execute(1, vec![address2.clone()]);
        
        // Verify balances
        assert_eq!(harness.contract.ooga_balance_of(&address1), 1);
        assert_eq!(harness.contract.ooga_balance_of(&address2), 1);
        assert_eq!(harness.contract.total_ooga(), 2);
        
        // User 1 exchanges OOGA for BOOGA
        let _ = harness.execute(2, vec![address1.clone()]);
        
        // Verify balances after exchange
        assert_eq!(harness.contract.ooga_balance_of(&address1), 0);
        assert_eq!(harness.contract.booga_balance_of(&address1), 1);
        assert_eq!(harness.contract.ooga_balance_of(&address2), 1);
        assert_eq!(harness.contract.booga_balance_of(&address2), 0);
        assert_eq!(harness.contract.total_ooga(), 1);
        assert_eq!(harness.contract.total_booga(), 1);
    }

    // New test for edge cases with addresses
    #[test]
    fn test_address_edge_cases() {
        let harness = TestHarness::new();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Test with empty address
        let empty_address = "".to_string();
        let result = harness.execute(1, vec![empty_address.clone()]);
        assert!(result.is_ok());
        assert_eq!(harness.contract.ooga_balance_of(&empty_address), 1);
        
        // Test with very long address
        let long_address = "a".repeat(1000);
        let result = harness.execute(1, vec![long_address.clone()]);
        assert!(result.is_ok());
        assert_eq!(harness.contract.ooga_balance_of(&long_address), 1);
        
        // Test with special characters
        let special_address = "!@#$%^&*()_+".to_string();
        let result = harness.execute(1, vec![special_address.clone()]);
        assert!(result.is_ok());
        assert_eq!(harness.contract.ooga_balance_of(&special_address), 1);
    }

    // New test for balance overflow
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

    // New test for stress testing with many operations
    #[test]
    fn test_stress_many_operations() {
        let harness = TestHarness::new();
        let address = test_address();
        
        // Initialize contract
        let _ = harness.execute(0, vec![]);
        
        // Perform many claim operations
        for _ in 0..100 {
            let _ = harness.execute(1, vec![address.clone()]);
        }
        
        // Verify balance
        assert_eq!(harness.contract.ooga_balance_of(&address), 100);
        assert_eq!(harness.contract.total_ooga(), 100);
        
        // Perform many exchange operations
        for _ in 0..50 {
            let _ = harness.execute(2, vec![address.clone()]);
        }
        
        // Verify final balances
        assert_eq!(harness.contract.ooga_balance_of(&address), 50);
        assert_eq!(harness.contract.booga_balance_of(&address), 50);
        assert_eq!(harness.contract.total_ooga(), 50);
        assert_eq!(harness.contract.total_booga(), 50);
    }
}
