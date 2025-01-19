use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_token::{instruction, state::GenericTokenAccount};

use crate::{instruction::EscrowInstruction, state::EscrowState};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        match instruction {
            EscrowInstruction::InitEscrow {
                amount_x,
                amount_y,
                pass,
            } => {
                msg!("Init escrow is working...");
                Self::process_init_escrow(program_id, accounts, amount_x, amount_y, pass)
            }
            EscrowInstruction::Exchange { pass } => {
                Self::exchange_token(program_id, accounts, pass)
            }
            EscrowInstruction::Withdraw { pass } => Ok(()),
        }?;

        Ok(())
    }

    pub fn process_init_escrow(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount_x: u64,
        amount_y: u64,
        pass: [u8; 32],
    ) -> ProgramResult {
        let mut account_info = accounts.iter();

        // All Accounts
        let initializer = next_account_info(&mut account_info)?;
        // No need to check this account since this account created programatically at client side.
        // This is a token address given by the users so we should check it.
        let temp_token_account = next_account_info(&mut account_info)?;
        // This is the account bob will send his Token Y.
        let token_to_recive_account = next_account_info(&mut account_info)?;
        // Escrow Account where all the escorow meta-data information will be stored(It is controlled by our Program)
        let escrow_account = next_account_info(&mut account_info)?;
        let rent = &Rent::from_account_info(next_account_info(&mut account_info)?)?;
        let token_program = next_account_info(&mut account_info)?;
        let _system_program = next_account_info(&mut account_info)?;

        // Checks
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *token_to_recive_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(ProgramError::AccountNotRentExempt);
        }

        // desrializing from bytes to struct.
        let mut escrow_info = EscrowState::try_from_slice(&escrow_account.data.borrow())?;

        if escrow_info.is_initialized {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // PDA account for initializer vault
        let (vault_x_token, _bump) =
            Pubkey::find_program_address(&[b"escrow", pass.as_ref()], program_id);

        // Hear we dont have to check the token_program account, bcoz while using spl-token crate, it will
        //  automaticlly checks the token_program_account.
        let owner_change_ix = instruction::set_authority(
            // - hear by using account key spl-crate can check weather this program
            // malicious or not.Thats why includind programId is required.
            token_program.key,
            temp_token_account.key,
            Some(&vault_x_token),
            instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key], // signer publickey in a array
        )?;

        // transferingg ownership from alice token account to vault_x account
        let cpi_accounts = [
            temp_token_account.clone(),
            initializer.clone(),
            token_program.clone(),
        ];

        // Invoking CPI ==> transfering ownership
        invoke(&owner_change_ix, &cpi_accounts)?;

        // updating the values
        escrow_info = EscrowState {
            is_initialized: true,
            initializer_pubkey: *initializer.key,
            expected_amount: amount_y,
            giving_amount: amount_x,
            temp_token_account_pubkey: *temp_token_account.key,
            initializer_token_to_recive_account: *token_to_recive_account.key,
        };

        // serializing back to bytes from struct.
        escrow_info.serialize(&mut &mut escrow_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn exchange_token(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        pass: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let taker = next_account_info(account_info_iter)?;

        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let takers_sending_token_account = next_account_info(account_info_iter)?;

        let takers_token_to_receive_account = next_account_info(account_info_iter)?;

        let pdas_temp_token_account = next_account_info(account_info_iter)?;
        let pdas_temp_token_account_info =
            GenericTokenAccount::unpack(&pdas_temp_token_account.try_borrow_data()?)?;
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        let initializers_main_account = next_account_info(account_info_iter)?;
        let initializers_token_to_receive_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;

        let escrow_info = EscrowState::try_from_slice(&escrow_account.try_borrow_data()?)?;

        if escrow_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if escrow_info.initializer_pubkey != *initializers_main_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if escrow_info.initializer_token_to_recive_account
            != *initializers_token_to_receive_account.key
        {
            return Err(ProgramError::InvalidAccountData);
        }

        let token_program = next_account_info(account_info_iter)?;

        let transfer_to_initializer_ix = spl_token::instruction::transfer(
            token_program.key,
            takers_sending_token_account.key,
            initializers_token_to_receive_account.key,
            taker.key,
            &[&taker.key],
            escrow_info.expected_amount,
        )?;
        msg!("Calling the token program to transfer tokens to the escrow's initializer...");
        invoke(
            &transfer_to_initializer_ix,
            &[
                takers_sending_token_account.clone(),
                initializers_token_to_receive_account.clone(),
                taker.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_to_taker_ix = spl_token::instruction::transfer(
            token_program.key,
            pdas_temp_token_account.key,
            takers_token_to_receive_account.key,
            &pda,
            &[&pda],
            pdas_temp_token_account_info.amount,
        )?;
        msg!("Calling the token program to transfer tokens to the taker...");
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                pdas_temp_token_account.clone(),
                takers_token_to_receive_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;

        let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            pdas_temp_token_account.key,
            initializers_main_account.key,
            &pda,
            &[&pda],
        )?;
        msg!("Calling the token program to close pda's temp account...");
        invoke_signed(
            &close_pdas_temp_acc_ix,
            &[
                pdas_temp_token_account.clone(),
                initializers_main_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;

        msg!("Closing the escrow account...");
        **initializers_main_account.lamports.borrow_mut() = initializers_main_account
            .lamports()
            .checked_add(escrow_account.lamports())
            .ok_or(ProgramError::ArithmeticOverflow)?;
        **escrow_account.lamports.borrow_mut() = 0;
        *escrow_account.try_borrow_mut_data()? = &mut [];

        Ok(())
    }
}

// +++++++++++++++++++++ Learnings +++++++++++++++++++++
// - All the bussiness logic will be inside of the processor file.
// - Hear in line no.71 borrow_mut() will allows you ref the raw byte data field in AccountInfo strcut and we can
//  modify or write to this data.{The escrow account is writable account}.
//  - [..] This syntax creates a slice of the mutable data reference, effectively allowing you to specify that
//    you want to work with all of the data in {escrow_account.data}.
