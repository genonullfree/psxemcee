use std::error::Error;
use std::fs::File;
use std::io::Write;

use psxmcrw::read_frame;

fn main() -> Result<(), Box<dyn Error>> {
    let mut output: File = File::create("output.bin")?;

    for i in 0..=0x3 {
        let mut frame = read_frame(i)?;
        if frame.len() != 128 {
            println!("rx invalid length: {}", frame.len());
            frame = frame[..128].to_vec();
        }
        println!("[{}] {:02x?}", i, frame);
        output.write(&frame)?;
    }

    Ok(())
}
