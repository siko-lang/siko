use crate::siko::hir::Data::Enum;
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

fn getChoice(p: &Pattern, moduleResolver: &ModuleResolver) -> (Choice, Option<String>) {
    match &p.pattern {
        SimplePattern::Named(n, _) => (Choice::Named(moduleResolver.resolverName(n)), None),
        SimplePattern::Bind(n, _) => (Choice::Wildcard, Some(n.toString())),
        SimplePattern::Tuple(_) => (Choice::Tuple, None),
        SimplePattern::StringLiteral(v) => (Choice::Literal(v.clone()), None),
        SimplePattern::IntegerLiteral(v) => (Choice::Literal(v.clone()), None),
        SimplePattern::Wildcard => (Choice::Wildcard, Some("_".to_string())),
    }
}

#[derive(Clone, Debug)]
enum Label {
    Simple(String),
    Bind(String, String),
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::Simple(n) => write!(f, "{}", n),
            Label::Bind(n, b) => write!(f, "{}/{}", n, b),
        }
    }
}

#[derive(Clone)]
struct LabeledPattern {
    pattern: Pattern,
    label: Label,
}

fn extendPatterns(p: &LabeledPattern, patterns: Vec<LabeledPattern>) -> Vec<LabeledPattern> {
    match p.pattern.pattern.clone() {
        SimplePattern::Named(_, args) => {
            let mut result = Vec::new();
            for (index, arg) in args.into_iter().enumerate() {
                let label = format!("{}.{}", p.label, index);
                result.push(LabeledPattern {
                    pattern: arg,
                    label: Label::Simple(label),
                });
            }
            result.extend(patterns);
            result
        }
        SimplePattern::Bind(_, _) => patterns,
        SimplePattern::Tuple(args) => {
            let mut result = Vec::new();
            for (index, arg) in args.into_iter().enumerate() {
                let label = format!("{}.{}", p.label, index);
                result.push(LabeledPattern {
                    pattern: arg,
                    label: Label::Simple(label),
                });
            }
            result.extend(patterns);
            result
        }
        SimplePattern::StringLiteral(_) => patterns,
        SimplePattern::IntegerLiteral(_) => patterns,
        SimplePattern::Wildcard => patterns,
    }
}

pub struct ChoiceNode {
    name: Label,
    choices: BTreeMap<Choice, ChoiceNode>,
    matches: Vec<usize>,
}

impl ChoiceNode {
    fn new(name: Label) -> ChoiceNode {
        ChoiceNode {
            name: name,
            choices: BTreeMap::new(),
            matches: Vec::new(),
        }
    }

    fn buildDot(&self, graph: &mut Graph) -> String {
        let label = if self.choices.is_empty() {
            Some(format!("{}/{:?}", self.name, self.matches.first()))
        } else {
            Some(format!("{}", self.name))
        };
        let name = graph.addNode(label);
        for (choice, child) in &self.choices {
            let childName = child.buildDot(graph);
            graph.edges.push((name.clone(), childName, Some(format!("{}", choice))));
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

    pub fn addWildcards(&mut self, variants: &BTreeMap<QualifiedName, QualifiedName>, enums: &BTreeMap<QualifiedName, Enum>) {
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
        if fullMatch && self.choices.len() > 1 {
            self.choices.remove(&Choice::Wildcard);
        }
        if wildcardNeeded && !self.choices.contains_key(&Choice::Wildcard) {
            self.choices.insert(Choice::Wildcard, ChoiceNode::new(Label::Simple("_".to_string())));
        }
    }

    fn build(&mut self, mut patterns: Vec<LabeledPattern>, moduleResolver: &ModuleResolver) {
        match patterns.first() {
            Some(_) => {
                let p = patterns.remove(0);
                let (choice, extra) = getChoice(&p.pattern, moduleResolver);
                let patterns = extendPatterns(&p, patterns);
                let label = match (p.label, extra) {
                    (Label::Simple(l), Some(e)) => Label::Bind(l, e),
                    (l, _) => l,
                };
                let next = self.choices.entry(choice).or_insert_with(|| ChoiceNode::new(label));
                next.build(patterns, moduleResolver);
            }
            None => {}
        }
    }

    fn apply(&mut self, mut patterns: Vec<LabeledPattern>, index: usize, moduleResolver: &ModuleResolver) -> bool {
        match patterns.first() {
            Some(_) => {
                let p = patterns.remove(0);
                let (choice, _) = getChoice(&p.pattern, moduleResolver);
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

pub struct MatchCompiler {
    choiceTree: ChoiceNode,
    branches: Vec<Pattern>,
}

impl MatchCompiler {
    pub fn new(branches: Vec<Pattern>) -> MatchCompiler {
        MatchCompiler {
            choiceTree: ChoiceNode::new(Label::Simple("main".to_string())),
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
            self.choiceTree.build(
                vec![LabeledPattern {
                    pattern: branch,
                    label: Label::Simple("root".to_string()),
                }],
                moduleResolver,
            );
        }
        self.choiceTree.addWildcards(variants, enums);

        let mut graph = Graph::new("matchtest".to_string());
        self.choiceTree.buildDot(&mut graph);
        graph.printDot();

        let mut lastLocation = None;
        for (index, branch) in self.branches.clone().into_iter().enumerate() {
            let location = branch.location.clone();
            lastLocation = Some(location.clone());
            if !self.choiceTree.apply(
                vec![LabeledPattern {
                    pattern: branch,
                    label: Label::Simple("root".to_string()),
                }],
                index,
                moduleResolver,
            ) {
                ResolverError::RedundantPattern(location).report();
            }
        }
        if self.choiceTree.checkMissing() {
            ResolverError::MissingPattern(lastLocation.expect("lastLocation was not found")).report();
        }
    }
}
