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
        Type::{formatTypes, Type},
        TypeVarAllocator::TypeVarAllocator,
        Unifier::Unifier,
        Variable::Variable,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::{builtins::getCloneFnName, QualifiedName},
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
    unifier: Unifier<'a>,
    knownConstraints: ConstraintContext,
}

impl<'a> FunctionCallResolver<'a> {
    pub fn new(
        program: &'a Program,
        allocator: TypeVarAllocator,
        ctx: &'a ReportContext,
        implStore: &'a ImplementationStore,
        knownConstraints: ConstraintContext,
        unifier: Unifier<'a>,
    ) -> FunctionCallResolver<'a> {
        FunctionCallResolver {
            program,
            allocator,
            ctx,
            implStore,
            knownConstraints,
            unifier,
        }
    }

    pub fn resolve(
        &mut self,
        f: &Function,
        args: &Vec<Variable>,
        resultVar: &Variable,
        location: Location,
    ) -> CheckFunctionCallResult {
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
        let neededConstraints =
            ConstraintExpander::new(self.program, self.allocator.clone(), f.constraintContext.clone())
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
                let ty = self.unifier.apply(arg.getType());
                //println!("Arg type: {}", ty);
                // if !ty.isSpecified(false) {
                //     TypecheckerError::TypeAnnotationNeeded(arg.location().clone()).report(self.ctx);
                // }
                argTypes.push(ty);
            }
            //println!("Expected args: {:?}, got args: {:?}", expectedArgs, argTypes);
        }
        if expectedArgs.len() > 0 {
            expectedResult = expectedResult.changeSelfType(expectedArgs[0].clone());
        }
        for (arg, expectedArg) in zip(args, &expectedArgs) {
            let expectedArg = self.unifier.apply(expectedArg.clone());
            self.unifier.updateConverterDestination(arg, &expectedArg);
        }
        self.unifier.unifyVar(resultVar, expectedResult.clone());
        let neededConstraints = self.unifier.apply(neededConstraints.clone());
        //println!("Needed constraints: {}", neededConstraints);
        let neededConstraints = neededConstraints.constraints;
        let mut remaining = neededConstraints.clone();
        let mut tryMore = true;
        if neededConstraints.len() > 0 {
            loop {
                let mut resolvedSomething = false;
                remaining = self.unifier.apply(remaining);
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
                            self.unifier.unify(a.ty.clone(), b.ty.clone(), location.clone());
                        }
                        //println!("---------- Using known implementation for {}", current);
                        let expectedFnType = self.unifier.apply(expectedFnType.clone());
                        let expectedResult = self.unifier.apply(expectedResult.clone());
                        self.unifier.unifyVar(resultVar, expectedResult);
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
                                    self.unifier.unify(a.ty.clone(), b.ty.clone(), location.clone());
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
                                            self.unifier.unify(
                                                expectedFnType.clone(),
                                                m.memberType.clone(),
                                                location.clone(),
                                            );
                                            let expectedFnType = self.unifier.apply(expectedFnType.clone());
                                            let expectedResult = self.unifier.apply(expectedResult.clone());
                                            self.unifier.unifyVar(resultVar, expectedResult);
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
                                                self.unifier.unify(
                                                    expectedFnType.clone(),
                                                    memberType,
                                                    location.clone(),
                                                );
                                                let expectedFnType = self.unifier.apply(expectedFnType.clone());
                                                let expectedResult = self.unifier.apply(expectedResult.clone());
                                                self.unifier.unifyVar(resultVar, expectedResult);
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
        let expectedFnType = self.unifier.apply(expectedFnType);
        let expectedResult = self.unifier.apply(expectedResult);
        checkResult.fnType = expectedFnType.clone();
        self.unifier.unifyVar(resultVar, expectedResult);
        //println!("result impl refs {:?}", checkResult.implRefs);
        //println!("result name {}", checkResult.fnName);
        assert_eq!(checkResult.implRefs.len(), neededConstraints.len());
        checkResult
    }

    pub fn resolveCloneCall(
        &mut self,
        arg: Variable,
        resultVar: Variable,
    ) -> (QualifiedName, Vec<ImplementationReference>) {
        let cloneFn = self
            .program
            .getFunction(&getCloneFnName())
            .expect("Clone function not found");
        let result = self.resolve(&cloneFn, &vec![arg.clone()], &resultVar, arg.location().clone());
        let implFn = self
            .program
            .getFunction(&result.fnName)
            .expect("Implementation of clone function not found");
        let result = self.resolve(&implFn, &vec![arg.clone()], &resultVar, arg.location().clone());
        (result.fnName, result.implRefs)
    }
}
