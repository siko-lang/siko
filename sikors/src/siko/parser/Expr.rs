use crate::siko::syntax::{
    Expr::Expr,
    Statement::{Block, Statement, StatementKind},
};

use super::{
    Parser::*,
    Pattern::PatternParser,
    Token::{BracketKind, KeywordKind, MiscKind, OperatorKind, TokenKind},
};

pub trait ExprParser {
    fn parseBlock(&mut self) -> Block;
    fn parseStatement(&mut self) -> (StatementKind, SemicolonRequirement);
    fn parseIf(&mut self) -> Expr;
    fn parseBinaryOp(&mut self, index: usize) -> Expr;
    fn parseExpr(&mut self) -> Expr;
    fn parseFunctionCall(&mut self) -> Expr;
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
        let trueBranch = self.parseExpr();
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        self.expect(TokenKind::Keyword(KeywordKind::Else));
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        let falseBranch = self.parseExpr();
        self.expect(TokenKind::RightBracket(BracketKind::Curly));
        Expr::If(Box::new(cond), Box::new(trueBranch), Box::new(falseBranch))
    }

    fn parseStatement(&mut self) -> (StatementKind, SemicolonRequirement) {
        match self.peek() {
            TokenKind::Keyword(KeywordKind::If) => {
                let expr = self.parseIf();
                (StatementKind::Expr(expr), SemicolonRequirement::Optional)
            }
            TokenKind::Keyword(KeywordKind::Let) => {
                self.expect(TokenKind::Keyword(KeywordKind::Let));
                let pattern = self.parsePattern();
                self.expect(TokenKind::Op(OperatorKind::Equal));
                let rhs = self.parseExpr();
                (
                    StatementKind::Let(pattern, rhs),
                    SemicolonRequirement::Required,
                )
            }
            _ => {
                let expr = self.parseExpr();
                if self.check(TokenKind::Op(OperatorKind::Equal)) {
                    self.expect(TokenKind::Op(OperatorKind::Equal));
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

    fn parseBinaryOp(&mut self, index: usize) -> Expr {
        let left = self.parseFunctionCall();
        return left;
    }

    fn callNext(&mut self, index: usize) -> Expr {
        if index >= self.opTable.len() {
            self.parseFunctionCall()
        } else {
            self.parseBinaryOp(index + 1)
        }
    }

    fn parseFunctionCall(&mut self) -> Expr {
        let callable = self.parsePrimary();
        if self.check(TokenKind::LeftBracket(BracketKind::Paren)) {
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
            Expr::Call(Box::new(callable), args)
        } else {
            callable
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
            TokenKind::Keyword(KeywordKind::If) => self.parseIf(),
            kind => self.reportError2("<expr>", kind),
        }
    }

    fn parseExpr(&mut self) -> Expr {
        self.parseBinaryOp(0)
    }
}
