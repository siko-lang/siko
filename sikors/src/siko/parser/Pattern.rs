use crate::siko::syntax::Pattern::Pattern;

use super::{Parser::Parser, Token::TokenKind};

pub trait PatternParser {
    fn parsePattern(&mut self) -> Pattern;
}

impl PatternParser for Parser {
    fn parsePattern(&mut self) -> Pattern {
        match self.peek() {
            TokenKind::VarIdentifier => {
                let name = self.parseVarIdentifier();
                Pattern::Bind(name)
            }
            kind => self.reportError2("<pattern>", kind),
        }
    }
}
