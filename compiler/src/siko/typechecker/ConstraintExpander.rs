use std::iter::zip;

use crate::siko::hir::{
    Apply::Apply,
    ConstraintContext::{Constraint, ConstraintContext},
    Instantiation::{instantiateProtocol, instantiateTrait},
    Program::Program,
    Substitution::Substitution,
    TypeVarAllocator::TypeVarAllocator,
};

pub struct ConstraintExpander<'a> {
    pub program: &'a Program,
    pub allocator: TypeVarAllocator,
    pub knownConstraints: ConstraintContext,
}

impl<'a> ConstraintExpander<'a> {
    pub fn new(program: &'a Program, allocator: TypeVarAllocator, knownConstraints: ConstraintContext) -> Self {
        ConstraintExpander {
            program,
            allocator,
            knownConstraints,
        }
    }

    pub fn expandKnownConstraints(mut self) -> ConstraintContext {
        //println!("expandKnownConstraints {}", self.f.name);
        let mut processed = Vec::new();
        let start = self.knownConstraints.constraints.clone();
        for c in &start {
            self.expandKnownConstraint(c, &mut processed);
        }
        self.knownConstraints.constraints.sort();
        self.knownConstraints.constraints.dedup();
        self.knownConstraints
    }

    fn expandKnownConstraint(&mut self, c: &Constraint, processed: &mut Vec<Constraint>) {
        if processed.contains(c) {
            return;
        }
        //println!("expandKnownConstraint {}", c);
        processed.push(c.clone());
        match self.program.getTrait(&c.name) {
            Some(traitDef) => {
                let traitDef = instantiateTrait(&mut self.allocator, &traitDef);
                let mut sub = Substitution::new();
                for (arg, ctxArg) in zip(&traitDef.params, &c.args) {
                    sub.add(arg.clone(), ctxArg.clone());
                }
                let traitDef = traitDef.apply(&sub);
                self.knownConstraints.constraints.push(c.clone());
                for c in traitDef.constraint.constraints {
                    self.expandKnownConstraint(&c, processed);
                }
            }
            None => {
                let protoDef = self.program.getProtocol(&c.name).expect("Protocol not found");
                let protoDef = instantiateProtocol(&mut self.allocator, &protoDef);
                let mut sub = Substitution::new();
                for (arg, ctxArg) in zip(&protoDef.params, &c.args) {
                    sub.add(arg.clone(), ctxArg.clone());
                }
                let protoDef = protoDef.apply(&sub);
                self.knownConstraints.constraints.push(c.clone());
                for c in protoDef.constraint.constraints {
                    self.expandKnownConstraint(&c, processed);
                }
            }
        };
    }
}
