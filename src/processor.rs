//! Program state processor

use crate::{
    instruction::BridgeProgramInstruction,
    state::{Bridge, Blockchain, Validator, Lock, Signature, User, LockTx, BlockchainId, Address, TxId},
    utils::*
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

/// Program state handler.
pub struct Processor {}
impl Processor {
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
        bridge_account_data.check_initialized(false)?;

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
        blockchain_id: BlockchainId,
        contract_address: Address
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let bridge_account_info = next_account_info(account_info_iter)?;
        let blockchain_account_info = next_account_info(account_info_iter)?;
        let payer_info = next_account_info(account_info_iter)?;
        let bridge_authority_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_account_info)?;

        let bump_seed = validate_bridge_authority_and_get_bump_seed(
            program_id,
            bridge_account_info.key,
            &bridge_authority_info.key,
        )?;

        let seed = check_and_get_blockchain_account_seed(
            program_id,
            blockchain_id,
            bridge_authority_info.key,
            blockchain_account_info.key
        )?;

        create_account_with_seed(
            payer_info,
            blockchain_account_info,
            bridge_authority_info,
            seed,
            Blockchain::LEN,
            rent,
            program_id,
            bridge_account_info.key.as_ref(),
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
        blockchain_id: BlockchainId,
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
        blockchain_account_data.check_initialized(true)?;

        let validator_index = blockchain_account_data.validators;

        let bump_seed = validate_bridge_authority_and_get_bump_seed(
            program_id,
            bridge_account_info.key,
            &bridge_authority_info.key,
        )?;

        let seed = check_and_get_validator_account_seed(
            program_id,
            blockchain_id,
            validator_index,
            bridge_authority_info.key,
            validator_account_info.key
        )?;

        create_account_with_seed(
            payer_info,
            validator_account_info,
            bridge_authority_info,
            seed,
            Validator::LEN,
            rent,
            program_id,
            bridge_account_info.key.as_ref(),
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
        token_source: BlockchainId,
        token_source_address: Address,
        source: BlockchainId,
        tx_id: TxId,
        lock_id: u64,
        destination: BlockchainId,
        sender: Address,
        recipient: Address,
        amount: u64,
        revert: bool
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
        bridge_account_data.check_initialized(true)?;

        let mut blockchain_account_data: Blockchain = Blockchain::try_from_slice(&blockchain_account_info.data.borrow_mut())?;
        blockchain_account_data.check_initialized(true)?;

        let validator_account_data: Validator = Validator::try_from_slice(&validator_account_info.data.borrow())?;
        validator_account_data.check_initialized(true)?;

        if validator_account_data.owner != *payer_info.key {
            msg!("Payer is not the validator");
            return Err(ProgramError::InvalidArgument);
        }

        if validator_account_data.blockchain_id != source {
            msg!("Invalid validator type");
            return Err(ProgramError::InvalidArgument);
        }

        let bump_seed = validate_bridge_authority_and_get_bump_seed(
            program_id,
            bridge_account_info.key,
            &bridge_authority_info.key,
        )?;

        let lock_seed = check_and_get_lock_account_seed(
            program_id,
            source,
            lock_id,
            revert,
            bridge_authority_info.key,
            lock_account_info.key
        )?;

        let mut lock_account_data = if lock_account_info.data_is_empty() {
            create_account_with_seed(
                payer_info,
                lock_account_info,
                bridge_authority_info,
                lock_seed,
                Lock::LEN,
                rent,
                program_id,
                bridge_account_info.key.as_ref(),
                bump_seed,
            )?;

            let index = blockchain_account_data.locks;
            blockchain_account_data.locks += 1;
            blockchain_account_data.serialize(&mut *blockchain_account_info.data.borrow_mut())?;

            let mut sender_user_data = Self::get_or_create_user_data(program_id, source, sender, sender_user_authority_info, sender_user_info, payer_info, rent)?;
            let mut recipient_user_data = Self::get_or_create_user_data(program_id, destination, recipient, recipient_user_authority_info, recipient_user_info, payer_info, rent)?;

            if sent_lock_info.lamports() > 0 {
                msg!("Sent lock account is initialized");
                return Err(ProgramError::AccountAlreadyInitialized);
            }
            Self::create_lock_tx_account(program_id,
                                         source,
                                         payer_info,
                                         sent_lock_info,
                                         lock_account_info,
                                         sender_user_authority_info,
                                         sender,
                                         sender_user_data.sent,
                                         tx_id,
                                         source,
                                         lock_id,
                                         false,
                                         "sent", rent)?;

            if received_lock_info.lamports() > 0 {
                msg!("Received lock account is initialized");
                return Err(ProgramError::AccountAlreadyInitialized);
            }
            Self::create_lock_tx_account(program_id,
                                         destination,
                                         payer_info,
                                         received_lock_info,
                                         lock_account_info,
                                         recipient_user_authority_info,
                                         recipient,
                                         recipient_user_data.received,
                                         tx_id,
                                         source,
                                         lock_id,
                                         false,
                                         "received",
                                         rent)?;

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

        lock_account_data.check_initialized(true)?;

        if
            lock_account_data.lock_id != lock_id ||
            lock_account_data.tx_id != tx_id ||
            lock_account_data.token_source_address != token_source_address ||
            lock_account_data.token_source != token_source ||
            lock_account_data.source != source ||
            lock_account_data.sender != sender ||
            lock_account_data.recipient != recipient ||
            lock_account_data.destination != destination ||
            lock_account_data.amount != amount
        {
            msg!("Existing lock does not match with the params");
            return Err(ProgramError::InvalidArgument);
        }

        let signature_seed = check_and_get_signature_account_seed(
            program_id,
            source,
            lock_id,
            validator_account_data.index,
            revert,
            bridge_authority_info.key,
            signature_account_info.key
        )?;

        create_account_with_seed(
            payer_info,
            signature_account_info,
            bridge_authority_info,
            signature_seed,
            Signature::LEN,
            rent,
            program_id,
            bridge_account_info.key.as_ref(),
            bump_seed,
        )?;

        let signature = Signature::new(
            source,
            lock_id,
            *bridge_account_info.key,
            signature,
            *validator_account_info.key,
            validator_account_data.index);
        signature.serialize(&mut *signature_account_info.data.borrow_mut())?;

        lock_account_data.signatures += 1;
        lock_account_data.serialize(&mut *lock_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn get_or_create_user_data<'a>(program_id: &Pubkey, blockchain_id: BlockchainId, user_address: Address, user_authority_info: & AccountInfo<'a>, user_info: & AccountInfo<'a>, payer_info: & AccountInfo<'a>, rent: & Rent) -> Result<User, ProgramError> {
        msg!("get_or_create_user_data");
        let bump_seed = validate_user_address_authority_and_get_bump_seed(program_id, user_address, user_authority_info.key)?;
        let seed = check_and_get_user_account_seed(program_id, blockchain_id, user_authority_info.key, user_info.key)?;
        return if user_info.data_is_empty() {
            create_account_with_seed(
                payer_info,
                user_info,
                user_authority_info,
                seed,
                User::LEN,
                rent,
                program_id,
                user_address.as_ref(),
                bump_seed,
            )?;
            Ok(User::new(blockchain_id, user_address))
        } else {
            Ok(User::try_from_slice(&user_info.data.borrow_mut())?)
        }
    }

    fn create_lock_tx_account<'a>(
        program_id: &Pubkey,
        blockchain_id: BlockchainId,
        payer_info: &AccountInfo<'a>,
        lock_tx_info: &AccountInfo<'a>,
        lock_info: &AccountInfo<'a>,
        user_authority_info: &AccountInfo<'a>,
        user_address: Address, index: u64,
        tx_id: TxId,
        source: BlockchainId,
        lock_id: u64,
        reverted: bool,
        tx_type: &str,
        rent: &Rent) -> ProgramResult {
        let bump_seed = validate_user_address_authority_and_get_bump_seed(program_id, user_address, user_authority_info.key)?;
        let seed = check_and_get_lock_tx_account_seed(program_id, blockchain_id, index, tx_type, user_authority_info.key, lock_tx_info.key)?;

        create_account_with_seed(
            payer_info,
            lock_tx_info,
            user_authority_info,
            seed,
            LockTx::LEN,
            rent,
            program_id,
            user_address.as_ref(),
            bump_seed,
        )?;

        LockTx::new(tx_id, source, lock_id, *lock_info.key, reverted).serialize(&mut *lock_tx_info.data.borrow_mut())?;
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
            BridgeProgramInstruction::AddSignature {signature, token_source, token_source_address, source, tx_id, lock_id, destination,sender,  recipient, amount, revert} => {
                msg!("Instruction: AddBlockchain");
                Self::process_add_signature(program_id, accounts, signature, token_source, token_source_address, source, tx_id, lock_id, destination, sender, recipient, amount, revert)
            }
        }
    }
}
