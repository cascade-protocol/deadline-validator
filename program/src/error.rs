//! Program error types

use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the deadline validator program
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeadlineError {
    /// The deadline has expired (current time > deadline)
    #[error("Deadline has expired")]
    DeadlineExpired,

    /// The instruction data is invalid or malformed
    #[error("Invalid instruction data")]
    InvalidInstructionData,
}

impl From<DeadlineError> for ProgramError {
    fn from(e: DeadlineError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
