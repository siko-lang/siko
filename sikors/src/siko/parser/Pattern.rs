use crate::siko::syntax::Pattern::Pattern;

use super::{
    Parser::Parser,
    Token::{BracketKind, MiscKind, TokenKind},
};

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
            TokenKind::TypeIdentifier => {
                let name = self.parseTypeIdentifier();
                let mut args = Vec::new();
                if self.check(TokenKind::LeftBracket(BracketKind::Paren)) {
                    self.expect(TokenKind::LeftBracket(BracketKind::Paren));
                    while !self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                        let arg = self.parsePattern();
                        args.push(arg);
                        if self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                            break;
                        }
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                    }
                    self.expect(TokenKind::RightBracket(BracketKind::Paren));
                }
                Pattern::Named(name, args)
            }
            kind => self.reportError2("<pattern>", kind),
        }
    }
}
