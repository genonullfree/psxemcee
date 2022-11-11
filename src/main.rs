use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Write;

use psxmcrw::read_frame;

fn main() -> Result<(), Box<dyn Error>> {
    let mut output: File = File::create("output.bin")?;

    for i in 0..=0x3 {
        let mut frame = read_frame(i)?;
        println!("Read frame: {}", i);
        output.write_all(&frame)?;
    }

    println!("\rMemory card read complete!");
    Ok(())
}
