use std::iter::zip;

use super::{
    Apply::Apply,
    Substitution::Substitution,
    Type::{Type, TypeVar},
};

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub allowNamed: bool,
    pub voidSeparate: bool,
}

impl Config {
    pub fn default() -> Config {
        Config {
            allowNamed: false,
            voidSeparate: false,
        }
    }

    pub fn allowNamed(&self) -> Config {
        let mut copy = *self;
        copy.allowNamed = true;
        copy
    }

    pub fn voidSeparate(&self) -> Config {
        let mut copy = *self;
        copy.voidSeparate = true;
        copy
    }
}

#[derive(Debug)]
pub struct Error {}

pub fn unify(sub: &mut Substitution, ty1: Type, ty2: Type, cfg: Config) -> Result<(), Error> {
    //println!("Unifying {}/{}", ty1, ty2);
    let ty1 = ty1.apply(sub).makeSingleRef();
    let ty2 = ty2.apply(sub).makeSingleRef();
    //println!("Unifying2 {}/{}", ty1, ty2);
    match (ty1, ty2) {
        (Type::Named(name1, args1), Type::Named(name2, args2)) => {
            if name1 != name2 {
                return Err(Error {});
            } else {
                for (arg1, arg2) in zip(args1, args2) {
                    unify(sub, arg1, arg2, cfg)?;
                }
                Ok(())
            }
        }
        (Type::Tuple(args1), Type::Tuple(args2)) => {
            if args1.len() != args2.len() {
                return Err(Error {});
            } else {
                for (arg1, arg2) in zip(args1, args2) {
                    unify(sub, arg1, arg2, cfg)?;
                }
                Ok(())
            }
        }
        (Type::Var(TypeVar::Named(n1)), Type::Var(TypeVar::Named(n2))) => {
            if n1 == n2 {
                return Ok(());
            } else {
                return Err(Error {});
            }
        }
        (Type::Var(TypeVar::Var(v1)), Type::Var(TypeVar::Var(v2))) if v1 == v2 => Ok(()),
        (Type::Never(false), Type::Var(_)) => Ok(()),
        (Type::Var(_), Type::Never(false)) => Ok(()),
        (Type::Var(v), Type::Never(true)) => {
            sub.add(Type::Var(v), Type::Never(false));
            Ok(())
        }
        (Type::Never(true), Type::Var(v)) => {
            sub.add(Type::Var(v), Type::Never(false));
            Ok(())
        }
        (ty1, Type::Var(TypeVar::Var(v))) => {
            sub.add(Type::Var(TypeVar::Var(v)), ty1);
            Ok(())
        }
        (Type::Var(TypeVar::Var(v)), ty2) => {
            sub.add(Type::Var(TypeVar::Var(v)), ty2);
            Ok(())
        }
        (ty1, Type::Var(v)) if cfg.allowNamed => {
            sub.add(Type::Var(v), ty1);
            Ok(())
        }
        (Type::Var(v), ty2) if cfg.allowNamed => {
            sub.add(Type::Var(v), ty2);
            Ok(())
        }
        (Type::Reference(v1), Type::Reference(v2)) => unify(sub, *v1, *v2, cfg),
        (Type::Ptr(v1), Type::Ptr(v2)) => unify(sub, *v1, *v2, cfg),
        (Type::Never(_), _) => Ok(()),
        (_, Type::Never(_)) => Ok(()),
        (Type::Function(args1, res1), Type::Function(args2, res2)) => {
            for (arg1, arg2) in zip(args1, args2) {
                unify(sub, arg1, arg2, cfg)?;
            }
            return unify(sub, *res1, *res2, cfg);
        }
        (Type::FunctionPtr(args1, res1), Type::FunctionPtr(args2, res2)) => {
            for (arg1, arg2) in zip(args1, args2) {
                unify(sub, arg1, arg2, cfg)?;
            }
            return unify(sub, *res1, *res2, cfg);
        }
        (Type::NumericConstant(v1), Type::NumericConstant(v2)) => {
            if v1 == v2 {
                Ok(())
            } else {
                Err(Error {})
            }
        }
        (Type::Void, Type::Void) => Ok(()),
        (Type::VoidPtr, Type::VoidPtr) => Ok(()),
        (Type::VoidPtr, Type::Ptr(_)) if !cfg.voidSeparate => Ok(()),
        (Type::Ptr(_), Type::VoidPtr) if !cfg.voidSeparate => Ok(()),
        (Type::Coroutine(yieldTy1, retTy1), Type::Coroutine(yieldTy2, retTy2)) => {
            unify(sub, *yieldTy1, *yieldTy2, cfg)?;
            unify(sub, *retTy1, *retTy2, cfg)
        }
        _ => return Err(Error {}),
    }
}
