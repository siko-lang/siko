use crate::siko::qualifiedname::QualifiedName;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Choice {
    Variant(QualifiedName, QualifiedName),
    Class(QualifiedName),
    Wildcard,
    Tuple,
    StringLiteral(String),
    IntegerLiteral(String),
}

impl Choice {
    pub fn kind(&self) -> ChoiceKind {
        match &self {
            Choice::Variant(v, e) => ChoiceKind::Variant(e.clone()),
            Choice::Class(n) => ChoiceKind::Class(n.clone()),
            Choice::Wildcard => ChoiceKind::Wildcard,
            Choice::Tuple => ChoiceKind::Tuple,
            Choice::StringLiteral(_) => ChoiceKind::StringLiteral,
            Choice::IntegerLiteral(_) => ChoiceKind::IntegerLiteral,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ChoiceKind {
    Variant(QualifiedName),
    Class(QualifiedName),
    Wildcard,
    Tuple,
    StringLiteral,
    IntegerLiteral,
}

impl ChoiceKind {
    pub fn isCompatible(&self, other: &ChoiceKind) -> bool {
        match (self, other) {
            (ChoiceKind::Wildcard, _) => true,
            (_, ChoiceKind::Wildcard) => true,
            (a, b) => *a == *b,
        }
    }
}

impl Display for ChoiceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChoiceKind::Variant(qn) => write!(f, "variant of {}", qn),
            ChoiceKind::Class(qn) => write!(f, "{}", qn),
            ChoiceKind::Wildcard => write!(f, "Wildcard"),
            ChoiceKind::Tuple => write!(f, "tuple"),
            ChoiceKind::StringLiteral => write!(f, "string iteral"),
            ChoiceKind::IntegerLiteral => write!(f, "integer literal"),
        }
    }
}
impl Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Choice::Variant(v, e) => write!(f, "variant({}, {})", v, e),
            Choice::Class(n) => write!(f, "named({})", n),
            Choice::Wildcard => write!(f, "wildcard"),
            Choice::Tuple => write!(f, "tuple"),
            Choice::StringLiteral(l) => write!(f, "string({})", l),
            Choice::IntegerLiteral(l) => write!(f, "integer({})", l),
        }
    }
}
