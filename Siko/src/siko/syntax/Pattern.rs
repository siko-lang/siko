use crate::siko::location::Location::Location;

use super::Identifier::Identifier;

#[derive(Debug)]
pub struct Pattern {
    pub pattern: SimplePattern,
    pub location: Location,
}

#[derive(Debug)]
pub enum SimplePattern {
    Named(Identifier, Vec<Pattern>),
    Bind(Identifier, bool),
    Tuple(Vec<Pattern>),
    StringLiteral(String, Location),
    IntegerLiteral(String, Location),
    Wildcard,
}
