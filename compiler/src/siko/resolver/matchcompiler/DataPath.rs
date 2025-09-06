use std::fmt;

use crate::siko::qualifiedname::QualifiedName;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataPath {
    Root,
    Tuple(Box<DataPath>, i64),
    TupleIndex(Box<DataPath>, i64),
    ItemIndex(Box<DataPath>, i64),
    Variant(Box<DataPath>, QualifiedName, QualifiedName),
    IntegerLiteral(Box<DataPath>, String),
    StringLiteral(Box<DataPath>, String),
    Struct(Box<DataPath>, QualifiedName),
    Wildcard(Box<DataPath>),
}

impl DataPath {
    pub fn isChild(&self, parent: &DataPath) -> bool {
        let mut selfParent = self.getParent();
        loop {
            if &selfParent == parent {
                return true;
            }
            if selfParent == DataPath::Root {
                return false;
            }
            selfParent = selfParent.getParent();
        }
    }

    pub fn getParent(&self) -> DataPath {
        match self {
            DataPath::Root => DataPath::Root,
            DataPath::Tuple(p, _) => *p.clone(),
            DataPath::TupleIndex(p, _) => *p.clone(),
            DataPath::ItemIndex(p, _) => *p.clone(),
            DataPath::Variant(p, _, _) => *p.clone(),
            DataPath::IntegerLiteral(p, _) => *p.clone(),
            DataPath::StringLiteral(p, _) => *p.clone(),
            DataPath::Struct(p, _) => *p.clone(),
            DataPath::Wildcard(p) => *p.clone(),
        }
    }
}

impl fmt::Display for DataPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataPath::Root => write!(f, "Root"),
            DataPath::Tuple(path, len) => write!(f, "{}/tuple{}", path, len),
            DataPath::TupleIndex(path, index) => {
                write!(f, "{}.t{}", path, index)
            }
            DataPath::ItemIndex(path, index) => {
                write!(f, "{}.i{}", path, index)
            }
            DataPath::Variant(path, name, _) => write!(f, "{}.{}", path, name),
            DataPath::IntegerLiteral(path, literal) => {
                write!(f, "{}[int:{}]", path, literal)
            }
            DataPath::StringLiteral(path, literal) => {
                write!(f, "{}[str:\"{}\"]", path, literal)
            }
            DataPath::Struct(path, name) => write!(f, "{}.{}", path, name),
            DataPath::Wildcard(path) => write!(f, "{}._", path),
        }
    }
}

impl fmt::Debug for DataPath {
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

pub fn matchDecisions(mut nodeDecisionPath: DecisionPath, mut matchDecisionPath: DecisionPath) -> bool {
    loop {
        if matchDecisionPath.decisions.is_empty() {
            return nodeDecisionPath.decisions.is_empty();
        }
        let path = matchDecisionPath.decisions.remove(0);
        nodeDecisionPath = removePaths(&path, nodeDecisionPath);
    }
}

fn removePaths(path: &DataPath, mut nodeDecisionPath: DecisionPath) -> DecisionPath {
    loop {
        if nodeDecisionPath.decisions.is_empty() {
            break;
        }
        let nodePath = &nodeDecisionPath.decisions[0];
        let remove = match (path, nodePath) {
            (DataPath::Wildcard(parent), _) => nodePath.isChild(parent),
            (p1, p2) => p1 == p2,
        };
        if remove {
            nodeDecisionPath.decisions.remove(0);
        } else {
            break;
        }
    }
    nodeDecisionPath
}
