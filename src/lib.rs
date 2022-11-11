use rppal::gpio::Gpio;
use std::error::Error;
use std::{thread, time};

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

/// Header for status response
const STATUS_RSP_START: [u8; 4] = [0x5a, 0x5d, 0x5c, 0x5d];
/// Header for read response
const READ_RSP_START: [u8; 2] = [0x5c, 0x5d];
/// Header for write response
const WRITE_RSP_START: [u8; 2] = [0x5a, 0x5d];

/// Length of frame for checksum calculation
const CHKSUM_FRAME_LEN: usize = 130;
/// Checksum location offset
const CHKSUM_OFS: usize = 130;

/// Command 'G'ood marker
const FRAME_STATUS: u8 = 0x47;
/// Frame status offset
const FRAME_STATUS_OFS: usize = 131;

/// Available commands for PS1 memory cards
#[derive(Debug, PartialEq)]
enum Command {
    Status,
    Read,
    Write,
}

// TODO Implement errors

fn cleanup_data(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::<u8>::new();

    for (i, d) in data.iter().enumerate() {
        if i + 1 >= data.len() {
            break;
        }
        out.push(d << 1 | (data[i + 1] & 0x80u8) >> 7);
    }
    out
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
pub fn read_all_frames() -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut data = Vec::<Vec<u8>>::new();

    // Read frames 0 through 1023
    for i in 0..0x400 {
        data.push(read_frame(i)?);
    }

    Ok(data)
}

/// Read a specific frame
pub fn read_frame(frame: u16) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut retry = 3;

    for i in 0..100 {
        //let _ = send_receive(0xff);
    }

    loop {
        // Try up to 3 times to read a frame, then give up
        if retry == 0 {
            // TODO: Return an error value
            println!("Err: retry limit reached!");
            return Ok(Vec::new());
        } else {
            retry -= 1;
        }

        // Execute a Read command
        let mut data = cmd_raw_frame(Command::Read, frame)?;
        if data.len() <= 128 {
            println!("Err: len is too short: {}", data.len());
            continue;
        }

        // Remvoe the garbage byte
        data.remove(0);

        // Find the beginning of the data
        let ofs = match find_haystack_end(&READ_RSP_START, &data) {
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
        if data[ofs + FRAME_STATUS_OFS] != FRAME_STATUS {
            println!(
                "Err: trailer: {} expected: {}",
                data[ofs + FRAME_STATUS_OFS],
                FRAME_STATUS
            );
            continue;
        }

        // Remove frame number
        let data = frame[2..].to_vec();

        return Ok(data);
    }
}

// Seek to the end of a needle in the frame
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

// TODO This needs an Option<&[u8]> for Command::Write frame data
fn cmd_raw_frame(com: Command, frame: u16) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut status = Vec::<u8>::new();
    // TODO: Fix this length
    let mut command = vec![0u8; 256];

    // Insert the proper command sequence
    match com {
        Command::Status => command[..2].copy_from_slice(&STATUS_CMD),
        Command::Read => command[..2].copy_from_slice(&READ_CMD),
        Command::Write => command[..2].copy_from_slice(&WRITE_CMD),
    };

    // If Read or Write, copy the frame address to the command
    if com != Command::Status {
        command[4..6].copy_from_slice(&frame.to_be_bytes());
    }

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
pub fn send_receive(input: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut clk = Gpio::new()?.get(CLK_GPIO)?.into_output();
    let mut cmd = Gpio::new()?.get(CMD_GPIO)?.into_output();
    let dat = Gpio::new()?.get(DAT_GPIO)?.into_input_pullup();
    let ack = Gpio::new()?.get(ACK_GPIO)?.into_input();

    let mut output = Vec::<u8>::new();

    thread::sleep(time::Duration::from_nanos(2000));
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
                break 'timeout;
            }

            if timeout.elapsed() > time::Duration::from_micros(1500) {
                break 'transmit;
            }
        }
    }

    Ok(output)
}
