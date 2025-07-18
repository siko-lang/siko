use super::{
    Parser::Parser,
    Token::{BracketKind, KeywordKind, MiscKind, TokenKind},
    Trait::TraitParser,
};

use crate::siko::{
    location::Location::{Location, Span},
    parser::{Data::DataParser, Function::FunctionParser},
    syntax::{
        Identifier::Identifier,
        Module::{Derive, Import, Module, ModuleItem},
    },
};
pub trait ModuleParser {
    fn parseImport(&mut self) -> Import;
    fn parseModule(&mut self) -> Module;
    fn parseDerives(&mut self) -> Vec<Derive>;
}

impl<'a> ModuleParser for Parser<'a> {
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
            implicitImport: false,
        }
    }

    fn parseDerives(&mut self) -> Vec<Derive> {
        self.expect(TokenKind::Misc(MiscKind::At));
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

    fn parseModule(&mut self) -> Module {
        self.expect(TokenKind::Keyword(KeywordKind::Module));
        let name = self.parseModuleName();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        let mut items = Vec::new();
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            let derives = if self.check(TokenKind::Misc(MiscKind::At)) {
                self.parseDerives()
            } else {
                Vec::new()
            };
            let mut public = false;
            if self.check(TokenKind::Keyword(KeywordKind::Pub)) {
                self.expect(TokenKind::Keyword(KeywordKind::Pub));
                public = true;
            }
            let item = match self.peek() {
                TokenKind::Keyword(KeywordKind::Extern) => {
                    self.expect(TokenKind::Keyword(KeywordKind::Extern));
                    let mut c = self.parseStruct(derives, public);
                    c.isExtern = true;
                    ModuleItem::Struct(c)
                }
                TokenKind::Keyword(KeywordKind::Struct) => ModuleItem::Struct(self.parseStruct(derives, public)),
                TokenKind::Keyword(KeywordKind::Enum) => ModuleItem::Enum(self.parseEnum(derives, public)),
                TokenKind::Keyword(KeywordKind::Fn) => ModuleItem::Function(self.parseFunction(public)),
                TokenKind::Keyword(KeywordKind::Import) => ModuleItem::Import(self.parseImport()),
                TokenKind::Keyword(KeywordKind::Trait) => ModuleItem::Trait(self.parseTrait(public)),
                TokenKind::Keyword(KeywordKind::Instance) => ModuleItem::Instance(self.parseInstance()),
                kind => self.reportError2("<module item>", kind),
            };
            items.push(item);
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        let implicitImports = vec![
            "String",
            "List",
            "Bool",
            "Int",
            "Char",
            "Result",
            "Option",
            "Ordering",
            "Show",
            "Iterator",
            "Std.Ops",
            "Std.Basic.Util",
            "Vec",
        ];
        for i in implicitImports {
            items.push(ModuleItem::Import(Import {
                moduleName: Identifier {
                    name: i.to_string(),
                    location: Location::new(self.fileId.clone(), Span::new()),
                },
                alias: None,
                implicitImport: true,
            }))
        }
        Module { name, items }
    }
}
