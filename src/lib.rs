use rppal::gpio::Gpio;
use std::error::Error;
use std::{thread, time};

// TODO use GPIO struct

const CMD_GPIO: u8 = 14;
const DAT_GPIO: u8 = 15;
const SEL_GPIO: u8 = 2;
const CLK_GPIO: u8 = 3;
const ACK_GPIO: u8 = 4;

pub enum Clk {
    Up,
    Down,
}

pub fn toggle(input: u8) -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(input)?.into_output();
    pin.toggle();

    Ok(())
}

pub fn get_mc_status() -> Result<(), Box<dyn Error>> {
    let mut status = Vec::<u8>::new();
    let command = vec![0x81, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let mut pin = Gpio::new()?.get(SEL_GPIO)?.into_output();

    pin.set_low();
    for c in command {
        status.push(send_receive(c)?);
    }
    pin.set_high();

    println!("{:02x?}", status);
    Ok(())
}

pub fn send_receive(tx: u8) -> Result<u8, Box<dyn Error>> {
    // LSB
    let mut rx: u8 = 0;
    for i in 0..8 {
        write_cmd_bit((tx >> i) & 0x01)?;
        thread::sleep(time::Duration::from_nanos(4000));
        clock(Clk::Down)?;
        // Runs at 250Khz
        thread::sleep(time::Duration::from_nanos(4000));
        clock(Clk::Up)?;
        let out = read_dat_bit()?;
        rx |= out << i;
    }

    wait_for_ack()?;

    Ok(rx)
}

fn write_cmd_bit(tx: u8) -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(CMD_GPIO)?.into_output();
    if tx > 0 {
        pin.set_high();
    } else {
        pin.set_low();
    }

    Ok(())
}

fn read_dat_bit() -> Result<u8, Box<dyn Error>> {
    let pin = Gpio::new()?.get(DAT_GPIO)?.into_input();
    Ok(pin.read() as u8)
}

fn clock(dir: Clk) -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(CLK_GPIO)?.into_output();
    match dir {
        Clk::Up => pin.set_high(),
        Clk::Down => pin.set_low(),
    };

    Ok(())
}

fn wait_for_ack() -> Result<(), Box<dyn Error>> {
    let pin = Gpio::new()?.get(ACK_GPIO)?.into_input();

    let timeout = time::Instant::now();
    loop {
        if pin.is_low() {
            break;
        }
        if timeout.elapsed() > time::Duration::from_secs(1) {
            println!("Timeout!");
            break;
        }
    }

    Ok(())
}
