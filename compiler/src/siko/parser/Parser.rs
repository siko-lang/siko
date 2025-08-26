use super::Module::ModuleParser;
use super::Token::{MiscKind, OperatorKind, Token, TokenInfo, TokenKind};
use crate::siko::location::Location::{Location, Position, Span};
use crate::siko::location::Report::{Report, ReportContext};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Module::Module;
use crate::siko::util::error;
use crate::siko::{location::Location::FileId, parser::Lexer::*};

pub struct Parser<'a> {
    ctx: &'a ReportContext,
    tokens: Vec<TokenInfo>,
    index: usize,
    pub fileId: FileId,
    modules: Vec<Module>,
    fileName: String,
    spans: Vec<Span>,
    pub opTable: Vec<Vec<OperatorKind>>,
}

impl<'a> Parser<'a> {
    pub fn new(ctx: &'a ReportContext, fileId: FileId, fileName: String) -> Parser<'a> {
        Parser {
            ctx: ctx,
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

    pub fn currentSpan(&mut self) -> Span {
        self.tokens[self.index].span.clone()
    }

    pub fn endSpan(&mut self) -> Span {
        if self.index > 0 {
            self.tokens[self.index - 1].span.clone()
        } else {
            self.tokens[self.index].span.clone()
        }
    }

    pub fn pushSpan(&mut self) {
        let span = self.tokens[self.index].span.clone();
        self.spans.push(span);
    }

    pub fn popSpan(&mut self) -> Location {
        let start = self.spans.pop().unwrap();
        let merged = start.merge(self.tokens[self.index - 1].span.clone());
        Location::new(self.fileId.clone(), merged)
    }

    pub fn useSpan(&self) -> Location {
        let start = self.spans.last().unwrap();
        let merged = start.clone().merge(self.tokens[self.index - 1].span.clone());
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
            "Expected {} found {} at {}:{}:{}",
            expected, found, self.fileName, line, offset,
        ));
    }

    pub fn reportError2(&mut self, expected: &str, found: TokenKind) -> ! {
        let line = self.tokens[self.index].span.start.line + 1;
        let offset = self.tokens[self.index].span.start.offset + 1;
        error(format!(
            "Expected {} found {} at {}:{}:{}",
            expected, found, self.fileName, line, offset,
        ));
    }

    pub fn reportError3(&mut self, msg: &str, location: Location) -> ! {
        let line = location.span.start.line + 1;
        let offset = location.span.start.offset + 1;
        error(format!("{} at {}:{}:{}", msg, self.fileName, line, offset));
    }

    pub fn step(&mut self) {
        self.index += 1;
    }

    pub fn undo(&mut self) {
        self.index -= 1;
    }

    pub fn parseQualifiedTypeName(&mut self) -> Identifier {
        let mut id = self.parseTypeIdentifier();
        while self.check(TokenKind::Misc(MiscKind::Dot)) {
            self.expect(TokenKind::Misc(MiscKind::Dot));
            id.dot(self.currentLocation());
            let next = self.parseTypeIdentifier();
            id.merge(next);
        }
        id
    }

    pub fn parseQualifiedVarName(&mut self) -> Identifier {
        if self.check(TokenKind::VarIdentifier) {
            return self.parseVarIdentifier();
        }
        let mut id = self.parseTypeIdentifier();
        while self.check(TokenKind::Misc(MiscKind::Dot)) {
            self.expect(TokenKind::Misc(MiscKind::Dot));
            id.dot(self.currentLocation());
            if self.check(TokenKind::TypeIdentifier) {
                id.merge(self.parseTypeIdentifier());
            } else if self.check(TokenKind::VarIdentifier) {
                id.merge(self.parseVarIdentifier());
                break;
            } else {
                self.reportError2("<qualified variable name>", self.peek());
            }
        }
        id
    }

    pub fn parseTypeIdentifier(&mut self) -> Identifier {
        match self.tokens[self.index].token.clone() {
            Token::TypeIdentifier(v) => {
                let i = Identifier::new(v, self.currentLocation());
                self.step();
                i
            }
            t => self.reportError(TokenKind::TypeIdentifier, t.kind()),
        }
    }

    pub fn parseVarIdentifier(&mut self) -> Identifier {
        match self.tokens[self.index].token.clone() {
            Token::VarIdentifier(v) => {
                let i = Identifier::new(v, self.currentLocation());
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

    pub fn parseQualifiedName(&mut self) -> Identifier {
        if self.check(TokenKind::TypeIdentifier) {
            let mut id = self.parseTypeIdentifier();
            while self.check(TokenKind::Misc(MiscKind::Dot)) {
                self.expect(TokenKind::Misc(MiscKind::Dot));
                id.dot(self.currentLocation());
                if self.check(TokenKind::TypeIdentifier) {
                    let next = self.parseTypeIdentifier();
                    id.merge(next);
                } else {
                    let next = self.parseVarIdentifier();
                    id.merge(next);
                    break;
                }
            }
            id
        } else {
            self.parseVarIdentifier()
        }
    }

    pub fn parse(&mut self) {
        let content = std::fs::read_to_string(&self.fileName).unwrap();
        let mut lexer = Lexer::new(content.chars().collect(), self.fileId.clone(), Position::new());
        let (tokens, errors) = lexer.lex(true);
        //println!("Tokens {:?}", tokens);
        let lexer_success = errors.is_empty();

        for e in errors {
            match e {
                super::Error::LexerError::InvalidIdentifier(n, span) => {
                    let slogan = format!("invalid identifier {}", self.ctx.yellow(&n));
                    let r = Report::new(self.ctx, slogan, Some(Location::new(self.fileId.clone(), span)));
                    r.print();
                }
                super::Error::LexerError::UnsupportedCharacter(c, span) => {
                    let slogan = format!("unsupported character {}", self.ctx.yellow(&format!("{}", c)));
                    let r = Report::new(self.ctx, slogan, Some(Location::new(self.fileId.clone(), span)));
                    r.print();
                }
                super::Error::LexerError::UnendingStringLiteral(span) => {
                    let slogan = format!("unending string literal");
                    let r = Report::new(self.ctx, slogan, Some(Location::new(self.fileId.clone(), span)));
                    r.print();
                }
                super::Error::LexerError::UnendingCharLiteral(span) => {
                    let slogan = format!("unending char literal");
                    let r = Report::new(self.ctx, slogan, Some(Location::new(self.fileId.clone(), span)));
                    r.print();
                }
                super::Error::LexerError::InvalidCharLiteral(s, span) => {
                    let slogan = format!("invalid char literal {}", self.ctx.yellow(&s));
                    let r = Report::new(self.ctx, slogan, Some(Location::new(self.fileId.clone(), span)));
                    r.print();
                }
                super::Error::LexerError::InvalidEscapeSequence(s, span) => {
                    let slogan = format!("invalid escape sequence {}", self.ctx.yellow(&s));
                    let r = Report::new(self.ctx, slogan, Some(Location::new(self.fileId.clone(), span)));
                    r.print();
                }
                super::Error::LexerError::UnexpectedCharacter(c, span) => {
                    let slogan = format!("unexpected character {}", self.ctx.yellow(&format!("{}", c)));
                    let r = Report::new(self.ctx, slogan, Some(Location::new(self.fileId.clone(), span)));
                    r.print();
                }
                super::Error::LexerError::UnexpectedEndOfFile(span) => {
                    let slogan = format!("unexpected end of file");
                    let r = Report::new(self.ctx, slogan, Some(Location::new(self.fileId.clone(), span)));
                    r.print();
                }
            }
        }
        if !lexer_success {
            error("parse error".to_string())
        }
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
