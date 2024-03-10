use crate::siko::syntax::{
    Data::{Class, Enum, Field, Variant},
    Module::Derive,
};

use super::{
    Function::FunctionParser,
    Parser::*,
    Token::{BracketKind, KeywordKind, MiscKind, TokenKind},
    Type::TypeParser,
};

pub trait DataParser {
    fn parseClass(&mut self, derives: Vec<Derive>) -> Class;
    fn parseEnum(&mut self, derives: Vec<Derive>) -> Enum;
    fn parseVariant(&mut self) -> Variant;
    fn parseField(&mut self) -> Field;
}

impl DataParser for Parser {
    fn parseClass(&mut self, derives: Vec<Derive>) -> Class {
        self.expect(TokenKind::Keyword(KeywordKind::Class));
        let name = self.parseTypeIdentifier();
        let typeParams = if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
            Some(self.parseTypeParameterDeclaration())
        } else {
            None
        };
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            match self.peek() {
                TokenKind::Keyword(KeywordKind::Fn) => {
                    let method = self.parseFunction();
                    methods.push(method);
                }
                TokenKind::VarIdentifier => {
                    let name = self.parseVarIdentifier();
                    self.expect(TokenKind::Misc(MiscKind::Colon));
                    let ty = self.parseType();
                    if self.check(TokenKind::Misc(MiscKind::Comma)) {
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                    }
                    fields.push(Field { name: name, ty: ty });
                }
                kind => self.reportError2("<class member>", kind),
            }
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Class {
            name,
            typeParams: typeParams,
            isExtern: false,
            fields: fields,
            methods: methods,
            derives,
        }
    }

    fn parseEnum(&mut self, derives: Vec<Derive>) -> Enum {
        self.expect(TokenKind::Keyword(KeywordKind::Enum));
        let name = self.parseTypeIdentifier();
        let mut variants = Vec::new();
        let mut methods = Vec::new();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        while self.check(TokenKind::TypeIdentifier) {
            let variant = self.parseVariant();
            variants.push(variant);
            if self.check(TokenKind::RightBracket(BracketKind::Curly)) {
                break;
            } else {
                self.expect(TokenKind::Misc(MiscKind::Comma));
            }
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Enum {
            name,
            variants,
            methods,
            derives,
        }
    }

    fn parseField(&mut self) -> Field {
        let name = self.parseVarIdentifier();
        self.expect(TokenKind::Misc(MiscKind::Colon));
        let ty = self.parseType();
        Field { name, ty }
    }

    fn parseVariant(&mut self) -> Variant {
        let name = self.parseTypeIdentifier();
        let mut items = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Paren)) {
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
        }
        Variant { name, items }
    }
}
