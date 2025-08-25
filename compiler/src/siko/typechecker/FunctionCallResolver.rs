use std::iter::zip;

use crate::siko::{
    hir::{
        Apply::Apply,
        ConstraintContext::ConstraintContext,
        Function::Function,
        ImplementationResolver::{ImplSearchResult, ImplementationResolver},
        ImplementationStore::ImplementationStore,
        Instantiation::{instantiateType, instantiateTypes},
        Instruction::ImplementationReference,
        Program::Program,
        Substitution::Substitution,
        Type::{formatTypes, Type},
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
        Variable::Variable,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::QualifiedName,
    typechecker::{ConstraintExpander::ConstraintExpander, Error::TypecheckerError},
};

pub struct CheckFunctionCallResult {
    pub fnType: Type,
    pub fnName: QualifiedName,
    pub implRefs: Vec<ImplementationReference>,
}

pub struct FunctionCallResolver<'a> {
    program: &'a Program,
    allocator: TypeVarAllocator,
    ctx: &'a ReportContext,
    implStore: &'a ImplementationStore,
    substitution: Substitution,
    knownConstraints: ConstraintContext,
}

impl<'a> FunctionCallResolver<'a> {
    pub fn new(
        program: &'a Program,
        allocator: TypeVarAllocator,
        ctx: &'a ReportContext,
        implStore: &'a ImplementationStore,
        knownConstraints: ConstraintContext,
        substitution: Substitution,
    ) -> FunctionCallResolver<'a> {
        FunctionCallResolver {
            program,
            allocator,
            ctx,
            implStore,
            knownConstraints,
            substitution,
        }
    }

    fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = unify(&mut self.substitution, ty1.clone(), ty2.clone(), false) {
            reportTypeMismatch(
                self.ctx,
                ty1.apply(&self.substitution),
                ty2.apply(&self.substitution),
                location,
            );
        }
    }

    fn tryUnify(&mut self, ty1: Type, ty2: Type) -> bool {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = unify(&mut self.substitution, ty1.clone(), ty2.clone(), false) {
            return false;
        }
        true
    }

    fn unifyVar(&mut self, var: &Variable, ty: Type) {
        self.unify(var.getType(), ty, var.location().clone());
    }

    fn createConstraintExpander(&mut self, constraints: ConstraintContext) -> ConstraintExpander {
        let expander = ConstraintExpander::new(self.program, self.allocator.clone(), constraints);
        expander
    }

    pub fn resolve(
        &mut self,
        f: &Function,
        args: &Vec<Variable>,
        resultVar: &Variable,
        location: Location,
    ) -> (CheckFunctionCallResult, Substitution) {
        let implResolver = ImplementationResolver::new(self.allocator.clone(), self.implStore, self.program);
        let fnType = f.getType();
        let fnType = fnType.resolveSelfType();
        let mut checkResult = CheckFunctionCallResult {
            fnType: fnType.clone(),
            fnName: f.name.clone(),
            implRefs: Vec::new(),
        };
        // println!(
        //     "Checking protocol (method?) call: {} {} {} {}",
        //     f.name, fnType, f.constraintContext, self.knownConstraints
        // );
        //let destType = self.getType(resultVar).apply(&self.substitution);
        //println!("Dest type: {}", destType);
        let mut types = f.constraintContext.typeParameters.clone();
        types.push(fnType.clone());
        let sub = instantiateTypes(&mut self.allocator, &types);
        let expectedFnType = fnType.apply(&sub);
        //println!("Instantiated fn type: {}", expectedFnType);
        let neededConstraints = self
            .createConstraintExpander(f.constraintContext.clone())
            .expandKnownConstraints();
        let neededConstraints = neededConstraints.apply(&sub);
        //println!("Needed constraints: {}", neededConstraints);
        let (expectedArgs, mut expectedResult) = match expectedFnType.clone().splitFnType() {
            Some((fnArgs, fnResult)) => (fnArgs, fnResult),
            None => panic!("Function type {} is not a function type!", expectedFnType),
        };
        if args.len() != expectedArgs.len() {
            TypecheckerError::ArgCountMismatch(expectedArgs.len() as u32, args.len() as u32, location.clone())
                .report(self.ctx);
        }
        {
            let mut argTypes = Vec::new();
            for arg in args {
                let ty = arg.getType().apply(&self.substitution);
                //println!("Arg type: {}", ty);
                if !ty.isSpecified(false) {
                    TypecheckerError::TypeAnnotationNeeded(arg.location().clone()).report(self.ctx);
                }
                argTypes.push(ty);
            }
            //println!("Expected args: {:?}, got args: {:?}", expectedArgs, argTypes);
        }
        if expectedArgs.len() > 0 {
            expectedResult = expectedResult.changeSelfType(expectedArgs[0].clone());
        }
        for (arg, expectedArg) in zip(args, &expectedArgs) {
            let expectedArg = expectedArg.clone().apply(&self.substitution);
            self.updateConverterDestination(arg, &expectedArg);
        }
        self.unifyVar(resultVar, expectedResult.clone());
        let neededConstraints = neededConstraints.clone().apply(&self.substitution);
        //println!("Needed constraints: {}", neededConstraints);
        let neededConstraints = neededConstraints.constraints;
        let mut remaining = neededConstraints.clone();
        let mut tryMore = true;
        if neededConstraints.len() > 0 {
            loop {
                let mut resolvedSomething = false;
                remaining = remaining.apply(&self.substitution);
                //println!("Remaining constraints: {:?} {:?}", remaining, neededConstraints);
                if remaining.is_empty() {
                    break;
                }
                for _ in 0..neededConstraints.len() {
                    if remaining.is_empty() {
                        break;
                    }
                    let current = remaining.remove(0);
                    if let Some((index, foundConstraint)) =
                        implResolver.findImplInKnownConstraints(&current, &self.knownConstraints)
                    {
                        // impl will be provided later during mono
                        for (a, b) in zip(&current.associatedTypes, &foundConstraint.associatedTypes) {
                            //println!("Unifying impl assoc {} with constraint assoc {}", a, b);
                            self.unify(a.ty.clone(), b.ty.clone(), location.clone());
                        }
                        //println!("---------- Using known implementation for {}", current);
                        let expectedFnType = expectedFnType.clone().apply(&self.substitution);
                        let expectedResult = expectedResult.clone().apply(&self.substitution);
                        self.unifyVar(resultVar, expectedResult);
                        //println!("expected fn type {}", expectedFnType);
                        checkResult.fnType = expectedFnType.clone();
                        checkResult.fnName = f.name.clone();
                        checkResult.implRefs.push(ImplementationReference::Indirect(index));
                    } else {
                        // we will select an impl now
                        //println!("---------- Trying to find implementation for {}", current);

                        match implResolver.findImplInScope(&current) {
                            ImplSearchResult::Found(implDef) => {
                                resolvedSomething = true;
                                //println!("Found impl {}", implDef.name);
                                for (a, b) in zip(&implDef.associatedTypes, &current.associatedTypes) {
                                    //println!("Unifying impl assoc {} with constraint assoc {}", a, b);
                                    self.unify(a.ty.clone(), b.ty.clone(), location.clone());
                                }
                                checkResult
                                    .implRefs
                                    .push(ImplementationReference::Direct(implDef.name.clone()));
                                if implDef.protocolName.add(f.name.getShortName()) == f.name {
                                    let mut found = false;
                                    for m in &implDef.members {
                                        if m.name == f.name.getShortName() {
                                            //Found matching member, apply substitution
                                            //println!("Will call {}", m.fullName);
                                            //println!("Impl member result ty {}", m.memberType);
                                            //println!("Impl member expected ty {}", expectedFnType);
                                            //println!("Unifying {} and {}", expectedFnType, m.memberType);
                                            self.unify(expectedFnType.clone(), m.memberType.clone(), location.clone());
                                            let expectedFnType = expectedFnType.clone().apply(&self.substitution);
                                            let expectedResult = expectedResult.clone().apply(&self.substitution);
                                            self.unifyVar(resultVar, expectedResult);
                                            checkResult.fnType = expectedFnType.clone();
                                            checkResult.fnName = m.fullName.clone();
                                            found = true;
                                        }
                                    }
                                    if !found {
                                        let protoDef = self
                                            .program
                                            .getProtocol(&implDef.protocolName)
                                            .expect("Protocol not found");
                                        for m in protoDef.members {
                                            if m.name == f.name.getShortName() {
                                                //Found matching member, apply substitution
                                                let memberType =
                                                    instantiateType(&mut self.allocator, m.memberType.clone());
                                                self.unify(expectedFnType.clone(), memberType, location.clone());
                                                let expectedFnType = expectedFnType.clone().apply(&self.substitution);
                                                let expectedResult = expectedResult.clone().apply(&self.substitution);
                                                self.unifyVar(resultVar, expectedResult);
                                                checkResult.fnType = expectedFnType.clone();
                                                checkResult.fnName = m.fullName.clone();
                                                found = true;
                                            }
                                        }
                                    }
                                    if !found {
                                        panic!("Method {} not found in implementation {}", f.name, implDef.name);
                                    }
                                }
                            }
                            ImplSearchResult::Ambiguous => {
                                if tryMore {
                                    //println!("Ambiguous implementations found for {}", current);
                                    remaining.push(current);
                                } else {
                                    TypecheckerError::AmbiguousImplementations(
                                        current.name.toString(),
                                        formatTypes(&current.args),
                                        location.clone(),
                                    )
                                    .report(self.ctx);
                                }
                            }
                            ImplSearchResult::NotFound => {
                                if tryMore {
                                    //println!("No implementation found for {}", current);
                                    remaining.push(current);
                                } else {
                                    TypecheckerError::NoImplementationFound(current.to_string(), location.clone())
                                        .report(self.ctx);
                                }
                            }
                        }
                    }
                }
                if !resolvedSomething {
                    tryMore = false;
                }
            }
        }
        let expectedFnType = expectedFnType.apply(&self.substitution);
        let expectedResult = expectedResult.apply(&self.substitution);
        checkResult.fnType = expectedFnType.clone();
        self.unifyVar(resultVar, expectedResult);
        //println!("result impl refs {:?}", checkResult.implRefs);
        //println!("result name {}", checkResult.fnName);
        assert_eq!(checkResult.implRefs.len(), neededConstraints.len());
        (checkResult, self.substitution.clone())
    }

    fn updateConverterDestination(&mut self, dest: &Variable, target: &Type) {
        let destTy = dest.getType().apply(&self.substitution);
        let targetTy = target.clone().apply(&self.substitution);
        //println!("Updating converter destination: {} -> {}", destTy, targetTy);
        if !self.tryUnify(destTy.clone(), targetTy.clone()) {
            match (destTy, targetTy.clone()) {
                (ty1, Type::Reference(ty2, _)) => {
                    self.tryUnify(ty1, *ty2.clone());
                }
                (Type::Reference(ty1, _), ty2) => {
                    self.tryUnify(*ty1.clone(), ty2);
                }
                (ty1, ty2) => {
                    self.tryUnify(ty1, ty2);
                }
            }
            let targetTy = target.clone().apply(&self.substitution);
            dest.setType(targetTy);
        }
    }
}

pub fn reportTypeMismatch(ctx: &ReportContext, ty1: Type, ty2: Type, location: Location) {
    TypecheckerError::TypeMismatch(format!("{}", ty1), format!("{}", ty2), location).report(ctx)
}
