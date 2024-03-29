use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
use psxemcee::{errors::PSXError, get_status, read_all_frames, read_at, write_at};

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

    // Open file for reading or writing, depending on command
    let mut file: File = match opt.cmd {
        Cmd::ReadAll | Cmd::ReadFrame(_) | Cmd::ReadBlock(_) | Cmd::Status => {
            File::create(opt.file)?
        }
        Cmd::WriteAll | Cmd::WriteFrame(_) | Cmd::WriteBlock(_) => File::open(opt.file)?,
    };

    // Execute the proper command
    let out = match opt.cmd {
        Cmd::ReadAll => read_all_frames(),
        Cmd::ReadFrame(opt) => read_at(frame_ofs(opt.offset)?, 1),
        Cmd::ReadBlock(opt) => read_at(block_ofs(opt.offset)?, 64),
        Cmd::Status => get_status(),
        Cmd::WriteFrame(opt) => {
            let mut buf = [0u8; 128];
            file.read_exact(&mut buf)?;
            write_at(frame_ofs(opt.offset)?, 1, buf.to_vec())
        }
        Cmd::WriteBlock(opt) => {
            let mut buf = [0u8; 128 * 64];
            file.read_exact(&mut buf)?;
            write_at(block_ofs(opt.offset)?, 64, buf.to_vec())
        }
        c => {
            println!("Command {:?} not implemented", c);
            return Ok(());
        }
    };

    // If len = 0 and no error, write was successful
    if let Ok(ref o) = out {
        if o.is_empty() {
            println!("Memory card write complete!");
            return Ok(());
        }
    };

    // Write output or print error
    match out {
        Ok(o) => {
            println!("Memory card read complete!");
            file.write_all(&o)?;
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
