use crate::siko::{location::Location::FileId, parser::Lexer::*};

pub struct Parser {}

impl Parser {
    pub fn new() -> Parser {
        Parser {}
    }

    pub fn parse(&mut self, file_name: &str) {
        let content = std::fs::read_to_string(file_name).unwrap();
        let mut lexer = Lexer::new(content, FileId::new(0));
        let (tokens, errors) = lexer.lex();
        println!("Errors {:?}", errors);
    }
}
