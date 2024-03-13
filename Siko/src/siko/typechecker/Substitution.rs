use std::{collections::BTreeMap, iter::zip};

use crate::siko::{
    ir::Type::{Type, TypeVar},
    util::error,
};

pub struct Substitution {
    substitutions: BTreeMap<TypeVar, Type>,
}

impl Substitution {
    pub fn new() -> Substitution {
        Substitution {
            substitutions: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, var: TypeVar, ty: Type) {
        self.substitutions.insert(var, ty);
    }

    pub fn apply(&self, ty: &Type) -> Type {
        let mut result = ty.clone();
        loop {
            match &result {
                Type::Named(n, args) => {
                    let newArgs = args.iter().map(|arg| self.apply(arg)).collect();
                    if newArgs == *args {
                        return result;
                    }
                    result = Type::Named(n.clone(), newArgs);
                }
                Type::Tuple(args) => {
                    let newArgs = args.iter().map(|arg| self.apply(arg)).collect();
                    if newArgs == *args {
                        return result;
                    }
                    result = Type::Tuple(newArgs);
                }
                Type::Function(args, fnResult) => {
                    let newArgs = args.iter().map(|arg| self.apply(arg)).collect();
                    let newFnResult = self.apply(fnResult);
                    if newArgs == *args && newFnResult == **fnResult {
                        return result;
                    }
                    result = Type::Function(newArgs, Box::new(newFnResult));
                }
                Type::Var(v) => match self.substitutions.get(&v) {
                    Some(ty) => {
                        result = ty.clone();
                    }
                    None => break result,
                },
                Type::SelfType => break result,
            }
        }
    }

    pub fn unify(&mut self, ty1: &Type, ty2: &Type) {
        println!("Unifying {}/{}", ty1, ty2);
        let ty1 = self.apply(ty1);
        let ty2 = self.apply(ty2);
        println!("Unifying2 {}/{}", ty1, ty2);
        match (&ty1, &ty2) {
            (Type::Named(name1, args1), Type::Named(name2, args2)) => {
                if name1 != name2 {
                    self.reportError(ty1, ty2);
                } else {
                    for (arg1, arg2) in zip(args1, args2) {
                        self.unify(arg1, arg2);
                    }
                }
            }
            (Type::Named(_, _), Type::Var(v)) => {
                self.add(v.clone(), ty1);
            }
            (Type::Var(v), Type::Named(_, _)) => {
                self.add(v.clone(), ty2);
            }
            _ => self.reportError(ty1, ty2),
        }
    }

    fn reportError(&self, ty1: Type, ty2: Type) {
        error(format!("Type mismatch {}/{}", ty1, ty2));
    }
}
