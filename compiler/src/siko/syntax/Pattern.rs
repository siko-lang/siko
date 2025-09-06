use std::fmt;

use crate::siko::{location::Location::Location, syntax::Expr::Expr};

use super::Identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub pattern: SimplePattern,
    pub location: Location,
}

impl Pattern {
    pub fn isGuarded(&self) -> bool {
        self.pattern.isGuarded()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimplePattern {
    Named(Identifier, Vec<Pattern>),
    Bind(Identifier, bool), // mutable
    Tuple(Vec<Pattern>),
    StringLiteral(String),
    IntegerLiteral(String),
    Wildcard,
    Guarded(Box<Pattern>, Box<Expr>),
}

impl SimplePattern {
    pub fn isGuarded(&self) -> bool {
        match self {
            SimplePattern::Guarded(_, _) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pattern)
    }
}

impl fmt::Display for SimplePattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimplePattern::Named(identifier, patterns) => {
                if patterns.is_empty() {
                    write!(f, "{}", identifier)
                } else {
                    write!(f, "{}(", identifier)?;
                    let mut first = true;
                    for pattern in patterns {
                        if !first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", pattern)?;
                        first = false;
                    }
                    write!(f, ")")
                }
            }
            SimplePattern::Bind(identifier, mutable) => {
                if *mutable {
                    write!(f, "mut {}", identifier)
                } else {
                    write!(f, "{}", identifier)
                }
            }
            SimplePattern::Tuple(patterns) => {
                write!(f, "(")?;
                let mut first = true;
                for pattern in patterns {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", pattern)?;
                    first = false;
                }
                write!(f, ")")
            }
            SimplePattern::StringLiteral(value) => write!(f, "\"{}\"", value),
            SimplePattern::IntegerLiteral(value) => write!(f, "{}", value),
            SimplePattern::Wildcard => write!(f, "_"),
            SimplePattern::Guarded(pattern, expr) => write!(f, "{} if {:?}", pattern, expr),
        }
    }
}
