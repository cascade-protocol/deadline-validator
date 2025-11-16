//! Deadline validation for Solana transactions
//!
//! This program validates that the current blockchain time is before
//! a specified deadline. Designed as public infrastructure for x402
//! payment protocol and other time-sensitive applications.
//!
//! ## Security Properties
//! - Stateless: No account storage
//! - Immutable: Deployed without upgrade authority
//! - Consensus-based: Uses Clock sysvar (consensus-managed)
//!
//! ## Usage
//! Include the ValidateDeadline instruction in your transaction with
//! a unix timestamp deadline. The instruction will fail atomically
//! if the current time exceeds the deadline.

pub mod error;
pub mod instruction;
pub mod processor;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint {
    use super::*;
    use solana_program::{
        account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
    };

    entrypoint!(process_instruction);

    pub fn process_instruction(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        processor::process_instruction(_program_id, _accounts, instruction_data)
    }
}

// Re-export for downstream users
pub use solana_program;

// Program ID - vanity address starting with "DEAD" (deadline validator)
solana_program::declare_id!("DEADaT1auZ8JjUMWUhhPWjQqFk9HSgHBkt5KaGMVnp1H");
