use crate::siko::location::Location::*;
use crate::siko::parser::Error::*;
use crate::siko::parser::Token::*;

fn isIdentifier(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
        _ => false,
    }
}

fn isInteger(c: char) -> bool {
    match c {
        '0'..='9' => true,
        _ => false,
    }
}

fn getSingleCharToken(c: char) -> Option<Token> {
    let token = match c {
        '(' => Token::LeftBracket(BracketKind::Paren),
        ')' => Token::RightBracket(BracketKind::Paren),
        '{' => Token::LeftBracket(BracketKind::Curly),
        '}' => Token::RightBracket(BracketKind::Curly),
        '[' => Token::LeftBracket(BracketKind::Square),
        ']' => Token::RightBracket(BracketKind::Square),
        ':' => Token::Misc(MiscKind::Colon),
        ',' => Token::Misc(MiscKind::Comma),
        ';' => Token::Misc(MiscKind::Semicolon),
        '@' => Token::Misc(MiscKind::At),
        '+' => Token::Op(OperatorKind::Add),
        '*' => Token::Op(OperatorKind::Mul),
        _ => return None,
    };
    Some(token)
}
pub struct Lexer {
    content: Vec<char>,
    index: usize,
    current: String,
    position: Position,
    fileId: FileId,
    span: Span,
    tokens: Vec<TokenInfo>,
    errors: Vec<LexerError>,
}

impl Lexer {
    pub fn new(content: String, fileId: FileId) -> Lexer {
        Lexer {
            content: content.chars().collect(),
            index: 0,
            current: String::new(),
            position: Position::new(),
            fileId: fileId,
            span: Span::new(),
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn isDone(self) -> bool {
        self.index >= self.content.len()
    }

    fn peek(&self) -> Option<char> {
        if self.index < self.content.len() {
            Some(self.content[self.index])
        } else {
            None
        }
    }

    fn step(&mut self) {
        if Some('\n') == self.peek() {
            self.position.offset = 0;
            self.position.line = self.position.line + 1;
        } else {
            self.position.offset += 1;
        }
        self.index = self.index + 1;
        self.span.end = self.position.clone();
    }

    fn ignore(&mut self) {
        self.step();
        self.resetSpan();
    }

    fn resetSpan(&mut self) {
        self.span.start = self.position.clone();
        self.span.end = self.position.clone();
    }

    fn resetCurrent(&mut self) {
        self.current = String::new();
    }

    fn addToken(&mut self, token: Token) {
        self.tokens.push(TokenInfo {
            token: token,
            span: self.span.clone(),
        });
        self.resetSpan();
        self.resetCurrent();
    }

    fn addError(&mut self, error: LexerError) {
        self.errors.push(error);
    }

    fn processIdentifier(&mut self, c: char) {
        let startsWithInteger = isInteger(c);
        let startsWithUpperCase = c.is_uppercase();
        self.current.push(c);
        self.step();
        loop {
            match self.peek() {
                Some(c) if isIdentifier(c) => {
                    self.current.push(c);
                    self.step();
                }
                _ => {
                    break;
                }
            }
        }
        if startsWithInteger {
            let mut invalidLiteral = false;
            for c in self.current.chars() {
                if !isInteger(c) {
                    invalidLiteral = true;
                    break;
                }
            }
            if invalidLiteral {
                self.addError(LexerError::InvalidIdentifier(
                    self.current.clone(),
                    self.span.clone(),
                ));
                self.resetSpan();
                self.resetCurrent();
            } else {
                self.addToken(Token::IntegerLiteral(self.current.clone()));
            }
        } else {
            if startsWithUpperCase {
                let token = match self.current.as_ref() {
                    "Self" => Token::Keyword(KeywordKind::TypeSelf),
                    _ => Token::TypeIdentifier(self.current.clone()),
                };
                self.addToken(token);
            } else {
                let token = match self.current.as_ref() {
                    "module" => Token::Keyword(KeywordKind::Module),
                    "class" => Token::Keyword(KeywordKind::Class),
                    "enum" => Token::Keyword(KeywordKind::Enum),
                    "fn" => Token::Keyword(KeywordKind::Fn),
                    "import" => Token::Keyword(KeywordKind::Import),
                    "if" => Token::Keyword(KeywordKind::If),
                    "else" => Token::Keyword(KeywordKind::Else),
                    "for" => Token::Keyword(KeywordKind::For),
                    "in" => Token::Keyword(KeywordKind::In),
                    "while" => Token::Keyword(KeywordKind::While),
                    "loop" => Token::Keyword(KeywordKind::Loop),
                    "match" => Token::Keyword(KeywordKind::Match),
                    "let" => Token::Keyword(KeywordKind::Let),
                    "derive" => Token::Keyword(KeywordKind::Derive),
                    "extern" => Token::Keyword(KeywordKind::Extern),
                    "trait" => Token::Keyword(KeywordKind::Trait),
                    "instance" => Token::Keyword(KeywordKind::Instance),
                    "effect" => Token::Keyword(KeywordKind::Effect),
                    "self" => Token::Keyword(KeywordKind::ValueSelf),
                    "mut" => Token::Keyword(KeywordKind::Mut),
                    "return" => Token::Keyword(KeywordKind::Return),
                    "continue" => Token::Keyword(KeywordKind::Continue),
                    "break" => Token::Keyword(KeywordKind::Break),
                    "implicit" => Token::Keyword(KeywordKind::Implicit),
                    "_" => Token::Misc(MiscKind::Wildcard),
                    _ => Token::VarIdentifier(self.current.clone()),
                };
                self.addToken(token);
            }
        }
    }
    fn processString(&mut self) {
        let mut literal = String::new();
        self.step();
        loop {
            match self.peek() {
                Some('"') => {
                    self.step();
                    break;
                }
                Some(c) => {
                    literal.push(c);
                    self.step();
                }
                None => self.addError(LexerError::UnendingStringLiteral(self.span.clone())),
            }
        }
        self.addToken(Token::StringLiteral(literal));
    }

    fn processSingle(&mut self, c: char) {
        if let Some(token) = getSingleCharToken(c) {
            self.step();
            self.addToken(token);
        }
    }

    pub fn lex(&mut self) -> (Vec<TokenInfo>, Vec<LexerError>) {
        loop {
            match self.peek() {
                Some(c) if isIdentifier(c) => self.processIdentifier(c),
                Some(c) if getSingleCharToken(c).is_some() => self.processSingle(c),
                Some(c) => match c {
                    '\n' => self.ignore(),
                    '\t' => self.ignore(),
                    '\r' => self.ignore(),
                    ' ' => self.ignore(),
                    '-' => {
                        self.step();
                        match self.peek() {
                            Some('>') => {
                                self.step();
                                self.addToken(Token::Arrow(ArrowKind::Right))
                            }
                            _ => self.addToken(Token::Op(OperatorKind::Sub)),
                        }
                    }
                    '>' => {
                        self.step();
                        match self.peek() {
                            Some('=') => {
                                self.step();
                                self.addToken(Token::Op(OperatorKind::GreaterThanOrEqual))
                            }
                            _ => self.addToken(Token::Op(OperatorKind::GreaterThan)),
                        }
                    }
                    '<' => {
                        self.step();
                        match self.peek() {
                            Some('=') => {
                                self.step();
                                self.addToken(Token::Op(OperatorKind::LessThanOrEqual))
                            }
                            _ => self.addToken(Token::Op(OperatorKind::LessThan)),
                        }
                    }
                    '=' => {
                        self.step();
                        match self.peek() {
                            Some('=') => {
                                self.step();
                                self.addToken(Token::Op(OperatorKind::Equal))
                            }
                            Some('>') => {
                                self.step();
                                self.addToken(Token::Arrow(ArrowKind::DoubleRight))
                            }
                            _ => self.addToken(Token::Misc(MiscKind::Equal)),
                        }
                    }
                    '.' => {
                        self.step();
                        match self.peek() {
                            Some('.') => {
                                self.step();
                                self.addToken(Token::Range(RangeKind::Exclusive))
                            }
                            _ => self.addToken(Token::Misc(MiscKind::Dot)),
                        }
                    }
                    '!' => {
                        self.step();
                        match self.peek() {
                            Some('=') => {
                                self.step();
                                self.addToken(Token::Op(OperatorKind::NotEqual))
                            }
                            _ => self.addToken(Token::Misc(MiscKind::ExclamationMark)),
                        }
                    }
                    '"' => {
                        self.processString();
                    }
                    _ => {
                        self.addError(LexerError::UnsupportedCharacter(c, self.span.clone()));
                        self.step();
                    }
                },
                None => break,
            }
        }
        self.addToken(Token::EOF);
        (self.tokens.clone(), self.errors.clone())
    }
}
