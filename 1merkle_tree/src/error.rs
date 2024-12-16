use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Operation will overflow")]
    Overflow,
    #[error("Operation will underflow")]
    Underflow,
}

impl From<CustomError> for ProgramError {
    fn from(e: CustomError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
