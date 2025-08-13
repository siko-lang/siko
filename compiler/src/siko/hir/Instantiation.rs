use std::collections::BTreeSet;

use crate::siko::hir::{
    Apply::Apply,
    Data::{Enum, Struct},
    Substitution::Substitution,
    Trait::{Instance, Trait},
    Type::Type,
    TypeVarAllocator::TypeVarAllocator,
    Unification::unify,
};

pub fn instantiateEnum(allocator: &mut TypeVarAllocator, e: &Enum, ty: &Type) -> Enum {
    let sub = instantiateTypes(allocator, &vec![e.ty.clone()]);
    let mut e = e.clone();
    e = e.apply(&sub);
    let mut sub = Substitution::new();
    let r = unify(&mut sub, ty.clone(), e.ty.clone(), false);
    assert!(r.is_ok());
    e.apply(&sub)
}

pub fn instantiateStruct(allocator: &mut TypeVarAllocator, c: &Struct, ty: &Type) -> Struct {
    let sub = instantiateTypes(allocator, &vec![c.ty.clone()]);
    let mut res = c.clone();
    res = res.apply(&sub);
    let mut sub = Substitution::new();
    let r = unify(&mut sub, ty.clone(), res.ty.clone(), false);
    assert!(r.is_ok());
    res.apply(&sub)
}

pub fn instantiateInstance(allocator: &mut TypeVarAllocator, i: &Instance) -> Instance {
    let mut vars = BTreeSet::new();
    for ty in &i.types {
        vars = ty.collectVars(vars);
    }
    let mut sub = Substitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    i.clone().apply(&sub)
}

pub fn instantiateTrait(allocator: &mut TypeVarAllocator, t: &Trait) -> Trait {
    let sub = instantiateTypes(allocator, &t.params);
    t.clone().apply(&sub)
}

pub fn instantiateTypes(allocator: &mut TypeVarAllocator, types: &Vec<Type>) -> Substitution {
    let mut vars = BTreeSet::new();
    for ty in types {
        vars = ty.collectVars(vars);
    }
    let mut sub = Substitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    sub
}

pub fn instantiateType(allocator: &mut TypeVarAllocator, ty: Type) -> Type {
    let mut vars = BTreeSet::new();
    vars = ty.collectVars(vars);
    let mut sub = Substitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    ty.apply(&sub)
}
