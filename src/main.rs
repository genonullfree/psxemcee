use std::error::Error;

use rppal::gpio::Gpio;

fn main() -> Result<(), Box<dyn Error>>{

    let mut pin = Gpio::new()?.get(4u8)?.into_output();
    pin.toggle();

    Ok(())
}
