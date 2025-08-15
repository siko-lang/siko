use crate::siko::{
    hir::{
        Apply::Apply, ConstraintContext::ConstraintContext, InstanceResolver::ResolutionResult, Program::Program,
        Substitution::Substitution, Type::formatTypes, TypeVarAllocator::TypeVarAllocator, Unification::unify,
    },
    location::{Location::Location, Report::ReportContext},
    typechecker::Error::TypecheckerError,
};

pub struct ConstraintChecker<'a> {
    pub allocator: TypeVarAllocator,
    ctx: &'a ReportContext,
    program: &'a Program,
    knownConstraints: &'a ConstraintContext,
    pub substitution: Substitution,
}

impl<'a> ConstraintChecker<'a> {
    pub fn new(
        allocator: &TypeVarAllocator,
        ctx: &'a ReportContext,
        program: &'a Program,
        knownConstraints: &'a ConstraintContext,
        substitution: &Substitution,
    ) -> ConstraintChecker<'a> {
        ConstraintChecker {
            allocator: allocator.clone(),
            ctx,
            program,
            knownConstraints,
            substitution: substitution.clone(),
        }
    }

    pub fn checkConstraint(
        &mut self,
        neededConstraints: &ConstraintContext,
        location: Location,
    ) -> Result<(), TypecheckerError> {
        // println!("------------------------------------");
        // println!("needed {}", neededConstraints);
        // println!("known {}", self.knownConstraints);
        for c in &neededConstraints.constraints {
            //println!("knownConstraints.contains(c) {}", self.knownConstraints.contains(c));
            if !self.knownConstraints.contains(c) {
                if let Some(instances) = self.program.instanceResolver.lookupInstances(&c.traitName) {
                    //println!("c.args {}", formatTypes(&c.args));
                    let mut fullyGeneric = true;
                    for arg in &c.args {
                        if !arg.isTypeVar() {
                            fullyGeneric = false;
                            break;
                        }
                    }
                    if fullyGeneric {
                        continue;
                    }
                    let resolutionResult = instances.find(&mut self.allocator, &c.args);
                    match resolutionResult {
                        ResolutionResult::Winner(instance) => {
                            //println!("Base Winner {} for {}", instance, formatTypes(&c.args));
                            let instance = instance.apply(&self.substitution);
                            //println!("Winner {} for {}", instance, formatTypes(&c.args));
                            for ctxAssocTy in &c.associatedTypes {
                                for instanceAssocTy in &instance.associatedTypes {
                                    if instanceAssocTy.name == ctxAssocTy.name {
                                        //println!("ASSOC MATCH {} {}", instanceAssocTy.ty, ctxAssocTy.ty);
                                        if let Err(_) = unify(
                                            &mut self.substitution,
                                            instanceAssocTy.ty.clone(),
                                            ctxAssocTy.ty.clone(),
                                            false,
                                        ) {
                                            return Err(TypecheckerError::TypeMismatch(
                                                format!("{}", instanceAssocTy.ty.clone().apply(&self.substitution)),
                                                format!("{}", ctxAssocTy.ty.clone().apply(&self.substitution)),
                                                location.clone(),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        ResolutionResult::Ambiguous(_) => {
                            return Err(TypecheckerError::AmbiguousInstances(
                                c.traitName.toString(),
                                formatTypes(&c.args),
                                location.clone(),
                                Vec::new(),
                            ));
                        }
                        ResolutionResult::NoInstanceFound => {
                            return Err(TypecheckerError::InstanceNotFound(
                                c.traitName.toString(),
                                formatTypes(&c.args),
                                location.clone(),
                            ));
                        }
                    }
                } else {
                    return Err(TypecheckerError::InstanceNotFound(
                        c.traitName.toString(),
                        formatTypes(&c.args),
                        location.clone(),
                    ));
                }
            }
        }
        Ok(())
    }
}
