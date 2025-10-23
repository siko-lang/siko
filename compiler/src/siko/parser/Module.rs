use super::{
    Parser::Parser,
    Token::{BracketKind, KeywordKind, TokenKind},
    Trait::TraitParser,
};

use crate::siko::{
    parser::{
        Attributes::AttributeParser, Data::DataParser, Effect::EffectParser, Function::FunctionParser,
        Global::GlobalParser, Implicit::ImplicitParser,
    },
    syntax::Module::{Import, Module, ModuleItem},
};
pub trait ModuleParser {
    fn parseImport(&mut self) -> Import;
    fn parseModule(&mut self) -> Module;
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

    fn parseModule(&mut self) -> Module {
        let (moduleAttributes, _) = self.parseAttributes();
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
                TokenKind::Keyword(KeywordKind::Struct) => {
                    ModuleItem::Struct(self.parseStruct(derives, public, attributes))
                }
                TokenKind::Keyword(KeywordKind::Enum) => ModuleItem::Enum(self.parseEnum(derives, public, attributes)),
                TokenKind::Keyword(KeywordKind::Fn) => {
                    ModuleItem::Function(self.parseFunction(attributes, public, true))
                }
                TokenKind::Keyword(KeywordKind::Let) => ModuleItem::Global(self.parseGlobal(attributes, public)),
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
        Module {
            name,
            items,
            attributes: moduleAttributes,
        }
    }
}
