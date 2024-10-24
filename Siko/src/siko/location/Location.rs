use std::fmt::Display;

use super::FileManager::FileManager;

#[derive(Clone)]
pub struct FileId {
    pub index: i64,
    pub fileManager: FileManager,
}

impl std::fmt::Debug for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl std::fmt::Display for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.index)
    }
}

impl PartialEq for FileId {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for FileId {}

impl PartialOrd for FileId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl Ord for FileId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl FileId {
    pub fn new(index: i64, fileManager: FileManager) -> FileId {
        FileId {
            index: index,
            fileManager: fileManager,
        }
    }

    pub fn getLines(&self) -> Vec<String> {
        let fileName = self.fileManager.get(&self);
        let content = std::fs::read(fileName).expect("Failed to read file");
        let content = String::from_utf8(content).expect("not utf8!");
        content.split("\n").map(|s| s.to_string()).collect()
    }

    pub fn getFileName(&self) -> String {
        self.fileManager.get(self)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Position {
    pub line: i64,
    pub offset: i64,
}

impl Position {
    pub fn new() -> Position {
        Position { line: 0, offset: 0 }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.offset)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Location {
    pub fileId: FileId,
    pub span: Span,
}

impl Location {
    pub fn new(fileId: FileId, span: Span) -> Location {
        Location { fileId: fileId, span: span }
    }

    pub fn merge(self, other: Location) -> Location {
        let Location { fileId: f1, span: s1 } = self;
        let Location { fileId: f2, span: s2 } = other;
        assert!(f1 == f2);
        Location {
            fileId: f1,
            span: s1.merge(s2),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.fileId, self.span)
    }
}
