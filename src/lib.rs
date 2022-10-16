use std::error::Error;

use rppal::gpio::Gpio;

fn toggle(input: u8) -> Result<(), Box<dyn Error>>{

    let mut pin = Gpio::new()?.get(input)?.into_output();
    pin.toggle();

    Ok(())
}
