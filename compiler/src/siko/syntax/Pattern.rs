use std::fmt;

use crate::siko::location::Location::Location;

use super::Identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pattern {
    pub pattern: SimplePattern,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimplePattern {
    Named(Identifier, Vec<Pattern>),
    Bind(Identifier, bool), // mutable
    Tuple(Vec<Pattern>),
    StringLiteral(String),
    IntegerLiteral(String),
    Wildcard,
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
        }
    }
}
