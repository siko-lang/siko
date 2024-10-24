use crate::siko::hir::Data::Enum;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::ModuleResolver::ModuleResolver;
use std::collections::BTreeMap;

pub struct Resolver<'a> {
    pub moduleResolver: &'a ModuleResolver,
    pub variants: &'a BTreeMap<QualifiedName, QualifiedName>,
    pub enums: &'a BTreeMap<QualifiedName, Enum>,
}

impl<'a> Resolver<'a> {
    pub fn new(
        moduleResolver: &'a ModuleResolver,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> Resolver<'a> {
        Resolver {
            moduleResolver: moduleResolver,
            variants: variants,
            enums: enums,
        }
    }
}
