//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
    system_program
};

fn str_to_chain_id(s: String) -> [u8; 4] {
    let len = s.len();
    let mut result = [0; 4];
    result[..len].copy_from_slice(s.as_bytes());
    result
}


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
        blockchain_id: [u8; 4],
        /// contract_address
        contract_address: [u8; 32]
    },

    ///Add new validator
    AddValidator {
        /// blockchain_id
        blockchain_id: [u8; 4],

        ///Validator public key
        pub_key: [u8; 32],
    },

    ///Add new signature
    AddSignature {
        /// signature
        signature: [u8; 65],

        /// token_source
        token_source: [u8; 4],

        /// token_source_address
        token_source_address: [u8; 32],

        /// source
        source: [u8; 4],

        /// lock_id
        lock_id: u64,

        /// destination
        destination: [u8; 4],

        /// recipient
        recipient: [u8; 32],

        /// amount
        amount: u64
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
    contract_address: [u8; 32]
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::AddBlockchain {blockchain_id: str_to_chain_id(blockchain_id), contract_address};
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
    let init_data = BridgeProgramInstruction::AddValidator {blockchain_id: str_to_chain_id(blockchain_id), pub_key};
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
    payer_account: &Pubkey,
    signature: [u8; 65],
    token_source: String,
    token_source_address: [u8; 32],
    source: String,
    lock_id: u64,
    destination: String,
    recipient: [u8; 32],
    amount: u64
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::AddSignature {
        signature, token_source: str_to_chain_id(token_source),
        token_source_address,
        source: str_to_chain_id(source),
        lock_id,
        destination: str_to_chain_id(destination),
        recipient,
        amount
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
