use psxmcrw::toggle;

fn main() -> Result<(), Box<dyn Error>>{

    toggle(4u8)?;

    Ok(())
}
