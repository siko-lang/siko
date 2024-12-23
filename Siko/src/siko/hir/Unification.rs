use std::iter::zip;

use super::{
    Apply::Apply,
    Substitution::TypeSubstitution,
    Type::{Type, TypeVar},
};

#[derive(Debug)]
pub struct Error {}

pub fn unify(sub: &mut TypeSubstitution, ty1: &Type, ty2: &Type, allowNamed: bool) -> Result<(), Error> {
    //println!("Unifying {}/{}", ty1, ty2);
    let ty1 = ty1.apply(sub).makeSingleRef();
    let ty2 = ty2.apply(sub).makeSingleRef();
    //println!("Unifying2 {}/{}", ty1, ty2);
    match (&ty1, &ty2) {
        (Type::Named(name1, args1, _), Type::Named(name2, args2, _)) => {
            if name1 != name2 {
                return Err(Error {});
            } else {
                for (arg1, arg2) in zip(args1, args2) {
                    unify(sub, arg1, arg2, allowNamed)?;
                }
                Ok(())
            }
        }
        (Type::Tuple(args1), Type::Tuple(args2)) => {
            if args1.len() != args2.len() {
                return Err(Error {});
            } else {
                for (arg1, arg2) in zip(args1, args2) {
                    unify(sub, arg1, arg2, allowNamed)?;
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
        (Type::Var(_), Type::Never(true)) => {
            sub.add(ty1, Type::Never(false));
            Ok(())
        }
        (Type::Never(true), Type::Var(_)) => {
            sub.add(ty2, Type::Never(false));
            Ok(())
        }
        (_, Type::Var(TypeVar::Var(_))) => {
            sub.add(ty2, ty1);
            Ok(())
        }
        (Type::Var(TypeVar::Var(_)), _) => {
            sub.add(ty1, ty2);
            Ok(())
        }
        (_, Type::Var(_)) if allowNamed => {
            sub.add(ty2, ty1);
            Ok(())
        }
        (Type::Var(_), _) if allowNamed => {
            sub.add(ty1, ty2);
            Ok(())
        }
        (Type::Reference(v1, _), Type::Reference(v2, _)) => unify(sub, &v1, &v2, allowNamed),
        (Type::Ptr(v1), Type::Ptr(v2)) => unify(sub, &v1, &v2, allowNamed),
        (Type::Never(_), _) => Ok(()),
        (_, Type::Never(_)) => Ok(()),
        (Type::Function(args1, res1), Type::Function(args2, res2)) => {
            for (arg1, arg2) in zip(args1, args2) {
                unify(sub, arg1, arg2, allowNamed)?;
            }
            return unify(sub, &res1, &res2, allowNamed);
        }
        _ => return Err(Error {}),
    }
}
