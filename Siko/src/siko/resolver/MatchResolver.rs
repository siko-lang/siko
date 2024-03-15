use crate::siko::ir::Data::Enum;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use crate::siko::util::Dot::Graph;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;

use super::Error::ResolverError;
use super::ModuleResolver::ModuleResolver;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Choice {
    Named(QualifiedName),
    Wildcard,
    Tuple,
    Literal(String),
}

impl Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Choice::Named(n) => write!(f, "named({})", n),
            Choice::Wildcard => write!(f, "wildcard"),
            Choice::Tuple => write!(f, "tuple"),
            Choice::Literal(l) => write!(f, "literal({})", l),
        }
    }
}

fn getChoice(p: &Pattern, moduleResolver: &ModuleResolver) -> Choice {
    match &p.pattern {
        SimplePattern::Named(n, _) => Choice::Named(moduleResolver.resolverName(n)),
        SimplePattern::Bind(_, _) => Choice::Wildcard,
        SimplePattern::Tuple(_) => Choice::Tuple,
        SimplePattern::StringLiteral(v) => Choice::Literal(v.clone()),
        SimplePattern::IntegerLiteral(v) => Choice::Literal(v.clone()),
        SimplePattern::Wildcard => Choice::Wildcard,
    }
}

fn extendPatterns(p: &Pattern, patterns: Vec<Pattern>) -> Vec<Pattern> {
    match p.pattern.clone() {
        SimplePattern::Named(_, mut args) => {
            args.extend(patterns);
            args
        }
        SimplePattern::Bind(_, _) => patterns,
        SimplePattern::Tuple(mut args) => {
            args.extend(patterns);
            args
        }
        SimplePattern::StringLiteral(_) => patterns,
        SimplePattern::IntegerLiteral(_) => patterns,
        SimplePattern::Wildcard => patterns,
    }
}

pub struct ChoiceNode {
    choices: BTreeMap<Choice, ChoiceNode>,
    matches: Vec<usize>,
}

impl ChoiceNode {
    pub fn new() -> ChoiceNode {
        ChoiceNode {
            choices: BTreeMap::new(),
            matches: Vec::new(),
        }
    }

    fn buildDot(&self, graph: &mut Graph) -> String {
        let label = if self.choices.is_empty() {
            Some(format!("{:?}", self.matches.first()))
        } else {
            None
        };
        let name = graph.addNode(label);
        for (choice, child) in &self.choices {
            let childName = child.buildDot(graph);
            graph
                .edges
                .push((name.clone(), childName, Some(format!("{}", choice))));
        }

        name
    }

    pub fn checkMissing(&self) -> bool {
        if self.choices.is_empty() {
            self.matches.is_empty()
        } else {
            for (_, next) in &self.choices {
                if next.checkMissing() {
                    return true;
                }
            }
            false
        }
    }

    pub fn addWildcards(
        &mut self,
        variants: &BTreeMap<QualifiedName, QualifiedName>,
        enums: &BTreeMap<QualifiedName, Enum>,
    ) {
        let mut allNames = BTreeSet::new();
        let mut wildcardNeeded = false;
        for (choice, next) in &mut self.choices {
            match &choice {
                Choice::Named(name) => {
                    allNames.insert(name.clone());
                }
                Choice::Literal(_) => {
                    wildcardNeeded = true;
                }
                Choice::Wildcard => {}
                Choice::Tuple => {}
            }
            next.addWildcards(variants, enums);
        }
        let mut fullMatch = true;
        for name in &allNames {
            if let Some(e) = variants.get(name) {
                let e = enums.get(e).unwrap();
                for variant in &e.variants {
                    if !allNames.contains(&variant.name) {
                        fullMatch = false;
                        wildcardNeeded = true;
                    }
                }
            }
        }
        if fullMatch {
            self.choices.remove(&Choice::Wildcard);
        }
        if wildcardNeeded && !self.choices.contains_key(&Choice::Wildcard) {
            self.choices.insert(Choice::Wildcard, ChoiceNode::new());
        }
    }

    pub fn build(&mut self, mut patterns: Vec<Pattern>, moduleResolver: &ModuleResolver) {
        match patterns.first() {
            Some(_) => {
                let p = patterns.remove(0);
                let choice = getChoice(&p, moduleResolver);
                let patterns = extendPatterns(&p, patterns);
                let next = self
                    .choices
                    .entry(choice)
                    .or_insert_with(|| ChoiceNode::new());
                next.build(patterns, moduleResolver);
            }
            None => {}
        }
    }

    pub fn apply(
        &mut self,
        mut patterns: Vec<Pattern>,
        index: usize,
        moduleResolver: &ModuleResolver,
    ) -> bool {
        match patterns.first() {
            Some(_) => {
                let p = patterns.remove(0);
                let choice = getChoice(&p, moduleResolver);
                let patterns = extendPatterns(&p, patterns);
                match choice {
                    Choice::Wildcard => {
                        let mut first = false;
                        for (_, next) in &mut self.choices {
                            if next.apply(patterns.clone(), index, moduleResolver) {
                                first = true;
                            }
                        }
                        first
                    }
                    choice => {
                        let mut first = false;
                        if let Some(next) = self.choices.get_mut(&choice) {
                            if next.apply(patterns, index, moduleResolver) {
                                first = true;
                            }
                        }
                        first
                    }
                }
            }
            _ => self.applyAll(index),
        }
    }

    pub fn applyAll(&mut self, index: usize) -> bool {
        if self.choices.is_empty() {
            let first = self.matches.is_empty();
            self.matches.push(index);
            first
        } else {
            let mut first = false;
            for (_, next) in &mut self.choices {
                if next.applyAll(index) {
                    first = true;
                }
            }
            first
        }
    }
}

pub struct MatchResolver {
    choiceTree: ChoiceNode,
    branches: Vec<Pattern>,
}

impl MatchResolver {
    pub fn new(branches: Vec<Pattern>) -> MatchResolver {
        MatchResolver {
            choiceTree: ChoiceNode::new(),
            branches: branches,
        }
    }

    pub fn check(
        &mut self,
        moduleResolver: &ModuleResolver,
        variants: &BTreeMap<QualifiedName, QualifiedName>,
        enums: &BTreeMap<QualifiedName, Enum>,
    ) {
        for branch in self.branches.clone() {
            self.choiceTree.build(vec![branch], moduleResolver);
        }
        self.choiceTree.addWildcards(variants, enums);
        let mut lastLocation = None;
        for (index, branch) in self.branches.clone().into_iter().enumerate() {
            let location = branch.location.clone();
            lastLocation = Some(location.clone());
            if !self.choiceTree.apply(vec![branch], index, moduleResolver) {
                ResolverError::RedundantPattern(location).report();
            }
        }
        if self.choiceTree.checkMissing() {
            ResolverError::MissingPattern(lastLocation.expect("lastLocation was not found"))
                .report();
        }
        let mut graph = Graph::new("matchtest".to_string());
        self.choiceTree.buildDot(&mut graph);
        graph.printDot();
    }
}
