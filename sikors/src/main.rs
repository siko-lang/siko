#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::parser::Parser::*;

use std::env::args;

fn main() {
    for arg in args().skip(1) {
        let mut parser = Parser::new();
        parser.parse(&arg);
    }
}
