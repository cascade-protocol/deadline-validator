//! Program instruction processor

use crate::{error::DeadlineError, instruction::DeadlineInstruction};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg, pubkey::Pubkey,
    sysvar::Sysvar,
};

/// Processes an instruction
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = DeadlineInstruction::unpack(instruction_data)?;

    match instruction {
        DeadlineInstruction::ValidateDeadline { deadline } => {
            msg!("Instruction: ValidateDeadline");
            process_validate_deadline(deadline)
        }
    }
}

/// Validates that current time is before or equal to deadline
///
/// ## Behavior
/// - Succeeds when: clock.unix_timestamp <= deadline (inclusive)
/// - Fails when: clock.unix_timestamp > deadline (exclusive)
///
/// ## Special Cases
/// - deadline = 0: Never expires (always succeeds)
/// - deadline < 0: Always expired (current time is positive)
///
/// ## Pattern
/// Follows Solana Foundation attestation service pattern for time validation.
/// Uses Clock::get() (modern Solana pattern, no account passing required).
fn process_validate_deadline(deadline: i64) -> ProgramResult {
    let clock = Clock::get()?;

    // Special case: 0 means never expires
    if deadline == 0 {
        msg!("Deadline: never expires (deadline = 0)");
        return Ok(());
    }

    // Standard validation: current_time > deadline → expired
    if clock.unix_timestamp > deadline {
        msg!(
            "Deadline expired: current={}, deadline={}",
            clock.unix_timestamp,
            deadline
        );
        return Err(DeadlineError::DeadlineExpired.into());
    }

    msg!(
        "Deadline valid: current={}, deadline={}, remaining={}s",
        clock.unix_timestamp,
        deadline,
        deadline - clock.unix_timestamp
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require the Clock sysvar which is only available
    // in on-chain execution. They are marked as #[ignore] for unit tests
    // and should be verified using integration tests with solana-program-test.

    #[test]
    #[ignore]
    fn test_validate_deadline_future() {
        // Far future deadline should succeed
        let future_deadline = i64::MAX;
        let result = process_validate_deadline(future_deadline);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_validate_deadline_expired() {
        // Past deadline should fail (Unix epoch)
        // Use 1 instead for actual past
        let past_deadline = 1i64;
        let result = process_validate_deadline(past_deadline);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DeadlineError::DeadlineExpired.into());
    }

    #[test]
    #[ignore]
    fn test_validate_deadline_never_expires() {
        // Zero deadline means never expires
        let result = process_validate_deadline(0);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_validate_deadline_negative() {
        // Negative deadlines always expired (current time is positive)
        let result = process_validate_deadline(-1000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DeadlineError::DeadlineExpired.into());

        let result = process_validate_deadline(i64::MIN);
        assert!(result.is_err());
    }

    #[test]
    #[ignore]
    fn test_validate_deadline_boundary() {
        // Test recent past (likely expired)
        let recent_past = 1000000000i64; // Year 2001
        let result = process_validate_deadline(recent_past);
        assert!(result.is_err());
    }
}
