use crate::siko::syntax::Trait::{Instance, Trait};

use super::{
    Parser::Parser,
    Token::{BracketKind, KeywordKind, MiscKind, OperatorKind, TokenKind},
};

pub trait TraitParser {
    fn parseTrait(&mut self) -> Trait;
    fn parseInstance(&mut self) -> Instance;
}

impl TraitParser for Parser {
    fn parseTrait(&mut self) -> Trait {
        self.expect(TokenKind::Keyword(KeywordKind::Trait));
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
        Trait {
            name: name,
            members: members,
        }
    }

    fn parseInstance(&mut self) -> Instance {
        Instance {}
    }
}
