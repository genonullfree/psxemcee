use rppal::gpio::Gpio;
use std::error::Error;
use std::{thread, time};

const DAT_GPIO: u8 = 23;
const CMD_GPIO: u8 = 24;
const SEL_GPIO: u8 = 17;
const CLK_GPIO: u8 = 27;
const ACK_GPIO: u8 = 22;

const STATUS_CMD: [u8; 2] = [0x81, 0x53];
const READ_CMD: [u8; 2] = [0x81, 0x52];
const WRITE_CMD: [u8; 2] = [0x81, 0x57];

const STATUS_CMD_START: [u8; 4] = [0x5a, 0x5d, 0x5c, 0x5d];
const READ_CMD_START: [u8; 2] = [0x5c, 0x5d];
const WRITE_CMD_START: [u8; 2] = [0x5a, 0x5d];

const CMD_TRAILER: u8 = 0x47;

const CHKSUM_FRAME_LEN: usize = 130;
const CHKSUM_OFS: usize = 130;
const FRAME_STATUS_OFS: usize = 131;

#[derive(Debug, PartialEq)]
enum Command {
    Status,
    Read,
    Write,
}

// TODO Implement errors

/// Calculate the Command checksum.
pub fn calc_checksum(d: &[u8]) -> u8 {
    let mut c = 0;
    for i in d.iter() {
        c ^= *i;
    }
    c
}

pub fn read_all_frames() -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut data = Vec::<Vec<u8>>::new();

    for i in 0..0x400 {
        data.push(read_frame(i)?);
    }

    Ok(data)
}

pub fn read_frame(frame: u16) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut retry = 3;

    loop {
        // Try up to 3 times to read a frame, then give up
        if retry == 0 {
            // TODO: Return an error value
            return Ok(Vec::new());
        } else {
            retry -= 1;
        }

        // Execute a Read command
        let mut data = cmd_raw_frame(Command::Read, frame)?;

        // Remvoe the garbage byte
        data.remove(0);

        // Find the beginning of the data
        let ofs = match find_haystack_end(&READ_CMD_START, &data) {
            Some(i) => i,
            None => continue,
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
        if data[ofs + FRAME_STATUS_OFS] != CMD_TRAILER {
            println!(
                "Err: trailer: {} expected: {}",
                data[ofs + 130],
                CMD_TRAILER
            );
            continue;
        }

        // Remove frame number
        let data = frame[2..].to_vec();

        return Ok(data);
    }
}

fn find_haystack_end(pattern: &[u8], data: &[u8]) -> Option<usize> {
    let mut idx = 0;
    for (i, d) in data.iter().enumerate() {
        if pattern[idx] == *d {
            idx += 1;
            if idx >= pattern.len() {
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
    let mut command = vec![0u8; 256];
    match com {
        Command::Status => command[..2].copy_from_slice(&STATUS_CMD),
        Command::Read => command[..2].copy_from_slice(&READ_CMD),
        Command::Write => command[..2].copy_from_slice(&WRITE_CMD),
    };
    if com != Command::Status {
        command[4..6].copy_from_slice(&frame.to_be_bytes());
    }

    let mut sel = Gpio::new()?.get(SEL_GPIO)?.into_output();

    sel.set_high();
    thread::sleep(time::Duration::from_millis(20));
    sel.set_low();
    let mut count = 0;
    for c in command {
        match send_receive(c)? {
            Some(s) => {
                status.push(s);
                count = 0;
            }
            None => count += 1,
        };
        if count > 1 {
            break;
        }
    }

    Ok(status)
}

pub fn send_receive(transmit: u8) -> Result<Option<u8>, Box<dyn Error>> {
    let mut clk = Gpio::new()?.get(CLK_GPIO)?.into_output();
    let mut tx = Gpio::new()?.get(CMD_GPIO)?.into_output();
    let dat = Gpio::new()?.get(DAT_GPIO)?.into_input_pullup();
    let ack = Gpio::new()?.get(ACK_GPIO)?.into_input();
    // LSB
    let mut rx: u8 = 0;
    clk.set_high();
    thread::sleep(time::Duration::from_nanos(2000));
    for i in 0..8 {
        thread::sleep(time::Duration::from_nanos(2000));
        clk.set_low();
        if (transmit >> i) & 0x01 == 0x01 {
            tx.set_high();
        } else {
            tx.set_low();
        }
        thread::sleep(time::Duration::from_nanos(2000));
        clk.set_high();
        let out = dat.read() as u8;
        rx |= out << i;
    }

    tx.set_low();

    let timeout = time::Instant::now();
    loop {
        if ack.is_low() {
            break;
        }

        if timeout.elapsed() > time::Duration::from_micros(1500) {
            return Ok(None);
        }
    }

    Ok(Some(rx))
}
