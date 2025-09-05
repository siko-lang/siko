use crate::siko::{
    parser::Module::ModuleParser,
    syntax::{
        Identifier::Identifier,
        Trait::{AssociatedType, AssociatedTypeDeclaration, Instance, Trait},
        Type::Type,
    },
};

use super::{
    Function::FunctionParser,
    Parser::Parser,
    Token::{BracketKind, KeywordKind, MiscKind, TokenKind},
    Type::TypeParser,
};

pub trait TraitParser {
    fn parseTrait(&mut self, public: bool) -> Trait;
    fn parseAssociatedTypeDeclaration(&mut self) -> AssociatedTypeDeclaration;
    fn parseAssociatedType(&mut self) -> AssociatedType;
    fn parseInstance(&mut self, public: bool) -> Instance;
}

impl<'a> TraitParser for Parser<'a> {
    fn parseTrait(&mut self, public: bool) -> Trait {
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
                let (attributes, _) = self.parseAttributes();
                if self.check(TokenKind::Keyword(KeywordKind::Fn)) {
                    let function = self.parseFunction(attributes, true);
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
            public: public,
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

    fn parseInstance(&mut self, public: bool) -> Instance {
        let location = self.currentLocation();
        self.expect(TokenKind::Keyword(KeywordKind::Instance));
        let typeParams = if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
            Some(self.parseTypeParameterDeclaration())
        } else {
            None
        };
        let nameLoc = self.currentLocation();
        let name = self.parseType();
        let defLoc = self.currentLocation();
        let (name, traitName, types) = if !self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            // name is impl name
            let name = if let Some((name, args)) = getNameAndArgs(name) {
                if args.is_empty() {
                    name
                } else {
                    self.reportError3("expected impl name", nameLoc)
                }
            } else {
                self.reportError3("expected impl name", nameLoc)
            };
            let def = self.parseType();
            if let Some((traitName, types)) = getNameAndArgs(def) {
                (Some(name), traitName, types)
            } else {
                self.reportError3("expected trait name and args", defLoc)
            }
        } else {
            if let Some((traitName, types)) = getNameAndArgs(name) {
                (None, traitName, types)
            } else {
                self.reportError3("expected trait name and args", defLoc);
            }
        };
        let mut methods = Vec::new();
        let mut associatedTypes = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
                let (attributes, _) = self.parseAttributes();
                if self.check(TokenKind::Keyword(KeywordKind::Fn)) {
                    let function = self.parseFunction(attributes, true);
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
            public,
            name,
            typeParams,
            traitName,
            types,
            associatedTypes,
            methods,
            location,
        }
    }
}

fn getNameAndArgs(ty: Type) -> Option<(Identifier, Vec<Type>)> {
    if let Type::Named(id, args) = ty {
        Some((id, args))
    } else {
        None
    }
}
