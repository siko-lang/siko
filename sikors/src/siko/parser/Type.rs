use crate::siko::{syntax::Type::*, util::error};

use super::{
    Parser::*,
    Token::{ArrowKind, BracketKind, KeywordKind, MiscKind, TokenKind},
};

pub trait TypeParser {
    fn parseType(&mut self) -> Type;
}

impl TypeParser for Parser {
    fn parseType(&mut self) -> Type {
        match self.peek() {
            Some(TokenKind::TypeIdentifier) => {
                let name = self.parseTypeIdentifier();
                let mut args = Vec::new();
                if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
                    self.expect(TokenKind::LeftBracket(BracketKind::Square));
                    while !self.check(TokenKind::RightBracket(BracketKind::Square)) {
                        let arg = self.parseType();
                        args.push(arg);
                        if self.check(TokenKind::RightBracket(BracketKind::Square)) {
                            break;
                        } else {
                            self.expect(TokenKind::Misc(MiscKind::Comma));
                        }
                    }
                    self.expect(TokenKind::RightBracket(BracketKind::Square));
                }
                Type::Named(name, args)
            }
            Some(TokenKind::LeftBracket(BracketKind::Paren)) => {
                let mut items = Vec::new();
                self.expect(TokenKind::LeftBracket(BracketKind::Paren));
                while !self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                    let item = self.parseType();
                    items.push(item);
                    if self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                        break;
                    } else {
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                    }
                }
                self.expect(TokenKind::RightBracket(BracketKind::Paren));
                Type::Tuple(items)
            }
            Some(TokenKind::Keyword(KeywordKind::Fn)) => {
                self.expect(TokenKind::Keyword(KeywordKind::Fn));
                let mut args = Vec::new();
                self.expect(TokenKind::LeftBracket(BracketKind::Paren));
                while !self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                    let arg = self.parseType();
                    args.push(arg);
                    if self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                        break;
                    } else {
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                    }
                }
                self.expect(TokenKind::RightBracket(BracketKind::Paren));
                self.expect(TokenKind::Arrow(ArrowKind::Right));
                let result = self.parseType();
                Type::Function(args, Box::new(result))
            }
            Some(kind) => self.reportError2("<type>", kind),
            None => error(format!("EOF")),
        }
    }
}
