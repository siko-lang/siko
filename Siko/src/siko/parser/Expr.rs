use crate::siko::{
    location::Location::{Location, Span},
    qualifiedname::{getFalseName, getTrueName},
    syntax::{
        Expr::{BinaryOp, Branch, Expr, SimpleExpr, UnaryOp},
        Identifier::Identifier,
        Pattern::{Pattern, SimplePattern},
        Statement::{Block, Statement, StatementKind},
    },
};

use super::{
    Parser::*,
    Pattern::PatternParser,
    Token::{ArrowKind, BracketKind, KeywordKind, MiscKind, OperatorKind, Token, TokenKind},
    Type::TypeParser,
};

pub trait ExprParser {
    fn parseBlock(&mut self) -> Block;
    fn parseStatement(&mut self) -> (StatementKind, SemicolonRequirement);
    fn parseIf(&mut self) -> Expr;
    fn parseFor(&mut self) -> Expr;
    fn parseLoop(&mut self) -> Expr;
    fn parseMatch(&mut self) -> Expr;
    fn parseFieldAccessOrCall(&mut self) -> Expr;
    fn parseBinaryOp(&mut self, index: usize) -> Expr;
    fn parseExpr(&mut self) -> Expr;
    fn parseUnary(&mut self) -> Expr;
    fn parsePrimary(&mut self) -> Expr;
    fn callNext(&mut self, index: usize) -> Expr;
    fn buildExpr(&mut self, e: SimpleExpr, start: Span) -> Expr;
}

pub enum SemicolonRequirement {
    Optional,
    TrailingOptional,
    Required,
}

impl<'a> ExprParser for Parser<'a> {
    fn parseBlock(&mut self) -> Block {
        let mut statements = Vec::new();
        let start = self.currentSpan();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            let (statementKind, requirement) = self.parseStatement();
            let trailing = self.check(TokenKind::RightBracket(BracketKind::Curly));
            let mut hasSemicolon = false;
            match requirement {
                SemicolonRequirement::Optional => {
                    if self.check(TokenKind::Misc(MiscKind::Semicolon)) {
                        hasSemicolon = true;
                        self.expect(TokenKind::Misc(MiscKind::Semicolon));
                    }
                }
                SemicolonRequirement::TrailingOptional => {
                    if trailing {
                        if self.check(TokenKind::Misc(MiscKind::Semicolon)) {
                            hasSemicolon = true;
                            self.expect(TokenKind::Misc(MiscKind::Semicolon));
                        }
                    } else {
                        hasSemicolon = true;
                        self.expect(TokenKind::Misc(MiscKind::Semicolon));
                    }
                }
                SemicolonRequirement::Required => {
                    hasSemicolon = true;
                    self.expect(TokenKind::Misc(MiscKind::Semicolon));
                }
            }

            statements.push(Statement {
                kind: statementKind,
                hasSemicolon,
            });
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        let end = self.currentSpan();
        Block {
            statements,
            location: Location::new(self.fileId.clone(), start.merge(end)),
        }
    }

    fn buildExpr(&mut self, e: SimpleExpr, start: Span) -> Expr {
        Expr {
            expr: e,
            location: Location::new(self.fileId.clone(), start.merge(self.currentSpan())),
        }
    }

    fn parseIf(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::If));
        let cond = self.parseExpr();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        self.undo();
        let mut branches = Vec::new();
        let trueBranch = self.parseExpr();
        branches.push(Branch {
            pattern: Pattern {
                pattern: SimplePattern::Named(
                    Identifier::new(&getTrueName().toString(), self.currentLocation()),
                    Vec::new(),
                ),
                location: self.currentLocation(),
            },
            body: trueBranch,
        });
        if self.check(TokenKind::Keyword(KeywordKind::Else)) {
            self.expect(TokenKind::Keyword(KeywordKind::Else));
            if !self.check(TokenKind::Keyword(KeywordKind::If)) {
                self.expect(TokenKind::LeftBracket(BracketKind::Curly));
                self.undo();
            }
            let falseBranch = self.parseExpr();
            branches.push(Branch {
                pattern: Pattern {
                    pattern: SimplePattern::Named(
                        Identifier::new(&getFalseName().toString(), self.currentLocation()),
                        Vec::new(),
                    ),
                    location: self.currentLocation(),
                },
                body: falseBranch,
            });
        } else {
            branches.push(Branch {
                pattern: Pattern {
                    pattern: SimplePattern::Named(
                        Identifier::new(&getFalseName().toString(), self.currentLocation()),
                        Vec::new(),
                    ),
                    location: self.currentLocation(),
                },
                body: Expr {
                    expr: SimpleExpr::Block(Block {
                        statements: Vec::new(),
                        location: self.currentLocation(),
                    }),
                    location: self.currentLocation(),
                },
            })
        }

        self.buildExpr(SimpleExpr::Match(Box::new(cond), branches), start)
    }

    fn parseFor(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::For));
        let pattern = self.parsePattern();
        self.expect(TokenKind::Keyword(KeywordKind::In));
        let source = self.parseExpr();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        self.undo();
        let body = self.parseExpr();
        self.buildExpr(SimpleExpr::For(pattern, Box::new(source), Box::new(body)), start)
    }

    fn parseLoop(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::Loop));
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            self.undo();
            let body = self.parseExpr();
            let pattern = Pattern {
                pattern: SimplePattern::Tuple(Vec::new()),
                location: self.currentLocation(),
            };
            let init = Expr {
                expr: SimpleExpr::Tuple(Vec::new()),
                location: self.currentLocation(),
            };
            self.buildExpr(SimpleExpr::Loop(pattern, Box::new(init), Box::new(body)), start)
        } else {
            let pattern = self.parsePattern();
            self.expect(TokenKind::Misc(MiscKind::Equal));
            let init = self.parseExpr();
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            self.undo();
            let body = self.parseExpr();
            self.buildExpr(SimpleExpr::Loop(pattern, Box::new(init), Box::new(body)), start)
        }
    }

    fn parseMatch(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::Match));
        let body = self.parseExpr();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        let mut branches = Vec::new();
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            let pattern = self.parsePattern();
            self.expect(TokenKind::Arrow(ArrowKind::Right));
            let body = if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
                let block = self.parseBlock();
                let expr = self.buildExpr(SimpleExpr::Block(block), start.clone());
                expr
            } else {
                let expr = self.parseExpr();
                if self.check(TokenKind::Misc(MiscKind::Comma)) {
                    self.expect(TokenKind::Misc(MiscKind::Comma));
                }
                expr
            };
            branches.push(Branch { pattern, body });
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        self.buildExpr(SimpleExpr::Match(Box::new(body), branches), start.clone())
    }

    fn parseStatement(&mut self) -> (StatementKind, SemicolonRequirement) {
        match self.peek() {
            TokenKind::Keyword(KeywordKind::If) => {
                let expr = self.parseIf();
                (StatementKind::Expr(expr), SemicolonRequirement::Optional)
            }
            TokenKind::Keyword(KeywordKind::For) => {
                let expr = self.parseFor();
                (StatementKind::Expr(expr), SemicolonRequirement::Optional)
            }
            TokenKind::Keyword(KeywordKind::Loop) => {
                let expr = self.parseLoop();
                (StatementKind::Expr(expr), SemicolonRequirement::Optional)
            }
            TokenKind::Keyword(KeywordKind::Match) => {
                let expr = self.parseMatch();
                (StatementKind::Expr(expr), SemicolonRequirement::Optional)
            }
            TokenKind::Keyword(KeywordKind::Let) => {
                self.expect(TokenKind::Keyword(KeywordKind::Let));
                let pattern = self.parsePattern();
                let mut ty = None;
                if self.peek() == TokenKind::Misc(MiscKind::Colon) {
                    self.expect(TokenKind::Misc(MiscKind::Colon));
                    ty = Some(self.parseType());
                }
                self.expect(TokenKind::Misc(MiscKind::Equal));
                let rhs = self.parseExpr();
                (StatementKind::Let(pattern, rhs, ty), SemicolonRequirement::Required)
            }
            _ => {
                let expr = self.parseExpr();
                if self.check(TokenKind::Misc(MiscKind::Equal)) {
                    self.expect(TokenKind::Misc(MiscKind::Equal));
                    let rhs = self.parseExpr();
                    (StatementKind::Assign(expr, rhs), SemicolonRequirement::Required)
                } else {
                    let semicolonRequirement = if let SimpleExpr::Block(_) = &expr.expr {
                        SemicolonRequirement::Optional
                    } else {
                        SemicolonRequirement::TrailingOptional
                    };
                    (StatementKind::Expr(expr), semicolonRequirement)
                }
            }
        }
    }

    fn parseFieldAccessOrCall(&mut self) -> Expr {
        let start = self.currentSpan();
        let mut current = self.parsePrimary();
        loop {
            if self.check(TokenKind::Misc(MiscKind::Dot)) {
                self.expect(TokenKind::Misc(MiscKind::Dot));
                if self.peek() == TokenKind::IntegerLiteral {
                    let name = self.parseIntegerLiteral();
                    current = self.buildExpr(SimpleExpr::TupleIndex(Box::new(current), name), start.clone());
                } else {
                    let name = self.parseVarIdentifier();
                    current = self.buildExpr(SimpleExpr::FieldAccess(Box::new(current), name), start.clone());
                }
            } else if self.check(TokenKind::LeftBracket(BracketKind::Paren)) {
                self.expect(TokenKind::LeftBracket(BracketKind::Paren));
                let mut args = Vec::new();
                while !self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                    let arg = self.parseExpr();
                    args.push(arg);
                    if self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                        break;
                    } else {
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                    }
                }
                self.expect(TokenKind::RightBracket(BracketKind::Paren));
                if let SimpleExpr::FieldAccess(receiver, name) = current.expr {
                    current = self.buildExpr(SimpleExpr::MethodCall(receiver, name, args), start.clone());
                } else {
                    current = self.buildExpr(SimpleExpr::Call(Box::new(current), args), start.clone());
                }
            } else {
                break;
            }
        }
        return current;
    }

    fn parseBinaryOp(&mut self, index: usize) -> Expr {
        let start = self.currentSpan();
        let mut left = self.callNext(index);
        loop {
            let ops = &self.opTable[index];
            let mut matchingOp = None;
            for op in ops {
                if self.check(TokenKind::Op(*op)) {
                    matchingOp = Some(op.clone());
                    break;
                }
            }
            if let Some(op) = matchingOp {
                self.expect(TokenKind::Op(op));
                let rhs = self.callNext(index);
                let op = match op {
                    OperatorKind::And => BinaryOp::And,
                    OperatorKind::Or => BinaryOp::Or,
                    OperatorKind::Add => BinaryOp::Add,
                    OperatorKind::Sub => BinaryOp::Sub,
                    OperatorKind::Mul => BinaryOp::Mul,
                    OperatorKind::Div => BinaryOp::Div,
                    OperatorKind::Equal => BinaryOp::Equal,
                    OperatorKind::NotEqual => BinaryOp::NotEqual,
                    OperatorKind::LessThan => BinaryOp::LessThan,
                    OperatorKind::GreaterThan => BinaryOp::GreaterThan,
                    OperatorKind::LessThanOrEqual => BinaryOp::LessThanOrEqual,
                    OperatorKind::GreaterThanOrEqual => BinaryOp::GreaterThanOrEqual,
                };
                left = self.buildExpr(SimpleExpr::BinaryOp(op, Box::new(left), Box::new(rhs)), start.clone());
            } else {
                break;
            }
        }
        return left;
    }

    fn callNext(&mut self, index: usize) -> Expr {
        if index + 1 >= self.opTable.len() {
            self.parseUnary()
        } else {
            self.parseBinaryOp(index + 1)
        }
    }

    fn parseUnary(&mut self) -> Expr {
        match self.peek() {
            TokenKind::Misc(MiscKind::ExclamationMark) => {
                let start = self.currentSpan();
                self.expect(TokenKind::Misc(MiscKind::ExclamationMark));
                let expr = self.parseFieldAccessOrCall();
                self.buildExpr(SimpleExpr::UnaryOp(UnaryOp::Not, Box::new(expr)), start)
            }
            _ => self.parseFieldAccessOrCall(),
        }
    }
    fn parsePrimary(&mut self) -> Expr {
        let start = self.currentSpan();
        match self.peek() {
            TokenKind::VarIdentifier => {
                let value = self.parseVarIdentifier();
                self.buildExpr(SimpleExpr::Value(value), start)
            }
            TokenKind::TypeIdentifier => {
                let mut value = self.parseTypeIdentifier();
                while self.check(TokenKind::Misc(MiscKind::Dot)) {
                    value.dot(self.currentLocation());
                    self.expect(TokenKind::Misc(MiscKind::Dot));
                    if self.check(TokenKind::VarIdentifier) {
                        let id = self.parseVarIdentifier();
                        value.merge(id);
                        break;
                    }
                    if self.check(TokenKind::TypeIdentifier) {
                        let id = self.parseTypeIdentifier();
                        value.merge(id);
                        continue;
                    }
                    self.reportError2("<identifier>", self.peek());
                }
                self.buildExpr(SimpleExpr::Name(value), start)
            }
            TokenKind::StringLiteral => {
                let literal = self.parseStringLiteral();
                self.buildExpr(SimpleExpr::StringLiteral(literal), start)
            }
            TokenKind::IntegerLiteral => {
                let literal = self.parseIntegerLiteral();
                self.buildExpr(SimpleExpr::IntegerLiteral(literal), start)
            }
            TokenKind::CharLiteral => {
                let tokenInfo = self.current().clone();
                self.step();
                if let Token::CharLiteral(value) = tokenInfo.token {
                    self.buildExpr(SimpleExpr::CharLiteral(value), start)
                } else {
                    unreachable!()
                }
            }
            TokenKind::Keyword(KeywordKind::ValueSelf) => {
                self.expect(TokenKind::Keyword(KeywordKind::ValueSelf));
                self.buildExpr(SimpleExpr::SelfValue, start)
            }
            TokenKind::Keyword(KeywordKind::Return) => {
                self.expect(TokenKind::Keyword(KeywordKind::Return));
                let arg = if self.check(TokenKind::Misc(MiscKind::Semicolon))
                    || self.check(TokenKind::Misc(MiscKind::Comma))
                {
                    None
                } else {
                    Some(Box::new(self.parseExpr()))
                };
                self.buildExpr(SimpleExpr::Return(arg), start)
            }
            TokenKind::Keyword(KeywordKind::Continue) => {
                self.expect(TokenKind::Keyword(KeywordKind::Continue));
                let arg = if self.check(TokenKind::Misc(MiscKind::Semicolon))
                    || self.check(TokenKind::Misc(MiscKind::Comma))
                {
                    None
                } else {
                    Some(Box::new(self.parseExpr()))
                };
                self.buildExpr(SimpleExpr::Continue(arg), start)
            }
            TokenKind::Keyword(KeywordKind::Break) => {
                self.expect(TokenKind::Keyword(KeywordKind::Break));
                let arg = if self.check(TokenKind::Misc(MiscKind::Semicolon))
                    || self.check(TokenKind::Misc(MiscKind::Comma))
                {
                    None
                } else {
                    Some(Box::new(self.parseExpr()))
                };
                self.buildExpr(SimpleExpr::Break(arg), start)
            }
            TokenKind::Keyword(KeywordKind::If) => self.parseIf(),
            TokenKind::Keyword(KeywordKind::For) => self.parseFor(),
            TokenKind::Keyword(KeywordKind::Loop) => self.parseLoop(),
            TokenKind::Keyword(KeywordKind::Match) => self.parseMatch(),
            TokenKind::LeftBracket(BracketKind::Curly) => {
                let block = self.parseBlock();
                self.buildExpr(SimpleExpr::Block(block), start)
            }
            TokenKind::LeftBracket(BracketKind::Paren) => {
                self.expect(TokenKind::LeftBracket(BracketKind::Paren));
                let mut args = Vec::new();
                let mut commaAtEnd = false;
                while !self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                    let arg = self.parseExpr();
                    args.push(arg);
                    if self.check(TokenKind::RightBracket(BracketKind::Paren)) {
                        commaAtEnd = false;
                        break;
                    } else {
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                        commaAtEnd = true;
                    }
                }
                self.expect(TokenKind::RightBracket(BracketKind::Paren));
                if args.len() == 1 && !commaAtEnd {
                    args.remove(0)
                } else {
                    self.buildExpr(SimpleExpr::Tuple(args), start)
                }
            }
            TokenKind::Misc(MiscKind::Ampersand) => {
                self.expect(TokenKind::Misc(MiscKind::Ampersand));
                let arg = self.parseExpr();
                self.buildExpr(SimpleExpr::Ref(Box::new(arg)), start)
            }
            kind => self.reportError2("<expr>", kind),
        }
    }

    fn parseExpr(&mut self) -> Expr {
        self.parseBinaryOp(0)
    }
}
