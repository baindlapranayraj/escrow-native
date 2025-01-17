use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum EscrowInstruction {
    InitEscrow { amount: u64 },
}

#[derive(BorshDeserialize)]
pub struct EscrowPayload {
    amount: u64,
}

impl EscrowInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // Extract the first byte and the remaining bytes
        let (variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        // Deserialize the remaining bytes into EscrowPayload
        let payload = EscrowPayload::try_from_slice(rest)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match variant {
            0 => Ok(Self::InitEscrow {
                amount: payload.amount,
            }),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

// +++++++++++++++++++++ Learnings +++++++++++++++++++++

// - Hear instruction.rs will acts like an API layer for the client-side
//
// - Hear the input data(Serialized form) will be converted to readable struct
//  and based on the data which is decoded it will find which specific function
//  to use for proccessing the user req
//
// The ? operator simplifies error handling by returning the error to the caller if it exists.
// This avoids unnecessary panics.
