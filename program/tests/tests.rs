//! Integration tests for deadline validator program
//!
//! These tests validate the deadline validator program using Mollusk,
//! which provides a lightweight SVM test harness with Clock sysvar support.
//!
//! ## Running Tests
//! For clean output without verbose DEBUG logs, use:
//! ```bash
//! RUST_LOG=off cargo test --test tests
//! ```

use {
    cascade_protocol_deadline_validator::{
        error::DeadlineError, id, instruction::DeadlineInstruction,
    },
    mollusk_svm::{result::Check, Mollusk},
    solana_instruction::Instruction,
    solana_program_error::ProgramError,
};

/// Helper function to create a Mollusk instance with the deadline validator program
fn setup_mollusk() -> Mollusk {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");
    Mollusk::new(&id(), "cascade_protocol_deadline_validator")
}

/// Helper to create a ValidateDeadline instruction
fn create_validate_deadline_instruction(deadline: i64) -> Instruction {
    let instruction_data = DeadlineInstruction::ValidateDeadline { deadline }.pack();
    Instruction::new_with_bytes(id(), &instruction_data, vec![])
}

#[test]
fn test_future_deadline_succeeds() {
    let mut mollusk = setup_mollusk();

    // Set clock to a known timestamp
    mollusk.sysvars.clock.unix_timestamp = 1700000000; // Nov 14, 2023

    // Deadline in the future should succeed
    let future_deadline = 1800000000; // Apr 23, 2027
    let instruction = create_validate_deadline_instruction(future_deadline);

    mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
}

#[test]
fn test_expired_deadline_fails() {
    let mut mollusk = setup_mollusk();

    // Set clock to a known timestamp
    mollusk.sysvars.clock.unix_timestamp = 1700000000; // Nov 14, 2023

    // Deadline in the past should fail
    let past_deadline = 1600000000; // Sep 13, 2020
    let instruction = create_validate_deadline_instruction(past_deadline);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[],
        &[Check::err(ProgramError::from(
            DeadlineError::DeadlineExpired,
        ))],
    );
}

#[test]
fn test_deadline_zero_never_expires() {
    let mut mollusk = setup_mollusk();

    // Set clock to any timestamp
    mollusk.sysvars.clock.unix_timestamp = 1700000000;

    // Deadline = 0 should never expire
    let instruction = create_validate_deadline_instruction(0);

    mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
}

#[test]
fn test_negative_deadline_always_expired() {
    let mut mollusk = setup_mollusk();

    // Set clock to current time (always positive)
    mollusk.sysvars.clock.unix_timestamp = 1700000000;

    // Negative deadline should always be expired
    let negative_deadline = -1000;
    let instruction = create_validate_deadline_instruction(negative_deadline);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[],
        &[Check::err(ProgramError::from(
            DeadlineError::DeadlineExpired,
        ))],
    );
}

#[test]
fn test_deadline_boundary_exact_match() {
    let mut mollusk = setup_mollusk();

    let timestamp = 1700000000;

    // Set clock to exact timestamp
    mollusk.sysvars.clock.unix_timestamp = timestamp;

    // Deadline exactly matching current time should succeed (inclusive behavior)
    let instruction = create_validate_deadline_instruction(timestamp);

    mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
}

#[test]
fn test_deadline_boundary_one_second_past() {
    let mut mollusk = setup_mollusk();

    let timestamp = 1700000000;

    // Set clock to timestamp
    mollusk.sysvars.clock.unix_timestamp = timestamp;

    // Deadline one second in the past should fail
    let instruction = create_validate_deadline_instruction(timestamp - 1);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[],
        &[Check::err(ProgramError::from(
            DeadlineError::DeadlineExpired,
        ))],
    );
}

#[test]
fn test_deadline_boundary_one_second_future() {
    let mut mollusk = setup_mollusk();

    let timestamp = 1700000000;

    // Set clock to timestamp
    mollusk.sysvars.clock.unix_timestamp = timestamp;

    // Deadline one second in the future should succeed
    let instruction = create_validate_deadline_instruction(timestamp + 1);

    mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
}

#[test]
fn test_time_progression_deadline_expiration() {
    let initial_timestamp = 1700000000;
    let deadline = initial_timestamp + 100; // 100 seconds in the future

    // First transaction should succeed
    {
        let mut mollusk = setup_mollusk();
        mollusk.sysvars.clock.unix_timestamp = initial_timestamp;

        let instruction = create_validate_deadline_instruction(deadline);

        mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
    }

    // Second transaction with same deadline should fail after time progresses
    {
        let mut mollusk = setup_mollusk();
        mollusk.sysvars.clock.unix_timestamp = initial_timestamp + 200; // 200 seconds later

        let instruction = create_validate_deadline_instruction(deadline);

        mollusk.process_and_validate_instruction(
            &instruction,
            &[],
            &[Check::err(ProgramError::from(
                DeadlineError::DeadlineExpired,
            ))],
        );
    }
}

#[test]
fn test_extreme_future_deadline() {
    let mut mollusk = setup_mollusk();

    // Set clock to current time
    mollusk.sysvars.clock.unix_timestamp = 1700000000;

    // Test with maximum i64 value (far future)
    let far_future_deadline = i64::MAX;
    let instruction = create_validate_deadline_instruction(far_future_deadline);

    mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
}

#[test]
fn test_extreme_past_deadline() {
    let mut mollusk = setup_mollusk();

    // Set clock to current time
    mollusk.sysvars.clock.unix_timestamp = 1700000000;

    // Test with minimum i64 value (far past)
    let far_past_deadline = i64::MIN;
    let instruction = create_validate_deadline_instruction(far_past_deadline);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[],
        &[Check::err(ProgramError::from(
            DeadlineError::DeadlineExpired,
        ))],
    );
}

#[test]
fn test_multiple_transactions_same_deadline() {
    let mut mollusk = setup_mollusk();

    let timestamp = 1700000000;
    let deadline = timestamp + 1000;

    // Set clock
    mollusk.sysvars.clock.unix_timestamp = timestamp;

    // Execute multiple transactions with the same deadline
    for _ in 0..5 {
        let instruction = create_validate_deadline_instruction(deadline);

        mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
    }
}

#[test]
fn test_unix_epoch_deadline() {
    let mut mollusk = setup_mollusk();

    // Set clock to current time
    mollusk.sysvars.clock.unix_timestamp = 1700000000;

    // Unix epoch (0) has special meaning, but 1 should be expired
    let epoch_plus_one = 1;
    let instruction = create_validate_deadline_instruction(epoch_plus_one);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[],
        &[Check::err(ProgramError::from(
            DeadlineError::DeadlineExpired,
        ))],
    );
}

#[test]
fn test_recent_past_deadline() {
    let mut mollusk = setup_mollusk();

    // Set clock to current time
    mollusk.sysvars.clock.unix_timestamp = 1700000000;

    // Test with year 2001 (definitely in the past)
    let year_2001 = 1000000000;
    let instruction = create_validate_deadline_instruction(year_2001);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[],
        &[Check::err(ProgramError::from(
            DeadlineError::DeadlineExpired,
        ))],
    );
}
