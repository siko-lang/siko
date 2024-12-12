use crate::siko::{
    syntax::{Identifier::Identifier, Type::*},
    util::error,
};

use super::{
    Parser::*,
    Token::{ArrowKind, BracketKind, KeywordKind, MiscKind, OperatorKind, TokenKind},
};

pub trait TypeParser {
    fn parseType(&mut self) -> Type;
    fn parseTypeConstraint(&mut self) -> Constraint;
    fn parseTypeConstraints(&mut self) -> Vec<Constraint>;
    fn parseTypeParams(&mut self) -> Vec<Identifier>;
    fn parseTypeParameterDeclaration(&mut self) -> TypeParameterDeclaration;
}

impl<'a> TypeParser for Parser<'a> {
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
            TokenKind::Misc(MiscKind::Ampersand) => {
                self.expect(TokenKind::Misc(MiscKind::Ampersand));
                let ty = self.parseType();
                Type::Reference(Box::new(ty))
            }
            TokenKind::Op(OperatorKind::Mul) => {
                self.expect(TokenKind::Op(OperatorKind::Mul));
                let ty = self.parseType();
                Type::Ptr(Box::new(ty))
            }
            kind => self.reportError2("<type>", kind),
        }
    }

    fn parseTypeConstraint(&mut self) -> Constraint {
        let traitName = self.parseTypeIdentifier();
        let mut args = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Square));
            while !self.check(TokenKind::RightBracket(BracketKind::Square)) {
                let ty = self.parseType();
                if self.check(TokenKind::Misc(MiscKind::Equal)) {
                    self.expect(TokenKind::Misc(MiscKind::Equal));
                    let associatedTy = self.parseType();
                    if let Type::Named(name, _) = ty {
                        args.push(ConstraintArgument::AssociatedType(name, associatedTy));
                    } else {
                        error(format!("Unexpected associated type {:?}", ty));
                    }
                } else {
                    args.push(ConstraintArgument::Type(ty));
                }
                if self.check(TokenKind::Misc(MiscKind::Comma)) {
                    self.expect(TokenKind::Misc(MiscKind::Comma));
                    continue;
                }
            }
        }
        self.expect(TokenKind::RightBracket(BracketKind::Square));
        Constraint {
            traitName: traitName,
            args: args,
        }
    }

    fn parseTypeConstraints(&mut self) -> Vec<Constraint> {
        let mut constraints = Vec::new();
        while self.check(TokenKind::TypeIdentifier) {
            let constraint = self.parseTypeConstraint();
            constraints.push(constraint);
            if self.check(TokenKind::Misc(MiscKind::Comma)) {
                self.expect(TokenKind::Misc(MiscKind::Comma));
                continue;
            } else {
                break;
            }
        }
        constraints
    }

    fn parseTypeParams(&mut self) -> Vec<Identifier> {
        let mut params = Vec::new();
        while self.check(TokenKind::TypeIdentifier) {
            let param = self.parseTypeIdentifier();
            params.push(param);
            if self.check(TokenKind::Misc(MiscKind::Comma)) {
                self.expect(TokenKind::Misc(MiscKind::Comma));
                continue;
            } else {
                break;
            }
        }
        params
    }

    fn parseTypeParameterDeclaration(&mut self) -> TypeParameterDeclaration {
        let mut constraints = Vec::new();
        self.expect(TokenKind::LeftBracket(BracketKind::Square));
        let params = self.parseTypeParams();
        if self.check(TokenKind::Misc(MiscKind::Colon)) {
            self.expect(TokenKind::Misc(MiscKind::Colon));
            constraints = self.parseTypeConstraints();
        }
        self.expect(TokenKind::RightBracket(BracketKind::Square));
        TypeParameterDeclaration { params, constraints }
    }
}
