use crate::siko::{
    syntax::{
        Attributes::Attributes,
        Function::{Function, FunctionExternKind, Parameter, ResultKind},
        Type::Type,
    },
    util::error,
};

use super::{
    Expr::ExprParser,
    Parser::*,
    Token::{ArrowKind, BracketKind, KeywordKind, MiscKind, TokenKind},
    Type::TypeParser,
};

pub trait FunctionParser {
    fn parseFunction(&mut self, attributes: Attributes, public: bool, allowGenerator: bool) -> Function;
}

impl<'a> FunctionParser for Parser<'a> {
    fn parseFunction(&mut self, attributes: Attributes, public: bool, allowGenerator: bool) -> Function {
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
            let param = if self.check(TokenKind::Misc(MiscKind::Ampersand)) {
                self.expect(TokenKind::Misc(MiscKind::Ampersand));
                self.expect(TokenKind::Keyword(KeywordKind::ValueSelf));
                Parameter::RefSelfParam
            } else {
                let mutable = if self.check(TokenKind::Keyword(KeywordKind::Mut)) {
                    self.expect(TokenKind::Keyword(KeywordKind::Mut));
                    true
                } else {
                    false
                };
                if self.check(TokenKind::Keyword(KeywordKind::ValueSelf)) {
                    self.expect(TokenKind::Keyword(KeywordKind::ValueSelf));
                    if mutable {
                        Parameter::MutSelfParam
                    } else {
                        Parameter::SelfParam
                    }
                } else {
                    let name = self.parseVarIdentifier();
                    self.expect(TokenKind::Misc(MiscKind::Colon));
                    let ty = self.parseType();
                    Parameter::Named(name, ty, mutable)
                }
            };
            params.push(param);
            if self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                break;
            }
            self.expect(TokenKind::Misc(MiscKind::Comma));
        }
        self.expect(TokenKind::RightBracket(BracketKind::Paren));
        let result = if self.check(TokenKind::Misc(MiscKind::Colon)) && allowGenerator {
            self.expect(TokenKind::Misc(MiscKind::Colon));
            let yieldTy = self.parseType();
            self.expect(TokenKind::Arrow(ArrowKind::Right));
            let returnTy = self.parseType();
            ResultKind::Generator(yieldTy, returnTy)
        } else {
            if self.check(TokenKind::Arrow(ArrowKind::Right)) {
                self.expect(TokenKind::Arrow(ArrowKind::Right));
                ResultKind::SingleReturn(self.parseType())
            } else {
                ResultKind::SingleReturn(Type::Tuple(Vec::new()))
            }
        };
        let (externKind, body) = if self.check(TokenKind::Misc(MiscKind::Equal)) {
            self.expect(TokenKind::Misc(MiscKind::Equal));
            self.expect(TokenKind::Keyword(KeywordKind::Extern));
            if self.check(TokenKind::StringLiteral) {
                let stringLiteral = self.parseStringLiteral();
                if stringLiteral == "C" {
                    if self.check(TokenKind::LeftBracket(BracketKind::Paren)) {
                        self.expect(TokenKind::LeftBracket(BracketKind::Paren));
                        let header = if self.check(TokenKind::StringLiteral) {
                            Some(self.parseStringLiteral())
                        } else {
                            None
                        };
                        self.expect(TokenKind::RightBracket(BracketKind::Paren));
                        (Some(FunctionExternKind::C(header)), None)
                    } else {
                        (Some(FunctionExternKind::C(None)), None)
                    }
                } else {
                    error(format!("Unknown extern kind: {}", stringLiteral));
                }
            } else {
                (Some(FunctionExternKind::Builtin), None)
            }
        } else {
            let body = if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
                Some(self.parseBlock())
            } else {
                None
            };
            (None, body)
        };

        Function {
            name,
            typeParams,
            params,
            result,
            body: body,
            externKind,
            public,
            attributes,
        }
    }
}
