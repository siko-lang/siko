use super::Module::ModuleParser;
use super::Token::{MiscKind, OperatorKind, Token, TokenInfo, TokenKind};
use crate::siko::location::Location::{Location, Span};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Module::Module;
use crate::siko::util::error;
use crate::siko::{location::Location::FileId, parser::Lexer::*};

pub struct Parser {
    tokens: Vec<TokenInfo>,
    index: usize,
    pub fileId: FileId,
    modules: Vec<Module>,
    fileName: String,
    spans: Vec<Span>,
    pub opTable: Vec<Vec<OperatorKind>>,
}

impl Parser {
    pub fn new(fileId: FileId, fileName: String) -> Parser {
        Parser {
            tokens: Vec::new(),
            index: 0,
            fileId: fileId,
            modules: Vec::new(),
            fileName: fileName,
            spans: Vec::new(),
            opTable: vec![
                vec![OperatorKind::And, OperatorKind::Or],
                vec![OperatorKind::Equal, OperatorKind::NotEqual],
                vec![
                    OperatorKind::LessThan,
                    OperatorKind::GreaterThan,
                    OperatorKind::LessThanOrEqual,
                    OperatorKind::GreaterThanOrEqual,
                ],
                vec![OperatorKind::Add, OperatorKind::Sub],
                vec![OperatorKind::Mul, OperatorKind::Div],
            ],
        }
    }

    pub fn pushSpan(&mut self) {
        self.spans.push(self.tokens[self.index].span.clone());
    }

    pub fn popSpan(&mut self) -> Location {
        let start = self.spans.pop().unwrap();
        let merged = start.merge(self.tokens[self.index - 1].span.clone());
        Location::new(self.fileId.clone(), merged)
    }

    pub fn useSpan(&self) -> Location {
        let start = self.spans.last().unwrap();
        let merged = start
            .clone()
            .merge(self.tokens[self.index - 1].span.clone());
        Location::new(self.fileId.clone(), merged)
    }

    pub fn peek(&self) -> TokenKind {
        self.tokens[self.index].kind()
    }

    pub fn current(&self) -> &TokenInfo {
        &self.tokens[self.index]
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    pub fn isDone(&self) -> bool {
        self.check(TokenKind::EOF)
    }

    pub fn expect(&mut self, kind: TokenKind) {
        if self.tokens[self.index].kind() != kind {
            self.reportError(kind, self.tokens[self.index].kind());
        } else {
            self.step();
        }
    }

    pub fn reportError(&mut self, expected: TokenKind, found: TokenKind) -> ! {
        let line = self.tokens[self.index].span.start.line + 1;
        let offset = self.tokens[self.index].span.start.offset + 1;
        error(format!(
            "Expected {:?} found {:?} at {}:{}:{}",
            expected, found, self.fileName, line, offset,
        ));
    }

    pub fn reportError2(&mut self, expected: &str, found: TokenKind) -> ! {
        let line = self.tokens[self.index].span.start.line + 1;
        let offset = self.tokens[self.index].span.start.offset + 1;
        error(format!(
            "Expected {:?} found {:?} at {}:{}:{}",
            expected, found, self.fileName, line, offset,
        ));
    }

    pub fn step(&mut self) {
        self.index += 1;
    }

    pub fn undo(&mut self) {
        self.index -= 1;
    }

    pub fn parseTypeIdentifier(&mut self) -> Identifier {
        match self.tokens[self.index].token.clone() {
            Token::TypeIdentifier(v) => {
                let i = Identifier {
                    name: v,
                    location: self.currentLocation(),
                };
                self.step();
                i
            }
            t => self.reportError(TokenKind::TypeIdentifier, t.kind()),
        }
    }

    pub fn parseVarIdentifier(&mut self) -> Identifier {
        match self.tokens[self.index].token.clone() {
            Token::VarIdentifier(v) => {
                let i = Identifier {
                    name: v,
                    location: self.currentLocation(),
                };
                self.step();
                i
            }
            t => self.reportError(TokenKind::VarIdentifier, t.kind()),
        }
    }

    pub fn parseIntegerLiteral(&mut self) -> String {
        match self.tokens[self.index].token.clone() {
            Token::IntegerLiteral(v) => {
                self.step();
                v.clone()
            }
            t => self.reportError(TokenKind::IntegerLiteral, t.kind()),
        }
    }

    pub fn parseStringLiteral(&mut self) -> String {
        match self.tokens[self.index].token.clone() {
            Token::StringLiteral(v) => {
                self.step();
                v.clone()
            }
            t => self.reportError(TokenKind::StringLiteral, t.kind()),
        }
    }

    pub fn currentLocation(&self) -> Location {
        Location::new(self.fileId.clone(), self.tokens[self.index].span.clone())
    }

    pub fn parseModuleName(&mut self) -> Identifier {
        let mut id = self.parseTypeIdentifier();
        while self.check(TokenKind::Misc(MiscKind::Dot)) {
            self.expect(TokenKind::Misc(MiscKind::Dot));
            id.dot(self.currentLocation());
            let next = self.parseTypeIdentifier();
            id.merge(next);
        }
        id
    }

    pub fn parse(&mut self) {
        let content = std::fs::read_to_string(&self.fileName).unwrap();
        let mut lexer = Lexer::new(content, self.fileId.clone());
        let (tokens, _errors) = lexer.lex();
        //println!("Tokens {:?}", tokens);
        self.tokens = tokens;
        //println!("Errors {:?}", errors);
        while !self.isDone() {
            let m = self.parseModule();
            self.modules.push(m);
        }
    }

    pub fn modules(self) -> Vec<Module> {
        self.modules
    }
}
