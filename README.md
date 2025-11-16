# Deadline Validator

Stateless Solana program for validating transaction deadlines. Enables timeout control beyond blockhash expiration limits (~80-90 seconds).

**Program ID**: `DEADaT1auZ8JjUMWUhhPWjQqFk9HSgHBkt5KaGMVnp1H`

Deployed on Mainnet/Devnet/Localnet. No upgrade authority. 525-1311 CU per call (varies by code path).

## Usage

### TypeScript

```typescript
import { TransactionInstruction, PublicKey } from '@solana/web3.js';

const PROGRAM_ID = new PublicKey('DEADaT1auZ8JjUMWUhhPWjQqFk9HSgHBkt5KaGMVnp1H');

function createValidateDeadlineInstruction(deadline: number | bigint): TransactionInstruction {
  const data = Buffer.alloc(9);
  data.writeUInt8(0, 0);
  data.writeBigInt64LE(BigInt(deadline), 1);
  return new TransactionInstruction({
    keys: [],
    programId: PROGRAM_ID,
    data,
  });
}

// Example: 5 minute deadline
const deadline = Math.floor(Date.now() / 1000) + 300;
transaction.add(createValidateDeadlineInstruction(deadline));
```

See full example in [examples/typescript/](examples/typescript/)

### Rust

```rust
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub const PROGRAM_ID: Pubkey = solana_sdk::pubkey!("DEADaT1auZ8JjUMWUhhPWjQqFk9HSgHBkt5KaGMVnp1H");

pub fn validate_deadline(deadline: i64) -> Instruction {
    let mut data = vec![0u8];
    data.extend_from_slice(&deadline.to_le_bytes());
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![],
        data,
    }
}
```

See full example in [examples/rust/](examples/rust/)

## Behavior

Validates `current_time <= deadline` (Unix timestamp, seconds since epoch).

- Succeeds if current time ≤ deadline
- Fails with `DeadlineExpired` (error code 0) if current time > deadline
- `deadline = 0` never expires
- Negative deadlines always expire

Uses Clock sysvar for consensus time. No accounts required.

## Building

```bash
# Build
cargo-build-sbf --manifest-path program/Cargo.toml

# Test
cargo test-sbf --manifest-path program/Cargo.toml

# Deploy
solana program deploy target/deploy/cascade_protocol_deadline_validator.so
```

Or use the Makefile: `make build`, `make test`, `make deploy-devnet`.

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | DeadlineExpired | Current time has exceeded the deadline |
| 1 | InvalidInstructionData | Instruction data is malformed |

## License

CC0 1.0 Universal - Public Domain. See [LICENSE](LICENSE).
