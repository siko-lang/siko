use crate::siko::syntax::Function::{Function, Parameter};

use super::{
    Expr::ExprParser,
    Parser::*,
    Token::{ArrowKind, BracketKind, KeywordKind, MiscKind, TokenKind},
    Type::TypeParser,
};

pub trait FunctionParser {
    fn parseFunction(&mut self) -> Function;
}

impl FunctionParser for Parser {
    fn parseFunction(&mut self) -> Function {
        self.expect(TokenKind::Keyword(KeywordKind::Fn));
        let name = self.parseVarIdentifier();
        let typeParams = if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
            Some(self.parseTypeParameterDeclaration())
        } else {
            None
        };
        self.expect(TokenKind::LeftBracket(BracketKind::Paren));
        let mut params = Vec::new();
        while !self.check(TokenKind::RightBracket(BracketKind::Paren)) {
            let mutable = if self.check(TokenKind::Keyword(KeywordKind::Mut)) {
                self.expect(TokenKind::Keyword(KeywordKind::Mut));
                true
            } else {
                false
            };
            let param = if self.check(TokenKind::Keyword(KeywordKind::ValueSelf)) {
                self.expect(TokenKind::Keyword(KeywordKind::ValueSelf));
                Parameter::SelfParam(mutable)
            } else {
                let name = self.parseVarIdentifier();
                self.expect(TokenKind::Misc(MiscKind::Colon));
                let ty = self.parseType();
                Parameter::Named(name, ty, mutable)
            };
            params.push(param);
            if self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                break;
            }
            self.expect(TokenKind::Misc(MiscKind::Comma));
        }
        self.expect(TokenKind::RightBracket(BracketKind::Paren));
        let result = if self.check(TokenKind::Arrow(ArrowKind::Right)) {
            self.expect(TokenKind::Arrow(ArrowKind::Right));
            Some(self.parseType())
        } else {
            None
        };
        let body = if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            Some(self.parseBlock())
        } else {
            None
        };
        Function {
            name,
            typeParams,
            params,
            result,
            body: body,
        }
    }
}
