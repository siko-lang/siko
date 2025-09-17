use crate::siko::{
    parser::{
        Parser::Parser,
        Token::{BracketKind, KeywordKind, MiscKind, TokenKind},
    },
    syntax::{
        Attributes::{Attributes, Safety},
        Module::Derive,
    },
};

pub trait AttributeParser {
    fn parseAttributes(&mut self) -> (Attributes, Vec<Derive>);
    fn parseDerives(&mut self) -> Vec<Derive>;
}

impl<'a> AttributeParser for Parser<'a> {
    fn parseAttributes(&mut self) -> (Attributes, Vec<Derive>) {
        let mut attributes = Attributes::new();
        let mut derives = Vec::new();
        while self.check(TokenKind::Misc(MiscKind::At)) {
            self.expect(TokenKind::Misc(MiscKind::At));
            if self.check(TokenKind::VarIdentifier) {
                let name = self.parseVarIdentifier();
                match name.name().as_str() {
                    "test" => attributes.testEntry = true,
                    "inline" => attributes.inline = true,
                    "unsafe" => attributes.safety = Safety::Unsafe,
                    "safe" => attributes.safety = Safety::Safe,
                    "builtin" => attributes.builtin = true,
                    _ => self.reportError3(&format!("Unknown attribute: {}", name), name.location()),
                }
            } else if self.check(TokenKind::Keyword(KeywordKind::Derive)) {
                derives = self.parseDerives();
            } else {
                self.reportError2("expected attribute", self.peek());
            }
        }
        (attributes, derives)
    }

    fn parseDerives(&mut self) -> Vec<Derive> {
        let mut derives = Vec::new();
        self.expect(TokenKind::Keyword(KeywordKind::Derive));
        self.expect(TokenKind::LeftBracket(BracketKind::Paren));
        while !self.check(TokenKind::RightBracket(BracketKind::Paren)) {
            let item = self.parseTypeIdentifier();
            derives.push(Derive { name: item });
            if self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                break;
            } else {
                self.expect(TokenKind::Misc(MiscKind::Comma));
            }
        }
        self.expect(TokenKind::RightBracket(BracketKind::Paren));
        derives
    }
}
