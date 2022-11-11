use std::error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PSXError {
    #[error("Error: {0}")]
    IO(#[from] Box<dyn error::Error>),

    #[error("Rpal::gpio::Error: {0}")]
    Rppal(#[from] rppal::gpio::Error),

    #[error("Error reading from MC")]
    Read,

    #[error("Error in checksum")]
    Checksum,

    #[error("Error: Bad status")]
    Status,

    #[error("Error writing to MC")]
    Write,
}
