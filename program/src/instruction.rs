//! Program instructions

use crate::error::DeadlineError;
use solana_program::program_error::ProgramError;

/// Instructions supported by the deadline validator program
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum DeadlineInstruction {
    /// Validates that current unix timestamp is before the given deadline
    ///
    /// Accounts expected: none (uses Clock sysvar directly)
    ///
    /// Instruction data layout:
    /// - Byte 0: Discriminator (0 = ValidateDeadline)
    /// - Bytes 1-8: deadline (i64, little-endian)
    ///
    /// Behavior:
    /// - Succeeds if: current_time <= deadline (inclusive)
    /// - Fails if: current_time > deadline (exclusive)
    ///
    /// Special case:
    /// - deadline = 0: Never expires (always succeeds)
    ///
    /// Note: deadline is i64 to match Clock.unix_timestamp type.
    /// Negative deadlines will always be expired (current time is positive).
    ValidateDeadline {
        /// Unix timestamp deadline (seconds since epoch)
        /// Use 0 for "never expires"
        deadline: i64,
    },
}

impl DeadlineInstruction {
    /// Unpacks instruction from byte buffer
    ///
    /// Expected format:
    /// - Byte 0: Discriminator (must be 0)
    /// - Bytes 1-8: Deadline as i64 little-endian
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(DeadlineError::InvalidInstructionData)?;

        Ok(match variant {
            0 => {
                if rest.len() != 8 {
                    return Err(DeadlineError::InvalidInstructionData.into());
                }
                let deadline = i64::from_le_bytes(
                    rest[..8]
                        .try_into()
                        .map_err(|_| DeadlineError::InvalidInstructionData)?,
                );
                Self::ValidateDeadline { deadline }
            }
            _ => return Err(DeadlineError::InvalidInstructionData.into()),
        })
    }

    /// Packs instruction into byte buffer
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(9);
        match self {
            Self::ValidateDeadline { deadline } => {
                buf.push(0); // instruction discriminator
                buf.extend_from_slice(&deadline.to_le_bytes());
            }
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_packing_roundtrip() {
        let deadline = 1234567890i64;
        let instruction = DeadlineInstruction::ValidateDeadline { deadline };

        let packed = instruction.pack();
        let unpacked = DeadlineInstruction::unpack(&packed).unwrap();

        assert_eq!(instruction, unpacked);
    }

    #[test]
    fn test_instruction_packing_zero_deadline() {
        let instruction = DeadlineInstruction::ValidateDeadline { deadline: 0 };
        let packed = instruction.pack();
        let unpacked = DeadlineInstruction::unpack(&packed).unwrap();
        assert_eq!(instruction, unpacked);
    }

    #[test]
    fn test_instruction_packing_negative_deadline() {
        let instruction = DeadlineInstruction::ValidateDeadline { deadline: -1000 };
        let packed = instruction.pack();
        let unpacked = DeadlineInstruction::unpack(&packed).unwrap();
        assert_eq!(instruction, unpacked);
    }

    #[test]
    fn test_invalid_instruction_empty_data() {
        assert!(DeadlineInstruction::unpack(&[]).is_err());
    }

    #[test]
    fn test_invalid_instruction_wrong_variant() {
        let data = [1u8, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(DeadlineInstruction::unpack(&data).is_err());
    }

    #[test]
    fn test_invalid_instruction_wrong_length() {
        // Missing bytes
        let data = [0u8, 1, 2, 3];
        assert!(DeadlineInstruction::unpack(&data).is_err());

        // Extra bytes
        let data = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert!(DeadlineInstruction::unpack(&data).is_err());
    }
}
