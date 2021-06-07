//! State transition types
//!
use crate::PROGRAM_VERSION;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

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
}

impl IsInitialized for Bridge {
    fn is_initialized(&self) -> bool {
        self.version == PROGRAM_VERSION
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
    pub blockchain_id: [u8; 4],

    /// Number of validators
    pub validators: u64,

    /// Number of locks
    pub locks: u64,

    /// Address of contract for the bridge
    pub contract_address: [u8; 32]

}

impl IsInitialized for Blockchain {
    fn is_initialized(&self) -> bool {
        self.version == PROGRAM_VERSION
    }
}

impl Blockchain {
    /// Struct size
    pub const LEN: usize = 85;
    /// Create new blockchain entity
    pub fn new(bridge: Pubkey, blockchain_id: [u8; 4], contract_address: [u8; 32]) -> Self {
        Self {
            version: PROGRAM_VERSION,
            bridge,
            blockchain_id,
            locks: 0,
            validators: 0,
            contract_address
        }
    }
}

/// Validator info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Validator {
    /// Data version
    pub version: u8,

    /// Blockchain id (4 bytes of UTF-8)
    pub blockchain_id: [u8; 4],

    /// Validator index
    pub index: u64,

    /// Validator public key for current blockchain
    pub pub_key: [u8; 32],

    /// Validator owner
    pub owner: Pubkey

}

impl IsInitialized for Validator {
    fn is_initialized(&self) -> bool {
        self.version == PROGRAM_VERSION
    }
}

impl Validator {
    /// Struct size
    pub const LEN: usize = 77;
    /// Create new validator entity
    pub fn new(blockchain_id: [u8; 4], index: u64, pub_key: [u8; 32], owner: Pubkey) -> Self {
        Self {
            version: PROGRAM_VERSION,
            blockchain_id,
            index,
            pub_key,
            owner
        }
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
    pub tx_id: [u8; 64],

    /// Bridge reference
    pub bridge: Pubkey,

    /// Token address from source blockchain
    pub token_source_address: [u8; 32],

    /// Token source
    pub token_source: [u8; 4],

    /// Source blockchain identifier
    pub source: [u8; 4],

    /// Sender address
    pub sender: [u8; 32],

    /// Recipient address
    pub recipient: [u8; 32],

    /// Destination blockchain identifier
    pub destination: [u8; 4],

    /// Amount to lock for the transfer
    pub amount: u64,

    /// Signature count
    pub signatures: u64
}

impl IsInitialized for Lock {
    fn is_initialized(&self) -> bool {
        self.version == PROGRAM_VERSION
    }
}

impl Lock {
    /// Struct size
    pub const LEN: usize = 237;
    /// Create new validator entity
    pub fn new(index: u64, lock_id: u64, tx_id: [u8; 64], bridge: Pubkey, token_source_address: [u8; 32], token_source: [u8; 4], source: [u8; 4], sender: [u8; 32], recipient: [u8; 32], destination: [u8; 4], amount: u64) -> Self {
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
}

/// Signature info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Signature {
    /// Data version
    pub version: u8,
    /// Lock index within the bridge
    pub index: u64,
    /// Bridge reference
    pub bridge: Pubkey,
    /// Oracle signature
    pub signature: [u8; 65],
    /// Validator public key
    pub validator: Pubkey,
}

impl IsInitialized for Signature {
    fn is_initialized(&self) -> bool {
        self.version == PROGRAM_VERSION
    }
}

impl Signature {
    /// Struct size
    pub const LEN: usize = 138;
    /// Create new validator entity
    pub fn new(index: u64,
               bridge: Pubkey,
               signature: [u8; 65],
               validator: Pubkey) -> Self {
        Self {
            version: PROGRAM_VERSION,
            index,
            bridge,
            signature,
            validator,
        }
    }
}


/// User info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct User {
    /// Data version
    pub version: u8,
    /// Blockchain ID
    pub blockchain_id: [u8; 4],
    /// User address
    pub address: [u8; 32],
    /// Number of sent transactions
    pub sent: u64,
    /// Number of received transactions
    pub received: u64
}

impl User {
    /// Struct size
    pub const LEN: usize = 53;
    /// Create new validator entity
    pub fn new(blockchain_id: [u8; 4],
               address: [u8; 32]) -> Self {
        Self {
            version: PROGRAM_VERSION,
            blockchain_id,
            address,
            sent: 0,
            received: 0,
        }
    }

    /// is initialized account method
    pub fn is_initialized(&self) -> bool {
        self.version == PROGRAM_VERSION
    }
}

/// Sent info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SentLock {
    /// Data version
    pub version: u8,
    /// Lock transaction id
    tx_id: [u8; 64]
}

impl SentLock {
    /// Struct size
    pub const LEN: usize = 65;
    /// Create new validator entity
    pub fn new(tx_id: [u8; 64]) -> Self {
        Self {
            version: PROGRAM_VERSION,
            tx_id
        }
    }

    /// is initialized account method
    pub fn is_initialized(&self) -> bool {
        self.version == PROGRAM_VERSION
    }
}

/// Received info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ReceivedLock {
    /// Data version
    pub version: u8,
    /// Lock transaction id
    tx_id: [u8; 64]
}

impl ReceivedLock {
    /// Struct size
    pub const LEN: usize = 65;
    /// Create new validator entity
    pub fn new(tx_id: [u8; 64]) -> Self {
        Self {
            version: PROGRAM_VERSION,
            tx_id
        }
    }

    /// is initialized account method
    pub fn is_initialized(&self) -> bool {
        self.version == PROGRAM_VERSION
    }
}
