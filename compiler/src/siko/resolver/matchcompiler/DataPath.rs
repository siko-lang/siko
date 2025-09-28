use crate::siko::qualifiedname::QualifiedName;
use std::fmt;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataPathSegment {
    Root,
    Tuple(i64),
    TupleIndex(i64),
    ItemIndex(i64),
    Variant(QualifiedName, QualifiedName),
    IntegerLiteral(String),
    StringLiteral(String),
    Struct(QualifiedName),
    Wildcard,
}

impl fmt::Display for DataPathSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataPathSegment::Root => write!(f, "Root"),
            DataPathSegment::Tuple(len) => write!(f, "/tuple{}", len),
            DataPathSegment::TupleIndex(index) => {
                write!(f, ".t{}", index)
            }
            DataPathSegment::ItemIndex(index) => {
                write!(f, ".i{}", index)
            }
            DataPathSegment::Variant(name, _) => write!(f, ".{}", name),
            DataPathSegment::IntegerLiteral(literal) => {
                write!(f, "[int:{}]", literal)
            }
            DataPathSegment::StringLiteral(literal) => {
                write!(f, "[str:\"{}\"]", literal)
            }
            DataPathSegment::Struct(name) => write!(f, ".{}", name),
            DataPathSegment::Wildcard => write!(f, "._"),
        }
    }
}

impl fmt::Debug for DataPathSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataPath {
    segments: Vec<DataPathSegment>,
}

impl DataPath {
    pub fn root() -> DataPath {
        DataPath {
            segments: vec![DataPathSegment::Root],
        }
    }

    pub fn asRef<'a>(&'a self) -> DataPathRef<'a> {
        DataPathRef {
            segments: &self.segments,
        }
    }

    pub fn push(&self, segment: DataPathSegment) -> DataPath {
        let mut segments = self.segments.clone();
        segments.push(segment);
        DataPath { segments }
    }
}

impl fmt::Display for DataPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.asRef())
    }
}

impl fmt::Debug for DataPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataPathRef<'a> {
    segments: &'a [DataPathSegment],
}

impl<'a> DataPathRef<'a> {
    pub fn isChild(&self, parent: &DataPathRef) -> bool {
        let s1 = self.segments();
        let s2 = parent.segments();
        if s1.len() <= s2.len() {
            return false;
        }
        for (a, b) in s1.iter().zip(s2.iter()) {
            if a != b {
                return false;
            }
        }
        true
    }

    pub fn segments(&self) -> &[DataPathSegment] {
        self.segments
    }

    pub fn isRoot(&self) -> bool {
        let s = self.segments();
        s.len() == 1 && s[0] == DataPathSegment::Root
    }

    pub fn asBindingPath(&self) -> DataPath {
        let mut segments = self.segments().to_vec();
        segments.push(DataPathSegment::Wildcard);
        DataPath { segments }
    }

    pub fn owned(&self) -> DataPath {
        DataPath {
            segments: self.segments().to_vec(),
        }
    }

    pub fn last(&self) -> &DataPathSegment {
        &self.segments()[self.segments().len() - 1]
    }

    pub fn getParent(&'a self) -> DataPathRef<'a> {
        let segments = self.segments();
        if segments.len() == 1 {
            return DataPathRef { segments };
        }
        DataPathRef {
            segments: &segments[0..segments.len() - 1],
        }
    }
}

impl fmt::Display for DataPathRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let segments = self
            .segments()
            .iter()
            .map(|s| format!("{}", s))
            .collect::<Vec<String>>()
            .join("");
        write!(f, "{}", segments)
    }
}

impl fmt::Debug for DataPathRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug)]
pub enum DataType {
    Struct(QualifiedName),
    Enum(QualifiedName),
    Tuple(i64),
    Integer,
    String,
    Wildcard,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Struct(name) => write!(f, "Struct({})", name),
            DataType::Enum(name) => write!(f, "Enum({})", name),
            DataType::Tuple(size) => write!(f, "Tuple({})", size),
            DataType::Integer => write!(f, "Integer"),
            DataType::String => write!(f, "String"),
            DataType::Wildcard => write!(f, "_"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecisionPath {
    pub decisions: Vec<DataPath>,
}

impl DecisionPath {
    pub fn new() -> DecisionPath {
        DecisionPath { decisions: Vec::new() }
    }

    pub fn add(&self, path: DataPath) -> DecisionPath {
        let mut d = self.clone();
        d.decisions.push(path);
        d
    }

    pub fn last(&self) -> &DataPath {
        self.decisions.last().unwrap()
    }
}

impl fmt::Display for DecisionPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decisions = self
            .decisions
            .iter()
            .map(|path| format!("{}", path))
            .collect::<Vec<String>>()
            .join(" -> ");

        write!(f, "{}", decisions)
    }
}

pub fn matchDecisions(mut nodeDecisionPath: &[DataPath], mut matchDecisionPath: &[DataPath]) -> bool {
    loop {
        if matchDecisionPath.is_empty() {
            return nodeDecisionPath.is_empty();
        }
        let path = &matchDecisionPath[0];
        matchDecisionPath = &matchDecisionPath[1..];
        nodeDecisionPath = removePaths(&path.asRef(), nodeDecisionPath);
    }
}

fn removePaths<'a, 'b>(path: &DataPathRef<'b>, mut nodeDecisionPath: &'a [DataPath]) -> &'a [DataPath] {
    loop {
        if nodeDecisionPath.is_empty() {
            break;
        }
        let nodePath = &nodeDecisionPath[0];
        let remove = match (path.last(), nodePath) {
            (DataPathSegment::Wildcard, _) => nodePath.asRef().isChild(&path.getParent()),
            (_, p2) => path == &p2.asRef(),
        };
        if remove {
            nodeDecisionPath = &nodeDecisionPath[1..];
        } else {
            break;
        }
    }
    nodeDecisionPath
}
