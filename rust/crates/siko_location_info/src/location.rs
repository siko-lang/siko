use crate::span::Span;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Location {
    pub line: usize,
    pub span: Span,
}

impl Location {
    pub fn new(line: usize, span: Span) -> Location {
        Location {
            line: line,
            span: span,
        }
    }
}
