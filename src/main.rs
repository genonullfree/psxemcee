use std::error::Error;

use psxmcrw::read_frame;

fn main() -> Result<(), Box<dyn Error>> {
    let mut data = Vec::<Vec<u8>>::new();
    data.push(read_frame(0)?);
    data.push(read_frame(1)?);
    data.push(read_frame(2)?);

    for i in data {
        println!("{:02x?}", i);
    }

    Ok(())
}
