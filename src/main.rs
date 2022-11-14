use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use psxmcrw::{errors::PSXError, read_all_frames, read_frame};

#[derive(Clone, Debug, Parser, PartialEq, Eq)]
pub enum Cmd {
    /// Read entire memory card
    ReadAll,

    /// Read a specific frame
    ReadFrame(ReadWriteOpt),

    /// Read a specific block
    ReadBlock(ReadWriteOpt),

    /// Get memory card status
    Status,

    /// Write file to memory card
    WriteAll,

    /// Write frame to memory card
    WriteFrame(ReadWriteOpt),

    /// Write block to memory card
    WriteBlock(ReadWriteOpt),
}

#[derive(Clone, Debug, Parser, PartialEq, Eq)]
pub struct Opt {
    /// Command
    #[command(subcommand)]
    cmd: Cmd,

    /// Filepath to save memory card data to, or write to the memory card from
    #[arg(short, long)]
    file: PathBuf,
}

#[derive(Clone, Debug, Parser, PartialEq, Eq)]
pub struct ReadWriteOpt {
    /// Offset to read or write at
    #[arg(short, long, default_value = "0")]
    offset: u16,
}

fn main() -> Result<(), PSXError> {
    // Process arguments
    let opt = Opt::parse();

    let mut output: File = File::create(opt.file)?;

    let out = match opt.cmd {
        Cmd::ReadAll => read_all_frames(),
        Cmd::ReadFrame(opt) => read_at(frame_ofs(opt.offset)?, 1),
        Cmd::ReadBlock(opt) => read_at(block_ofs(opt.offset)?, 64),
        c => {
            println!("Command {:?} not implemented", c);
            return Ok(());
        }
    };

    match out {
        Ok(o) => {
            println!("Memory card read complete!");
            output.write_all(&o)?;
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return Ok(());
        }
    }

    Ok(())
}

fn frame_ofs(offset: u16) -> Result<u16, PSXError> {
    if offset > 0x3ff {
        println!("Error: Frame offset must be between 0 and 1023 (0x3ff)");
        return Err(PSXError::FrameOfs);
    }
    Ok(offset)
}

fn block_ofs(offset: u16) -> Result<u16, PSXError> {
    if offset > 15 {
        println!("Error: Block offset must be between 0 and 15");
        return Err(PSXError::BlockOfs);
    }
    Ok(offset * 64)
}

fn read_at(offset: u16, length: u16) -> Result<Vec<u8>, PSXError> {
    let mut output = Vec::<u8>::new();
    for i in 0..length {
        println!("Read frame: {}", offset + i as u16);
        output.append(&mut read_frame(offset + i as u16)?);
    }

    Ok(output)
}
