use crate::siko::syntax::{
    Expr::{BinaryOp, Branch, Expr},
    Statement::{Block, Statement, StatementKind},
};

use super::{
    Parser::*,
    Pattern::PatternParser,
    Token::{ArrowKind, BracketKind, KeywordKind, MiscKind, OperatorKind, Token, TokenKind},
};

pub trait ExprParser {
    fn parseBlock(&mut self) -> Block;
    fn parseStatement(&mut self) -> (StatementKind, SemicolonRequirement);
    fn parseIf(&mut self) -> Expr;
    fn parseFor(&mut self) -> Expr;
    fn parseMatch(&mut self) -> Expr;
    fn parseFieldAccessOrCall(&mut self) -> Expr;
    fn parseBinaryOp(&mut self, index: usize) -> Expr;
    fn parseExpr(&mut self) -> Expr;
    fn parsePrimary(&mut self) -> Expr;
    fn callNext(&mut self, index: usize) -> Expr;
}

pub enum SemicolonRequirement {
    Optional,
    TrailingOptional,
    Required,
}

impl ExprParser for Parser {
    fn parseBlock(&mut self) -> Block {
        let mut statements = Vec::new();
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
        Block { statements }
    }

    fn parseIf(&mut self) -> Expr {
        self.expect(TokenKind::Keyword(KeywordKind::If));
        let cond = self.parseExpr();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        self.undo();
        let trueBranch = self.parseExpr();
        let falseBranch = if self.check(TokenKind::Keyword(KeywordKind::Else)) {
            self.expect(TokenKind::Keyword(KeywordKind::Else));
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            self.undo();
            let falseBranch = self.parseExpr();
            Some(Box::new(falseBranch))
        } else {
            None
        };
        Expr::If(Box::new(cond), Box::new(trueBranch), falseBranch)
    }

    fn parseFor(&mut self) -> Expr {
        self.expect(TokenKind::Keyword(KeywordKind::For));
        let pattern = self.parsePattern();
        self.expect(TokenKind::Keyword(KeywordKind::In));
        let source = self.parseExpr();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        self.undo();
        let body = self.parseExpr();
        Expr::For(pattern, Box::new(source), Box::new(body))
    }

    fn parseMatch(&mut self) -> Expr {
        self.expect(TokenKind::Keyword(KeywordKind::Match));
        let body = self.parseExpr();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        let mut branches = Vec::new();
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            let pattern = self.parsePattern();
            self.expect(TokenKind::Arrow(ArrowKind::Right));
            let body = self.parseExpr();
            if self.check(TokenKind::Misc(MiscKind::Comma)) {
                self.expect(TokenKind::Misc(MiscKind::Comma));
            }
            branches.push(Branch { pattern, body });
        }
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Expr::Match(Box::new(body), branches)
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
            TokenKind::Keyword(KeywordKind::Match) => {
                let expr = self.parseMatch();
                (StatementKind::Expr(expr), SemicolonRequirement::Optional)
            }
            TokenKind::Keyword(KeywordKind::Let) => {
                self.expect(TokenKind::Keyword(KeywordKind::Let));
                let pattern = self.parsePattern();
                self.expect(TokenKind::Misc(MiscKind::Equal));
                let rhs = self.parseExpr();
                (
                    StatementKind::Let(pattern, rhs),
                    SemicolonRequirement::Required,
                )
            }
            _ => {
                let expr = self.parseExpr();
                if self.check(TokenKind::Misc(MiscKind::Equal)) {
                    self.expect(TokenKind::Misc(MiscKind::Equal));
                    let rhs = self.parseExpr();
                    (
                        StatementKind::Assign(expr, rhs),
                        SemicolonRequirement::Required,
                    )
                } else {
                    (
                        StatementKind::Expr(expr),
                        SemicolonRequirement::TrailingOptional,
                    )
                }
            }
        }
    }

    fn parseFieldAccessOrCall(&mut self) -> Expr {
        let mut current = self.parsePrimary();
        loop {
            if self.check(TokenKind::Misc(MiscKind::Dot)) {
                self.expect(TokenKind::Misc(MiscKind::Dot));
                let name = self.parseVarIdentifier();
                current = Expr::FieldAccess(Box::new(current), name);
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
                current = Expr::Call(Box::new(current), args);
            } else {
                break;
            }
        }
        return current;
    }

    fn parseBinaryOp(&mut self, index: usize) -> Expr {
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
                left = Expr::BinaryOp(op, Box::new(left), Box::new(rhs));
            } else {
                break;
            }
        }
        return left;
    }

    fn callNext(&mut self, index: usize) -> Expr {
        if index + 1 >= self.opTable.len() {
            self.parseFieldAccessOrCall()
        } else {
            self.parseBinaryOp(index + 1)
        }
    }

    fn parsePrimary(&mut self) -> Expr {
        match self.peek() {
            TokenKind::VarIdentifier => {
                let value = self.parseVarIdentifier();
                Expr::Value(value)
            }
            TokenKind::TypeIdentifier => {
                let value = self.parseTypeIdentifier();
                Expr::Name(value)
            }
            TokenKind::StringLiteral => {
                let tokenInfo = self.current().clone();
                self.step();
                if let Token::StringLiteral(value) = tokenInfo.token {
                    Expr::StringLiteral(value)
                } else {
                    unreachable!()
                }
            }
            TokenKind::IntegerLiteral => {
                let tokenInfo = self.current().clone();
                self.step();
                if let Token::IntegerLiteral(value) = tokenInfo.token {
                    Expr::IntegerLiteral(value)
                } else {
                    unreachable!()
                }
            }
            TokenKind::CharLiteral => {
                let tokenInfo = self.current().clone();
                self.step();
                if let Token::CharLiteral(value) = tokenInfo.token {
                    Expr::CharLiteral(value)
                } else {
                    unreachable!()
                }
            }
            TokenKind::Keyword(KeywordKind::ValueSelf) => {
                self.expect(TokenKind::Keyword(KeywordKind::ValueSelf));
                Expr::SelfValue
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
                Expr::Return(arg)
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
                Expr::Continue(arg)
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
                Expr::Break(arg)
            }
            TokenKind::Keyword(KeywordKind::If) => self.parseIf(),
            TokenKind::Keyword(KeywordKind::For) => self.parseFor(),
            TokenKind::Keyword(KeywordKind::Match) => self.parseMatch(),
            TokenKind::LeftBracket(BracketKind::Curly) => {
                let block = self.parseBlock();
                Expr::Block(block)
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
                    Expr::Tuple(args)
                }
            }
            kind => self.reportError2("<expr>", kind),
        }
    }

    fn parseExpr(&mut self) -> Expr {
        self.parseBinaryOp(0)
    }
}
