

mod mir;
mod mir_loader;

use mir::*;
use mir_loader::*;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    for arg in args {
        println!("Parsing json from {}", arg);
        match load_mir(arg) {
            Ok(mir_program) => {
                println!("MIR loaded");
            }
            Err(e) => {
                println!("Failed to parse {:?}", e);
            }
        };
    }
}
