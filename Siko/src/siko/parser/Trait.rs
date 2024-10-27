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

impl<'a> TraitParser for Parser<'a> {
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
        let mut methods = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
                let function = self.parseFunction();
                methods.push(function);
            }
            self.expect(TokenKind::RightBracket(BracketKind::Curly));
        }
        Trait {
            name: name,
            params: params,
            deps: depParams,
            typeParams: typeParams,
            methods,
        }
    }

    fn parseInstance(&mut self) -> Instance {
        let location = self.currentLocation();
        self.expect(TokenKind::Keyword(KeywordKind::Instance));
        let typeParams = if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
            Some(self.parseTypeParameterDeclaration())
        } else {
            None
        };
        let ty = self.parseType();
        let mut methods = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
                let function = self.parseFunction();
                methods.push(function);
            }
            self.expect(TokenKind::RightBracket(BracketKind::Curly));
        }
        Instance {
            id: 0,
            typeParams,
            ty,
            methods,
            location,
        }
    }
}
