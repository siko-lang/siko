use crate::siko::{
    parser::{
        Parser::Parser,
        Token::{KeywordKind, MiscKind, TokenKind},
        Type::TypeParser,
    },
    syntax::Implicit::Implicit,
};

pub trait ImplicitParser {
    fn parseImplicit(&mut self, public: bool) -> Implicit;
}

impl ImplicitParser for Parser<'_> {
    fn parseImplicit(&mut self, public: bool) -> Implicit {
        self.expect(TokenKind::Keyword(KeywordKind::Implicit));
        let isMutable = if self.check(TokenKind::Keyword(KeywordKind::Mut)) {
            self.expect(TokenKind::Keyword(KeywordKind::Mut));
            true
        } else {
            false
        };
        let name = self.parseVarIdentifier();
        self.expect(TokenKind::Misc(MiscKind::Colon));
        let ty = self.parseType();
        Implicit {
            name,
            ty,
            mutable: isMutable,
            public,
        }
    }
}
