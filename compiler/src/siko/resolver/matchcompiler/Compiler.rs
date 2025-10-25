use crate::siko::hir::Variable::Variable;
use crate::siko::location::Location::Location;
use crate::siko::resolver::matchcompiler::DataPath::{
    matchDecisions, DataPath, DataPathSegment, DataType, DecisionPath,
};
use crate::siko::resolver::matchcompiler::IrCompiler::IrCompiler;
use crate::siko::resolver::matchcompiler::Tree::{
    Bindings, Case, Leaf, Match, MatchKind, Node, Switch, SwitchKind, Tuple, Wildcard,
};
use crate::siko::resolver::Environment::Environment;
use crate::siko::resolver::Error::ResolverError;
use crate::siko::resolver::ExprResolver::ExprResolver;
use crate::siko::syntax::Expr::Branch;
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use crate::siko::util::Runner::Runner;
use std::collections::{BTreeMap, BTreeSet};
use std::iter::repeat;

pub struct MatchCompiler<'a, 'b> {
    bodyLocation: Location,
    branches: Vec<Branch>,
    errors: Vec<ResolverError>,
    pub usedPatterns: BTreeSet<i64>,
    pub missingPatterns: BTreeSet<String>,
    irCompiler: IrCompiler<'a, 'b>,
    runner: Runner,
}

impl<'a, 'b> MatchCompiler<'a, 'b> {
    pub fn new(
        resolver: &'a mut ExprResolver<'b>,
        bodyId: Variable,
        matchLocation: Location,
        bodyLocation: Location,
        branches: Vec<Branch>,
        parentEnv: &'a Environment<'a>,
        runner: Runner,
    ) -> MatchCompiler<'a, 'b> {
        let irCompiler = IrCompiler::new(
            bodyId,
            branches.clone(),
            resolver,
            parentEnv,
            bodyLocation.clone(),
            matchLocation.clone(),
        );
        MatchCompiler {
            bodyLocation: bodyLocation,
            branches: branches,
            errors: Vec::new(),
            usedPatterns: BTreeSet::new(),
            missingPatterns: BTreeSet::new(),
            irCompiler: irCompiler,
            runner: runner,
        }
    }

    pub fn compile(&mut self) -> Variable {
        let runner = self.runner.child("match_processing");
        let node = runner.clone().run(|| self.processAndVerifyPatterns(runner));
        let runner = self.runner.child("match_ir_compilation");
        let v = runner.run(|| self.irCompiler.compileIr(node));
        v
    }

    fn resolve(&self, pattern: &Pattern) -> Pattern {
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self
                    .irCompiler
                    .resolver
                    .moduleResolver
                    .resolveTypeOrVariantName(&origId);
                let id = Identifier::new(name.toString(), Location::empty());
                let args = args.iter().map(|p| self.resolve(p)).collect();
                Pattern {
                    pattern: SimplePattern::Named(id, args),
                    location: Location::empty(),
                }
            }
            SimplePattern::Bind(_, _) => pattern.clone(),
            SimplePattern::Tuple(args) => {
                let args = args.iter().map(|p| self.resolve(p)).collect();
                Pattern {
                    pattern: SimplePattern::Tuple(args),
                    location: Location::empty(),
                }
            }
            SimplePattern::StringLiteral(_) => pattern.clone(),
            SimplePattern::IntegerLiteral(_) => pattern.clone(),
            SimplePattern::Wildcard => pattern.clone(),
            SimplePattern::Guarded(pat, expr) => {
                let resolvedPat = self.resolve(pat);
                Pattern {
                    pattern: SimplePattern::Guarded(Box::new(resolvedPat), expr.clone()),
                    location: Location::empty(),
                }
            }
            SimplePattern::OrPattern(_) => {
                unreachable!("OrPattern should have been handled earlier")
            }
        }
    }

    fn generateChoices(&self, pattern: &Pattern) -> Vec<Pattern> {
        let wildcardPattern = Pattern {
            pattern: SimplePattern::Wildcard,
            location: Location::empty(),
        };
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.irCompiler.resolver.moduleResolver.resolveName(&origId);
                let mut result = Vec::new();
                if let Some(_enumName) = self.irCompiler.resolver.variants.get(&name) {
                    // Instead of generating O(n²) specific variant alternatives,
                    // generate a single wildcard to represent "any other variant"
                    result.push(wildcardPattern.clone());
                    for (index, arg) in args.iter().enumerate() {
                        let choices = self.generateChoices(arg);
                        for choice in choices {
                            let mut choiceArgs = Vec::new();
                            choiceArgs.extend(args.iter().cloned().take(index));
                            choiceArgs.push(choice);
                            choiceArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                            let id = Identifier::new(name.toString(), Location::empty());
                            let pat = Pattern {
                                pattern: SimplePattern::Named(id, choiceArgs),
                                location: Location::empty(),
                            };
                            result.push(pat);
                        }
                    }
                }
                result
            }
            SimplePattern::Bind(_, _) => Vec::new(),
            SimplePattern::Tuple(args) => {
                let mut result = Vec::new();
                for (index, arg) in args.iter().enumerate() {
                    let choices = self.generateChoices(arg);
                    for choice in choices {
                        let mut choiceArgs = Vec::new();
                        choiceArgs.extend(args.iter().cloned().take(index));
                        choiceArgs.push(choice);
                        choiceArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                        let pat = Pattern {
                            pattern: SimplePattern::Tuple(choiceArgs),
                            location: Location::empty(),
                        };
                        result.push(pat);
                    }
                }
                result
            }
            SimplePattern::StringLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::IntegerLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::Wildcard => Vec::new(),
            SimplePattern::Guarded(pat, _) => self.generateChoices(pat),
            SimplePattern::OrPattern(_) => {
                unreachable!("OrPattern should have been handled earlier")
            }
        }
    }

    fn generateDecisions(
        &mut self,
        pattern: &Pattern,
        parentData: &DataPath,
        decision: &DecisionPath,
        mut bindings: Bindings,
    ) -> (DecisionPath, Bindings) {
        //println!("generateDecisions: {}, {}, {}", pattern, parentData, decision);
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.irCompiler.resolver.moduleResolver.resolveName(&origId);
                if let Some(enumName) = self.irCompiler.resolver.variants.get(&name) {
                    let path = parentData.push(DataPathSegment::Variant(name, enumName.clone()));
                    let mut decision = decision.add(path.clone());
                    for (index, arg) in args.iter().enumerate() {
                        let path = path.push(DataPathSegment::ItemIndex(index as i64));
                        (decision, bindings) = self.generateDecisions(arg, &path, &decision, bindings);
                    }
                    (decision, bindings)
                } else {
                    (decision.add(parentData.push(DataPathSegment::Struct(name))), bindings)
                }
            }
            SimplePattern::Bind(name, _) => {
                let path = parentData.asRef().asBindingPath();
                let bindPath = decision.add(parentData.clone());
                //println!("Binding {} to {}", name, bindPath);
                bindings.bindings.insert(bindPath, name.toString());
                (decision.add(path), bindings)
            }
            SimplePattern::Tuple(args) => {
                let mut decision = decision.clone();
                let path = parentData.push(DataPathSegment::Tuple(args.len() as i64));
                decision = decision.add(path.clone());
                for (index, arg) in args.iter().enumerate() {
                    let path = path.push(DataPathSegment::TupleIndex(index as i64));
                    (decision, bindings) = self.generateDecisions(arg, &path, &decision, bindings);
                }
                (decision, bindings)
            }
            SimplePattern::StringLiteral(v) => (
                decision.add(parentData.push(DataPathSegment::StringLiteral(v.clone()))),
                bindings,
            ),
            SimplePattern::IntegerLiteral(v) => (
                decision.add(parentData.push(DataPathSegment::IntegerLiteral(v.clone()))),
                bindings,
            ),
            SimplePattern::Wildcard => (decision.add(parentData.push(DataPathSegment::Wildcard)), bindings),
            SimplePattern::Guarded(pat, _) => self.generateDecisions(pat, parentData, decision, bindings),
            SimplePattern::OrPattern(_) => {
                unreachable!("OrPattern should have been handled earlier")
            }
        }
    }

    fn processAndVerifyPatterns(&mut self, runner: Runner) -> Node {
        let mut matches = Vec::new();
        for (index, branch) in self.branches.clone().iter().enumerate() {
            let branchPattern = self.resolve(&branch.pattern);
            let (decision, bindings) =
                self.generateDecisions(&branchPattern, &DataPath::root(), &DecisionPath::new(), Bindings::new());
            //println!("{} Pattern {}\n decision: {}", index, branch.pattern, decision);
            let choices = self.generateChoices(&branchPattern);
            matches.push(Match::new(
                MatchKind::UserDefined(index as i64, branchPattern.isGuarded()),
                branchPattern,
                decision,
                bindings,
            ));
            for choice in choices {
                //println!("   Alt: {}", choice);
                let (decision, bindings) =
                    self.generateDecisions(&choice, &DataPath::root(), &DecisionPath::new(), Bindings::new());
                matches.push(Match::new(MatchKind::Alternative, choice, decision, bindings));
            }
        }

        let mut dataTypes = BTreeMap::new();
        for m in &matches {
            for path in &m.decisionPath.decisions {
                match path.asRef().last() {
                    DataPathSegment::Root => {}
                    DataPathSegment::Tuple(count) => {
                        let parent = path.asRef().getParent().owned();
                        dataTypes.insert(parent, DataType::Tuple(count.clone()));
                    }
                    DataPathSegment::TupleIndex(_) => {}
                    DataPathSegment::ItemIndex(_) => {}
                    DataPathSegment::Variant(_, enumName) => {
                        let parent = path.asRef().getParent().owned();
                        dataTypes.insert(parent, DataType::Enum(enumName.clone()));
                    }
                    DataPathSegment::IntegerLiteral(_) => {
                        let parent = path.asRef().getParent().owned();
                        dataTypes.insert(parent, DataType::Integer);
                    }
                    DataPathSegment::StringLiteral(_) => {
                        let parent = path.asRef().getParent().owned();
                        dataTypes.insert(parent, DataType::String);
                    }
                    DataPathSegment::Struct(name) => {
                        let parent = path.asRef().getParent().owned();
                        dataTypes.insert(parent, DataType::Struct(name.clone()));
                    }
                    DataPathSegment::Wildcard => {
                        let parent = path.asRef().getParent().owned();
                        if !dataTypes.contains_key(&parent) {
                            dataTypes.insert(parent.clone(), DataType::Wildcard);
                        }
                    }
                }
            }
        }

        // for (path, ty) in &dataTypes {
        //     println!("{} {}", path, ty);
        // }
        // for m in &matches {
        //     println!("Decision {}", m.decisionPath);
        // }

        let mut pendingPaths = Vec::new();
        pendingPaths.push(DataPath::root());
        let nodeBuilderRunner = runner.child("node_builder");
        let mut node =
            nodeBuilderRunner.run(|| self.buildNode(pendingPaths, &DecisionPath::new(), &dataTypes, &matches));
        //node.dump(0);
        let addRunner = runner.child("node_add");
        let decisionPathMatcheRunner = addRunner.child("decision_path_matches");
        addRunner.run(|| node.add(self, &matches, &decisionPathMatcheRunner));
        //node.dump(0);
        for (index, branch) in self.branches.clone().iter().enumerate() {
            if !self.usedPatterns.contains(&(index as i64)) {
                self.errors
                    .push(ResolverError::RedundantPattern(branch.pattern.location.clone()));
            }
        }

        let missingPatterns: Vec<_> = self.missingPatterns.iter().map(|p| p.to_string()).collect();
        if !missingPatterns.is_empty() {
            self.errors.push(ResolverError::MissingPattern(
                missingPatterns,
                self.bodyLocation.clone(),
            ));
        }

        for err in &self.errors {
            err.reportOnly(self.irCompiler.resolver.ctx);
        }

        if !self.errors.is_empty() {
            std::process::exit(1);
        }
        node
    }

    fn buildNode(
        &mut self,
        mut pendingPaths: Vec<DataPath>,
        currentDecision: &DecisionPath,
        dataTypes: &BTreeMap<DataPath, DataType>,
        allMatches: &Vec<Match>,
    ) -> Node {
        //println!("buildNode: {:?} | {}", pendingPaths, currentDecision);
        if pendingPaths.is_empty() {
            let end = Leaf::new(currentDecision.clone());
            return Node::Leaf(end);
        }
        let currentPath = pendingPaths.remove(0);
        if let Some(ty) = dataTypes.get(&currentPath) {
            //println!("Building node for {}, {} / [{}] / {:?}", currentPath, ty, currentDecision, pendingPaths);
            match ty {
                DataType::Struct(_) => todo!(),
                DataType::Enum(enumName) => {
                    let e = self
                        .irCompiler
                        .resolver
                        .enums
                        .get(enumName)
                        .expect("enumName not found");
                    let mut values = BTreeSet::new();
                    for m in allMatches {
                        if matchDecisions(&currentDecision.decisions, &m.decisionPath.decisions) {
                            if m.decisionPath.decisions.len() > currentDecision.decisions.len() {
                                match &m.decisionPath.decisions[currentDecision.decisions.len()].asRef().last() {
                                    DataPathSegment::Variant(value, _) => {
                                        values.insert(value.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    let mut cases = BTreeMap::new();
                    for variant in &e.variants {
                        if !values.contains(&variant.name) {
                            continue;
                        }
                        let casePath =
                            currentPath.push(DataPathSegment::Variant(variant.name.clone(), enumName.clone()));
                        let currentDecision = currentDecision.add(casePath.clone());
                        let mut pendings = pendingPaths.clone();
                        for index in 0..variant.items.len() {
                            pendings.insert(
                                0,
                                casePath.push(DataPathSegment::ItemIndex((variant.items.len() - index - 1) as i64)),
                            );
                        }
                        let node = self.buildNode(pendings, &currentDecision, dataTypes, allMatches);
                        cases.insert(Case::Variant(variant.name.clone()), node);
                    }
                    if values.len() < e.variants.len() {
                        let path = currentPath.push(DataPathSegment::Wildcard);
                        let mut pendings = pendingPaths.clone();
                        pendings.insert(0, path.clone());
                        let currentDecision = &currentDecision.add(path);
                        let node = self.buildNode(pendings, currentDecision, dataTypes, allMatches);
                        cases.insert(Case::Default, node);
                    }
                    //println!("Enum switch on {}", currentPath.asBindingPath());
                    let switch = Switch {
                        dataPath: currentPath.clone(),
                        kind: SwitchKind::Enum(enumName.clone()),
                        cases: cases,
                    };
                    Node::Switch(switch)
                }
                DataType::Tuple(size) => {
                    let path = currentPath.push(DataPathSegment::Tuple(*size));
                    let currentDecision = currentDecision.add(path.clone());
                    let mut pendings = Vec::new();
                    for index in 0..*size {
                        let argPath = path.push(DataPathSegment::TupleIndex(index));
                        pendings.insert(0, argPath);
                    }
                    pendings.reverse();
                    pendings.extend(pendingPaths.clone());
                    let node = self.buildNode(pendings, &currentDecision, dataTypes, allMatches);
                    let tuple = Tuple {
                        size: *size,
                        dataPath: path.clone(),
                        next: Box::new(node),
                    };
                    Node::Tuple(tuple)
                }
                DataType::Integer => {
                    let mut cases = BTreeMap::new();
                    let mut values = BTreeSet::new();
                    for m in allMatches {
                        if matchDecisions(&currentDecision.decisions, &m.decisionPath.decisions) {
                            if m.decisionPath.decisions.len() > currentDecision.decisions.len() {
                                match &m.decisionPath.decisions[currentDecision.decisions.len()].asRef().last() {
                                    DataPathSegment::IntegerLiteral(value) => {
                                        values.insert(value.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    for value in values {
                        let path = currentPath.push(DataPathSegment::IntegerLiteral(value.clone()));
                        let mut pendingPaths = pendingPaths.clone();
                        pendingPaths.insert(0, path.clone());
                        let currentDecision = &currentDecision.add(path);
                        let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                        cases.insert(Case::Integer(value.clone()), node);
                    }
                    let path = currentPath.push(DataPathSegment::Wildcard);
                    let mut pendingPaths = pendingPaths.clone();
                    pendingPaths.insert(0, path.clone());
                    let currentDecision = &currentDecision.add(path);
                    let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                    cases.insert(Case::Default, node);
                    let switch = Switch {
                        dataPath: currentPath.clone(),
                        kind: SwitchKind::Integer,
                        cases: cases,
                    };
                    Node::Switch(switch)
                }
                DataType::String => {
                    let mut cases = BTreeMap::new();
                    let mut values = BTreeSet::new();
                    for m in allMatches {
                        if matchDecisions(&currentDecision.decisions, &m.decisionPath.decisions) {
                            if m.decisionPath.decisions.len() >= currentDecision.decisions.len() + 1 {
                                match &m.decisionPath.decisions[currentDecision.decisions.len()].asRef().last() {
                                    DataPathSegment::StringLiteral(value) => {
                                        values.insert(value.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    for value in values {
                        let path = currentPath.push(DataPathSegment::StringLiteral(value.clone()));
                        let mut pendingPaths = pendingPaths.clone();
                        pendingPaths.insert(0, path.clone());
                        let currentDecision = &currentDecision.add(path);
                        let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                        cases.insert(Case::String(value.clone()), node);
                    }
                    let path = currentPath.push(DataPathSegment::Wildcard);
                    let mut pendingPaths = pendingPaths.clone();
                    pendingPaths.insert(0, path.clone());
                    let currentDecision = &currentDecision.add(path);
                    let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                    cases.insert(Case::Default, node);
                    let switch = Switch {
                        dataPath: currentPath.clone(),
                        kind: SwitchKind::String,
                        cases: cases,
                    };
                    Node::Switch(switch)
                }
                DataType::Wildcard => {
                    let path = currentPath.push(DataPathSegment::Wildcard);
                    pendingPaths.insert(0, path.clone());
                    let currentDecision = &currentDecision.add(path.clone());
                    let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                    Node::Wildcard(Wildcard {
                        path: currentPath,
                        next: Box::new(node),
                    })
                }
            }
        } else {
            self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches)
        }
    }
}
