use std::error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PSXError {
    #[error("Error: {0}")]
    EError(#[from] Box<dyn error::Error>),

    #[error("Rpal::gpio::Error: {0}")]
    RpalError(#[from] rppal::gpio::Error),

    #[error("Error reading from MC")]
    ReadError,

    #[error("Error in checksum")]
    ChecksumError,

    #[error("Error: Bad status")]
    StatusError,

    #[error("Error writing to MC")]
    WriteError,
}
