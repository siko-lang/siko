use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::{
    Instantiation::{instantiateEnum, instantiateStruct},
    Program::Program,
    Type::Type,
    TypeVarAllocator::TypeVarAllocator,
};

pub struct DataGroupHandler<'a> {
    program: &'a Program,
    visited: BTreeSet<Type>,
    recursive: BTreeMap<Type, bool>,
    allocator: TypeVarAllocator,
}

impl<'a> DataGroupHandler<'a> {
    pub fn new(program: &'a Program) -> DataGroupHandler<'a> {
        DataGroupHandler {
            program,
            visited: BTreeSet::new(),
            recursive: BTreeMap::new(),
            allocator: TypeVarAllocator::new(),
        }
    }

    pub fn isRecursive(&mut self, ty: &Type) -> bool {
        println!("Checking if {} is recursive", ty);
        if self.recursive.contains_key(ty) {
            return self.recursive[ty];
        }

        if self.visited.contains(ty) {
            self.recursive.insert(ty.clone(), true);
            return true;
        }
        self.visited.insert(ty.clone());

        let result = self.check(ty);
        self.recursive.insert(ty.clone(), result);
        result
    }

    pub fn check(&mut self, ty: &Type) -> bool {
        match ty {
            Type::Named(s, _) => {
                if let Some(s) = self.program.structs.get(s) {
                    let s = instantiateStruct(&mut self.allocator, s, ty);
                    for field in &s.fields {
                        if self.isRecursive(&field.ty) {
                            self.recursive.insert(ty.clone(), true);
                            return true;
                        }
                    }
                }
                if let Some(e) = self.program.enums.get(s) {
                    let e = instantiateEnum(&mut self.allocator, e, ty);
                    for v in &e.variants {
                        for item in &v.items {
                            if self.isRecursive(&item) {
                                self.recursive.insert(ty.clone(), true);
                                return true;
                            }
                        }
                    }
                }
            }
            Type::Tuple(args) => {
                for arg in args {
                    if self.isRecursive(arg) {
                        self.recursive.insert(ty.clone(), true);
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    pub fn dump(&self) {
        for (ty, is_recursive) in &self.recursive {
            println!("Type: {}, Recursive: {}", ty, is_recursive);
        }
    }
}

pub fn processDataGroup(program: &Program) {
    let mut handler = DataGroupHandler::new(program);
    for (_, s) in &program.structs {
        handler.isRecursive(&s.ty);
    }
    for (_, e) in &program.enums {
        handler.isRecursive(&e.ty);
    }

    handler.dump();
}
