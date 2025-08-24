use std::iter::zip;

use crate::siko::{
    hir::{
        Apply::Apply, ConstraintContext::Constraint, ImplementationStore::ImplementationStore,
        Instantiation::instantiateImplementation, Program::Program, Substitution::Substitution, Trait::Implementation,
        TypeVarAllocator::TypeVarAllocator, Unification::unify,
    },
    qualifiedname::QualifiedName,
};

pub enum ImplSearchResult {
    Found(Implementation),
    Ambiguous,
    NotFound,
}
impl ImplSearchResult {
    fn isFound(&self) -> bool {
        match self {
            ImplSearchResult::Found(_) => true,
            _ => false,
        }
    }
}

pub struct ImplementationResolver<'a> {
    allocator: TypeVarAllocator,
    implementationStore: &'a ImplementationStore,
    program: &'a Program,
}

impl<'a> ImplementationResolver<'a> {
    pub fn new(
        allocator: TypeVarAllocator,
        implementationStore: &'a ImplementationStore,
        program: &'a Program,
    ) -> Self {
        ImplementationResolver {
            allocator,
            implementationStore,
            program,
        }
    }

    fn findImplementationForConstraint(
        &self,
        constraint: &Constraint,
        candidates: &Vec<QualifiedName>,
    ) -> ImplSearchResult {
        //println!("Finding implementation for constraint {}", constraint);
        let mut matchingImpls = Vec::new();
        for implName in candidates {
            let implDef = self.program.getImplementation(&implName).expect("Impl not found");
            if constraint.name == implDef.protocolName {
                if implDef.types.len() != constraint.args.len() {
                    continue;
                }
                let mut implDef = instantiateImplementation(&self.allocator, &implDef);
                //println!("Trying impl {}", implDef);
                let mut allMatch = true;
                let mut sub = Substitution::new();
                for (implArg, cArg) in zip(&implDef.types, &constraint.args) {
                    //println!("  Unifying impl arg {} with constraint arg {}", implArg, cArg);
                    if !unify(&mut sub, implArg.clone(), cArg.clone(), false).is_ok() {
                        //println!("  Arg {} does not match {}", implArg, cArg);
                        allMatch = false;
                        break;
                    }
                }
                if allMatch {
                    //println!("Applying substitution: {}", sub);
                    implDef = implDef.apply(&sub);
                    //println!("Impl after applying substitution: {}", implDef);
                    let mut allSubConstraintsMatch = true;
                    for c in &implDef.constraintContext.constraints {
                        //println!("  checking sub constraint: {}", c);
                        if !self.findImplInScope(c).isFound() {
                            allSubConstraintsMatch = false;
                            break;
                        }
                    }
                    if allSubConstraintsMatch {
                        matchingImpls.push(implDef);
                    } else {
                        //println!("  sub constraints do not match");
                    }
                }
            }
        }
        //println!("Found {} matching impls for {}", matchingImpls.len(), constraint);
        if matchingImpls.len() > 1 {
            ImplSearchResult::Ambiguous
        } else {
            match matchingImpls.pop() {
                Some(implDef) => ImplSearchResult::Found(implDef),
                None => ImplSearchResult::NotFound,
            }
        }
    }

    pub fn findImplInScope(&self, constraint: &Constraint) -> ImplSearchResult {
        if let ImplSearchResult::Found(implDef) =
            self.findImplementationForConstraint(constraint, &self.implementationStore.localImplementations)
        {
            return ImplSearchResult::Found(implDef);
        }
        self.findImplementationForConstraint(constraint, &self.implementationStore.importedImplementations)
    }
}
