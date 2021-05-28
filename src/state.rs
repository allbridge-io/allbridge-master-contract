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
    /// Number of locks in this bridge
    pub locks: u64,
}

impl Bridge {
    /// Struct size
    pub const LEN: usize = 41;
    /// Create new bridge entity
    pub fn new(owner: Pubkey) -> Self {
        Self {
            version: PROGRAM_VERSION,
            owner,
            locks: 0,
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
    pub const LEN: usize = 77;
    /// Create new blockchain entity
    pub fn new(bridge: Pubkey, blockchain_id_str: &str, contract_address: [u8; 32]) -> Self {
        let id_len = blockchain_id_str.len();
        let mut blockchain_id = [0; 4];
        blockchain_id[..id_len].copy_from_slice(blockchain_id_str.as_bytes());
        Self {
            version: PROGRAM_VERSION,
            bridge,
            blockchain_id,
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
    pub fn new(blockchain_id_str: &str, index: u64, pub_key: [u8; 32], owner: Pubkey) -> Self {
        let id_len = blockchain_id_str.len();
        let mut blockchain_id = [0; 4];
        blockchain_id[..id_len].copy_from_slice(blockchain_id_str.as_bytes());
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

    /// Bridge reference
    pub bridge: Pubkey,

    /// Token address from source blockchain
    pub token_source_address: [u8; 32],

    /// Token source
    pub token_source: [u8; 4],

    /// Source blockchain identifier
    pub source: [u8; 4],

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
    pub const LEN: usize = 141;
    /// Create new validator entity
    pub fn new(index: u64, lock_id: u64, bridge: Pubkey, token_source_address: [u8; 32], token_source_str: &str, source_str: &str, recipient: [u8; 32], destination_str: &str, amount: u64) -> Self {
        let token_source_len = token_source_str.len();
        let mut token_source = [0; 4];
        token_source[..token_source_len].copy_from_slice(token_source_str.as_bytes());

        let source_len = source_str.len();
        let mut source = [0; 4];
        source[..source_len].copy_from_slice(source_str.as_bytes());

        let destination_str_len = destination_str.len();
        let mut destination = [0; 4];
        destination[..destination_str_len].copy_from_slice(destination_str.as_bytes());

        Self {
            version: PROGRAM_VERSION,
            index,
            lock_id,
            bridge,
            token_source,
            token_source_address,
            source,
            recipient,
            destination,
            amount,
            signatures: 0
        }
    }
}

/// Signature type
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum SigType {
    /// Simple lock type
    Lock
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
    /// Action type for signature ("lock")
    pub sig_type: SigType,
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
    pub const LEN: usize = 150;
    /// Create new validator entity
    pub fn new(index: u64,
               bridge: Pubkey,
               sig_type: SigType,
               signature: [u8; 65],
               validator: Pubkey) -> Self {
        Self {
            version: PROGRAM_VERSION,
            index,
            bridge,
            sig_type,
            signature,
            validator,
        }
    }
}
