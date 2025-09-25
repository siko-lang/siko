use std::{cell::RefCell, rc::Rc};

use crate::siko::{
    hir::{Apply::Apply, Substitution::Substitution, Type::Type, Unification::unify, Variable::Variable},
    location::{
        Location::Location,
        Report::{Report, ReportContext},
    },
    util::Runner::Runner,
};

pub trait UnificationErrorHandler {
    fn handleError(&self, error: UnifierError);
}

#[derive(Clone)]
pub struct Unifier {
    handler: Rc<dyn UnificationErrorHandler>,
    pub substitution: Rc<RefCell<Substitution>>,
    pub verbose: bool,
    pub runner: Runner,
    pub applyRunner: Runner,
}

impl Unifier {
    pub fn withContext(ctx: &ReportContext, runner: Runner) -> Unifier {
        let applyRunner = runner.child("apply");
        Unifier {
            handler: Rc::new(DefaultUnificationErrorHandler::new(ctx.clone())),
            substitution: Rc::new(RefCell::new(Substitution::new())),
            verbose: false,
            runner,
            applyRunner,
        }
    }

    pub fn new(runner: Runner) -> Unifier {
        let applyRunner = runner.child("apply");
        Unifier {
            handler: Rc::new(InternalUnificationErrorHandler {}),
            substitution: Rc::new(RefCell::new(Substitution::new())),
            verbose: false,
            runner,
            applyRunner,
        }
    }

    pub fn apply<T: Apply>(&self, item: T) -> T {
        self.applyRunner.run(|| {
            let sub = self.substitution.borrow();
            item.apply(&sub)
        })
    }

    pub fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        if self.verbose {
            println!("Unifying {} and {}", ty1, ty2);
        }
        self.runner.run(|| {
            let mut sub = self.substitution.borrow_mut();
            if let Err(_) = unify(&mut sub, ty1.clone(), ty2.clone(), false) {
                let ty = ty1.apply(&sub);
                let ty2 = ty2.apply(&sub);
                self.handler.handleError(UnifierError::TypeMismatch(
                    format!("{}", ty),
                    format!("{}", ty2),
                    location,
                ));
            }
        });
    }

    pub fn tryUnify(&mut self, ty1: Type, ty2: Type) -> bool {
        //println!("UNIFY {} {}", ty1, ty2);
        self.runner.run(|| {
            let mut sub = self.substitution.borrow_mut();
            if let Err(_) = unify(&mut sub, ty1.clone(), ty2.clone(), false) {
                return false;
            }
            true
        })
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

pub struct DefaultUnificationErrorHandler {
    ctx: ReportContext,
}

impl DefaultUnificationErrorHandler {
    pub fn new(ctx: ReportContext) -> Self {
        DefaultUnificationErrorHandler { ctx }
    }
}

impl UnificationErrorHandler for DefaultUnificationErrorHandler {
    fn handleError(&self, error: UnifierError) {
        error.report(&self.ctx);
    }
}

pub struct InternalUnificationErrorHandler {}

impl UnificationErrorHandler for InternalUnificationErrorHandler {
    fn handleError(&self, error: UnifierError) {
        match error {
            UnifierError::TypeMismatch(ty1, ty2, location) => {
                panic!("Type mismatch: {} vs {} at {}", ty1, ty2, location);
            }
        }
    }
}
