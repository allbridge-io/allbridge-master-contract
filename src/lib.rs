// #![deny(missing_docs)]

//! A minimal Solana program template

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

/// Current program version
pub const PROGRAM_VERSION: u8 = 1;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

// FIXME: change this address before run tests/program
solana_program::declare_id!("2svg2dmcS8L3s2GyBBjfoeQW89Rx1zH3rkDC5EhmpXVi");
