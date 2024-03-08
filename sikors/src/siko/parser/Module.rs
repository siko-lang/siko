use super::{
    Parser::Parser,
    Token::{BracketKind, KeywordKind, TokenKind},
};

use crate::siko::{
    parser::{Data::DataParser, Function::FunctionParser},
    syntax::Module::{Import, Module, ModuleItem},
    util::error,
};
pub trait ModuleParser {
    fn parseImport(&mut self) -> Import;
    fn parseModule(&mut self) -> Module;
}

impl ModuleParser for Parser {
    fn parseImport(&mut self) -> Import {
        self.expect(TokenKind::Keyword(KeywordKind::Import));
        let name = self.parseModuleName();
        let alias = if self.check(TokenKind::Keyword(KeywordKind::As)) {
            self.expect(TokenKind::Keyword(KeywordKind::As));
            let alias = self.parseModuleName();
            Some(alias)
        } else {
            None
        };
        Import {
            moduleName: name,
            alias,
        }
    }
    fn parseModule(&mut self) -> Module {
        self.expect(TokenKind::Keyword(KeywordKind::Module));
        let name = self.parseModuleName();
        println!("Module name {:?}", name);
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        let mut items = Vec::new();
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            let item = match self.peek() {
                Some(TokenKind::Keyword(KeywordKind::Class)) => {
                    ModuleItem::Class(self.parseClass())
                }
                Some(TokenKind::Keyword(KeywordKind::Enum)) => ModuleItem::Enum(self.parseEnum()),
                Some(TokenKind::Keyword(KeywordKind::Fn)) => {
                    ModuleItem::Function(self.parseFunction())
                }
                Some(TokenKind::Keyword(KeywordKind::Import)) => {
                    ModuleItem::Import(self.parseImport())
                }
                Some(kind) => self.reportError2("<module item>", kind),
                None => error(format!("EOF")),
            };
            items.push(item);
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Module { name, items }
    }
}
