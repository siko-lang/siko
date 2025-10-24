use std::{cell::RefCell, collections::BTreeMap, iter::zip, rc::Rc};

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
        Unification::{unify, Config},
    },
    qualifiedname::{
        builtins::{getCopyName, getDropName, getImplicitConvertName},
        QualifiedName,
    },
    util::Runner::Runner,
};

pub enum InstanceSearchResult {
    Found(Instance),
    Ambiguous(Vec<QualifiedName>),
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
    program: &'a Program,
    knownConstraints: ConstraintContext,
    localInstances: BTreeMap<QualifiedName, Vec<Rc<Instance>>>,
    importedInstances: BTreeMap<QualifiedName, Vec<Rc<Instance>>>,
    cache: Rc<RefCell<BTreeMap<Constraint, Instance>>>,
}

impl<'a> InstanceResolver<'a> {
    pub fn new(
        allocator: TypeVarAllocator,
        instanceStore: &'a InstanceStore,
        program: &'a Program,
        knownConstraints: ConstraintContext,
    ) -> Self {
        let mut localInstances = BTreeMap::new();
        for localInstance in &instanceStore.localInstances {
            if let Some(instanceDef) = program.getInstance(localInstance) {
                let entry = localInstances
                    .entry(instanceDef.traitName.clone())
                    .or_insert_with(Vec::new);
                entry.push(instanceDef);
            }
        }
        let mut importedInstances = BTreeMap::new();
        for importedInstance in &instanceStore.importedInstances {
            if let Some(instanceDef) = program.getInstance(importedInstance) {
                let entry = importedInstances
                    .entry(instanceDef.traitName.clone())
                    .or_insert_with(Vec::new);
                entry.push(instanceDef);
            }
        }
        InstanceResolver {
            allocator,
            program,
            knownConstraints,
            localInstances,
            importedInstances,
            cache: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    fn findInstanceForConstraint(
        &self,
        constraint: &Constraint,
        candidates: &Vec<Rc<Instance>>,
        level: u32,
        runner: Runner,
    ) -> InstanceSearchResult {
        //println!("Finding instance for constraint {}", constraint);
        let mut matchingImpls = Vec::new();
        for candidate in candidates {
            let instanceDef = candidate.as_ref();
            if constraint.name == instanceDef.traitName {
                if instanceDef.types.len() != constraint.args.len() {
                    continue;
                }
                let mut instanceDef = instantiateInstance(&self.allocator, &instanceDef);
                //println!("Trying impl {}", instanceDef);
                let mut allMatch = true;
                let mut sub = Substitution::new();
                for (implArg, cArg) in zip(&instanceDef.types, &constraint.args) {
                    if cArg.isGeneric() && !implArg.isGeneric() {
                        allMatch = false;
                        break;
                    }
                    // println!("  Unifying impl arg {} with constraint arg {}", implArg, cArg);
                    if !unify(
                        &mut sub,
                        implArg.clone(),
                        cArg.clone(),
                        Config::default().voidSeparate(),
                    )
                    .is_ok()
                    {
                        //println!("  Arg {} does not match {}", implArg, cArg);
                        allMatch = false;
                        break;
                    }
                }
                if allMatch {
                    //println!("Applying substitution: {}", sub);
                    instanceDef = instanceDef.apply(&sub);
                    //println!("Impl after applying substitution: {}", instanceDef);
                    if runner.getConfig().dumpCfg.instanceResolverTraceEnabled {
                        let indent = "  ".repeat(level as usize);
                        println!(
                            "{}Candidate {} matches for {}, sub constraints: {}",
                            indent, instanceDef.name, constraint, instanceDef.constraintContext
                        );
                    }
                    let mut allSubConstraintsMatch = true;
                    for c in &instanceDef.constraintContext.constraints {
                        //println!("  checking sub constraint: {}", c);
                        if self
                            .findImplInKnownConstraints(
                                c,
                                &self.knownConstraints,
                                runner.child("find_impl_in_known_constraints"),
                            )
                            .is_some()
                        {
                            //println!("  Found in known constraints");
                            continue;
                        }
                        if !self.findInstanceInScopeInner(c, level + 1, runner.clone()).isFound() {
                            //println!("  No instance found for sub constraint {}", c);
                            allSubConstraintsMatch = false;
                            break;
                        }
                    }
                    if allSubConstraintsMatch {
                        //println!("  matching impl found: {}", instanceDef);
                        matchingImpls.push(instanceDef);
                    } else {
                        //println!("  sub constraints do not match");
                    }
                }
            }
        }
        //println!("Found {} matching impls for {}", matchingImpls.len(), constraint);
        if matchingImpls.len() > 1 {
            let names = matchingImpls.iter().map(|i| i.name.clone()).collect();
            InstanceSearchResult::Ambiguous(names)
        } else {
            match matchingImpls.pop() {
                Some(instanceDef) => InstanceSearchResult::Found(instanceDef),
                None => InstanceSearchResult::NotFound,
            }
        }
    }

    pub fn findInstanceInScope(&self, constraint: &Constraint, runner: Runner) -> InstanceSearchResult {
        if runner.getConfig().dumpCfg.instanceResolverTraceEnabled {
            println!("Finding instance in scope for constraint {}", constraint);
            for (_, instances) in &self.localInstances {
                for instance in instances {
                    println!("Local instance: {}", instance.name);
                }
            }
            for (_, instances) in &self.importedInstances {
                for instance in instances {
                    println!("Imported instance: {}", instance.name);
                }
            }
        }
        {
            let mut stats = runner.statistics.borrow_mut();
            stats.instanceLookup += 1;
        }
        if constraint.isConcrete() {
            {
                let mut stats = runner.statistics.borrow_mut();
                stats.instanceCacheLookup += 1;
            }
            let cache = &self.cache.borrow();
            if let Some(cached) = cache.get(constraint) {
                //println!("Found cached instance for constraint {}", constraint);
                let instantiate = instantiateInstance(&self.allocator, cached);
                {
                    let mut stats = runner.statistics.borrow_mut();
                    stats.instanceCacheHit += 1;
                }
                return InstanceSearchResult::Found(instantiate);
            } else {
                let mut stats = runner.statistics.borrow_mut();
                stats.instanceCacheMiss += 1;
            }
        }
        let r = runner
            .clone()
            .run(|| self.findInstanceInScopeInner(constraint, 0, runner));
        if let InstanceSearchResult::Found(i) = &r {
            if constraint.isConcrete() {
                let normalized = i.normalize();
                let cache = &mut self.cache.borrow_mut();
                cache.insert(constraint.clone(), normalized);
            }
        }
        r
    }

    pub fn findInstanceInScopeInner(
        &self,
        constraint: &Constraint,
        level: u32,
        runner: Runner,
    ) -> InstanceSearchResult {
        if runner.getConfig().dumpCfg.instanceResolverTraceEnabled {
            let indent = "  ".repeat(level as usize);
            println!("{}-> resolve level {} constraint {}", indent, level, constraint);
        }
        // println!(
        //     "Finding instance in scope for constraint {} at level {}",
        //     constraint, level
        // );
        if level > 10 {
            // Prevent infinite recursion
            panic!("Instance resolution exceeded maximum recursion depth");
        }
        if let Some(localCandidates) = self.localInstances.get(&constraint.name) {
            match self.findInstanceForConstraint(constraint, &localCandidates, level, runner.clone()) {
                InstanceSearchResult::Found(instanceDef) => return InstanceSearchResult::Found(instanceDef),
                InstanceSearchResult::Ambiguous(names) => return InstanceSearchResult::Ambiguous(names),
                InstanceSearchResult::NotFound => {}
            }
        }
        if let Some(importedCandidates) = self.importedInstances.get(&constraint.name) {
            match self.findInstanceForConstraint(constraint, &importedCandidates, level, runner.clone()) {
                InstanceSearchResult::Found(instanceDef) => return InstanceSearchResult::Found(instanceDef),
                InstanceSearchResult::Ambiguous(names) => return InstanceSearchResult::Ambiguous(names),
                InstanceSearchResult::NotFound => {}
            }
        }
        let mut canonTypes = Vec::new();
        for arg in &constraint.args {
            if let Some(canon) = self.canonicalizeType(arg.clone()) {
                canonTypes.push(canon);
            } else {
                return InstanceSearchResult::NotFound;
            }
        }
        if let Some(instanceName) = self.program.canonicalImplStore.get(&constraint.name, &canonTypes) {
            //println!("Found canonical impl {} for {}", instanceName, formatTypes(&canonTypes));
            let instanceDef = self
                .program
                .getInstance(&instanceName)
                .expect("Canonical impl not found");
            return self.findInstanceForConstraint(constraint, &vec![instanceDef], level, runner);
        }
        InstanceSearchResult::NotFound
    }

    pub fn findImplInKnownConstraints(
        &self,
        constraint: &Constraint,
        knownConstraints: &ConstraintContext,
        runner: Runner,
    ) -> Option<(u32, Constraint)> {
        runner.run(|| {
            for (index, known) in knownConstraints.constraints.iter().enumerate() {
                if constraint.name == known.name && constraint.args.len() == known.args.len() {
                    let mut sub = Substitution::new();
                    let mut allMatch = true;
                    for (arg, karg) in zip(&constraint.args, &known.args) {
                        if unify(&mut sub, arg.clone(), karg.clone(), Config::default()).is_err() {
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
        })
    }

    pub fn isCopy(&self, ty: &Type, runner: Runner) -> bool {
        if ty.isFunction() {
            return true;
        }
        let constraint = Constraint {
            name: getCopyName(),
            args: vec![ty.clone()],
            associatedTypes: Vec::new(),
            main: false,
        };
        self.findInstanceInScope(&constraint, runner).isFound()
    }

    pub fn isDrop(&self, ty: &Type, runner: Runner) -> bool {
        let constraint = Constraint {
            name: getDropName(),
            args: vec![ty.clone()],
            associatedTypes: Vec::new(),
            main: false,
        };
        self.findInstanceInScope(&constraint, runner).isFound()
    }

    pub fn isImplicitConvert(&self, src: &Type, dest: &Type, runner: Runner) -> bool {
        //println!("Checking implicit convert from {} to {}", src, dest);
        let constraint = Constraint {
            name: getImplicitConvertName(),
            args: vec![src.clone(), dest.clone()],
            associatedTypes: Vec::new(),
            main: false,
        };
        // println!("Constraint: {}", constraint);
        self.findInstanceInScope(&constraint, runner).isFound()
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
