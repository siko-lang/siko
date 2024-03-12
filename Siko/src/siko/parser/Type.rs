use crate::siko::syntax::Type::*;

use super::{
    Parser::*,
    Token::{ArrowKind, BracketKind, KeywordKind, MiscKind, OperatorKind, TokenKind},
};

pub trait TypeParser {
    fn parseType(&mut self) -> Type;
    fn parseTypeParameterDeclaration(&mut self) -> TypeParameterDeclaration;
}

impl TypeParser for Parser {
    fn parseType(&mut self) -> Type {
        match self.peek() {
            TokenKind::TypeIdentifier => {
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
            TokenKind::LeftBracket(BracketKind::Paren) => {
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
            TokenKind::Keyword(KeywordKind::Fn) => {
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
            TokenKind::Keyword(KeywordKind::TypeSelf) => {
                self.expect(TokenKind::Keyword(KeywordKind::TypeSelf));
                Type::SelfType
            }
            kind => self.reportError2("<type>", kind),
        }
    }

    fn parseTypeParameterDeclaration(&mut self) -> TypeParameterDeclaration {
        let mut params = Vec::new();
        let mut constraints = Vec::new();
        let mut afterParam = false;
        self.expect(TokenKind::LeftBracket(BracketKind::Square));
        while !self.check(TokenKind::RightBracket(BracketKind::Square)) {
            if afterParam {
                let constraint = self.parseType();
                constraints.push(constraint);
                if self.check(TokenKind::Misc(MiscKind::Comma)) {
                    self.expect(TokenKind::Misc(MiscKind::Comma));
                    continue;
                } else {
                    break;
                }
            } else {
                let param = self.parseTypeIdentifier();
                let mut deps = Vec::new();
                if self.check(TokenKind::Misc(MiscKind::Colon)) {
                    self.expect(TokenKind::Misc(MiscKind::Colon));
                    loop {
                        let dep = self.parseType();
                        deps.push(dep);
                        if self.check(TokenKind::Op(OperatorKind::Add)) {
                            self.expect(TokenKind::Op(OperatorKind::Add));
                        } else {
                            break;
                        }
                    }
                }
                params.push(TypeParameter {
                    name: param,
                    constraints: deps,
                });
                if self.check(TokenKind::Misc(MiscKind::Comma)) {
                    self.expect(TokenKind::Misc(MiscKind::Comma));
                    continue;
                }
                if self.check(TokenKind::Arrow(ArrowKind::DoubleRight)) {
                    self.expect(TokenKind::Arrow(ArrowKind::DoubleRight));
                    afterParam = true;
                }
            }
        }
        self.expect(TokenKind::RightBracket(BracketKind::Square));
        TypeParameterDeclaration {
            params,
            constraints,
        }
    }
}
