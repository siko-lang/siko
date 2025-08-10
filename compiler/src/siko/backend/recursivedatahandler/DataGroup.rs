use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    hir::{
        Data::Enum,
        Function::Parameter,
        Instantiation::{instantiateEnum, instantiateStruct},
        Program::Program,
        Substitution::Substitution,
        Type::{formatTypes, Type},
        TypeVarAllocator::TypeVarAllocator,
    },
    location::Report::{Report, ReportContext},
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

pub struct DataGroupHandler<'a> {
    program: &'a Program,
    visited: BTreeSet<Type>,
    recursive: BTreeMap<Type, bool>,
    allocator: TypeVarAllocator,
    deps: BTreeMap<Type, Vec<Type>>,
}

impl<'a> DataGroupHandler<'a> {
    pub fn new(program: &'a Program) -> DataGroupHandler<'a> {
        DataGroupHandler {
            program,
            visited: BTreeSet::new(),
            recursive: BTreeMap::new(),
            allocator: TypeVarAllocator::new(),
            deps: BTreeMap::new(),
        }
    }

    pub fn isRecursive(&mut self, ty: &Type) -> bool {
        // println!("Checking if {} is recursive", ty);
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

    pub fn addDep(&mut self, ty: &Type, dep: &Type) {
        //println!("Adding dependency: {} -> {}", ty, dep);
        if ty.isNamed() {
            if dep.isNamed() {
                self.deps.entry(ty.clone()).or_default().push(dep.clone());
            } else {
                self.deps.entry(ty.clone()).or_default();
            }
        }
    }

    pub fn check(&mut self, ty: &Type) -> bool {
        match ty {
            Type::Named(s, _) => {
                if let Some(s) = self.program.structs.get(s) {
                    let s = instantiateStruct(&mut self.allocator, s, ty);
                    if s.fields.is_empty() {
                        self.deps.entry(ty.clone()).or_default();
                        return false; // No fields, not recursive
                    }
                    let mut recursive = false;
                    for field in &s.fields {
                        self.addDep(ty, &field.ty);
                        if self.isRecursive(&field.ty) {
                            recursive = true;
                        }
                    }
                    if recursive {
                        self.recursive.insert(ty.clone(), true);
                    }
                    return recursive;
                }
                if let Some(e) = self.program.enums.get(s) {
                    let e = instantiateEnum(&mut self.allocator, e, ty);
                    let mut recursive = false;
                    self.deps.entry(ty.clone()).or_default();
                    for v in &e.variants {
                        for item in &v.items {
                            self.addDep(ty, item);
                            if self.isRecursive(item) {
                                recursive = true;
                            }
                        }
                    }
                    if recursive {
                        self.recursive.insert(ty.clone(), true);
                    }
                    return recursive;
                }
                panic!("Unknown type: {}", s);
            }
            Type::Tuple(args) => {
                let mut recursive = false;
                for arg in args {
                    if self.isRecursive(arg) {
                        recursive = true;
                    }
                }
                if recursive {
                    self.recursive.insert(ty.clone(), true);
                }
                return recursive;
            }
            _ => false,
        }
    }

    pub fn dump(&self) {
        for (ty, is_recursive) in &self.recursive {
            println!("Type: {}, Recursive: {}", ty, is_recursive);
        }
    }
}

fn processEnum(mut e: Enum, sub: &Substitution) -> Enum {
    //println!("Processed enum: {}", e.name);
    for v in &mut e.variants {
        for item in &mut v.items {
            *item = sub.get(item.clone());
        }
    }
    e
}

fn processSingleDataGroup(mut program: Program, group: DependencyGroup<Type>) -> Program {
    //println!("Group: {}", formatTypes(&group.items));
    let mut sub = Substitution::new();
    for item in &group.items {
        sub.add(item.clone(), item.getBoxedType());
    }
    for item in &group.items {
        if let Some(name) = item.getName() {
            //if let Some(s) = program.structs.get(&name) {}
            if let Some(e) = program.enums.get(&name) {
                //println!("Processing enum: {}", e.name);
                let e = processEnum(e.clone(), &sub);
                for v in &e.variants {
                    let ctorFunc = program.functions.get_mut(&v.name).expect("Variant ctor not found");
                    for param in ctorFunc.params.iter_mut() {
                        match param {
                            Parameter::Named(_, ty, _) => {
                                *ty = sub.get(ty.clone());
                            }
                            Parameter::SelfParam(_, ty) => {
                                *ty = sub.get(ty.clone());
                            }
                        }
                    }
                }
                //println!("Processed enum: {}", e);
                program.enums.insert(name.clone(), e);
                // for v in &e.variants {
                //     for item in &v.items {
                //         if group.items.contains(item) {
                //             println!("Enum variant item need to be boxed: {} {}", item, item);
                //         }
                //     }
                // }
            }
        }
    }
    program
}

fn getDataGroups(program: &Program) -> (Vec<DependencyGroup<Type>>, BTreeMap<Type, bool>) {
    let mut handler = DataGroupHandler::new(program);
    for (_, s) in &program.structs {
        handler.isRecursive(&s.ty);
    }
    for (_, e) in &program.enums {
        handler.isRecursive(&e.ty);
    }

    //handler.dump();

    // for (ty, deps) in &handler.deps {
    //     println!("Type: {}, Dependencies: {}", ty, formatTypes(deps));
    // }

    (processDependencies(&handler.deps), handler.recursive)
}

pub fn processDataGroups(ctx: &ReportContext, mut program: Program) -> Program {
    let (groups, cache) = getDataGroups(&program);
    for group in groups {
        let mut recursive = false;
        if group.items.len() > 1 {
            recursive = true;
        }
        if group.items.len() == 1 && cache.get(&group.items[0]) == Some(&true) {
            recursive = true;
        }
        if recursive {
            program = processSingleDataGroup(program, group);
        }
    }
    let (groups, cache) = getDataGroups(&program);
    let mut success = true;
    for group in groups {
        let mut recursive = false;
        if group.items.len() > 1 {
            recursive = true;
        }
        if group.items.len() == 1 && cache.get(&group.items[0]) == Some(&true) {
            recursive = true;
        }
        if recursive {
            for item in &group.items {
                if let Some(name) = item.getName() {
                    if let Some(s) = program.structs.get(&name) {
                        let location = s.location.clone();
                        let slogan = format!("Recursive data types {}", formatTypes(&group.items));
                        let report = Report::new(ctx, slogan, Some(location));
                        report.print();
                        success = false;
                    }
                    if let Some(e) = program.enums.get(&name) {
                        let location = e.location.clone();
                        let slogan = format!("Recursive data types {}", formatTypes(&group.items));
                        let report = Report::new(ctx, slogan, Some(location));
                        report.print();
                        success = false;
                    }
                }
            }
        }
    }
    if !success {
        std::process::exit(1);
    }
    program
}
