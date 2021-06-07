//! Program state processor

use crate::{
    instruction::BridgeProgramInstruction,
    state::{Bridge, Blockchain, Validator, Lock, Signature, User, SentLock, ReceivedLock},
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

    fn validate_user_address_authority_and_get_bump_seed(
        program_id: &Pubkey,
        user_address: [u8; 32],
        authority_account: &Pubkey,
    ) -> Result<u8, ProgramError> {
        msg!("validate_user_address_authority_and_get_bump_seed");
        let signer_seeds = &[user_address.as_ref()];
        let (expected_authority_account, bump_seed) =
            Pubkey::find_program_address(signer_seeds, program_id);
        if expected_authority_account != *authority_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(bump_seed)
    }

    fn check_and_get_blockchain_account_seed(
        program_id: &Pubkey,
        blockchain_id: [u8; 4],
        bridge_authority: &Pubkey,
        blockchain_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        let seed = format!("blockchain_{}", Self::chain_id_to_str(&blockchain_id)?);
        let expected_blockchain_account =
            Pubkey::create_with_seed(bridge_authority, seed.as_str(), program_id)?;
        if expected_blockchain_account != *blockchain_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn check_and_get_validator_account_seed(
        program_id: &Pubkey,
        blockchain_id: [u8; 4],
        index: u64,
        bridge_authority: &Pubkey,
        validator_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        let seed = format!("validator_{}_{}", Self::chain_id_to_str(&blockchain_id)?, index);
        let expected_validator_account =
            Pubkey::create_with_seed(bridge_authority, seed.as_str(), program_id)?;
        if expected_validator_account != *validator_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn check_and_get_lock_account_seed(
        program_id: &Pubkey,
        source: [u8; 4],
        tx_id: [u8; 64],
        bridge_authority: &Pubkey,
        lock_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        let seed = format!("lock_{}_{}", Self::chain_id_to_str(&source)?, &bs58::encode(&tx_id).into_string()[..20]);
        let expected_lock_account =
            Pubkey::create_with_seed(bridge_authority, seed.as_str(), program_id)?;
        if expected_lock_account != *lock_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn check_and_get_signature_account_seed(
        program_id: &Pubkey,
        source: [u8; 4],
        lock_id: u64,
        index: u64,
        bridge_authority: &Pubkey,
        signature_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        let seed = format!("signature_lock_{}_{}_{}", Self::chain_id_to_str(&source)?, lock_id, index);
        let expected_signature_account =
            Pubkey::create_with_seed(bridge_authority, seed.as_str(), program_id)?;
        if expected_signature_account != *signature_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn check_and_get_user_account_seed(
        program_id: &Pubkey,
        blockchain_id: [u8; 4],
        user_authority: &Pubkey,
        user_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        msg!("check_and_get_user_account_seed");
        let seed = format!("user_{}", Self::chain_id_to_str(&blockchain_id)?);
        let expected_user_account =
            Pubkey::create_with_seed(user_authority, seed.as_str(), program_id)?;
        if expected_user_account != *user_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn check_and_get_sent_lock_account_seed(
        program_id: &Pubkey,
        blockchain_id: [u8; 4],
        user_authority: &Pubkey,
        index: u64,
        sent_lock_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        msg!("check_and_get_sent_lock_account_seed");
        let seed = format!("sent_{}_{}", Self::chain_id_to_str(&blockchain_id)?, index);
        let expected_account =
            Pubkey::create_with_seed(user_authority, seed.as_str(), program_id)?;
        if expected_account != *sent_lock_account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(seed)
    }

    fn check_and_get_received_lock_account_seed(
        program_id: &Pubkey,
        blockchain_id: [u8; 4],
        user_authority: &Pubkey,
        index: u64,
        received_lock_account: &Pubkey,
    ) -> Result<String, ProgramError> {
        msg!("check_and_get_received_lock_account_seed");
        let seed = format!("received_{}_{}", Self::chain_id_to_str(&blockchain_id)?, index);
        let expected_account =
            Pubkey::create_with_seed(user_authority, seed.as_str(), program_id)?;
        if expected_account != *received_lock_account {
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

    fn chain_id_to_str(chain_id: &[u8; 4]) -> Result<&str, ProgramError> {
        std::str::from_utf8(chain_id)
            .map_err(|_| ProgramError::InvalidArgument)
            .map(|s| s.trim_matches(0 as char))
    }

    fn create_account_with_seed<'a>(
        payer_info: &AccountInfo<'a>,
        new_account: &AccountInfo<'a>,
        authority_info: &AccountInfo<'a>,
        seed: String,
        data_size: usize,
        rent: &Rent,
        program_id: &Pubkey,
        bridge_account: &Pubkey,
        bump_seed: u8,
    ) -> ProgramResult {
        msg!("create_account_with_seed");
        invoke_signed(
            &system_instruction::create_account_with_seed(
                &payer_info.key,
                &new_account.key,
                &authority_info.key,
                seed.as_str(),
                rent.minimum_balance(data_size),
                data_size as u64,
                &program_id,
            ),
            &[
                payer_info.clone(),
                new_account.clone(),
                authority_info.clone(),
            ],
            &[&[
                bridge_account.as_ref(),
                &[bump_seed],
            ]],
        )
    }

    fn create_account_with_user_seed<'a>(
        payer_info: &AccountInfo<'a>,
        new_account: &AccountInfo<'a>,
        authority_info: &AccountInfo<'a>,
        seed: String,
        data_size: usize,
        rent: &Rent,
        program_id: &Pubkey,
        address: [u8; 32],
        bump_seed: u8,
    ) -> ProgramResult {
        msg!("create_account_with_seed");
        invoke_signed(
            &system_instruction::create_account_with_seed(
                &payer_info.key,
                &new_account.key,
                &authority_info.key,
                seed.as_str(),
                rent.minimum_balance(data_size),
                data_size as u64,
                &program_id,
            ),
            &[
                payer_info.clone(),
                new_account.clone(),
                authority_info.clone(),
            ],
            &[&[
                address.as_ref(),
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
        blockchain_id: [u8; 4],
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
            blockchain_id,
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
            blockchain_id,
            contract_address);
        blockchain.serialize(&mut *blockchain_account_info.data.borrow_mut())?;
        Ok(())
    }


    /// Process add validator
    pub fn process_add_validator(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        blockchain_id: [u8; 4],
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
            blockchain_id,
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
            blockchain_id,
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
        token_source: [u8; 4],
        token_source_address: [u8; 32],
        source: [u8; 4],
        tx_id: [u8; 64],
        lock_id: u64,
        destination: [u8; 4],
        sender: [u8; 32],
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
        let sender_user_info = next_account_info(account_info_iter)?;
        let sender_user_authority_info = next_account_info(account_info_iter)?;
        let recipient_user_info = next_account_info(account_info_iter)?;
        let recipient_user_authority_info = next_account_info(account_info_iter)?;
        let sent_lock_info = next_account_info(account_info_iter)?;
        let received_lock_info = next_account_info(account_info_iter)?;
        let payer_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_account_info)?;

        if !payer_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let bridge_account_data: Bridge = Bridge::try_from_slice(&bridge_account_info.data.borrow())?;
        if !bridge_account_data.is_initialized() {
            msg!("Bridge account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        let mut blockchain_account_data: Blockchain = Blockchain::try_from_slice(&blockchain_account_info.data.borrow_mut())?;
        if !blockchain_account_data.is_initialized() {
            msg!("Blockchain account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        let validator_account_data: Validator = Validator::try_from_slice(&validator_account_info.data.borrow())?;
        if !validator_account_data.is_initialized() {
            msg!("Blockchain account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        if validator_account_data.owner != *payer_info.key {
            msg!("Payer is not the validator");
            return Err(ProgramError::InvalidArgument);
        }

        if validator_account_data.blockchain_id != source {
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
            source,
            tx_id,
            bridge_authority_info.key,
            lock_account_info.key
        )?;
        msg!("lock_account_data");
        let mut lock_account_data = if lock_account_info.data_is_empty() {
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

            let index = blockchain_account_data.locks;
            blockchain_account_data.locks += 1;
            blockchain_account_data.serialize(&mut *blockchain_account_info.data.borrow_mut())?;

            let mut sender_user_data = Self::get_or_create_user_data(program_id, source, sender, sender_user_authority_info, sender_user_info, payer_info, bridge_account_info, rent)?;
            let mut recipient_user_data = Self::get_or_create_user_data(program_id, destination, recipient, recipient_user_authority_info, recipient_user_info, payer_info, bridge_account_info, rent)?;

            Self::create_sent_lock_account(program_id, source, payer_info, sent_lock_info, sender_user_authority_info, sender, sender_user_data.sent, tx_id, bridge_account_info, rent)?;
            Self::create_received_lock_account(program_id, destination, payer_info, received_lock_info, recipient_user_authority_info, recipient, recipient_user_data.received, tx_id, bridge_account_info, rent)?;

            sender_user_data.sent += 1;
            recipient_user_data.received += 1;

            sender_user_data.serialize(&mut *sender_user_info.data.borrow_mut())?;
            recipient_user_data.serialize(&mut *recipient_user_info.data.borrow_mut())?;

            Lock::new(
                index,
                lock_id,
                tx_id,
                *bridge_account_info.key,
                token_source_address,
                token_source,
                source,
                sender,
                recipient,
                destination,
                amount)
        } else {
            Lock::try_from_slice(&lock_account_info.data.borrow_mut())?
        };



        if !lock_account_data.is_initialized() {
            msg!("Lock account is not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        let signature_seed = Self::check_and_get_signature_account_seed(
            program_id,
            source,
            lock_id,
            lock_account_data.signatures,
            bridge_authority_info.key,
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
            signature,
            *validator_account_info.key);
        signature.serialize(&mut *signature_account_info.data.borrow_mut())?;

        lock_account_data.signatures += 1;
        lock_account_data.serialize(&mut *lock_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn get_or_create_user_data<'a>(program_id: &Pubkey, blockchain_id: [u8; 4], user_address: [u8; 32], user_authority_info: & AccountInfo<'a>, user_info: & AccountInfo<'a>, payer_info: & AccountInfo<'a>, bridge_account_info: & AccountInfo<'a>, rent: & Rent) -> Result<User, ProgramError> {
        msg!("get_or_create_user_data");
        let bump_seed = Self::validate_user_address_authority_and_get_bump_seed(program_id, user_address, user_authority_info.key)?;
        let seed = Self::check_and_get_user_account_seed(program_id, blockchain_id, user_authority_info.key, user_info.key)?;
        return if user_info.data_is_empty() {
            Self::create_account_with_user_seed(
                payer_info,
                user_info,
                user_authority_info,
                seed,
                User::LEN,
                rent,
                program_id,
                user_address,
                bump_seed,
            )?;
            Ok(User::new(blockchain_id, user_address))
        } else {
            Ok(User::try_from_slice(&user_info.data.borrow_mut())?)
        }
    }

    fn create_sent_lock_account<'a>(program_id: &Pubkey, blockchain_id: [u8; 4], payer_info: &AccountInfo<'a>, sent_lock_info: &AccountInfo<'a>, user_authority_info: &AccountInfo<'a>, user_address: [u8; 32], index: u64, tx_id: [u8; 64], bridge_account_info: &AccountInfo<'a>, rent: &Rent) -> ProgramResult {
        msg!("create_sent_lock_account");
        let bump_seed = Self::validate_user_address_authority_and_get_bump_seed(program_id, user_address, user_authority_info.key)?;
        let seed = Self::check_and_get_sent_lock_account_seed(program_id, blockchain_id, user_authority_info.key, index, sent_lock_info.key)?;

        Self::create_account_with_user_seed(
            payer_info,
            sent_lock_info,
            user_authority_info,
            seed,
            SentLock::LEN,
            rent,
            program_id,
            user_address,
            bump_seed,
        )?;

        SentLock::new(tx_id).serialize(&mut *sent_lock_info.data.borrow_mut())?;
        Ok(())
    }

    fn create_received_lock_account<'a>(program_id: &Pubkey, blockchain_id: [u8; 4], payer_info: &AccountInfo<'a>, received_lock_info: &AccountInfo<'a>, user_authority_info: &AccountInfo<'a>, user_address: [u8; 32], index: u64, tx_id: [u8; 64], bridge_account_info: &AccountInfo<'a>, rent: &Rent) -> ProgramResult {
        msg!("create_received_lock_account");
        let bump_seed = Self::validate_user_address_authority_and_get_bump_seed(program_id, user_address, user_authority_info.key)?;
        let seed = Self::check_and_get_received_lock_account_seed(program_id, blockchain_id, user_authority_info.key, index, received_lock_info.key)?;

        Self::create_account_with_user_seed(
            payer_info,
            received_lock_info,
            user_authority_info,
            seed,
            ReceivedLock::LEN,
            rent,
            program_id,
            user_address,
            bump_seed,
        )?;

        ReceivedLock::new(tx_id).serialize(&mut *received_lock_info.data.borrow_mut())?;
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
                Self::process_add_blockchain(program_id, accounts, blockchain_id, contract_address)
            },
            BridgeProgramInstruction::AddValidator {blockchain_id, pub_key} => {
                msg!("Instruction: AddBlockchain");
                Self::process_add_validator(program_id, accounts, blockchain_id, pub_key)
            }
            BridgeProgramInstruction::AddSignature {signature, token_source, token_source_address, source, tx_id, lock_id, destination,sender,  recipient, amount} => {
                msg!("Instruction: AddBlockchain");
                Self::process_add_signature(program_id, accounts, signature, token_source, token_source_address, source, tx_id, lock_id, destination, sender, recipient, amount)
            }
        }
    }
}
