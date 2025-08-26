use std::iter::zip;

use crate::siko::{
    hir::{
        Apply::Apply,
        ConstraintContext::{Constraint, ConstraintContext},
        ImplementationStore::ImplementationStore,
        Instantiation::instantiateImplementation,
        Program::Program,
        Substitution::Substitution,
        Trait::Implementation,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
    },
    qualifiedname::{
        builtins::{getCopyName, getDropName, getImplicitConvertName},
        QualifiedName,
    },
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
        if let ImplSearchResult::Found(implDef) =
            self.findImplementationForConstraint(constraint, &self.implementationStore.importedImplementations)
        {
            return ImplSearchResult::Found(implDef);
        }
        let mut canonTypes = Vec::new();
        for arg in &constraint.args {
            if let Some(canon) = self.canonicalizeType(arg.clone()) {
                canonTypes.push(canon);
            } else {
                return ImplSearchResult::NotFound;
            }
        }
        if let Some(implName) = self.program.canonicalImplStore.get(&constraint.name, &canonTypes) {
            //println!("Found canonical impl {} for {}", implName, formatTypes(&canonTypes));
            return self.findImplementationForConstraint(constraint, &vec![implName.clone()]);
        }
        ImplSearchResult::NotFound
    }

    pub fn findImplInKnownConstraints(
        &self,
        constraint: &Constraint,
        knownConstraints: &ConstraintContext,
    ) -> Option<(u32, Constraint)> {
        for (index, known) in knownConstraints.constraints.iter().enumerate() {
            if constraint.name == known.name && constraint.args.len() == known.args.len() {
                let mut sub = Substitution::new();
                let mut allMatch = true;
                for (arg, karg) in zip(&constraint.args, &known.args) {
                    if unify(&mut sub, arg.clone(), karg.clone(), false).is_err() {
                        allMatch = false;
                        break;
                    }
                }
                if allMatch {
                    let mut foundConstraint = known.clone();
                    foundConstraint = foundConstraint.apply(&sub);
                    return Some((index as u32, foundConstraint));
                }
            }
        }
        None
    }

    pub fn isCopy(&self, ty: &Type) -> bool {
        let constraint = Constraint {
            name: getCopyName(),
            args: vec![ty.clone()],
            associatedTypes: Vec::new(),
        };
        self.findImplInScope(&constraint).isFound()
    }

    pub fn isDrop(&self, ty: &Type) -> bool {
        let constraint = Constraint {
            name: getDropName(),
            args: vec![ty.clone()],
            associatedTypes: Vec::new(),
        };
        self.findImplInScope(&constraint).isFound()
    }

    pub fn isImplicitConvert(&self, src: &Type, dest: &Type) -> bool {
        //println!("Checking implicit convert from {} to {}", src, dest);
        let constraint = Constraint {
            name: getImplicitConvertName(),
            args: vec![src.clone(), dest.clone()],
            associatedTypes: Vec::new(),
        };
        // println!("Constraint: {}", constraint);
        self.findImplInScope(&constraint).isFound()
    }

    fn canonicalizeType(&self, ty: Type) -> Option<Type> {
        match ty {
            Type::Named(name, args) => {
                let mut newArgs = Vec::new();
                for _ in args {
                    newArgs.push(self.allocator.next());
                }
                Some(Type::Named(name, newArgs))
            }
            _ => None,
        }
    }
}
