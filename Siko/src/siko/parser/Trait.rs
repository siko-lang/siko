use crate::siko::syntax::Trait::{AssociatedType, AssociatedTypeDeclaration, Instance, Trait};

use super::{
    Function::FunctionParser,
    Parser::Parser,
    Token::{BracketKind, KeywordKind, MiscKind, TokenKind},
    Type::TypeParser,
};

pub trait TraitParser {
    fn parseTrait(&mut self) -> Trait;
    fn parseAssociatedTypeDeclaration(&mut self) -> AssociatedTypeDeclaration;
    fn parseAssociatedType(&mut self) -> AssociatedType;
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
        self.expect(TokenKind::LeftBracket(BracketKind::Square));
        let params = self.parseTypeParams();
        self.expect(TokenKind::RightBracket(BracketKind::Square));
        let mut methods = Vec::new();
        let mut associatedTypes = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
                if self.check(TokenKind::Keyword(KeywordKind::Fn)) {
                    let function = self.parseFunction();
                    methods.push(function);
                    continue;
                }
                if self.check(TokenKind::Keyword(KeywordKind::Type)) {
                    let associatedType = self.parseAssociatedTypeDeclaration();
                    associatedTypes.push(associatedType);
                    continue;
                }
                self.reportError2("expected trait member or associated type", self.peek());
            }
            self.expect(TokenKind::RightBracket(BracketKind::Curly));
        }
        Trait {
            name: name,
            params: params,
            typeParams: typeParams,
            associatedTypes: associatedTypes,
            methods,
        }
    }

    fn parseAssociatedTypeDeclaration(&mut self) -> AssociatedTypeDeclaration {
        self.expect(TokenKind::Keyword(KeywordKind::Type));
        let name = self.parseTypeIdentifier();
        let constraints = if self.check(TokenKind::Misc(MiscKind::Colon)) {
            self.expect(TokenKind::Misc(MiscKind::Colon));
            self.parseTypeConstraints()
        } else {
            Vec::new()
        };
        AssociatedTypeDeclaration {
            name: name,
            constraints: constraints,
        }
    }

    fn parseAssociatedType(&mut self) -> AssociatedType {
        self.expect(TokenKind::Keyword(KeywordKind::Type));
        let name = self.parseTypeIdentifier();
        self.expect(TokenKind::Misc(MiscKind::Equal));
        let ty = self.parseType();
        AssociatedType { name: name, ty: ty }
    }

    fn parseInstance(&mut self) -> Instance {
        let location = self.currentLocation();
        self.expect(TokenKind::Keyword(KeywordKind::Instance));
        let typeParams = if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
            Some(self.parseTypeParameterDeclaration())
        } else {
            None
        };
        let traitName = self.parseTypeIdentifier();
        let mut types = Vec::new();
        self.expect(TokenKind::LeftBracket(BracketKind::Square));
        while !self.check(TokenKind::RightBracket(BracketKind::Square)) {
            let ty = self.parseType();
            types.push(ty);
            if self.check(TokenKind::Misc(MiscKind::Comma)) {
                self.expect(TokenKind::Misc(MiscKind::Comma));
                continue;
            } else {
                break;
            }
        }
        self.expect(TokenKind::RightBracket(BracketKind::Square));
        let mut methods = Vec::new();
        let mut associatedTypes = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
                if self.check(TokenKind::Keyword(KeywordKind::Fn)) {
                    let function = self.parseFunction();
                    methods.push(function);
                    continue;
                }
                if self.check(TokenKind::Keyword(KeywordKind::Type)) {
                    let associatedType = self.parseAssociatedType();
                    associatedTypes.push(associatedType);
                    continue;
                }
                self.reportError2("expected trait member or associated type", self.peek());
            }
            self.expect(TokenKind::RightBracket(BracketKind::Curly));
        }
        Instance {
            id: 0,
            typeParams,
            traitName,
            types,
            associatedTypes,
            methods,
            location,
        }
    }
}
