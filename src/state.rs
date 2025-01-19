use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::Sealed, pubkey::Pubkey};

// This account is owned  by escrow program
#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct EscrowState {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,                  // Alic PublicKey
    pub temp_token_account_pubkey: Pubkey, //  token_account for Alice sends his token x to this account
    pub initializer_token_to_recive_account: Pubkey, // token account for bob to send his token y to this account so that alice can take it.
    pub expected_amount: u64,
    pub giving_amount: u64,
}

impl Sealed for EscrowState {}
