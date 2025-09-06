use std::{collections::BTreeMap, fmt};

use crate::siko::{
    qualifiedname::QualifiedName,
    resolver::matchcompiler::{
        Compiler::MatchCompiler,
        DataPath::{matchDecisions, DataPath, DecisionPath},
    },
    syntax::Pattern::Pattern,
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

#[derive(Clone, Debug)]
pub struct Switch {
    pub dataPath: DataPath,
    pub kind: SwitchKind,
    pub cases: BTreeMap<Case, Node>,
}

#[derive(Clone, Debug)]
pub struct End {
    pub decisionPath: DecisionPath,
    pub matches: Vec<Match>,
}

#[derive(Clone, Debug)]
pub struct Wildcard {
    pub next: Box<Node>,
}

#[derive(Clone, Debug)]
pub enum Node {
    Tuple(Tuple),
    Switch(Switch),
    End(End),
    Wildcard(Wildcard),
}

impl Node {
    pub fn getDataPath(&self) -> DataPath {
        match self {
            Node::Tuple(tuple) => tuple.dataPath.getParent(),
            Node::Switch(switch) => switch.dataPath.clone(),
            Node::End(_) => unreachable!(),
            Node::Wildcard(_) => unreachable!(),
        }
    }

    pub fn add(&mut self, compiler: &mut MatchCompiler, matches: &Vec<Match>) {
        match self {
            Node::Tuple(tuple) => tuple.next.add(compiler, matches),
            Node::Switch(switch) => {
                for (_, node) in &mut switch.cases {
                    node.add(compiler, matches);
                }
            }
            Node::Wildcard(w) => {
                w.next.add(compiler, matches);
            }
            Node::End(end) => {
                let mut localMatch: Option<Match> = None;
                for m in matches {
                    let matchResult = matchDecisions(end.decisionPath.clone(), m.decisionPath.clone());
                    if matchResult {
                        //println!("MATCH end {} //// {}", end.decisionPath, m.decisionPath);
                        match &localMatch {
                            Some(local) => match (&local.kind, &m.kind) {
                                (MatchKind::Alternative, MatchKind::UserDefined(_)) => {
                                    localMatch = Some(m.clone());
                                }
                                (MatchKind::UserDefined(i1), MatchKind::UserDefined(i2)) => {
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
                            compiler.missingPatterns.insert(m.pattern.clone());
                        }
                        MatchKind::UserDefined(index) => {
                            compiler.usedPatterns.insert(*index);
                        }
                    }
                    //println!("M {}", m.decisionPath);
                    //println!("FINAL MATCH {} for {}, bindings: {}", end.decisionPath, m.kind, m.bindings);
                    end.matches.push(localMatch.unwrap());
                } else {
                    panic!("Empty end node in decision tree");
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Match {
    pub kind: MatchKind,
    pub pattern: Pattern,
    pub decisionPath: DecisionPath,
    pub bindings: Bindings,
}

#[derive(Clone, Debug)]
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
    UserDefined(i64),
    Alternative,
}

impl fmt::Display for MatchKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MatchKind::UserDefined(value) => {
                write!(f, "UserDefined({})", value)
            }
            MatchKind::Alternative => write!(f, "Alternative"),
        }
    }
}
