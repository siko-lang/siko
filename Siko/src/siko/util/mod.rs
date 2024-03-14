use std::process::exit;

pub fn error(msg: String) -> ! {
    println!("{}", msg);
    exit(1);
}

pub mod SCC;
