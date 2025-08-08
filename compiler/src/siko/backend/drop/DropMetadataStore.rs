use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    rc::Rc,
};

use crate::siko::{
    backend::drop::Path::{Path, SimplePath},
    hir::Variable::VariableName,
};

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetadataKind {
    DeclarationList(VariableName),
}

impl Display for MetadataKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataKind::DeclarationList(name) => write!(f, "DeclarationList({})", name),
        }
    }
}

struct DeclarationListInner {
    name: VariableName,
    paths: BTreeSet<SimplePath>,
}

#[derive(Clone)]
pub struct DeclarationList {
    list: Rc<RefCell<DeclarationListInner>>,
}

impl DeclarationList {
    pub fn new(name: VariableName) -> Self {
        DeclarationList {
            list: Rc::new(RefCell::new(DeclarationListInner {
                name,
                paths: BTreeSet::new(),
            })),
        }
    }

    pub fn addPath(&self, path: SimplePath) {
        self.list.borrow_mut().paths.insert(path);
    }

    pub fn paths(&self) -> BTreeSet<SimplePath> {
        self.list.borrow().paths.clone()
    }
}

struct DropMetadata {
    declaration_list: DeclarationList,
}

pub struct DropMetadataStore {
    drop_lists: BTreeMap<u32, DropList>,
    variableMetadata: BTreeMap<VariableName, DropMetadata>,
}

impl DropMetadataStore {
    pub fn new() -> Self {
        DropMetadataStore {
            drop_lists: BTreeMap::new(),
            variableMetadata: BTreeMap::new(),
        }
    }

    pub fn createDropList(&mut self, kind: Kind) -> u32 {
        let id = self.drop_lists.len() as u32 + 1; // Generate a new ID
        let drop_list = DropList::new(kind);
        self.drop_lists.insert(id, drop_list);
        id
    }

    pub fn addVariable(&mut self, name: VariableName) {
        let declaration_list = DeclarationList::new(name.clone());
        self.variableMetadata.insert(name, DropMetadata { declaration_list });
    }

    pub fn getDeclarationList(&self, name: &VariableName) -> Option<DeclarationList> {
        self.variableMetadata
            .get(name)
            .map(|metadata| metadata.declaration_list.clone())
    }

    pub fn addPath(&mut self, id: u32, path: Path) {
        if let Some(drop_list) = self.drop_lists.get_mut(&id) {
            drop_list.addPath(path);
        } else {
            panic!("DropList with id {} not found", id);
        }
    }

    pub fn getDropListIds(&self) -> Vec<u32> {
        self.drop_lists.keys().cloned().collect()
    }

    pub fn getDropList(&self, id: u32) -> &DropList {
        self.drop_lists.get(&id).expect("DropList not found")
    }
}
