/**
 * Deadline Validator - Rust Client Example
 *
 * Simple helper to create ValidateDeadline instructions.
 * Just copy this module into your project - no crate dependency needed!
 */

use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
};

/// Program ID for Deadline Validator
pub const PROGRAM_ID: Pubkey = solana_sdk::pubkey!("DEADaT1auZ8JjUMWUhhPWjQqFk9HSgHBkt5KaGMVnp1H");

/// Create a ValidateDeadline instruction
///
/// # Arguments
/// * `deadline` - Unix timestamp (seconds since epoch)
///                Use 0 for "never expires"
///                Negative values are always expired
///
/// # Returns
/// Instruction ready to add to your transaction
pub fn validate_deadline(deadline: i64) -> Instruction {
    // Instruction data format:
    // [0]: Discriminator (u8) = 0
    // [1..9]: Deadline (i64, little-endian)
    let mut data = vec![0u8];  // Discriminator
    data.extend_from_slice(&deadline.to_le_bytes());  // Deadline

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![],  // No accounts needed - uses Clock sysvar internally
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_creation() {
        let deadline = 1735689600i64;  // Jan 1, 2025
        let ix = validate_deadline(deadline);

        assert_eq!(ix.program_id, PROGRAM_ID);
        assert_eq!(ix.accounts.len(), 0);
        assert_eq!(ix.data.len(), 9);
        assert_eq!(ix.data[0], 0);  // Discriminator
        assert_eq!(i64::from_le_bytes(ix.data[1..9].try_into().unwrap()), deadline);
    }

    #[test]
    fn test_never_expires() {
        let ix = validate_deadline(0);
        assert_eq!(i64::from_le_bytes(ix.data[1..9].try_into().unwrap()), 0);
    }
}
