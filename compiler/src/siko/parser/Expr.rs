use crate::siko::{
    location::Location::{Location, Span},
    parser::Token::RangeKind,
    qualifiedname::builtins::{getFalseName, getRangeCtorName, getTrueName},
    syntax::{
        Expr::{BinaryOp, Branch, ContextHandler, Expr, FunctionArg, SimpleExpr, UnaryOp, With},
        Identifier::Identifier,
        Pattern::{Pattern, SimplePattern},
        Statement::{Block, Statement, StatementKind},
    },
    util::error,
};

use super::{
    Parser::*,
    Pattern::PatternParser,
    Token::{ArrowKind, BracketKind, KeywordKind, MiscKind, OperatorKind, TokenKind},
    Type::TypeParser,
};

pub trait ExprParser {
    fn parseBlock(&mut self) -> Block;
    fn parseStatement(&mut self) -> (StatementKind, SemicolonRequirement);
    fn parseIf(&mut self) -> Expr;
    fn parseFor(&mut self) -> Expr;
    fn parseLoop(&mut self) -> Expr;
    fn parseWhile(&mut self) -> Expr;
    fn parseMatch(&mut self) -> Expr;
    fn parseMatchIf(&mut self) -> Expr;
    fn parseWith(&mut self) -> Expr;
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
        let end = self.endSpan();
        Block {
            statements,
            location: Location::new(self.fileId.clone(), start.merge(end)),
        }
    }

    fn buildExpr(&mut self, e: SimpleExpr, start: Span) -> Expr {
        let end = self.endSpan();
        Expr {
            expr: e,
            location: Location::new(self.fileId.clone(), start.merge(end)),
        }
    }

    fn parseIf(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::If));
        if self.check(TokenKind::Keyword(KeywordKind::Let)) {
            self.expect(TokenKind::Keyword(KeywordKind::Let));
            let pattern = self.parsePattern();
            self.expect(TokenKind::Misc(MiscKind::Equal));
            let cond = self.parseExpr();
            let block = self.parseBlock();
            let mut branches = Vec::new();
            let trueBranch = self.buildExpr(SimpleExpr::Block(block), start.clone());
            branches.push(Branch {
                pattern,
                body: trueBranch,
            });
            let elseBranch = if self.check(TokenKind::Keyword(KeywordKind::Else)) {
                self.expect(TokenKind::Keyword(KeywordKind::Else));
                if self.check(TokenKind::Keyword(KeywordKind::If)) {
                    self.parseIf()
                } else {
                    let block = self.parseBlock();
                    self.buildExpr(SimpleExpr::Block(block), start.clone())
                }
            } else {
                Expr {
                    expr: SimpleExpr::Block(Block {
                        statements: Vec::new(),
                        location: self.currentLocation(),
                    }),
                    location: self.currentLocation(),
                }
            };
            branches.push(Branch {
                pattern: Pattern {
                    pattern: SimplePattern::Wildcard,
                    location: self.currentLocation(),
                },
                body: elseBranch,
            });
            return self.buildExpr(SimpleExpr::Match(Box::new(cond), branches), start);
        }
        let cond = self.parseExpr();
        let block = self.parseBlock();
        let mut branches = Vec::new();
        let trueBranch = self.buildExpr(SimpleExpr::Block(block), start.clone());
        branches.push(Branch {
            pattern: Pattern {
                pattern: SimplePattern::Named(
                    Identifier::new(getTrueName().toString(), self.currentLocation()),
                    Vec::new(),
                ),
                location: self.currentLocation(),
            },
            body: trueBranch,
        });
        if self.check(TokenKind::Keyword(KeywordKind::Else)) {
            self.expect(TokenKind::Keyword(KeywordKind::Else));
            if !self.check(TokenKind::Keyword(KeywordKind::If)) {
                let block = self.parseBlock();
                let falseBranch = self.buildExpr(SimpleExpr::Block(block), start.clone());
                branches.push(Branch {
                    pattern: Pattern {
                        pattern: SimplePattern::Named(
                            Identifier::new(getFalseName().toString(), self.currentLocation()),
                            Vec::new(),
                        ),
                        location: self.currentLocation(),
                    },
                    body: falseBranch,
                });
            } else {
                let falseBranch = self.parseIf();
                branches.push(Branch {
                    pattern: Pattern {
                        pattern: SimplePattern::Named(
                            Identifier::new(getFalseName().toString(), self.currentLocation()),
                            Vec::new(),
                        ),
                        location: self.currentLocation(),
                    },
                    body: falseBranch,
                });
            }
        } else {
            branches.push(Branch {
                pattern: Pattern {
                    pattern: SimplePattern::Named(
                        Identifier::new(getFalseName().toString(), self.currentLocation()),
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
        let body = self.parseBlock();
        let end = self.endSpan();
        let location = Location::new(self.fileId.clone(), start.merge(end));
        let mut statements = Vec::new();
        let source = self.buildExpr(
            SimpleExpr::Call(
                Box::new(Expr {
                    expr: SimpleExpr::Value(Identifier::new(
                        "IntoIterator.intoIterator".to_string(),
                        location.clone(),
                    )),
                    location: location.clone(),
                }),
                vec![FunctionArg::Positional(source)],
            ),
            start.clone(),
        );
        let iter = Pattern {
            pattern: SimplePattern::Bind(Identifier::new(".iter".to_string(), location.clone()), true),
            location: location.clone(),
        };
        statements.push(Statement {
            kind: StatementKind::Let(iter, source, None),
            hasSemicolon: false,
        });
        let someBranch = Branch {
            pattern: Pattern {
                pattern: SimplePattern::Named(
                    Identifier::new("Option.Some".to_string(), location.clone()),
                    vec![pattern],
                ),
                location: location.clone(),
            },
            body: Expr {
                expr: SimpleExpr::Block(Block {
                    statements: vec![Statement {
                        kind: StatementKind::Expr(Expr {
                            expr: SimpleExpr::Block(body),
                            location: location.clone(),
                        }),
                        hasSemicolon: true,
                    }],
                    location: location.clone(),
                }),
                location: location.clone(),
            },
        };
        let noneBranch = Branch {
            pattern: Pattern {
                pattern: SimplePattern::Named(Identifier::new("Option.None".to_string(), location.clone()), Vec::new()),
                location: location.clone(),
            },
            body: Expr {
                expr: SimpleExpr::Block(Block {
                    statements: vec![Statement {
                        kind: StatementKind::Expr(Expr {
                            expr: SimpleExpr::Break(None),
                            location: location.clone(),
                        }),
                        hasSemicolon: true,
                    }],
                    location: location.clone(),
                }),
                location: location.clone(),
            },
        };
        let nextCall = Expr {
            expr: SimpleExpr::MethodCall(
                Box::new(Expr {
                    expr: SimpleExpr::Value(Identifier::new(".iter".to_string(), location.clone())),
                    location: location.clone(),
                }),
                Identifier::new("next".to_string(), location.clone()),
                vec![],
            ),
            location: location.clone(),
        };
        let matchExpr = Expr {
            expr: SimpleExpr::Match(Box::new(nextCall), vec![someBranch, noneBranch]),
            location: location.clone(),
        };
        let loopBody = Expr {
            expr: SimpleExpr::Block(Block {
                statements: vec![Statement {
                    kind: StatementKind::Expr(matchExpr),
                    hasSemicolon: true,
                }],
                location: location.clone(),
            }),
            location: location.clone(),
        };
        let loopExpr = Expr {
            expr: SimpleExpr::Loop(
                Pattern {
                    pattern: SimplePattern::Tuple(Vec::new()),
                    location: location.clone(),
                },
                Box::new(Expr {
                    expr: SimpleExpr::Tuple(Vec::new()),
                    location: location.clone(),
                }),
                Box::new(loopBody),
            ),
            location: location.clone(),
        };
        statements.push(Statement {
            kind: StatementKind::Expr(loopExpr),
            hasSemicolon: true,
        });
        let block = SimpleExpr::Block(Block {
            statements: statements,
            location: location.clone(),
        });
        self.buildExpr(block, start)
    }

    fn parseLoop(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::Loop));
        if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            self.undo();
            let body = self.parseBlock();
            let pattern = Pattern {
                pattern: SimplePattern::Tuple(Vec::new()),
                location: self.currentLocation(),
            };
            let init = Expr {
                expr: SimpleExpr::Tuple(Vec::new()),
                location: self.currentLocation(),
            };
            self.buildExpr(
                SimpleExpr::Loop(
                    pattern,
                    Box::new(init),
                    Box::new(Expr {
                        expr: SimpleExpr::Block(body),
                        location: self.currentLocation(),
                    }),
                ),
                start,
            )
        } else {
            let pattern = self.parsePattern();
            self.expect(TokenKind::Misc(MiscKind::Equal));
            let init = self.parseExpr();
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            self.undo();
            let body = self.parseBlock();
            self.buildExpr(
                SimpleExpr::Loop(
                    pattern,
                    Box::new(init),
                    Box::new(Expr {
                        expr: SimpleExpr::Block(body),
                        location: self.currentLocation(),
                    }),
                ),
                start,
            )
        }
    }

    fn parseWhile(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::While));
        if self.check(TokenKind::Keyword(KeywordKind::Let)) {
            self.expect(TokenKind::Keyword(KeywordKind::Let));
            let pattern = self.parsePattern();
            self.expect(TokenKind::Misc(MiscKind::Equal));
            let value = self.parseExpr();
            self.expect(TokenKind::LeftBracket(BracketKind::Curly));
            self.undo();
            let block = self.parseBlock();
            let matchExpr = Expr {
                expr: SimpleExpr::Match(
                    Box::new(value),
                    vec![
                        Branch {
                            pattern,
                            body: Expr {
                                expr: SimpleExpr::Block(block),
                                location: self.currentLocation(),
                            },
                        },
                        Branch {
                            pattern: Pattern {
                                pattern: SimplePattern::Wildcard,
                                location: self.currentLocation(),
                            },
                            body: Expr {
                                expr: SimpleExpr::Break(None),
                                location: self.currentLocation(),
                            },
                        },
                    ],
                ),
                location: self.currentLocation(),
            };
            let loopBody = Expr {
                expr: SimpleExpr::Block(Block {
                    statements: vec![Statement {
                        kind: StatementKind::Expr(matchExpr),
                        hasSemicolon: true,
                    }],
                    location: self.currentLocation(),
                }),
                location: self.currentLocation(),
            };
            return self.buildExpr(
                SimpleExpr::Loop(
                    Pattern {
                        pattern: SimplePattern::Tuple(Vec::new()),
                        location: self.currentLocation(),
                    },
                    Box::new(Expr {
                        expr: SimpleExpr::Tuple(Vec::new()),
                        location: self.currentLocation(),
                    }),
                    Box::new(loopBody),
                ),
                start,
            );
        }
        let cond = self.parseExpr();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        self.undo();
        let body = self.parseBlock();
        let body = Expr {
            expr: SimpleExpr::Match(
                Box::new(cond),
                vec![
                    Branch {
                        pattern: Pattern {
                            pattern: SimplePattern::Named(
                                Identifier::new(getTrueName().toString(), self.currentLocation()),
                                Vec::new(),
                            ),
                            location: self.currentLocation(),
                        },
                        body: Expr {
                            expr: SimpleExpr::Block(body),
                            location: self.currentLocation(),
                        },
                    },
                    Branch {
                        pattern: Pattern {
                            pattern: SimplePattern::Named(
                                Identifier::new(getFalseName().toString(), self.currentLocation()),
                                Vec::new(),
                            ),
                            location: self.currentLocation(),
                        },
                        body: Expr {
                            expr: SimpleExpr::Break(None),
                            location: self.currentLocation(),
                        },
                    },
                ],
            ),
            location: self.currentLocation(),
        };
        let body = Expr {
            expr: SimpleExpr::Block(Block {
                statements: vec![Statement {
                    kind: StatementKind::Expr(body),
                    hasSemicolon: true,
                }],
                location: self.currentLocation(),
            }),
            location: self.currentLocation(),
        };
        self.buildExpr(
            SimpleExpr::Loop(
                Pattern {
                    pattern: SimplePattern::Tuple(Vec::new()),
                    location: self.currentLocation(),
                },
                Box::new(Expr {
                    expr: SimpleExpr::Tuple(Vec::new()),
                    location: self.currentLocation(),
                }),
                Box::new(body),
            ),
            start,
        )
    }

    fn parseMatchIf(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::If));
        let (body, bodyGiven) = if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            (self.buildExpr(SimpleExpr::Tuple(Vec::new()), start), false)
        } else {
            (self.parseExpr(), true)
        };
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        let mut branches = Vec::new();
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            let pattern = if self.check(TokenKind::Misc(MiscKind::Wildcard)) {
                self.expect(TokenKind::Misc(MiscKind::Wildcard));
                let pattern = Pattern {
                    pattern: SimplePattern::Wildcard,
                    location: self.currentLocation(),
                };
                self.expect(TokenKind::Arrow(ArrowKind::Right));
                pattern
            } else {
                let guardExpr = self.parseExpr();
                let pattern = if bodyGiven {
                    let loc = self.currentLocation();
                    let identifier = Identifier::new("value".to_string(), loc.clone());
                    let extendedGuardExpr = if self.check(TokenKind::Arrow(ArrowKind::Right)) {
                        let extendedGuardExpr = SimpleExpr::MethodCall(
                            Box::new(guardExpr),
                            Identifier::new("contains".to_string(), loc.clone()),
                            vec![FunctionArg::Positional(Expr {
                                expr: SimpleExpr::Value(identifier.clone()),
                                location: loc.clone(),
                            })],
                        );
                        self.expect(TokenKind::Arrow(ArrowKind::Right));
                        extendedGuardExpr
                    } else {
                        self.expect(TokenKind::Arrow(ArrowKind::DoubleRight));
                        let extendedGuardExpr = SimpleExpr::BinaryOp(
                            BinaryOp::Equal,
                            Box::new(guardExpr),
                            Box::new(Expr {
                                expr: SimpleExpr::Value(identifier.clone()),
                                location: loc.clone(),
                            }),
                        );
                        extendedGuardExpr
                    };
                    let guardExpr = Expr {
                        expr: extendedGuardExpr,
                        location: loc.clone(),
                    };
                    let pattern = Pattern {
                        pattern: SimplePattern::Guarded(
                            Box::new(Pattern {
                                pattern: SimplePattern::Bind(identifier, true),
                                location: loc.clone(),
                            }),
                            Box::new(guardExpr),
                        ),
                        location: loc.clone(),
                    };

                    pattern
                } else {
                    let pattern = Pattern {
                        pattern: SimplePattern::Guarded(
                            Box::new(Pattern {
                                pattern: SimplePattern::Bind(
                                    Identifier::new("value".to_string(), self.currentLocation()),
                                    false,
                                ),
                                location: self.currentLocation(),
                            }),
                            Box::new(guardExpr),
                        ),
                        location: self.currentLocation(),
                    };
                    self.expect(TokenKind::Arrow(ArrowKind::Right));
                    pattern
                };
                pattern
            };
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

    fn parseMatch(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::Match));
        if self.check(TokenKind::Keyword(KeywordKind::If)) {
            return self.parseMatchIf();
        }
        let body = self.parseExpr();
        self.expect(TokenKind::LeftBracket(BracketKind::Curly));
        let mut branches = Vec::new();
        while !self.check(TokenKind::RightBracket(BracketKind::Curly)) {
            let patternStart = self.currentSpan();
            let mut pattern = self.parsePattern();
            if self.check(TokenKind::Op(OperatorKind::BitOr)) {
                let mut patterns = vec![pattern];
                while self.check(TokenKind::Op(OperatorKind::BitOr)) {
                    self.expect(TokenKind::Op(OperatorKind::BitOr));
                    let p = self.parsePattern();
                    patterns.push(p);
                }
                pattern = Pattern {
                    pattern: SimplePattern::OrPattern(patterns),
                    location: self.currentLocation(),
                };
            }
            if self.check(TokenKind::Keyword(KeywordKind::If)) {
                self.expect(TokenKind::Keyword(KeywordKind::If));
                let guardExpr = self.parseExpr();
                let location = Location::new(self.fileId.clone(), patternStart.merge(self.currentSpan()));
                pattern = Pattern {
                    pattern: SimplePattern::Guarded(Box::new(pattern), Box::new(guardExpr)),
                    location: location.clone(),
                };
            }
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

    fn parseWith(&mut self) -> Expr {
        let start = self.currentSpan();
        self.expect(TokenKind::Keyword(KeywordKind::With));
        let mut contexts = Vec::new();
        while !self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
            let method = self.parseQualifiedVarName();
            self.expect(TokenKind::Misc(MiscKind::Equal));
            let optional = if self.check(TokenKind::Keyword(KeywordKind::Try)) {
                self.expect(TokenKind::Keyword(KeywordKind::Try));
                true
            } else {
                false
            };
            let handler = self.parseQualifiedVarName();
            contexts.push(ContextHandler {
                name: method,
                handler,
                optional,
            });
            if self.check(TokenKind::LeftBracket(BracketKind::Curly)) {
                break;
            } else {
                self.expect(TokenKind::Misc(MiscKind::Comma));
            }
        }
        let body = self.parseBlock();
        let body = Expr {
            expr: SimpleExpr::Block(body),
            location: self.currentLocation(),
        };
        let with = With {
            handlers: contexts,
            body,
        };
        self.buildExpr(SimpleExpr::With(Box::new(with)), start)
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
            TokenKind::Keyword(KeywordKind::While) => {
                let expr = self.parseWhile();
                (StatementKind::Expr(expr), SemicolonRequirement::Optional)
            }
            TokenKind::Keyword(KeywordKind::Match) => {
                let expr = self.parseMatch();
                (StatementKind::Expr(expr), SemicolonRequirement::Optional)
            }
            TokenKind::Keyword(KeywordKind::With) => {
                let expr = self.parseWith();
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
                } else if self.check(TokenKind::Op(OperatorKind::AddAssign)) {
                    self.expect(TokenKind::Op(OperatorKind::AddAssign));
                    let start = self.currentSpan();
                    let rhs = self.parseExpr();
                    let addExpr = self.buildExpr(
                        SimpleExpr::BinaryOp(BinaryOp::Add, Box::new(expr.clone()), Box::new(rhs)),
                        start,
                    );
                    (StatementKind::Assign(expr, addExpr), SemicolonRequirement::Required)
                } else if self.check(TokenKind::Op(OperatorKind::SubAssign)) {
                    self.expect(TokenKind::Op(OperatorKind::SubAssign));
                    let start = self.currentSpan();
                    let rhs = self.parseExpr();
                    let subExpr = self.buildExpr(
                        SimpleExpr::BinaryOp(BinaryOp::Sub, Box::new(expr.clone()), Box::new(rhs)),
                        start,
                    );
                    (StatementKind::Assign(expr, subExpr), SemicolonRequirement::Required)
                } else if self.check(TokenKind::Op(OperatorKind::MulAssign)) {
                    self.expect(TokenKind::Op(OperatorKind::MulAssign));
                    let start = self.currentSpan();
                    let rhs = self.parseExpr();
                    let subExpr = self.buildExpr(
                        SimpleExpr::BinaryOp(BinaryOp::Mul, Box::new(expr.clone()), Box::new(rhs)),
                        start,
                    );
                    (StatementKind::Assign(expr, subExpr), SemicolonRequirement::Required)
                } else if self.check(TokenKind::Op(OperatorKind::DivAssign)) {
                    self.expect(TokenKind::Op(OperatorKind::DivAssign));
                    let start = self.currentSpan();
                    let rhs = self.parseExpr();
                    let subExpr = self.buildExpr(
                        SimpleExpr::BinaryOp(BinaryOp::Div, Box::new(expr.clone()), Box::new(rhs)),
                        start,
                    );
                    (StatementKind::Assign(expr, subExpr), SemicolonRequirement::Required)
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
                    if self.check(TokenKind::Misc(MiscKind::Colon)) {
                        self.expect(TokenKind::Misc(MiscKind::Colon));
                        let name = if let SimpleExpr::Value(id) = arg.expr {
                            id
                        } else {
                            error("Expected identifier for named argument".to_string());
                        };
                        let value = self.parseExpr();
                        args.push(FunctionArg::Named(name, value));
                    } else {
                        args.push(FunctionArg::Positional(arg));
                    }
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
            } else if self.check(TokenKind::LeftBracket(BracketKind::Square)) {
                self.expect(TokenKind::LeftBracket(BracketKind::Square));
                let indexExpr = self.parseExpr();
                self.expect(TokenKind::RightBracket(BracketKind::Square));
                let call = SimpleExpr::MethodCall(
                    Box::new(current.clone()),
                    Identifier::new("index".to_string(), self.currentLocation()),
                    vec![FunctionArg::Positional(indexExpr.clone())],
                );
                current = self.buildExpr(call, start.clone());
            } else {
                break;
            }
        }
        return current;
    }

    fn parseBinaryOp(&mut self, index: usize) -> Expr {
        let start = self.currentSpan();
        let mut left = self.callNext(index);
        if self.check(TokenKind::Range(RangeKind::Exclusive)) {
            self.expect(TokenKind::Range(RangeKind::Exclusive));
            let end = self.parseExpr();
            return self.buildExpr(
                SimpleExpr::Call(
                    Box::new(Expr {
                        expr: SimpleExpr::Value(Identifier::new(getRangeCtorName().toString(), self.currentLocation())),
                        location: self.currentLocation(),
                    }),
                    vec![FunctionArg::Positional(left), FunctionArg::Positional(end)],
                ),
                start,
            );
        } else if self.check(TokenKind::Range(RangeKind::Inclusive)) {
            self.expect(TokenKind::Range(RangeKind::Inclusive));
            let end = self.parseExpr();
            let trueValue = self.buildExpr(
                SimpleExpr::Name(Identifier::new(getTrueName().toString(), self.currentLocation())),
                start.clone(),
            );
            return self.buildExpr(
                SimpleExpr::Call(
                    Box::new(Expr {
                        expr: SimpleExpr::Value(Identifier::new(getRangeCtorName().toString(), self.currentLocation())),
                        location: self.currentLocation(),
                    }),
                    vec![
                        FunctionArg::Positional(left),
                        FunctionArg::Positional(end),
                        FunctionArg::Positional(trueValue),
                    ],
                ),
                start,
            );
        }
        loop {
            if self.check(TokenKind::Keyword(KeywordKind::In)) {
                self.expect(TokenKind::Keyword(KeywordKind::In));
                let item = left;
                let container = self.parseExpr();
                let containsCall = Expr {
                    expr: SimpleExpr::MethodCall(
                        Box::new(container),
                        Identifier::new("contains".to_string(), self.currentLocation()),
                        vec![FunctionArg::Positional(item)],
                    ),
                    location: self.currentLocation(),
                };
                return self.buildExpr(containsCall.expr, start);
            }
            if self.check(TokenKind::Keyword(KeywordKind::Not)) {
                self.expect(TokenKind::Keyword(KeywordKind::Not));
                self.expect(TokenKind::Keyword(KeywordKind::In));
                let item = left;
                let container = self.parseExpr();
                let containsCall = Expr {
                    expr: SimpleExpr::MethodCall(
                        Box::new(container),
                        Identifier::new("notContains".to_string(), self.currentLocation()),
                        vec![FunctionArg::Positional(item)],
                    ),
                    location: self.currentLocation(),
                };
                return self.buildExpr(containsCall.expr, start);
            }
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
                    OperatorKind::ShiftLeft => BinaryOp::ShiftLeft,
                    OperatorKind::ShiftRight => BinaryOp::ShiftRight,
                    OperatorKind::BitAnd => BinaryOp::BitAnd,
                    OperatorKind::BitOr => BinaryOp::BitOr,
                    OperatorKind::BitXor => BinaryOp::BitXor,
                    OperatorKind::AddAssign => {
                        error("AddAssign is not supported in expressions".to_string());
                    }
                    OperatorKind::SubAssign => {
                        error("SubAssign is not supported in expressions".to_string());
                    }
                    OperatorKind::MulAssign => {
                        error("MulAssign is not supported in expressions".to_string());
                    }
                    OperatorKind::DivAssign => {
                        error("DivAssign is not supported in expressions".to_string());
                    }
                };
                if op == BinaryOp::And {
                    let trueBranch = Branch {
                        pattern: Pattern {
                            pattern: SimplePattern::Named(
                                Identifier::new(getTrueName().toString(), self.currentLocation()),
                                Vec::new(),
                            ),
                            location: self.currentLocation(),
                        },
                        body: rhs,
                    };
                    let falseBranch = Branch {
                        pattern: Pattern {
                            pattern: SimplePattern::Named(
                                Identifier::new(getFalseName().toString(), self.currentLocation()),
                                Vec::new(),
                            ),
                            location: self.currentLocation(),
                        },
                        body: self.buildExpr(
                            SimpleExpr::Call(
                                Box::new(Expr {
                                    expr: SimpleExpr::Value(Identifier::new(
                                        getFalseName().toString(),
                                        self.currentLocation(),
                                    )),
                                    location: self.currentLocation(),
                                }),
                                Vec::new(),
                            ),
                            start,
                        ),
                    };
                    left = self.buildExpr(SimpleExpr::Match(Box::new(left), vec![trueBranch, falseBranch]), start);
                } else if op == BinaryOp::Or {
                    let trueBranch = Branch {
                        pattern: Pattern {
                            pattern: SimplePattern::Named(
                                Identifier::new(getTrueName().toString(), self.currentLocation()),
                                Vec::new(),
                            ),
                            location: self.currentLocation(),
                        },
                        body: self.buildExpr(
                            SimpleExpr::Call(
                                Box::new(Expr {
                                    expr: SimpleExpr::Value(Identifier::new(
                                        getTrueName().toString(),
                                        self.currentLocation(),
                                    )),
                                    location: self.currentLocation(),
                                }),
                                Vec::new(),
                            ),
                            start,
                        ),
                    };
                    let falseBranch = Branch {
                        pattern: Pattern {
                            pattern: SimplePattern::Named(
                                Identifier::new(getFalseName().toString(), self.currentLocation()),
                                Vec::new(),
                            ),
                            location: self.currentLocation(),
                        },
                        body: rhs,
                    };
                    left = self.buildExpr(SimpleExpr::Match(Box::new(left), vec![trueBranch, falseBranch]), start);
                } else {
                    assert_ne!(op, BinaryOp::And);
                    assert_ne!(op, BinaryOp::Or);
                    left = self.buildExpr(SimpleExpr::BinaryOp(op, Box::new(left), Box::new(rhs)), start.clone());
                }
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
            TokenKind::Op(OperatorKind::Sub) => {
                let start = self.currentSpan();
                self.expect(TokenKind::Op(OperatorKind::Sub));
                let expr = self.parseFieldAccessOrCall();
                self.buildExpr(SimpleExpr::UnaryOp(UnaryOp::Neg, Box::new(expr)), start)
            }
            TokenKind::Op(OperatorKind::Mul) => {
                let start = self.currentSpan();
                self.expect(TokenKind::Op(OperatorKind::Mul));
                let expr = self.parseFieldAccessOrCall();
                self.buildExpr(SimpleExpr::UnaryOp(UnaryOp::Deref, Box::new(expr)), start)
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
                let value = self.parseQualifiedName();
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
                let literal = self.parseCharLiteral();
                self.buildExpr(SimpleExpr::CharLiteral(literal), start)
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
            TokenKind::Keyword(KeywordKind::While) => self.parseWhile(),
            TokenKind::Keyword(KeywordKind::Match) => self.parseMatch(),
            TokenKind::Keyword(KeywordKind::With) => self.parseWith(),
            TokenKind::Keyword(KeywordKind::Try) => {
                self.expect(TokenKind::Keyword(KeywordKind::Try));
                let arg = self.parseExpr();
                let matchExpr = Expr {
                    expr: SimpleExpr::Match(
                        Box::new(arg),
                        vec![
                            Branch {
                                pattern: Pattern {
                                    pattern: SimplePattern::Named(
                                        Identifier::new(format!("Result.Result.Ok"), self.currentLocation()),
                                        vec![Pattern {
                                            pattern: SimplePattern::Bind(
                                                Identifier::new("value".to_string(), self.currentLocation()),
                                                false,
                                            ),
                                            location: self.currentLocation(),
                                        }],
                                    ),
                                    location: self.currentLocation(),
                                },
                                body: Expr {
                                    expr: SimpleExpr::Value(Identifier::new(
                                        "value".to_string(),
                                        self.currentLocation(),
                                    )),
                                    location: self.currentLocation(),
                                },
                            },
                            Branch {
                                pattern: Pattern {
                                    pattern: SimplePattern::Named(
                                        Identifier::new(format!("Result.Result.Err"), self.currentLocation()),
                                        vec![Pattern {
                                            pattern: SimplePattern::Bind(
                                                Identifier::new("err".to_string(), self.currentLocation()),
                                                false,
                                            ),
                                            location: self.currentLocation(),
                                        }],
                                    ),
                                    location: self.currentLocation(),
                                },
                                body: Expr {
                                    expr: SimpleExpr::Return(Some(Box::new(Expr {
                                        expr: SimpleExpr::Call(
                                            Box::new(Expr {
                                                expr: SimpleExpr::Value(Identifier::new(
                                                    format!("Result.Result.Err"),
                                                    self.currentLocation(),
                                                )),
                                                location: self.currentLocation(),
                                            }),
                                            vec![FunctionArg::Positional(Expr {
                                                expr: SimpleExpr::Value(Identifier::new(
                                                    "err".to_string(),
                                                    self.currentLocation(),
                                                )),
                                                location: self.currentLocation(),
                                            })],
                                        ),
                                        location: self.currentLocation(),
                                    }))),
                                    location: self.currentLocation(),
                                },
                            },
                        ],
                    ),
                    location: self.currentLocation(),
                };
                self.buildExpr(matchExpr.expr, start)
            }
            TokenKind::Keyword(KeywordKind::Yield) => {
                self.expect(TokenKind::Keyword(KeywordKind::Yield));
                let arg = self.parseExpr();
                self.buildExpr(SimpleExpr::Yield(Box::new(arg)), start)
            }
            TokenKind::Keyword(KeywordKind::Co) => {
                self.expect(TokenKind::Keyword(KeywordKind::Co));
                let arg = self.parseExpr();
                self.buildExpr(SimpleExpr::SpawnCoroutine(Box::new(arg)), start)
            }
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
            TokenKind::Op(OperatorKind::BitAnd) => {
                self.expect(TokenKind::Op(OperatorKind::BitAnd));
                let isRaw = if self.check(TokenKind::Keyword(KeywordKind::Raw)) {
                    self.expect(TokenKind::Keyword(KeywordKind::Raw));
                    true
                } else {
                    false
                };
                let arg = self.parseExpr();
                self.buildExpr(SimpleExpr::Ref(Box::new(arg), isRaw), start)
            }

            TokenKind::LeftBracket(BracketKind::Square) => {
                self.expect(TokenKind::LeftBracket(BracketKind::Square));
                let mut args = Vec::new();
                while !self.check(TokenKind::RightBracket(BracketKind::Square)) {
                    let arg = self.parseExpr();
                    args.push(arg);
                    if self.check(TokenKind::RightBracket(BracketKind::Square)) {
                        break;
                    } else {
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                    }
                }
                self.expect(TokenKind::RightBracket(BracketKind::Square));
                self.buildExpr(SimpleExpr::List(args), start)
            }
            TokenKind::Op(OperatorKind::BitOr) => {
                self.expect(TokenKind::Op(OperatorKind::BitOr));
                let mut params = Vec::new();
                loop {
                    if self.check(TokenKind::Op(OperatorKind::BitOr)) {
                        break;
                    }
                    let param = self.parsePattern();
                    params.push(param);
                    if self.check(TokenKind::Op(OperatorKind::BitOr)) {
                        break;
                    } else {
                        self.expect(TokenKind::Misc(MiscKind::Comma));
                    }
                }
                self.expect(TokenKind::Op(OperatorKind::BitOr));
                let body = self.parseExpr();
                let block = SimpleExpr::Block(Block {
                    statements: vec![Statement {
                        kind: StatementKind::Expr(body),
                        hasSemicolon: false,
                    }],
                    location: self.currentLocation(),
                });
                let block = Expr {
                    expr: block,
                    location: self.currentLocation(),
                };
                self.buildExpr(SimpleExpr::Lambda(params, Box::new(block)), start)
            }
            kind => self.reportError2("<expr>", kind),
        }
    }

    fn parseExpr(&mut self) -> Expr {
        self.parseBinaryOp(0)
    }
}
