use std::{collections::BTreeMap, fmt};

use crate::siko::{
    qualifiedname::QualifiedName,
    resolver::matchcompiler::{
        Compiler::MatchCompiler,
        DataPath::{matchDecisions, DataPath, DecisionPath},
    },
    syntax::Pattern::Pattern,
    util::Runner::Runner,
};

#[derive(Clone, Debug)]
pub struct Tuple {
    pub size: i64,
    pub dataPath: DataPath,
    pub next: Box<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Case {
    Variant(QualifiedName),
    Integer(String),
    String(String),
    Default,
}

#[derive(Clone, Debug)]
pub enum SwitchKind {
    Enum(QualifiedName),
    Integer,
    String,
}

impl SwitchKind {
    pub fn isEnum(&self) -> bool {
        match self {
            SwitchKind::Enum(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Switch {
    pub dataPath: DataPath,
    pub kind: SwitchKind,
    pub cases: BTreeMap<Case, Node>,
}

#[derive(Clone, Debug)]
pub struct Leaf {
    pub decisionPath: DecisionPath,
    pub guardedMatches: Vec<Match>,
    pub finalMatch: Option<Match>,
}

impl Leaf {
    pub fn new(decisionPath: DecisionPath) -> Leaf {
        Leaf {
            decisionPath,
            guardedMatches: Vec::new(),
            finalMatch: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Wildcard {
    pub path: DataPath,
    pub next: Box<Node>,
}

#[derive(Clone, Debug)]
pub enum Node {
    Tuple(Tuple),
    Switch(Switch),
    Leaf(Leaf),
    Wildcard(Wildcard),
}

impl Node {
    pub fn getDataPath(&self) -> DataPath {
        match self {
            Node::Tuple(tuple) => tuple.dataPath.asRef().getParent().owned(),
            Node::Switch(switch) => switch.dataPath.clone(),
            Node::Leaf(l) => l.decisionPath.last().clone(),
            Node::Wildcard(wildcard) => wildcard.path.clone(),
        }
    }

    pub fn isLeaf(&self) -> bool {
        match self {
            Node::Leaf(_) => true,
            _ => false,
        }
    }

    pub fn add(&mut self, compiler: &mut MatchCompiler, matches: &Vec<Match>, runner: &Runner) {
        match self {
            Node::Tuple(tuple) => tuple.next.add(compiler, matches, runner),
            Node::Switch(switch) => {
                for (_, node) in &mut switch.cases {
                    node.add(compiler, matches, runner);
                }
            }
            Node::Wildcard(w) => {
                w.next.add(compiler, matches, runner);
            }
            Node::Leaf(leaf) => {
                let mut localMatch: Option<Match> = None;
                for m in matches {
                    let matchResult =
                        runner.run(|| matchDecisions(&leaf.decisionPath.decisions, &m.decisionPath.decisions));
                    if matchResult {
                        if m.kind.isGuarded() {
                            leaf.guardedMatches.push(m.clone());
                            continue;
                        }
                        //println!("MATCH end {} //// {}", end.decisionPath, m.decisionPath);
                        match &localMatch {
                            Some(local) => match (&local.kind, &m.kind) {
                                (MatchKind::Alternative, MatchKind::UserDefined(_, _)) => {
                                    localMatch = Some(m.clone());
                                }
                                (MatchKind::UserDefined(i1, _), MatchKind::UserDefined(i2, _)) => {
                                    if *i2 < *i1 {
                                        localMatch = Some(m.clone());
                                    }
                                }
                                _ => {}
                            },
                            None => {
                                localMatch = Some(m.clone());
                            }
                        }
                    }
                }
                if let Some(m) = &localMatch {
                    match &m.kind {
                        MatchKind::Alternative => {
                            compiler.missingPatterns.insert(m.pattern.to_string());
                        }
                        MatchKind::UserDefined(index, guarded) => {
                            assert_eq!(*guarded, false);
                            compiler.usedPatterns.insert(*index);
                            let mut filteredGuardedMatches = Vec::new();
                            // Keep only guarded matches with index < index
                            // This ensures that we only keep guardedmatches that are checked before this one
                            for guardedMatch in &leaf.guardedMatches {
                                if let MatchKind::UserDefined(guardedIndex, _) = guardedMatch.kind {
                                    if guardedIndex < *index {
                                        compiler.usedPatterns.insert(guardedIndex);
                                        filteredGuardedMatches.push(guardedMatch.clone());
                                    }
                                }
                            }
                            leaf.guardedMatches = filteredGuardedMatches;
                        }
                    }
                    //println!("M {}", m.decisionPath);
                    //println!("FINAL MATCH {} for {}, bindings: {}", end.decisionPath, m.kind, m.bindings);
                    leaf.finalMatch = Some(localMatch.unwrap());
                } else {
                    panic!("Empty leaf node in decision tree");
                }
            }
        }
    }

    pub fn dump(&self, indent: i64) {
        let indentStr = " ".repeat((indent * 2) as usize);
        match self {
            Node::Tuple(tuple) => {
                println!("{}Tuple(size: {}, path: {})", indentStr, tuple.size, tuple.dataPath);
                tuple.next.dump(indent + 1);
            }
            Node::Switch(switch) => {
                // println!(
                //     "{}Switch(kind: {:?}, path: {})",
                //     indentStr, switch.kind, switch.dataPath
                // );
                println!("{}Switch(kind: {:?})", indentStr, switch.kind);
                for (case, node) in &switch.cases {
                    println!("{}  Case: {:?}", indentStr, case);
                    node.dump(indent + 2);
                }
            }
            Node::Leaf(leaf) => {
                println!("{}Leaf(path: {})", indentStr, leaf.decisionPath);
                //println!("{}Leaf", indentStr);
                if let Some(finalMatch) = &leaf.finalMatch {
                    println!(
                        "{}  Final Match: {}, bindings: {}, pattern: {}",
                        indentStr, finalMatch.kind, finalMatch.bindings, finalMatch.pattern
                    );
                } else {
                    println!("{}  No final match", indentStr);
                }
                for guardedMatch in &leaf.guardedMatches {
                    println!(
                        "{}  Guarded Match: {}, bindings: {}, pattern: {}",
                        indentStr, guardedMatch.kind, guardedMatch.bindings, guardedMatch.pattern
                    );
                }
            }
            Node::Wildcard(wildcard) => {
                println!("{}Wildcard(path: {})", indentStr, wildcard.path);
                wildcard.next.dump(indent + 1);
            }
        }
    }
}

struct ChildInfo {
    case: Case,
    leaf: Leaf,
}

#[derive(Clone, Debug)]
pub struct Match {
    pub kind: MatchKind,
    pub pattern: Pattern,
    pub decisionPath: DecisionPath,
    pub bindings: Bindings,
}

impl Match {
    pub fn new(kind: MatchKind, pattern: Pattern, decisionPath: DecisionPath, bindings: Bindings) -> Match {
        Match {
            kind,
            pattern,
            decisionPath,
            bindings,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bindings {
    pub bindings: BTreeMap<DecisionPath, String>,
}

impl Bindings {
    pub fn new() -> Bindings {
        Bindings {
            bindings: BTreeMap::new(),
        }
    }
}

impl fmt::Display for Bindings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bindings {{ ")?;
        for (key, value) in &self.bindings {
            write!(f, "{}: {}, ", key, value)?;
        }
        write!(f, "}}")
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MatchKind {
    UserDefined(i64, bool), // guarded
    Alternative,
}

impl MatchKind {
    pub fn isGuarded(&self) -> bool {
        match self {
            MatchKind::UserDefined(_, guarded) => *guarded,
            MatchKind::Alternative => false,
        }
    }
}

impl fmt::Display for MatchKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MatchKind::UserDefined(value, guarded) => {
                write!(f, "UserDefined({}, guarded:{})", value, guarded)
            }
            MatchKind::Alternative => write!(f, "Alternative"),
        }
    }
}
