use std::process::exit;

pub fn error(msg: String) -> ! {
    println!("{}", msg);
    exit(1);
}

pub mod Config;
pub mod DependencyProcessor;
pub mod Dot;
pub mod Instantiator;
pub mod Runner;
pub mod SCC;
