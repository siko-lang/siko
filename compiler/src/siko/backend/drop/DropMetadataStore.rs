use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    rc::Rc,
};

use crate::siko::{
    backend::drop::Path::{Path, PathSegment, SimplePath},
    hir::{
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
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

struct PathListInner {
    name: VariableName,
    ty: Type,
    paths: BTreeSet<SimplePath>,
}

impl PathListInner {
    pub fn expand(&mut self, program: &Program) {
        let var = Variable {
            name: self.name.clone(),
            location: Location::empty(),
            ty: Some(self.ty.clone()),
        };
        let rootPath = var.toPath().toSimplePath();
        self.expandPath(program, &rootPath, &self.ty.clone());
    }

    fn expandPath(&mut self, program: &Program, rootPath: &SimplePath, ty: &Type) {
        if let Some(name) = ty.getName() {
            if let Some(structDef) = program.getStruct(&name) {
                let mut subPaths = Vec::new();
                let mut present = Vec::new();
                for field in &structDef.fields {
                    let segment = PathSegment::Named(field.name.clone(), field.ty.clone());
                    let fieldPath = rootPath.add(segment);
                    for p in &self.paths {
                        if p.contains(&fieldPath) {
                            present.push(fieldPath.clone());
                        }
                        subPaths.push(fieldPath.clone());
                    }
                }
                if !present.is_empty() {
                    for s in subPaths {
                        self.paths.insert(s);
                    }
                    for p in present {
                        self.expandPath(program, &p, p.items.last().unwrap().getType());
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct PathList {
    list: Rc<RefCell<PathListInner>>,
}

impl PathList {
    pub fn new(name: VariableName, ty: Type) -> Self {
        PathList {
            list: Rc::new(RefCell::new(PathListInner {
                name,
                ty,
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

    pub fn expand(&self, program: &Program) {
        self.list.borrow_mut().expand(program);
    }
}

pub struct DropMetadataStore {
    variableMetadata: BTreeMap<VariableName, PathList>,
}

impl DropMetadataStore {
    pub fn new() -> Self {
        DropMetadataStore {
            variableMetadata: BTreeMap::new(),
        }
    }

    pub fn addVariable(&mut self, name: VariableName, ty: Type) {
        let path_list = PathList::new(name.clone(), ty);
        self.variableMetadata.insert(name, path_list);
    }

    pub fn getPathList(&self, name: &VariableName) -> Option<PathList> {
        self.variableMetadata.get(name).cloned()
    }

    pub fn expandPathLists(&mut self, program: &Program) {
        for path_list in self.variableMetadata.values_mut() {
            path_list.expand(program);
        }
    }
}

// fn dropPath(&self, rootPath: &Path, ty: &Type, context: &Context, dropList: &mut DropList) {
//     match context.isMoved(&&rootPath) {
//         MoveKind::NotMoved => {
//             //println!("not moved - drop {}", rootPath);
//             dropList.add(rootPath.clone());
//         }
//         MoveKind::Partially => {
//             //println!("partially moved {}", rootPath);
//             //println!("already moved (maybe partially?) {}", rootPath);
//             if let Some(structName) = ty.getName() {
//                 if let Some(structDef) = self.program.getStruct(&structName) {
//                     let mut allocator = TypeVarAllocator::new();
//                     let structInstance = instantiateStruct(&mut allocator, &structDef, ty);
//                     for field in &structInstance.fields {
//                         let path = rootPath.add(field.name.clone());
//                         self.dropPath(&path, &field.ty, context, dropList);
//                     }
//                 }
//             }
//         }
//         MoveKind::Fully(var) => {
//             //println!("already moved {} by {}", rootPath, var);
//         }
//     }
// }
