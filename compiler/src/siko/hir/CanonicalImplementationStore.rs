use std::collections::{btree_map::Entry, BTreeMap};

use crate::siko::{
    hir::Type::{normalizeTypes, Type},
    location::{
        Location::Location,
        Report::{Entry as ReportEntry, Report, ReportContext},
    },
    qualifiedname::QualifiedName,
};

pub enum CanonicalImplStoreError {
    NotCanonicalType(String, Location),
    ConflictingCanonicalInstances(String, Location, Location),
}

impl CanonicalImplStoreError {
    pub fn report(&self, ctx: &ReportContext) -> ! {
        match &self {
            CanonicalImplStoreError::NotCanonicalType(ty, l) => {
                let slogan = format!("Not a canonical type: {}", ctx.yellow(ty));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            CanonicalImplStoreError::ConflictingCanonicalInstances(name, l1, l2) => {
                let slogan = format!("Conflicting canonical instances for {}", ctx.yellow(name));
                let e1 = ReportEntry::new(Some("First instance defined here".to_string()), l1.clone());
                let e2 = ReportEntry::new(Some("Second instance defined here".to_string()), l2.clone());
                let r = Report::build(ctx, slogan, vec![e1, e2]);
                r.print();
            }
        }
        std::process::exit(1);
    }
}

#[derive(Clone)]
struct CanonicalInstanceInfo {
    name: QualifiedName,
    location: Location,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    name: QualifiedName,
    types: Vec<Type>,
}

#[derive(Clone)]
pub struct CanonicalImplementationStore {
    implementations: BTreeMap<Key, CanonicalInstanceInfo>,
}

impl CanonicalImplementationStore {
    pub fn new() -> Self {
        CanonicalImplementationStore {
            implementations: BTreeMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        name: QualifiedName,
        types: Vec<Type>,
        implementation: QualifiedName,
        location: Location,
        ctx: &ReportContext,
    ) {
        for ty in &types {
            if !isCanonicalType(ty) {
                let error = CanonicalImplStoreError::NotCanonicalType(format!("{}", ty), location);
                error.report(ctx);
            }
        }
        let types = normalizeTypes(&types);
        let set = Key { name, types };
        match self.implementations.entry(set) {
            Entry::Vacant(e) => {
                e.insert(CanonicalInstanceInfo {
                    name: implementation,
                    location,
                });
            }
            Entry::Occupied(e) => {
                let existing = e.get();
                let error = CanonicalImplStoreError::ConflictingCanonicalInstances(
                    format!("{}", existing.name),
                    existing.location.clone(),
                    location,
                );
                error.report(ctx);
            }
        }
    }

    pub fn get(&self, name: &QualifiedName, types: &Vec<Type>) -> Option<&QualifiedName> {
        let types = normalizeTypes(types);
        let set = Key {
            name: name.clone(),
            types: types.clone(),
        };
        self.implementations.get(&set).map(|info| &info.name)
    }
}

fn isCanonicalType(ty: &Type) -> bool {
    match ty {
        Type::Named(_, args) => args.iter().all(|arg| arg.isGeneric()),
        _ => false,
    }
}
