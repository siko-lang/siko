use crate::siko::{
    parser::{
        Expr::ExprParser,
        Parser::Parser,
        Token::{KeywordKind, MiscKind, TokenKind},
        Type::TypeParser,
    },
    syntax::{Attributes::Attributes, Global::Global},
};

pub trait GlobalParser {
    fn parseGlobal(&mut self, attributes: Attributes, public: bool) -> Global;
}

impl<'a> GlobalParser for Parser<'a> {
    fn parseGlobal(&mut self, attributes: Attributes, public: bool) -> Global {
        self.expect(TokenKind::Keyword(KeywordKind::Let));
        let name = self.parseTypeIdentifier();
        self.expect(TokenKind::Misc(MiscKind::Colon));
        let ty = self.parseType();
        self.expect(TokenKind::Misc(MiscKind::Equal));
        let value = self.parseExpr();
        self.expect(TokenKind::Misc(MiscKind::Semicolon));
        Global {
            name,
            ty,
            value,
            public,
            attributes,
        }
    }
}
