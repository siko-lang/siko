use crate::siko::syntax::Pattern::{Pattern, SimplePattern};

use super::{
    Parser::Parser,
    Token::{BracketKind, KeywordKind, MiscKind, TokenKind},
};

pub trait PatternParser {
    fn buildPattern(&mut self, p: SimplePattern) -> Pattern;
    fn parsePattern(&mut self) -> Pattern;
}

impl PatternParser for Parser {
    fn buildPattern(&mut self, p: SimplePattern) -> Pattern {
        Pattern {
            pattern: p,
            location: self.popSpan(),
        }
    }

    fn parsePattern(&mut self) -> Pattern {
        self.pushSpan();
        match self.peek() {
            TokenKind::Keyword(KeywordKind::Mut) => {
                self.expect(TokenKind::Keyword(KeywordKind::Mut));
                let name = self.parseVarIdentifier();
                self.buildPattern(SimplePattern::Bind(name, true))
            }
            TokenKind::VarIdentifier => {
                let name = self.parseVarIdentifier();
                self.buildPattern(SimplePattern::Bind(name, false))
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
                self.buildPattern(SimplePattern::Named(name, args))
            }
            TokenKind::LeftBracket(BracketKind::Paren) => {
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
                self.buildPattern(SimplePattern::Tuple(args))
            }
            TokenKind::Misc(MiscKind::Wildcard) => {
                self.expect(TokenKind::Misc(MiscKind::Wildcard));
                self.buildPattern(SimplePattern::Wildcard)
            }
            kind => self.reportError2("<pattern>", kind),
        }
    }
}
