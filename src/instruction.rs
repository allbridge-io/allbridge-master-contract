//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
    system_program,
};
use crate::utils::str_to_chain_id;
use crate::state::{Address, BlockchainId, TxId};

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum BridgeProgramInstruction {
    /// Initializes new bridge account
    /// 0. `[W]`  Uninitialized bridge account
    /// 1. `[RS]` Bridge account owner
    /// 2. `[R]`  System rent variable, used to check if bridge and token account have enough SOL to be rent-exempt
    InitializeBridge,

    ///Add new blockchain
    AddBlockchain {
        /// blockchain_id
        blockchain_id: BlockchainId,
        /// contract_address
        contract_address: Address
    },

    ///Add new validator
    AddValidator {
        /// blockchain_id
        blockchain_id: BlockchainId,

        ///Validator public key
        pub_key: [u8; 32],
    },

    ///Add new signature
    AddSignature {
        /// signature
        signature: [u8; 65],

        /// token_source
        token_source: BlockchainId,

        /// token_source_address
        token_source_address: Address,

        /// source
        source: BlockchainId,

        /// lock_id
        lock_id: u64,

        /// tx_id
        tx_id: TxId,

        /// destination
        destination: BlockchainId,

        /// sender
        sender: Address,

        /// recipient
        recipient: Address,

        /// amount
        amount: u64,

        /// Is reverted transfer by user
        revert: bool,
    },
}

/// Create `InitBridge` instruction
pub fn init_bridge(
    program_id: &Pubkey,
    bridge_account: &Pubkey,
    owner_account: &Pubkey
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::InitializeBridge;
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;
    let accounts = vec![
        AccountMeta::new(*bridge_account, false),
        AccountMeta::new_readonly(*owner_account, true),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `AddBlockchain` instruction
pub fn add_blockchain(
    program_id: &Pubkey,
    bridge_account: &Pubkey,
    blockchain_account: &Pubkey,
    payer_account: &Pubkey,
    bridge_authority: &Pubkey,
    blockchain_id: String,
    contract_address: Address
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::AddBlockchain {blockchain_id: str_to_chain_id(blockchain_id.as_str()), contract_address};
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;
    let accounts = vec![
        AccountMeta::new(*bridge_account, false),
        AccountMeta::new(*blockchain_account, false),
        AccountMeta::new_readonly(*payer_account, true),
        AccountMeta::new_readonly(*bridge_authority, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `AddBlockchain` instruction
pub fn add_validator(
    program_id: &Pubkey,
    bridge_account: &Pubkey,
    blockchain_account: &Pubkey,
    validator_account: &Pubkey,
    payer_account: &Pubkey,
    bridge_authority: &Pubkey,
    blockchain_id: String,
    pub_key: [u8; 32]
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::AddValidator {blockchain_id: str_to_chain_id(blockchain_id.as_str()), pub_key};
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;
    let accounts = vec![
        AccountMeta::new(*bridge_account, false),
        AccountMeta::new(*blockchain_account, false),
        AccountMeta::new(*validator_account, false),
        AccountMeta::new_readonly(*payer_account, true),
        AccountMeta::new_readonly(*bridge_authority, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `AddSignature` instruction
pub fn add_signature(
    program_id: &Pubkey,
    bridge_account: &Pubkey,
    blockchain_account: &Pubkey,
    validator_account: &Pubkey,
    lock_account: &Pubkey,
    signature_account: &Pubkey,
    bridge_authority: &Pubkey,
    sender_user: &Pubkey,
    sender_user_authority: &Pubkey,
    recipient_user: &Pubkey,
    recipient_user_authority: &Pubkey,
    sent_lock: &Pubkey,
    received_lock: &Pubkey,
    payer_account: &Pubkey,
    signature: [u8; 65],
    token_source: String,
    token_source_address: Address,
    source: String,
    tx_id: TxId,
    lock_id: u64,
    destination: String,
    sender: Address,
    recipient: Address,
    amount: u64,
    revert: bool
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::AddSignature {
        signature, token_source: str_to_chain_id(token_source.as_str()),
        token_source_address,
        source: str_to_chain_id(source.as_str()),
        tx_id,
        lock_id,
        destination: str_to_chain_id(destination.as_str()),
        sender,
        recipient,
        amount,
        revert
    };
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;
    let accounts = vec![
        AccountMeta::new(*bridge_account, false),
        AccountMeta::new(*blockchain_account, false),
        AccountMeta::new(*validator_account, false),
        AccountMeta::new(*lock_account, false),
        AccountMeta::new(*signature_account, false),
        AccountMeta::new_readonly(*bridge_authority, false),
        AccountMeta::new(*sender_user, false),
        AccountMeta::new_readonly(*sender_user_authority, false),
        AccountMeta::new(*recipient_user, false),
        AccountMeta::new_readonly(*recipient_user_authority, false),
        AccountMeta::new(*sent_lock, false),
        AccountMeta::new(*received_lock, false),
        AccountMeta::new_readonly(*payer_account, true),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false)
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
