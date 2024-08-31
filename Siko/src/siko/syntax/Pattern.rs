use crate::siko::location::Location::Location;

use super::Identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub pattern: SimplePattern,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimplePattern {
    Named(Identifier, Vec<Pattern>),
    Bind(Identifier, bool), // mutable
    Tuple(Vec<Pattern>),
    StringLiteral(String),
    IntegerLiteral(String),
    Wildcard,
}
