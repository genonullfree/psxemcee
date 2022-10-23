use rppal::gpio::Gpio;
use std::error::Error;
use std::{thread, time};

const DAT_GPIO: u8 = 23;
const CMD_GPIO: u8 = 24;
const SEL_GPIO: u8 = 17;
const CLK_GPIO: u8 = 27;
const ACK_GPIO: u8 = 22;

/*
const CMD_GPIO: u8 = 14;
const DAT_GPIO: u8 = 15;
const SEL_GPIO: u8 = 2;
const CLK_GPIO: u8 = 3;
const ACK_GPIO: u8 = 4;
*/

const STATUS_CMD: [u8; 2] = [0x81, 0x53];
const READ_CMD: [u8; 2] = [0x81, 0x52];
const WRITE_CMD: [u8; 2] = [0x81, 0x57];

const STATUS_CMD_START: [u8; 4] = [0x5a, 0x5d, 0x5c, 0x5d];
const READ_CMD_START: [u8; 2] = [0x5c, 0x5d];
const WRITE_CMD_START: [u8; 2] = [0x5a, 0x5d];

// TODO this is not a good method...
const NON_SONY_MC_TRAILER: [u8; 5] = [0x5c; 5];

#[derive(Debug, PartialEq)]
enum Command {
    Status,
    Read,
    Write,
}

pub fn read_frame(frame: u16) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut data = cmd_raw_frame(Command::Read, frame)?;

    data.remove(0);

    let idx = match find_data_index(&READ_CMD_START, &data) {
        Some(i) => i,
        None => return Ok(data), // TODO This is an error case :(
    };

    // Remove header info
    let data = data[idx + READ_CMD_START.len() + 3..].to_vec();

    // Remove trailer info
    let idx = match find_data_index(&NON_SONY_MC_TRAILER, &data) {
        Some(i) => i,
        None => return Ok(data), // TODO This is an error case :(
    };
    let data = data[..idx - 1].to_vec();

    Ok(data)
}

fn find_data_index(pattern: &[u8], data: &[u8]) -> Option<usize> {
    let mut idx = 0;
    for (i, d) in data.iter().enumerate() {
        if pattern[idx] == *d {
            idx += 1;
            if idx >= pattern.len() {
                return Some(i - idx);
            }
        } else {
            idx = 0;
        }
    }
    return None;
}

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
    let dat = Gpio::new()?.get(DAT_GPIO)?.into_input();
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
