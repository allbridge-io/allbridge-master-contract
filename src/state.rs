//! State transition types
//!
use crate::PROGRAM_VERSION;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
    program_error::ProgramError,
    entrypoint::ProgramResult,
    msg
};

pub type TxId = [u8; 64];
pub type Address = [u8; 32];
pub type BlockchainId = [u8; 4];


/// Information about the bridge
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Bridge {
    /// Data version
    pub version: u8,
    /// Bridge owner account, signs secure instructions to the bridge
    pub owner: Pubkey,
}

impl Bridge {
    /// Struct size
    pub const LEN: usize = 33;
    /// Create new bridge entity
    pub fn new(owner: Pubkey) -> Self {
        Self {
            version: PROGRAM_VERSION,
            owner,
        }
    }

    pub fn check_initialized(&self, expect_initialized: bool) -> ProgramResult {
        if expect_initialized && self.version != PROGRAM_VERSION {
            msg!("Account not initialized");
            return Err(ProgramError::UninitializedAccount);
        } else if !expect_initialized && self.version == PROGRAM_VERSION {
            msg!("Account already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(())
    }
}

///Information about blockchain
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Blockchain {
    /// Data version
    pub version: u8,

    /// Associated bridge address
    pub bridge: Pubkey,

    /// Blockchain id (4 bytes of UTF-8)
    pub blockchain_id: BlockchainId,

    /// Number of validators
    pub validators: u64,

    /// Number of locks
    pub locks: u64,

    /// Address of contract for the bridge
    pub contract_address: Address

}

impl Blockchain {
    /// Struct size
    pub const LEN: usize = 85;
    /// Create new blockchain entity
    pub fn new(bridge: Pubkey, blockchain_id: BlockchainId, contract_address: Address) -> Self {
        Self {
            version: PROGRAM_VERSION,
            bridge,
            blockchain_id,
            locks: 0,
            validators: 0,
            contract_address
        }
    }

    pub fn check_initialized(&self, expect_initialized: bool) -> ProgramResult {
        if expect_initialized && self.version != PROGRAM_VERSION {
            msg!("Account not initialized");
            return Err(ProgramError::UninitializedAccount);
        } else if !expect_initialized && self.version == PROGRAM_VERSION {
            msg!("Account already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(())
    }
}

/// Validator info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Validator {
    /// Data version
    pub version: u8,

    /// Blockchain id (4 bytes of UTF-8)
    pub blockchain_id: BlockchainId,

    /// Validator index
    pub index: u64,

    /// Validator public key for current blockchain
    pub pub_key: [u8; 32],

    /// Validator owner
    pub owner: Pubkey

}

impl Validator {
    /// Struct size
    pub const LEN: usize = 77;
    /// Create new validator entity
    pub fn new(blockchain_id: BlockchainId, index: u64, pub_key: [u8; 32], owner: Pubkey) -> Self {
        Self {
            version: PROGRAM_VERSION,
            blockchain_id,
            index,
            pub_key,
            owner
        }
    }

    pub fn check_initialized(&self, expect_initialized: bool) -> ProgramResult {
        if expect_initialized && self.version != PROGRAM_VERSION {
            msg!("Account not initialized");
            return Err(ProgramError::UninitializedAccount);
        } else if !expect_initialized && self.version == PROGRAM_VERSION {
            msg!("Account already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(())
    }
}


/// Lock info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Lock {
    /// Data version
    pub version: u8,

    /// Lock index within the bridge
    pub index: u64,

    /// Lock id
    pub lock_id: u64,

    /// Lock transaction id
    pub tx_id: TxId,

    /// Bridge reference
    pub bridge: Pubkey,

    /// Token address from source blockchain
    pub token_source_address: Address,

    /// Token source
    pub token_source: BlockchainId,

    /// Source blockchain identifier
    pub source: BlockchainId,

    /// Sender address
    pub sender: Address,

    /// Recipient address
    pub recipient: Address,

    /// Destination blockchain identifier
    pub destination: BlockchainId,

    /// Amount to lock for the transfer
    pub amount: u64,

    /// Signature count
    pub signatures: u64
}

impl Lock {
    /// Struct size
    pub const LEN: usize = 237;
    /// Create new validator entity
    pub fn new(index: u64, lock_id: u64, tx_id: TxId, bridge: Pubkey, token_source_address: Address, token_source: BlockchainId, source: BlockchainId, sender: Address, recipient: Address, destination: BlockchainId, amount: u64) -> Self {
        Self {
            version: PROGRAM_VERSION,
            index,
            lock_id,
            tx_id,
            bridge,
            token_source,
            token_source_address,
            source,
            sender,
            recipient,
            destination,
            amount,
            signatures: 0
        }
    }

    pub fn check_initialized(&self, expect_initialized: bool) -> ProgramResult {
        if expect_initialized && self.version != PROGRAM_VERSION {
            msg!("Account not initialized");
            return Err(ProgramError::UninitializedAccount);
        } else if !expect_initialized && self.version == PROGRAM_VERSION {
            msg!("Account already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(())
    }
}

/// Signature info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Signature {
    /// Data version
    pub version: u8,
    /// Source blockchain identifier
    pub source: BlockchainId,
    /// Lock id
    pub lock_id: u64,
    /// Bridge reference
    pub bridge: Pubkey,
    /// Oracle signature
    pub signature: [u8; 65],
    /// Validator public key
    pub validator: Pubkey,
    /// Validator index
    pub validator_index: u64
}

impl Signature {
    /// Struct size
    pub const LEN: usize = 150;
    /// Create new validator entity
    pub fn new(source: BlockchainId,
               lock_id: u64,
               bridge: Pubkey,
               signature: [u8; 65],
               validator: Pubkey,
               validator_index: u64) -> Self {
        Self {
            version: PROGRAM_VERSION,
            source,
            lock_id,
            bridge,
            signature,
            validator,
            validator_index
        }
    }

    pub fn check_initialized(&self, expect_initialized: bool) -> ProgramResult {
        if expect_initialized && self.version != PROGRAM_VERSION {
            msg!("Account not initialized");
            return Err(ProgramError::UninitializedAccount);
        } else if !expect_initialized && self.version == PROGRAM_VERSION {
            msg!("Account already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(())
    }
}


/// User info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct User {
    /// Data version
    pub version: u8,
    /// Blockchain ID
    pub blockchain_id: BlockchainId,
    /// User address
    pub address: Address,
    /// Number of sent transactions
    pub sent: u64,
    /// Number of received transactions
    pub received: u64
}

impl User {
    /// Struct size
    pub const LEN: usize = 53;
    /// Create new validator entity
    pub fn new(blockchain_id: BlockchainId,
               address: Address) -> Self {
        Self {
            version: PROGRAM_VERSION,
            blockchain_id,
            address,
            sent: 0,
            received: 0,
        }
    }

    /// is initialized account method
    pub fn check_initialized(&self, expect_initialized: bool) -> ProgramResult {
        if expect_initialized && self.version != PROGRAM_VERSION {
            msg!("Account not initialized");
            return Err(ProgramError::UninitializedAccount);
        } else if !expect_initialized && self.version == PROGRAM_VERSION {
            msg!("Account already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(())
    }
}

/// Sent info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct LockTx {
    /// Data version
    pub version: u8,
    /// Lock transaction id
    pub tx_id: TxId,

    pub source: BlockchainId,

    pub lock_id: u64,

    pub lock_account: Pubkey,

    pub reverted: bool
}

impl LockTx {
    /// Struct size
    pub const LEN: usize = 110;
    /// Create new validator entity
    pub fn new(tx_id: TxId, source: BlockchainId, lock_id: u64, lock_account: Pubkey, reverted: bool) -> Self {
        Self {
            version: PROGRAM_VERSION,
            tx_id,
            source,
            lock_id,
            lock_account,
            reverted
        }
    }

    /// is initialized account method
    pub fn check_initialized(&self, expect_initialized: bool) -> ProgramResult {
        if expect_initialized && self.version != PROGRAM_VERSION {
            msg!("Account not initialized");
            return Err(ProgramError::UninitializedAccount);
        } else if !expect_initialized && self.version == PROGRAM_VERSION {
            msg!("Account already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(())
    }
}
