use std::iter::zip;

use crate::siko::{
    hir::{
        Apply::Apply,
        ConstraintContext::ConstraintContext,
        Function::Function,
        InstanceResolver::{InstanceResolver, InstanceSearchResult},
        InstanceStore::InstanceStore,
        Instantiation::{instantiateType, instantiateTypes},
        Instruction::InstanceReference,
        Program::Program,
        Type::{formatTypes, Type},
        TypeVarAllocator::TypeVarAllocator,
        Unifier::Unifier,
        Variable::Variable,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::{
        builtins::{getCloneFnName, getDropFnName},
        QualifiedName,
    },
    typechecker::{ConstraintExpander::ConstraintExpander, Error::TypecheckerError},
};

pub struct CheckFunctionCallResult {
    pub fnType: Type,
    pub fnName: QualifiedName,
    pub instanceRefs: Vec<InstanceReference>,
}

pub struct FunctionCallResolver<'a> {
    program: &'a Program,
    allocator: TypeVarAllocator,
    ctx: &'a ReportContext,
    implStore: &'a InstanceStore,
    unifier: Unifier<'a>,
    knownConstraints: ConstraintContext,
}

impl<'a> FunctionCallResolver<'a> {
    pub fn new(
        program: &'a Program,
        allocator: TypeVarAllocator,
        ctx: &'a ReportContext,
        implStore: &'a InstanceStore,
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
        let implResolver = InstanceResolver::new(
            self.allocator.clone(),
            self.implStore,
            self.program,
            self.knownConstraints.clone(),
        );
        let fnType = f.getType();
        let fnType = fnType.resolveSelfType();
        let mut checkResult = CheckFunctionCallResult {
            fnType: fnType.clone(),
            fnName: f.name.clone(),
            instanceRefs: Vec::new(),
        };
        // println!(
        //     "Checking trait (method?) call: {} {} {} {}",
        //     f.name, fnType, f.constraintContext, self.knownConstraints
        // );
        // let destType = self.unifier.apply(resultVar.getType());
        // println!("Dest type: {}", destType);
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
                //println!("Remaining constraints: {:?}", remaining);
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
                        //println!("---------- Using known instance for {}", current);
                        let expectedFnType = self.unifier.apply(expectedFnType.clone());
                        let expectedResult = self.unifier.apply(expectedResult.clone());
                        self.unifier.unifyVar(resultVar, expectedResult);
                        //println!("expected fn type {}", expectedFnType);
                        checkResult.fnType = expectedFnType.clone();
                        checkResult.fnName = f.name.clone();
                        checkResult.instanceRefs.push(InstanceReference::Indirect(index));
                    } else {
                        // we will select an instance now
                        //println!("---------- Trying to find instance for {}", current);

                        match implResolver.findInstanceInScope(&current) {
                            InstanceSearchResult::Found(instanceDef) => {
                                resolvedSomething = true;
                                //println!("Found impl {}", instanceDef.name);
                                for (a, b) in zip(&instanceDef.associatedTypes, &current.associatedTypes) {
                                    //println!("Unifying impl assoc {} with constraint assoc {}", a, b);
                                    self.unifier.unify(a.ty.clone(), b.ty.clone(), location.clone());
                                }
                                checkResult
                                    .instanceRefs
                                    .push(InstanceReference::Direct(instanceDef.name.clone()));
                                if instanceDef.traitName.add(f.name.getShortName()) == f.name {
                                    let mut found = false;
                                    for m in &instanceDef.members {
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
                                        let traitDef =
                                            self.program.getTrait(&instanceDef.traitName).expect("Trait not found");
                                        for m in traitDef.members {
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
                                        panic!("Method {} not found in instance {}", f.name, instanceDef.name);
                                    }
                                }
                            }
                            InstanceSearchResult::Ambiguous(names) => {
                                if tryMore {
                                    //println!("Ambiguous implementations found for {}", current);
                                    remaining.push(current);
                                } else {
                                    let instances = names.iter().map(|n| n.toString()).collect();
                                    TypecheckerError::AmbiguousImplementations(
                                        current.name.toString(),
                                        formatTypes(&current.args),
                                        instances,
                                        location.clone(),
                                    )
                                    .report(self.ctx);
                                }
                            }
                            InstanceSearchResult::NotFound => {
                                if tryMore {
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
        //println!("result impl refs {:?}", checkResult.instanceRefs);
        //println!("result name {}", checkResult.fnName);
        assert_eq!(checkResult.instanceRefs.len(), neededConstraints.len());
        checkResult
    }

    pub fn resolveCloneCall(&mut self, arg: Variable, resultVar: Variable) -> (QualifiedName, Vec<InstanceReference>) {
        let cloneFn = self
            .program
            .getFunction(&getCloneFnName())
            .expect("Clone function not found");
        let result = self.resolve(&cloneFn, &vec![arg.clone()], &resultVar, arg.location().clone());
        let implFn = self
            .program
            .getFunction(&result.fnName)
            .expect("Instance of clone function not found");
        let result = self.resolve(&implFn, &vec![arg.clone()], &resultVar, arg.location().clone());
        (result.fnName, result.instanceRefs)
    }

    pub fn resolveDropCall(&mut self, arg: Variable, resultVar: Variable) -> (QualifiedName, Vec<InstanceReference>) {
        let dropFn = self
            .program
            .getFunction(&getDropFnName())
            .expect("Drop function not found");
        let result = self.resolve(&dropFn, &vec![arg.clone()], &resultVar, arg.location().clone());
        let implFn = self
            .program
            .getFunction(&result.fnName)
            .expect("Instance of drop function not found");
        let result = self.resolve(&implFn, &vec![arg.clone()], &resultVar, arg.location().clone());
        (result.fnName, result.instanceRefs)
    }
}
