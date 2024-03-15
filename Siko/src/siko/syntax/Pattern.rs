use crate::siko::location::Location::Location;

use super::Identifier::Identifier;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern: SimplePattern,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub enum SimplePattern {
    Named(Identifier, Vec<Pattern>),
    Bind(Identifier, bool),
    Tuple(Vec<Pattern>),
    StringLiteral(String),
    IntegerLiteral(String),
    Wildcard,
}
