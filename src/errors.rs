use std::error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PSXError {
    #[error("Error: {0}")]
    Error(#[from] Box<dyn error::Error>),

    #[error("IO::Error: {0}")]
    IO(#[from] std::io::Error),

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

    #[error("Error: Invalid data length to write (need 128bytes)")]
    WriteLen,

    #[error("Error: Short write")]
    WriteShort,

    #[error("Error: Frame offset must be between 0 and 1023 (0x3ff)")]
    FrameOfs,

    #[error("Error: Block offset must be between 0 and 15")]
    BlockOfs,
}
