use crate::siko::syntax::Data::{Class, Enum};

use super::{
    Parser::*,
    Token::{BracketKind, KeywordKind, TokenKind},
};

pub trait DataParser {
    fn parseClass(&mut self) -> Class;
    fn parseEnum(&mut self) -> Enum;
}

impl DataParser for Parser {
    fn parseClass(&mut self) -> Class {
        self.expect(TokenKind::Keyword(KeywordKind::Class));
        let name = self.parseTypeIdentifier();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Class { name }
    }

    fn parseEnum(&mut self) -> Enum {
        self.expect(TokenKind::Keyword(KeywordKind::Class));
        let name = self.parseTypeIdentifier();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Enum { name }
    }
}
