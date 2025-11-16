/**
 * Deadline Validator - TypeScript Client Example
 *
 * Simple helper to create ValidateDeadline instructions.
 * Just copy this function into your project - no SDK installation needed!
 */

import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  Keypair,
} from '@solana/web3.js';

/** Program ID for Deadline Validator */
export const PROGRAM_ID = new PublicKey('DEADaT1auZ8JjUMWUhhPWjQqFk9HSgHBkt5KaGMVnp1H');

/**
 * Create a ValidateDeadline instruction
 *
 * @param deadline - Unix timestamp (seconds since epoch)
 *                   Use 0 for "never expires"
 *                   Negative values are always expired
 * @returns TransactionInstruction ready to add to your transaction
 */
export function createValidateDeadlineInstruction(
  deadline: number | bigint
): TransactionInstruction {
  // Instruction data format:
  // [0]: Discriminator (u8) = 0
  // [1..9]: Deadline (i64, little-endian)
  const data = Buffer.alloc(9);
  data.writeUInt8(0, 0);  // Discriminator
  data.writeBigInt64LE(BigInt(deadline), 1);  // Deadline

  return new TransactionInstruction({
    keys: [],  // No accounts needed - uses Clock sysvar internally
    programId: PROGRAM_ID,
    data,
  });
}

/**
 * Example Usage
 */
async function example() {
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  const payer = Keypair.generate();  // In production, use your actual keypair

  // Example 1: Deadline 5 minutes from now
  const fiveMinutesFromNow = Math.floor(Date.now() / 1000) + 300;
  const instruction = createValidateDeadlineInstruction(fiveMinutesFromNow);

  // Example 2: Never expires
  const neverExpires = createValidateDeadlineInstruction(0);

  // Add to transaction
  const transaction = new Transaction().add(instruction);

  // Send transaction
  try {
    const signature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [payer]
    );
    console.log('Transaction confirmed:', signature);
  } catch (error) {
    if (error.message.includes('custom program error: 0x0')) {
      console.log('Deadline expired!');
    } else {
      throw error;
    }
  }
}

// Run example (uncomment to test)
// example().catch(console.error);
