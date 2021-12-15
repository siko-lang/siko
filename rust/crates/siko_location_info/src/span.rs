#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(s: usize, e: usize) -> Span {
        Span { start: s, end: e }
    }

    pub fn single(offset: usize) -> Span {
        Span {
            start: offset,
            end: offset + 1,
        }
    }
}
