#![allow(non_snake_case)]
#![allow(dead_code)]

mod Compiler;
mod PackageFinder;
mod siko;

fn main() {
    let mut c = Compiler::Compiler::new();
    c.run();
}
