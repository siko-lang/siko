use super::{
    Parser::Parser,
    Token::{BracketKind, KeywordKind, MiscKind, TokenKind},
    Trait::TraitParser,
};

use crate::siko::{
    location::Location::{Location, Span},
    parser::{Data::DataParser, Effect::EffectParser, Function::FunctionParser, Implicit::ImplicitParser},
    syntax::{
        Function::Attributes,
        Identifier::Identifier,
        Module::{Derive, Import, Module, ModuleItem},
    },
};
pub trait ModuleParser {
    fn parseImport(&mut self) -> Import;
    fn parseModule(&mut self) -> Module;
    fn parseAttributes(&mut self) -> (Attributes, Vec<Derive>);
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

    fn parseAttributes(&mut self) -> (Attributes, Vec<Derive>) {
        let mut attributes = Attributes::new();
        let mut derives = Vec::new();
        while self.check(TokenKind::Misc(MiscKind::At)) {
            self.expect(TokenKind::Misc(MiscKind::At));
            if self.check(TokenKind::VarIdentifier) {
                let name = self.parseVarIdentifier();
                match name.name().as_str() {
                    "test" => attributes.testEntry = true,
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

    fn parseModule(&mut self) -> Module {
        self.expect(TokenKind::Keyword(KeywordKind::Module));
        let name = self.parseModuleName();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        let mut items = Vec::new();
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            let (attributes, derives) = self.parseAttributes();
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
                TokenKind::Keyword(KeywordKind::Fn) => ModuleItem::Function(self.parseFunction(attributes, public)),
                TokenKind::Keyword(KeywordKind::Import) => ModuleItem::Import(self.parseImport()),
                TokenKind::Keyword(KeywordKind::Effect) => ModuleItem::Effect(self.parseEffect(public)),
                TokenKind::Keyword(KeywordKind::Implicit) => ModuleItem::Implicit(self.parseImplicit(public)),
                TokenKind::Keyword(KeywordKind::Trait) => ModuleItem::Trait(self.parseTrait(public)),
                TokenKind::Keyword(KeywordKind::Instance) => ModuleItem::Instance(self.parseInstance(public)),
                kind => self.reportError2("<module item>", kind),
            };
            items.push(item);
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        let implicitImports = vec![
            "String",
            "List",
            "Bool",
            "Box",
            "Int",
            "U8",
            "U16",
            "U32",
            "U64",
            "I8",
            "I16",
            "I32",
            "I64",
            "Result",
            "Option",
            "Ordering",
            "Show",
            "Iterator",
            "Std.Ops.Basic",
            "Std.Fmt",
            "Std.Cmp",
            "Std.Basic.Util",
            "Vec",
            "Range",
        ];
        for i in implicitImports {
            items.push(ModuleItem::Import(Import {
                moduleName: Identifier::new(i.to_string(), Location::new(self.fileId.clone(), Span::new())),
                alias: None,
                implicitImport: true,
            }))
        }
        Module { name, items }
    }
}
