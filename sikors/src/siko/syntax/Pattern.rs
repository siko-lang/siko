use crate::siko::location::Location::Location;

use super::Identifier::Identifier;

#[derive(Debug)]
pub enum Pattern {
    Named(Identifier, Vec<Pattern>),
    Bind(Identifier),
    Tuple(Vec<Pattern>),
    StringLiteral(String, Location),
    IntegerLiteral(String, Location),
    Wildcard,
}
