use std::collections::BTreeSet;

use crate::siko::hir::{
    Apply::Apply,
    Data::{Enum, Struct},
    Substitution::Substitution,
    Trait::{Instance, Trait},
    Type::Type,
    TypeVarAllocator::TypeVarAllocator,
    Unification::{unify, Config},
};

pub fn instantiateEnum(allocator: &TypeVarAllocator, e: &Enum, ty: &Type) -> Enum {
    //println!("Instantiating enum {} {} to type {}", e.name, e.ty, ty);
    let sub = instantiateTypes(allocator, &vec![e.ty.clone()]);
    let mut e = e.clone();
    e = e.apply(&sub);
    let mut sub = Substitution::new();
    let r = unify(&mut sub, ty.clone(), e.ty.clone(), Config::default());
    assert!(r.is_ok());
    e.apply(&sub)
}

pub fn instantiateStruct(allocator: &TypeVarAllocator, c: &Struct, ty: &Type) -> Struct {
    let sub = instantiateTypes(allocator, &vec![c.ty.clone()]);
    let mut res = c.clone();
    res = res.apply(&sub);
    let mut sub = Substitution::new();
    let r = unify(&mut sub, ty.clone(), res.ty.clone(), Config::default());
    assert!(r.is_ok());
    res.apply(&sub)
}

pub fn instantiateInstance(allocator: &TypeVarAllocator, i: &Instance) -> Instance {
    let mut vars = BTreeSet::new();
    for ty in &i.types {
        vars = ty.collectVars(vars);
    }
    for ty in &i.typeParams {
        vars = ty.collectVars(vars);
    }
    for at in &i.associatedTypes {
        vars = at.ty.collectVars(vars);
    }
    for ty in &i.constraintContext.typeParameters {
        vars = ty.collectVars(vars);
    }
    for constraint in &i.constraintContext.constraints {
        for ty in &constraint.args {
            vars = ty.collectVars(vars);
        }
        for at in &constraint.associatedTypes {
            vars = at.ty.collectVars(vars);
        }
    }
    for member in &i.members {
        vars = member.memberType.collectVars(vars);
        for ty in &member.constraint.typeParameters {
            vars = ty.collectVars(vars);
        }
        for constraint in &member.constraint.constraints {
            for ty in &constraint.args {
                vars = ty.collectVars(vars);
            }
            for at in &constraint.associatedTypes {
                vars = at.ty.collectVars(vars);
            }
        }
    }
    let mut sub = Substitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    i.clone().apply(&sub)
}

pub fn instantiateTrait(allocator: &TypeVarAllocator, t: &Trait) -> Trait {
    let sub = instantiateTypes(allocator, &t.params);
    t.clone().apply(&sub)
}

pub fn instantiateTraitWithSub(allocator: &TypeVarAllocator, t: &Trait) -> (Trait, Substitution) {
    let sub = instantiateTypes(allocator, &t.params);
    (t.clone().apply(&sub), sub)
}

pub fn instantiateTypes(allocator: &TypeVarAllocator, types: &Vec<Type>) -> Substitution {
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

pub fn instantiateType(allocator: &TypeVarAllocator, ty: Type) -> Type {
    let mut vars = BTreeSet::new();
    vars = ty.collectVars(vars);
    let mut sub = Substitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    ty.apply(&sub)
}
