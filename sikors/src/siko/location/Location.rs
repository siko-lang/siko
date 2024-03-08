#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FileId {
    index: i64,
}

impl FileId {
    pub fn new(index: i64) -> FileId {
        FileId { index: index }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Position {
    pub line: i64,
    pub offset: i64,
}

impl Position {
    pub fn new() -> Position {
        Position { line: 0, offset: 0 }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}
impl Span {
    pub fn new() -> Span {
        Span {
            start: Position::new(),
            end: Position::new(),
        }
    }

    pub fn merge(self, other: Span) -> Span {
        let Span { start: s1, end: e1 } = self;
        let Span { start: s2, end: e2 } = other;
        assert!(s1 <= s2);
        assert!(e1 <= e2);
        Span { start: s1, end: e2 }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Location {
    pub fileId: FileId,
    pub span: Span,
}

impl Location {
    pub fn new(fileId: FileId) -> Location {
        Location {
            fileId: fileId,
            span: Span::new(),
        }
    }

    pub fn merge(self, other: Location) -> Location {
        let Location {
            fileId: f1,
            span: s1,
        } = self;
        let Location {
            fileId: f2,
            span: s2,
        } = other;
        assert!(f1 == f2);
        Location {
            fileId: f1,
            span: s1.merge(s2),
        }
    }
}
