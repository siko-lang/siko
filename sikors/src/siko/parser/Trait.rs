use std::mem;

use crate::siko::syntax::Trait::{Instance, Trait};

use super::{
    Function::FunctionParser,
    Parser::Parser,
    Token::{BracketKind, KeywordKind, MiscKind, OperatorKind, TokenKind},
    Type::TypeParser,
};

pub trait TraitParser {
    fn parseTrait(&mut self) -> Trait;
    fn parseInstance(&mut self) -> Instance;
}

impl TraitParser for Parser {
    fn parseTrait(&mut self) -> Trait {
        self.expect(TokenKind::Keyword(KeywordKind::Trait));
        let typeParams = if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
            Some(self.parseTypeParameterDeclaration())
        } else {
            None
        };
        let name = self.parseTypeIdentifier();
        let mut params = Vec::new();
        let mut depParams = Vec::new();
        let mut isDep = false;
        self.expect(TokenKind::LeftBracket(BracketKind::Square));
        loop {
            let param = self.parseTypeIdentifier();
            if isDep {
                depParams.push(param);
            } else {
                params.push(param);
            }
            match self.peek() {
                TokenKind::RightBracket(BracketKind::Square) => break,
                TokenKind::Misc(MiscKind::Comma) => {
                    self.expect(TokenKind::Misc(MiscKind::Comma));
                }
                TokenKind::Op(OperatorKind::GreaterThan) => {
                    if isDep {
                        self.reportError2(", or ]", TokenKind::Op(OperatorKind::GreaterThan));
                    }
                    isDep = true;
                    self.expect(TokenKind::Op(OperatorKind::GreaterThan));
                }
                kind => self.reportError2(", or ]", kind),
            }
        }
        self.expect(TokenKind::RightBracket(BracketKind::Square));
        let mut members = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
                let function = self.parseFunction();
                members.push(function);
            }
            self.expect(TokenKind::RightBracket(BracketKind::Curly));
        }
        Trait {
            name: name,
            typeParams: typeParams,
            members: members,
        }
    }

    fn parseInstance(&mut self) -> Instance {
        self.expect(TokenKind::Keyword(KeywordKind::Instance));
        let typeParams = if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
            Some(self.parseTypeParameterDeclaration())
        } else {
            None
        };
        let ty = self.parseType();
        let mut members = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
                let function = self.parseFunction();
                members.push(function);
            }
            self.expect(TokenKind::RightBracket(BracketKind::Curly));
        }
        Instance {}
    }
}
