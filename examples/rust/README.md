# Rust Example

Simple Rust client for Deadline Validator. No crate dependency needed - just copy the module!

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
solana-sdk = "~2.3"
```

## Usage

Copy the `client.rs` module into your project and use it:

```rust
use crate::client::validate_deadline;

// Create instruction with deadline 5 minutes from now
let deadline = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64 + 300;

let instruction = validate_deadline(deadline);

// Add to your transaction
transaction.add_instruction(instruction);
```

## Examples

### Payment with 24-hour deadline

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

let deadline = now + 86400;  // 24 hours from now
let instruction = validate_deadline(deadline);
```

### Never expires

```rust
let instruction = validate_deadline(0);  // 0 = never expires
```

### Specific timestamp

```rust
let deadline = 1735689600;  // Jan 1, 2025 00:00:00 UTC
let instruction = validate_deadline(deadline);
```

### In a Solana Program (CPI)

```rust
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::invoke,
};

pub fn process_with_deadline(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deadline: i64,
) -> ProgramResult {
    let deadline_validator_program = /* AccountInfo for deadline validator */;

    let instruction = validate_deadline(deadline);

    invoke(
        &instruction,
        &[deadline_validator_program.clone()],
    )?;

    // Continue with your logic...
    Ok(())
}
```

## Error Handling

```rust
use solana_sdk::transaction::TransactionError;

match result {
    Err(TransactionError::InstructionError(_, error)) if error == 0 => {
        println!("Deadline has expired!");
    }
    Ok(signature) => {
        println!("Transaction confirmed: {}", signature);
    }
    Err(e) => {
        eprintln!("Transaction failed: {}", e);
    }
}
```

## That's It!

No crate to add, no version conflicts, no dependency hell. Just copy ~30 lines and go.
