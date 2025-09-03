use std::{cell::RefCell, rc::Rc};

use crate::siko::{
    hir::{Apply::Apply, Substitution::Substitution, Type::Type, Unification::unify, Variable::Variable},
    location::{Location::Location, Report::Report, Report::ReportContext},
};

#[derive(Clone)]
pub struct Unifier<'a> {
    ctx: &'a ReportContext,
    pub substitution: Rc<RefCell<Substitution>>,
}

impl<'a> Unifier<'a> {
    pub fn new(ctx: &'a ReportContext) -> Unifier<'a> {
        Unifier {
            ctx,
            substitution: Rc::new(RefCell::new(Substitution::new())),
        }
    }

    pub fn apply<T: Apply>(&self, item: T) -> T {
        let sub = self.substitution.borrow();
        item.apply(&sub)
    }

    pub fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        //println!("UNIFY {} {}", ty1, ty2);
        let mut sub = self.substitution.borrow_mut();
        if let Err(_) = unify(&mut sub, ty1.clone(), ty2.clone(), false) {
            let ty = ty1.apply(&sub);
            let ty2 = ty2.apply(&sub);
            UnifierError::TypeMismatch(format!("{}", ty), format!("{}", ty2), location).report(self.ctx)
        }
    }

    pub fn tryUnify(&mut self, ty1: Type, ty2: Type) -> bool {
        //println!("UNIFY {} {}", ty1, ty2);
        let mut sub = self.substitution.borrow_mut();
        if let Err(_) = unify(&mut sub, ty1.clone(), ty2.clone(), false) {
            return false;
        }
        true
    }

    pub fn unifyVar(&mut self, var: &Variable, ty: Type) {
        self.unify(var.getType(), ty, var.location().clone());
    }

    pub fn unifyVars(&mut self, var1: &Variable, var2: &Variable) {
        self.unify(var1.getType(), var2.getType(), var1.location().clone());
    }

    pub fn updateConverterDestination(&mut self, dest: &Variable, target: &Type) {
        let destTy = self.apply(dest.getType());
        let targetTy = self.apply(target.clone());
        //println!("Updating converter destination: {} -> {}", destTy, targetTy);
        if !self.tryUnify(destTy.clone(), targetTy.clone()) {
            match (destTy, targetTy.clone()) {
                (ty1, Type::Reference(ty2)) => {
                    self.tryUnify(ty1, *ty2);
                }
                (Type::Reference(ty1), ty2) => {
                    self.tryUnify(*ty1, ty2);
                }
                (ty1, ty2) => {
                    self.tryUnify(ty1, ty2);
                }
            }
            let targetTy = self.apply(target.clone());
            dest.setType(targetTy);
        }
    }
}

pub enum UnifierError {
    TypeMismatch(String, String, Location),
}

impl UnifierError {
    pub fn report(&self, ctx: &ReportContext) -> ! {
        match &self {
            UnifierError::TypeMismatch(ty1, ty2, l) => {
                let slogan = format!("Type mismatch: {}, {}", ctx.yellow(ty1), ctx.yellow(ty2));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
        }
        std::process::exit(1);
    }
}
