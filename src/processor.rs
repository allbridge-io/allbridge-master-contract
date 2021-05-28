//! Program state processor

use crate::{
    instruction::BridgeProgramInstruction,
    state::{Bridge, Blockchain, Validator, Lock, Signature, SigType},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed},
    system_instruction,
    program_error::ProgramError,
    program_pack::{IsInitialized},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

/// Program state handler.
pub struct Processor {}
impl Processor {

    fn validate_bridge_authority_and_get_bump_seed(
        program_id: &Pubkey,
        bridge_account: &Pubkey,
        authority_account: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let signer_seeds = &[bridge_account.as_ref()];
        let (expected_authority_account, bump_seed) =
            Pubkey::find_program_address(signer_seeds, program_id);
        if expected_authority_account != *authority_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(bump_seed)
    }

    fn check_and_get_blockchain_account_seed(
        program_id: &Pubkey,
        blockchain_id: &str,
        bridge_authority: &Pubkey,
        blockchain_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        let seed = format!("blockchain_{}", blockchain_id);
        let expected_blockchain_account =
            Pubkey::create_with_seed(bridge_authority, seed.as_str(), program_id)?;
        if expected_blockchain_account != *blockchain_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn check_and_get_validator_account_seed(
        program_id: &Pubkey,
        blockchain_id: &str,
        index: u64,
        bridge_authority: &Pubkey,
        validator_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        let seed = format!("validator_{}_{}", blockchain_id, index);
        let expected_validator_account =
            Pubkey::create_with_seed(bridge_authority, seed.as_str(), program_id)?;
        if expected_validator_account != *validator_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn check_and_get_lock_account_seed(
        program_id: &Pubkey,
        index: u64,
        bridge_authority: &Pubkey,
        lock_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        let seed = format!("lock_{}", index);
        let expected_lock_account =
            Pubkey::create_with_seed(bridge_authority, seed.as_str(), program_id)?;
        if expected_lock_account != *lock_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn check_and_get_signature_account_seed(
        program_id: &Pubkey,
        lock_index: u64,
        index: u64,
        bridge_authority: &Pubkey,
        signature_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        let seed = format!("signature_{}_{}", lock_index, index);
        let expected_signature_account =
            Pubkey::create_with_seed(bridge_authority, seed.as_str(), program_id)?;
        if expected_signature_account != *signature_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn str_to_hex(str: &str) -> [u8; 4] {
        let str_len = str.len();
        let mut result = [0; 4];
        result[..str_len].copy_from_slice(str.as_bytes());
        result
    }

    fn create_account_with_seed<'a>(
        payer_info: &AccountInfo<'a>,
        new_account: &AccountInfo<'a>,
        bridge_authority_info: &AccountInfo<'a>,
        seed: String,
        data_size: usize,
        rent: &Rent,
        program_id: &Pubkey,
        bridge_account: &Pubkey,
        bump_seed: u8,
    ) -> ProgramResult {
        invoke_signed(
            &system_instruction::create_account_with_seed(
                &payer_info.key,
                &new_account.key,
                &bridge_authority_info.key,
                seed.as_str(),
                rent.minimum_balance(data_size),
                data_size as u64,
                &program_id,
            ),
            &[
                payer_info.clone(),
                new_account.clone(),
                bridge_authority_info.clone(),
            ],
            &[&[
                bridge_account.as_ref(),
                &[bump_seed],
            ]],
        )
    }

    /// Initialize the bridge
    pub fn process_init_bridge(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let bridge_account_info = next_account_info(account_info_iter)?;
        let owner_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_account_info)?;

        if !owner_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let bridge_account_data = Bridge::try_from_slice(&bridge_account_info.data.borrow())?;
        if bridge_account_data.is_initialized() {
            msg!("Record account already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        if !rent.is_exempt(
            bridge_account_info.lamports(),
            bridge_account_info.data_len(),
        ) {
            return Err(ProgramError::AccountNotRentExempt);
        }

        let bridge = Bridge::new(*owner_account_info.key);
        bridge.serialize(&mut *bridge_account_info.data.borrow_mut())?;
        Ok(())
    }

    /// Process add blockchain
    pub fn process_add_blockchain(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        blockchain_id_str: &str,
        contract_address: [u8; 32]
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let bridge_account_info = next_account_info(account_info_iter)?;
        let blockchain_account_info = next_account_info(account_info_iter)?;
        let payer_info = next_account_info(account_info_iter)?;
        let bridge_authority_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_account_info)?;

        let bump_seed = Self::validate_bridge_authority_and_get_bump_seed(
            program_id,
            bridge_account_info.key,
            &bridge_authority_info.key,
        )?;

        let seed = Self::check_and_get_blockchain_account_seed(
            program_id,
            blockchain_id_str,
            bridge_authority_info.key,
            blockchain_account_info.key
        )?;

        Self::create_account_with_seed(
            payer_info,
            blockchain_account_info,
            bridge_authority_info,
            seed,
            Blockchain::LEN,
            rent,
            program_id,
            bridge_account_info.key,
            bump_seed,
        )?;

        let blockchain = Blockchain::new(
            *bridge_account_info.key,
            &blockchain_id_str.as_ref(),
            contract_address);
        blockchain.serialize(&mut *blockchain_account_info.data.borrow_mut())?;
        Ok(())
    }


    /// Process add validator
    pub fn process_add_validator(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        blockchain_id_str: &str,
        pub_key: [u8; 32]
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let bridge_account_info = next_account_info(account_info_iter)?;
        let blockchain_account_info = next_account_info(account_info_iter)?;
        let validator_account_info = next_account_info(account_info_iter)?;
        let payer_info = next_account_info(account_info_iter)?;
        let bridge_authority_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_account_info)?;

        let mut blockchain_account_data = Blockchain::try_from_slice(&blockchain_account_info.data.borrow_mut())?;
        if !blockchain_account_data.is_initialized() {
            msg!("Blockchain account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        let validator_index = blockchain_account_data.validators;

        let bump_seed = Self::validate_bridge_authority_and_get_bump_seed(
            program_id,
            bridge_account_info.key,
            &bridge_authority_info.key,
        )?;

        let seed = Self::check_and_get_validator_account_seed(
            program_id,
            blockchain_id_str,
            validator_index,
            bridge_authority_info.key,
            validator_account_info.key
        )?;

        Self::create_account_with_seed(
            payer_info,
            validator_account_info,
            bridge_authority_info,
            seed,
            Validator::LEN,
            rent,
            program_id,
            bridge_account_info.key,
            bump_seed,
        )?;

        blockchain_account_data.validators += 1;


        blockchain_account_data.serialize(&mut *blockchain_account_info.data.borrow_mut())?;

        let validator = Validator::new(
            blockchain_id_str,
            validator_index,
            pub_key,
            *payer_info.key);
        validator.serialize(&mut *validator_account_info.data.borrow_mut())?;

        Ok(())
    }

    /// Add signature
    pub fn process_add_signature(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        signature: [u8; 65],
        token_source: &str,
        token_source_address: [u8; 32],
        source: &str,
        lock_id: u64,
        destination: &str,
        recipient: [u8; 32],
        amount: u64
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let bridge_account_info = next_account_info(account_info_iter)?;
        let blockchain_account_info = next_account_info(account_info_iter)?;
        let validator_account_info = next_account_info(account_info_iter)?;
        let lock_account_info = next_account_info(account_info_iter)?;
        let signature_account_info = next_account_info(account_info_iter)?;
        let bridge_authority_info = next_account_info(account_info_iter)?;
        let payer_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_account_info)?;

        if !payer_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut bridge_account_data: Bridge = Bridge::try_from_slice(&bridge_account_info.data.borrow_mut())?;
        if !bridge_account_data.is_initialized() {
            msg!("Bridge account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        let mut blockchain_account_data: Blockchain = Blockchain::try_from_slice(&blockchain_account_info.data.borrow_mut())?;
        if !blockchain_account_data.is_initialized() {
            msg!("Blockchain account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        let mut validator_account_data: Validator = Validator::try_from_slice(&validator_account_info.data.borrow_mut())?;
        if !validator_account_data.is_initialized() {
            msg!("Blockchain account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        if validator_account_data.owner != *payer_info.key {
            msg!("Payer is not the validator");
            return Err(ProgramError::InvalidArgument);
        }

        let destination_hex = Self::str_to_hex(destination);
        if validator_account_data.blockchain_id != destination_hex {
            msg!("Invalid validator type");
            return Err(ProgramError::InvalidArgument);
        }

        let bump_seed = Self::validate_bridge_authority_and_get_bump_seed(
            program_id,
            bridge_account_info.key,
            &bridge_authority_info.key,
        )?;

        let lock_seed = Self::check_and_get_lock_account_seed(
            program_id,
            bridge_account_data.locks,
            bridge_authority_info.key,
            validator_account_info.key
        )?;

        if lock_account_info.data_is_empty() {
            Self::create_account_with_seed(
                payer_info,
                lock_account_info,
                bridge_authority_info,
                lock_seed,
                Lock::LEN,
                rent,
                program_id,
                bridge_account_info.key,
                bump_seed,
            )?;

            let lock = Lock::new(
                bridge_account_data.locks,
                lock_id,
                *bridge_account_info.key,
                token_source_address,
                token_source,
                source,
                recipient,
                destination,
                amount);
            lock.serialize(&mut *lock_account_info.data.borrow_mut())?;
        }

        let mut lock_account_data: Lock = Lock::try_from_slice(&lock_account_info.data.borrow_mut())?;
        if !lock_account_data.is_initialized() {
            msg!("Lock account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        let signature_seed = Self::check_and_get_signature_account_seed(
            program_id,
            lock_account_data.lock_id,
            lock_account_data.signatures,
            validator_account_info.key,
            signature_account_info.key
        )?;

        Self::create_account_with_seed(
            payer_info,
            signature_account_info,
            bridge_authority_info,
            signature_seed,
            Signature::LEN,
            rent,
            program_id,
            bridge_account_info.key,
            bump_seed,
        )?;

        let signature = Signature::new(
            lock_account_data.signatures,
            *bridge_account_info.key,
            SigType::Lock,
            signature,
            *validator_account_info.key);
        signature.serialize(&mut *signature_account_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes an instruction
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        let instruction = BridgeProgramInstruction::try_from_slice(input)
            .or(Err(ProgramError::InvalidInstructionData))?;
        match instruction {
            BridgeProgramInstruction::InitializeBridge => {
                msg!("Instruction: InitializeBridge");
                Self::process_init_bridge(program_id, accounts)
            },
            BridgeProgramInstruction::AddBlockchain {contract_address, blockchain_id} => {
                msg!("Instruction: AddBlockchain");
                Self::process_add_blockchain(program_id, accounts, blockchain_id.as_ref(), contract_address)
            },
            BridgeProgramInstruction::AddValidator {blockchain_id, pub_key} => {
                msg!("Instruction: AddBlockchain");
                Self::process_add_validator(program_id, accounts, blockchain_id.as_ref(), pub_key)
            }
            BridgeProgramInstruction::AddSignature {signature, token_source, token_source_address, source, lock_id, destination, recipient, amount} => {
                msg!("Instruction: AddBlockchain");
                Self::process_add_signature(program_id, accounts, signature, token_source.as_str(), token_source_address, source.as_str(), lock_id, destination.as_str(), recipient, amount)
            }
        }
    }
}
