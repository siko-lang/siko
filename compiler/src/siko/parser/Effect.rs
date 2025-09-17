use crate::siko::{parser::Attributes::AttributeParser, syntax::Effect::Effect};

use super::{
    Function::FunctionParser,
    Parser::Parser,
    Token::{BracketKind, KeywordKind, TokenKind},
};

pub trait EffectParser {
    fn parseEffect(&mut self, public: bool) -> Effect;
}

impl<'a> EffectParser for Parser<'a> {
    fn parseEffect(&mut self, public: bool) -> Effect {
        self.expect(TokenKind::Keyword(KeywordKind::Effect));
        let name = self.parseTypeIdentifier();
        let mut methods = Vec::new();
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
                let (attributes, _) = self.parseAttributes();
                if self.check(TokenKind::Keyword(KeywordKind::Fn)) {
                    let function = self.parseFunction(attributes, true, false);
                    methods.push(function);
                    continue;
                }
                self.reportError2("expected effect member", self.peek());
            }
            self.expect(TokenKind::RightBracket(BracketKind::Curly));
        }
        Effect {
            name: name,
            methods,
            public: public,
        }
    }
}
