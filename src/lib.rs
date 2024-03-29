use rppal::gpio::Gpio;
use std::{thread, time};

pub mod errors;
use errors::PSXError;

/// Data GPIO pin
const DAT_GPIO: u8 = 23;
/// Command GPIO pin
const CMD_GPIO: u8 = 24;
/// Chip Select GPIO pin
const SEL_GPIO: u8 = 17;
/// Clock GPIO pin
const CLK_GPIO: u8 = 27;
/// Acknowledge GPIO pin
const ACK_GPIO: u8 = 22;

/// Status command sequence
const STATUS_CMD: [u8; 2] = [0x81, 0x53];
/// Read command sequence
const READ_CMD: [u8; 2] = [0x81, 0x52];
/// Write command sequence
const WRITE_CMD: [u8; 2] = [0x81, 0x57];

/// Receive Command Acknowledge
const COMMAND_ACK: [u8; 2] = [0x5c, 0x5d];

/// Length of frame for checksum calculation
const CHKSUM_FRAME_LEN: usize = 130;
/// Checksum location offset
const CHKSUM_OFS: usize = 130;

/// Command 'G'ood marker
const FRAME_STATUS_GOOD: u8 = 0x47;
/// Frame status offset
const FRAME_STATUS_GOOD_OFS: usize = 131;

/// Available commands for PS1 memory cards
#[derive(Debug, PartialEq)]
enum Command<'a> {
    Status,
    Read(u16),
    Write(u16, &'a [u8]),
}

/// Calculate the Command checksum.
pub fn calc_checksum(d: &[u8]) -> u8 {
    let mut c = 0;
    for i in d.iter() {
        c ^= *i;
    }

    c
}

/// Read all frames from the memory card
pub fn read_all_frames() -> Result<Vec<u8>, PSXError> {
    let mut data = Vec::<u8>::new();

    // Read frames 0 through 1023
    for i in 0..0x400 {
        data.append(&mut read_frame(i)?);
    }

    Ok(data)
}

/// Read a certain number of frames
pub fn read_at(offset: u16, length: u16) -> Result<Vec<u8>, PSXError> {
    let mut output = Vec::<u8>::new();
    for i in 0..length {
        output.append(&mut read_frame(offset + i as u16)?);
    }

    Ok(output)
}

/// Read a specific frame
pub fn read_frame(frame: u16) -> Result<Vec<u8>, PSXError> {
    let mut retry = 3;

    loop {
        // Try up to 3 times to read a frame, then give up
        if retry == 0 {
            println!("Err: retry limit reached!");
            return Err(PSXError::Read);
        } else {
            retry -= 1;
        }

        // Execute a Read command
        let mut data = cmd_raw_frame(Command::Read(frame))?;
        if data.len() <= 128 {
            println!("Err: len is too short: {}", data.len());
            continue;
        }

        // Remvoe the garbage byte
        data.remove(0);

        // Find the beginning of the data
        let ofs = match find_haystack_end(&COMMAND_ACK, &data) {
            Some(i) => i,
            None => {
                println!("Err: Couldnt find the response start");
                continue;
            }
        };

        // Calculate checksum
        let frame = &data[ofs..ofs + CHKSUM_FRAME_LEN];
        let checksum = data[ofs + CHKSUM_OFS];
        let calc = calc_checksum(frame);
        if checksum != calc {
            println!("Err: calc: {} expected: {}", calc, checksum);
            continue;
        }

        // Verify we received a good 'G' status trailer
        if data[ofs + FRAME_STATUS_GOOD_OFS] != FRAME_STATUS_GOOD {
            println!(
                "Err: trailer: {} expected: {}",
                data[ofs + FRAME_STATUS_GOOD_OFS],
                FRAME_STATUS_GOOD
            );
            continue;
        }

        // Remove frame number
        let data = frame[2..].to_vec();

        return Ok(data);
    }
}

/// Read the status from the memory card. Note: This is only supported by official Sony memory cards
pub fn get_status() -> Result<Vec<u8>, PSXError> {
    cmd_raw_frame(Command::Status)
}

/// Write a certain number of frames
pub fn write_at(offset: u16, length: u16, data: Vec<u8>) -> Result<Vec<u8>, PSXError> {
    if data.len() % 128 != 0 || data.len() / 128 != length.into() {
        return Err(PSXError::WriteLen);
    }

    for (i, d) in data.chunks(128).enumerate() {
        write_frame(offset + i as u16, d.to_vec())?;
    }

    // Return empty vec
    Ok(Vec::<u8>::new())
}

/// Write a specific frame
pub fn write_frame(frame: u16, data: Vec<u8>) -> Result<(), PSXError> {
    // Execute a Write command
    let _ = cmd_raw_frame(Command::Write(frame, &data))?;

    Ok(())
}

/// Seek to the end of a needle in the frame
fn find_haystack_end(needle: &[u8], data: &[u8]) -> Option<usize> {
    let mut idx = 0;
    for (i, d) in data.iter().enumerate() {
        if needle[idx] == *d {
            idx += 1;
            if idx >= needle.len() {
                return Some(i + 1);
            }
        } else {
            idx = 0;
        }
    }

    None
}

/// Generate a command buffer from the supplied Command
fn cmd_raw_frame(com: Command) -> Result<Vec<u8>, PSXError> {
    // TODO: Fix this length
    let mut command = vec![0u8; 256];

    // Insert the proper command sequence
    match com {
        Command::Status => command[..2].copy_from_slice(&STATUS_CMD),
        Command::Read(frame) => {
            command[..2].copy_from_slice(&READ_CMD);
            command[4..6].copy_from_slice(&frame.to_be_bytes());
        }
        Command::Write(frame, data) => {
            command[..2].copy_from_slice(&WRITE_CMD);
            command[4..6].copy_from_slice(&frame.to_be_bytes());
            if data.len() != 128 {
                return Err(PSXError::WriteLen);
            }
            command[6..134].copy_from_slice(data);
            command[134] = calc_checksum(&command[4..CHKSUM_FRAME_LEN]);
        }
    };

    // Trigger the Chip Select high then low to select the card
    let mut sel = Gpio::new()?.get(SEL_GPIO)?.into_output();
    sel.set_high();
    thread::sleep(time::Duration::from_millis(20));
    sel.set_low();

    // Send the command buffer
    let status = send_receive(&command)?;

    Ok(status)
}

/// Send and receive many bytes of data, LSB first
pub fn send_receive(input: &[u8]) -> Result<Vec<u8>, PSXError> {
    let mut clk = Gpio::new()?.get(CLK_GPIO)?.into_output();
    let mut cmd = Gpio::new()?.get(CMD_GPIO)?.into_output();
    let dat = Gpio::new()?.get(DAT_GPIO)?.into_input_pullup();
    let ack = Gpio::new()?.get(ACK_GPIO)?.into_input();

    let mut output = Vec::<u8>::new();

    'transmit: for transmit in input {
        // Byte for storing response data from DAT
        let mut rx: u8 = 0;

        for i in 0..8 {
            // Write data to the card when the clock is low
            thread::sleep(time::Duration::from_nanos(2000));
            clk.set_low();
            if (transmit >> i) & 0x01 == 0x01 {
                cmd.set_high();
            } else {
                cmd.set_low();
            }

            // Read data from the card when the clock is high
            thread::sleep(time::Duration::from_nanos(2000));
            clk.set_high();
            let out = (dat.read() as u8) & 0x01;
            rx |= out << i;
        }

        // Wait for ACK, fail if timeout is triggered first
        let timeout = time::Instant::now();
        'timeout: loop {
            if ack.is_low() {
                output.push(rx);
                // Exit the ACK loop and transmit next byte
                break 'timeout;
            }

            if timeout.elapsed() > time::Duration::from_micros(1500) {
                // Exit the transmit loop and return any data we have already received
                break 'transmit;
            }
        }
    }

    Ok(output)
}
