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
    let mut command = vec![0u8; 256];
    command[0] = 0x81;
    command[1] = 0x52;
    //let command = vec![0x81, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let mut pin = Gpio::new()?.get(SEL_GPIO)?.into_output();

    pin.set_high();
    thread::sleep(time::Duration::from_millis(20));
    pin.set_low();
    for c in command {
        status.push(send_receive(c)?);
    }
    //pin.set_low();

    // debug
    println!("{:02x?}", status);
    Ok(())
}

pub fn send_receive(transmit: u8) -> Result<u8, Box<dyn Error>> {
    let mut clk = Gpio::new()?.get(CLK_GPIO)?.into_output();
    let mut tx = Gpio::new()?.get(CMD_GPIO)?.into_output();
    let dat = Gpio::new()?.get(DAT_GPIO)?.into_input();
    // LSB
    let mut rx: u8 = 0;
    clk.set_high();
    for i in 0..8 {
        if (transmit >> i) & 0x01 == 0x01 {
            tx.set_high();
        } else {
            tx.set_low();
        }
        // Runs at 250Khz (*.5)
        thread::sleep(time::Duration::from_nanos(2000));
        clk.set_low();
        thread::sleep(time::Duration::from_nanos(2000));
        clk.set_high();
        let out = dat.read() as u8;
        rx |= out << i;
    }

    wait_for_ack()?;
    //println!();

    Ok(rx)
}

fn write_cmd_bit(tx: u8) -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(CMD_GPIO)?.into_output();
    if tx > 0 {
        pin.set_high();
        // debug
        //print!("1");
    } else {
        pin.set_low();
        // debug
        //print!("0");
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
    let mut count = 0;
    loop {
        if pin.is_low() {
            break;
        }

        // TODO fail with an actual error
        if timeout.elapsed() > time::Duration::from_micros(150) {
            //println!("Timeout!");
            return Ok(())
        }
    }

    loop {
        if pin.is_high() {
            break;
        }

        if timeout.elapsed() > time::Duration::from_micros(150) {
            println!("Timeout2!");
            return Ok(())
        }
    }

    Ok(())
}
