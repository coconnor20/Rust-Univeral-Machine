use std::env;
use rum::rumload;
use rum::emulation_unit;


/// Main Function
fn main() {
    let input = env::args().nth(1);
    let instructions = rumload::load(input.as_deref());
    emulation_unit::run_um(instructions);
}
