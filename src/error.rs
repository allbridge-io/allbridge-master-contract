//! Error types

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the Bridge program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum SolBridgeProgramError {
    /// Example error
    #[error("Invalid signature")]
    InvalidSignature,
    /// Secp256 instruction losing
    #[error("Secp256 instruction losing")]
    Secp256InstructionLosing,
}
impl From<SolBridgeProgramError> for ProgramError {
    fn from(e: SolBridgeProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for SolBridgeProgramError {
    fn type_of() -> &'static str {
        "BridgeProgramError"
    }
}

impl PrintProgramError for SolBridgeProgramError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            SolBridgeProgramError::InvalidSignature => msg!("Invalid signature"),
            SolBridgeProgramError::Secp256InstructionLosing => msg!("Secp256 instruction losing"),
        }
    }
}
