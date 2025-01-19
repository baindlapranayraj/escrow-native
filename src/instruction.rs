use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

// API Endpoint
pub enum EscrowInstruction {
    // 0. Initializer Account
    // 1. temp_token_account
    // 2. tokent_to_recive_account
    // 3. escrow_meta_data_account
    // 4. rent_sysver_account
    // 5. token_program
    InitEscrow {
        amount_x: u64,
        amount_y: u64,
        pass: [u8; 32],
    }, // External Data which is manually given my initiator
    Exchange {
        pass: [u8; 32],
    },
    Withdraw {
        pass: [u8; 32],
    },
}

//This borsh traits allow us to deserailze the bytes to readable structs
#[derive(BorshDeserialize)]
pub struct EscrowPayload {
    amount_x: u64,
    amount_y: u64,
    pass: [u8; 32],
}

#[derive(BorshDeserialize)]
pub struct EscrowExchangePayload {
    pass: [u8; 32],
}

#[derive(BorshDeserialize)]
pub struct EscrowWithdrawPayload {
    pass: [u8; 32],
}

impl EscrowInstruction {
    // unpack function is used as helper function to convert raw bytes to readable struct
    pub fn unpack(instrcution_data: &[u8]) -> Result<Self, ProgramError> {
        // Extract the first byte and the remaining bytes
        let (variant, rest) = instrcution_data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        match variant {
            0 => {
                // Deserialize the remaining bytes into EscrowPayload
                let payload = EscrowPayload::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                Ok(Self::InitEscrow {
                    amount_x: payload.amount_x,
                    amount_y: payload.amount_y,
                    pass: payload.pass,
                })
            }
            1 => {
                // Deserialize the remaining bytes into EscrowExchangePayload
                let payload = EscrowExchangePayload::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::Exchange { pass: payload.pass })
            }
            2 => {
                let payload = EscrowWithdrawPayload::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::Exchange { pass: payload.pass })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

// +++++++++++++++++++++ Learnings +++++++++++++++++++++

// - Hear instruction.rs will acts like an API layer for the client-side
//
// - Hear the instrcution_data (Serialized form) will be converted to readable struct
//  and based on the data which is decoded it will find which specific function
//  to use for proccessing the user req
//
// The ? operator simplifies error handling by returning the error to the caller if it exists.
// This avoids unnecessary panics.
