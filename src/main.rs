use std::error::Error;

use psxmcrw::get_mc_status;

fn main() -> Result<(), Box<dyn Error>> {
    get_mc_status()
}
