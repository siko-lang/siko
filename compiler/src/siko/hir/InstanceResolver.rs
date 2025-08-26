use std::iter::zip;

use crate::siko::{
    hir::{
        Apply::Apply,
        ConstraintContext::{Constraint, ConstraintContext},
        InstanceStore::InstanceStore,
        Instantiation::instantiateInstance,
        Program::Program,
        Substitution::Substitution,
        Trait::Instance,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
    },
    qualifiedname::{
        builtins::{getCopyName, getDropName, getImplicitConvertName},
        QualifiedName,
    },
};

pub enum InstanceSearchResult {
    Found(Instance),
    Ambiguous,
    NotFound,
}
impl InstanceSearchResult {
    fn isFound(&self) -> bool {
        match self {
            InstanceSearchResult::Found(_) => true,
            _ => false,
        }
    }
}

pub struct InstanceResolver<'a> {
    allocator: TypeVarAllocator,
    instanceStore: &'a InstanceStore,
    program: &'a Program,
}

impl<'a> InstanceResolver<'a> {
    pub fn new(allocator: TypeVarAllocator, instanceStore: &'a InstanceStore, program: &'a Program) -> Self {
        InstanceResolver {
            allocator,
            instanceStore: instanceStore,
            program,
        }
    }

    fn findInstanceForConstraint(
        &self,
        constraint: &Constraint,
        candidates: &Vec<QualifiedName>,
    ) -> InstanceSearchResult {
        //println!("Finding instance for constraint {}", constraint);
        let mut matchingImpls = Vec::new();
        for implName in candidates {
            let instanceDef = self.program.getInstance(&implName).expect("Impl not found");
            if constraint.name == instanceDef.traitName {
                if instanceDef.types.len() != constraint.args.len() {
                    continue;
                }
                let mut instanceDef = instantiateInstance(&self.allocator, &instanceDef);
                //println!("Trying impl {}", instanceDef);
                let mut allMatch = true;
                let mut sub = Substitution::new();
                for (implArg, cArg) in zip(&instanceDef.types, &constraint.args) {
                    //println!("  Unifying impl arg {} with constraint arg {}", implArg, cArg);
                    if !unify(&mut sub, implArg.clone(), cArg.clone(), false).is_ok() {
                        //println!("  Arg {} does not match {}", implArg, cArg);
                        allMatch = false;
                        break;
                    }
                }
                if allMatch {
                    //println!("Applying substitution: {}", sub);
                    instanceDef = instanceDef.apply(&sub);
                    //println!("Impl after applying substitution: {}", instanceDef);
                    let mut allSubConstraintsMatch = true;
                    for c in &instanceDef.constraintContext.constraints {
                        //println!("  checking sub constraint: {}", c);
                        if !self.findInstanceInScope(c).isFound() {
                            allSubConstraintsMatch = false;
                            break;
                        }
                    }
                    if allSubConstraintsMatch {
                        matchingImpls.push(instanceDef);
                    } else {
                        //println!("  sub constraints do not match");
                    }
                }
            }
        }
        //println!("Found {} matching impls for {}", matchingImpls.len(), constraint);
        if matchingImpls.len() > 1 {
            InstanceSearchResult::Ambiguous
        } else {
            match matchingImpls.pop() {
                Some(instanceDef) => InstanceSearchResult::Found(instanceDef),
                None => InstanceSearchResult::NotFound,
            }
        }
    }

    pub fn findInstanceInScope(&self, constraint: &Constraint) -> InstanceSearchResult {
        if let InstanceSearchResult::Found(instanceDef) =
            self.findInstanceForConstraint(constraint, &self.instanceStore.localInstances)
        {
            return InstanceSearchResult::Found(instanceDef);
        }
        if let InstanceSearchResult::Found(instanceDef) =
            self.findInstanceForConstraint(constraint, &self.instanceStore.importedInstances)
        {
            return InstanceSearchResult::Found(instanceDef);
        }
        let mut canonTypes = Vec::new();
        for arg in &constraint.args {
            if let Some(canon) = self.canonicalizeType(arg.clone()) {
                canonTypes.push(canon);
            } else {
                return InstanceSearchResult::NotFound;
            }
        }
        if let Some(implName) = self.program.canonicalImplStore.get(&constraint.name, &canonTypes) {
            //println!("Found canonical impl {} for {}", implName, formatTypes(&canonTypes));
            return self.findInstanceForConstraint(constraint, &vec![implName.clone()]);
        }
        InstanceSearchResult::NotFound
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
        self.findInstanceInScope(&constraint).isFound()
    }

    pub fn isDrop(&self, ty: &Type) -> bool {
        let constraint = Constraint {
            name: getDropName(),
            args: vec![ty.clone()],
            associatedTypes: Vec::new(),
        };
        self.findInstanceInScope(&constraint).isFound()
    }

    pub fn isImplicitConvert(&self, src: &Type, dest: &Type) -> bool {
        //println!("Checking implicit convert from {} to {}", src, dest);
        let constraint = Constraint {
            name: getImplicitConvertName(),
            args: vec![src.clone(), dest.clone()],
            associatedTypes: Vec::new(),
        };
        // println!("Constraint: {}", constraint);
        self.findInstanceInScope(&constraint).isFound()
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
