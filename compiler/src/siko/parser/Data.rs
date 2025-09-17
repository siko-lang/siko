use crate::siko::{
    parser::Attributes::AttributeParser,
    syntax::{
        Attributes::Attributes,
        Data::{Enum, Field, Struct, Variant},
        Module::Derive,
    },
};

use super::{
    Function::FunctionParser,
    Parser::*,
    Token::{BracketKind, KeywordKind, MiscKind, TokenKind},
    Type::TypeParser,
};

pub trait DataParser {
    fn parseStruct(&mut self, derives: Vec<Derive>, public: bool, attributes: Attributes) -> Struct;
    fn parseEnum(&mut self, derives: Vec<Derive>, public: bool, attributes: Attributes) -> Enum;
    fn parseVariant(&mut self) -> Variant;
    fn parseField(&mut self, public: bool) -> Field;
}

impl<'a> DataParser for Parser<'a> {
    fn parseStruct(&mut self, derives: Vec<Derive>, public: bool, attributes: Attributes) -> Struct {
        self.expect(TokenKind::Keyword(KeywordKind::Struct));
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
            let (fnAttributes, _) = self.parseAttributes();
            let mut public = false;
            if self.check(TokenKind::Keyword(KeywordKind::Pub)) {
                self.expect(TokenKind::Keyword(KeywordKind::Pub));
                public = true;
            }
            match self.peek() {
                TokenKind::Keyword(KeywordKind::Fn) => {
                    let method = self.parseFunction(fnAttributes, public, false);
                    methods.push(method);
                }
                TokenKind::VarIdentifier => {
                    let field = self.parseField(public);
                    fields.push(field);
                    if self.check(TokenKind::Misc(MiscKind::Comma)) {
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                    }
                }
                kind => self.reportError2("<structDef member>", kind),
            }
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Struct {
            name,
            typeParams: typeParams,
            isExtern: attributes.builtin,
            fields: fields,
            methods: methods,
            derives,
            public,
        }
    }

    fn parseEnum(&mut self, derives: Vec<Derive>, public: bool, _: Attributes) -> Enum {
        self.expect(TokenKind::Keyword(KeywordKind::Enum));
        let name = self.parseTypeIdentifier();
        let typeParams = if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
            Some(self.parseTypeParameterDeclaration())
        } else {
            None
        };
        let mut variants = Vec::new();
        let mut methods = Vec::new();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            let (fnAttributes, _) = self.parseAttributes();
            let mut public = false;
            if self.check(TokenKind::Keyword(KeywordKind::Pub)) {
                self.expect(TokenKind::Keyword(KeywordKind::Pub));
                public = true;
            }
            match self.peek() {
                TokenKind::Keyword(KeywordKind::Fn) => {
                    let method = self.parseFunction(fnAttributes, public, false);
                    methods.push(method);
                }
                TokenKind::TypeIdentifier => {
                    let variant = self.parseVariant();
                    variants.push(variant);
                    if self.check(TokenKind::Misc(MiscKind::Comma)) {
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                    }
                }
                kind => self.reportError2("<enum member>", kind),
            }
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Enum {
            name,
            typeParams,
            variants,
            methods,
            derives,
            public,
        }
    }

    fn parseField(&mut self, public: bool) -> Field {
        let name = self.parseVarIdentifier();
        self.expect(TokenKind::Misc(MiscKind::Colon));
        let ty = self.parseType();
        Field { name, ty, public }
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
