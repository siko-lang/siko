use std::collections::BTreeMap;

use crate::siko::{location::Location::Location, resolver::Error::ResolverError};

use super::{
    Choice::{Choice, ChoiceKind},
    Decision::Path,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub path: Path,
    pub choice: Choice,
}

pub struct Collector {
    pub kinds: BTreeMap<Path, ChoiceKind>,
    pub edges: BTreeMap<Edge, Path>,
}

impl Collector {
    pub fn new() -> Collector {
        Collector {
            kinds: BTreeMap::new(),
            edges: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, path: Path, kind: ChoiceKind, location: Location) {
        let current = self.kinds.entry(path).or_insert(kind.clone());
        if *current == ChoiceKind::Wildcard {
            *current = kind;
        } else {
            if !current.isCompatible(&kind) {
                ResolverError::IncompatiblePattern(format!("{}", kind), format!("{}", current), location).report();
            }
        }
    }

    pub fn addEdge(&mut self, src: Path, choice: Choice, dest: Path) {
        let edge = Edge { path: src, choice: choice };
        self.edges.insert(edge, dest);
    }

    pub fn kind(&self, path: &Path) -> ChoiceKind {
        self.kinds.get(path).expect("path kind not found").clone()
    }
}
