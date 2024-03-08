use super::{
    Parser::Parser,
    Token::{BracketKind, KeywordKind, TokenKind},
};

use crate::siko::{
    parser::{Data::DataParser, Function::FunctionParser},
    syntax::Module::{Import, Module, ModuleItem},
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
                TokenKind::Keyword(KeywordKind::Class) => ModuleItem::Class(self.parseClass()),
                TokenKind::Keyword(KeywordKind::Enum) => ModuleItem::Enum(self.parseEnum()),
                TokenKind::Keyword(KeywordKind::Fn) => ModuleItem::Function(self.parseFunction()),
                TokenKind::Keyword(KeywordKind::Import) => ModuleItem::Import(self.parseImport()),
                kind => self.reportError2("<module item>", kind),
            };
            items.push(item);
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Module { name, items }
    }
}
