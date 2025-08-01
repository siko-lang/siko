use std::collections::{BTreeMap, BTreeSet};

use crate::siko::backend::drop::Path::{Path, SimplePath};

pub enum Kind {
    VariableAssign(SimplePath),
    FieldAssign(SimplePath),
}

pub struct DropList {
    paths: BTreeSet<SimplePath>,
    kind: Kind,
}

impl DropList {
    pub fn new(kind: Kind) -> Self {
        DropList {
            paths: BTreeSet::new(),
            kind,
        }
    }

    pub fn addPath(&mut self, path: Path) {
        self.paths.insert(path.toSimplePath());
    }

    pub fn paths(&self) -> &BTreeSet<SimplePath> {
        &self.paths
    }

    pub fn getRoot(&self) -> SimplePath {
        match &self.kind {
            Kind::VariableAssign(path) => path.clone(),
            Kind::FieldAssign(path) => path.clone(),
        }
    }
}

pub struct DropListHandler {
    dropLists: BTreeMap<u32, DropList>,
}

impl DropListHandler {
    pub fn new() -> Self {
        DropListHandler {
            dropLists: BTreeMap::new(),
        }
    }

    pub fn createDropList(&mut self, id: u32, kind: Kind) {
        let drop_list = DropList::new(kind);
        self.dropLists.insert(id, drop_list);
    }

    pub fn addPath(&mut self, id: u32, path: Path) {
        if let Some(drop_list) = self.dropLists.get_mut(&id) {
            drop_list.addPath(path);
        } else {
            panic!("DropList with id {} not found", id);
        }
    }

    pub fn getDropListIds(&self) -> Vec<u32> {
        self.dropLists.keys().cloned().collect()
    }

    pub fn getDropList(&self, id: u32) -> &DropList {
        self.dropLists.get(&id).expect("DropList not found")
    }
}
