//! Compute unit benchmarks for deadline validator program
//!
//! Run with: cargo bench --bench compute_units
//! Results are written to: ../target/benches/compute_units.md (gitignored)

use {
    cascade_protocol_deadline_validator::{id, instruction::DeadlineInstruction},
    mollusk_svm::Mollusk,
    mollusk_svm_bencher::MolluskComputeUnitBencher,
};

fn main() {
    // Optionally disable logging for clean output
    solana_logger::setup_with("");

    // Setup Mollusk
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");
    let mollusk = Mollusk::new(&id(), "cascade_protocol_deadline_validator");

    // Create test instructions
    let future_deadline_ix = {
        let instruction_data = DeadlineInstruction::ValidateDeadline {
            deadline: 1800000000,
        }
        .pack();
        solana_instruction::Instruction::new_with_bytes(id(), &instruction_data, vec![])
    };

    let zero_deadline_ix = {
        let instruction_data = DeadlineInstruction::ValidateDeadline { deadline: 0 }.pack();
        solana_instruction::Instruction::new_with_bytes(id(), &instruction_data, vec![])
    };

    let max_deadline_ix = {
        let instruction_data = DeadlineInstruction::ValidateDeadline { deadline: i64::MAX }.pack();
        solana_instruction::Instruction::new_with_bytes(id(), &instruction_data, vec![])
    };

    // Run benchmarks
    MolluskComputeUnitBencher::new(mollusk)
        .bench(("validate_future_deadline", &future_deadline_ix, &[]))
        .bench(("validate_zero_deadline", &zero_deadline_ix, &[]))
        .bench(("validate_max_deadline", &max_deadline_ix, &[]))
        .must_pass(true)
        .out_dir("../target/benches")
        .execute();
}
