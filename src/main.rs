use std::error::Error;

use rppal::gpio::Gpio;

fn main() -> Result<(), Box<dyn Error>>{

    let mut pin = Gpio::new()?.get(23u8)?.into_output();
    pin.set_high();

    Ok(())
}
