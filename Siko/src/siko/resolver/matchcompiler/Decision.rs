use crate::siko::{
    location::Location::Location,
    syntax::Pattern::{Pattern, SimplePattern},
};
use std::fmt::Display;

use super::{Choice::Choice, Resolver::Resolver};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Path {
    Root,
    Variant(Box<Path>, i64),
    Index(Box<Path>, i64),
    End(i64),
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Path::Root => write!(f, "root"),
            Path::Variant(parent, i) => write!(f, "{}.v{}", parent, i),
            Path::Index(parent, i) => write!(f, "{}.{}", parent, i),
            Path::End(i) => write!(f, "end#{}", i),
        }
    }
}

#[derive(Clone)]
pub struct Decision {
    pub choice: Choice,
    pub path: Path,
    pub location: Location,
}

impl Decision {
    pub fn new(choice: Choice, path: Path, location: Location) -> Decision {
        Decision {
            choice: choice,
            path: path,
            location: location,
        }
    }
}

impl Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.path, self.choice)
    }
}

#[derive(Clone)]
pub struct Decisions {
    pub decisions: Vec<Decision>,
}

impl Decisions {
    fn add(&mut self, decision: Decision) {
        self.decisions.push(decision);
    }
}

impl Display for Decisions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, d) in self.decisions.iter().enumerate() {
            if index == 0 {
                write!(f, "{}", d)?;
            } else {
                write!(f, ", {}", d)?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

pub struct DecisionBuilder<'a> {
    pub decisions: Decisions,
    resolver: &'a Resolver<'a>,
}

impl<'a> DecisionBuilder<'a> {
    pub fn new(compiler: &'a Resolver<'a>) -> DecisionBuilder<'a> {
        DecisionBuilder {
            decisions: Decisions { decisions: Vec::new() },
            resolver: compiler,
        }
    }

    pub fn build(&mut self, pattern: Pattern, parent: Path) {
        match pattern.pattern {
            SimplePattern::Named(name, args) => {
                let name = self.resolver.moduleResolver.resolverName(&name);
                let mut choice = Choice::Class(name.clone());
                let mut path = parent.clone();
                if let Some(e) = self.resolver.variants.get(&name) {
                    //println!("{} is a variant of {}", name, e);
                    choice = Choice::Variant(name.clone(), e.clone());
                    let enumDef = self.resolver.enums.get(e).expect("enum not found");
                    for (index, v) in enumDef.variants.iter().enumerate() {
                        if v.name == name {
                            path = Path::Variant(Box::new(parent.clone()), index as i64);
                        }
                    }
                }
                self.decisions.add(Decision::new(choice, parent.clone(), pattern.location.clone()));
                for (index, arg) in args.clone().into_iter().enumerate() {
                    self.build(arg, Path::Index(Box::new(path.clone()), index as i64));
                }
            }
            SimplePattern::Bind(_, _) => {
                self.decisions
                    .add(Decision::new(Choice::Wildcard, parent.clone(), pattern.location.clone()));
            }
            SimplePattern::Tuple(args) => {
                self.decisions.add(Decision::new(Choice::Tuple, parent.clone(), pattern.location.clone()));
                for (index, arg) in args.clone().into_iter().enumerate() {
                    self.build(arg, Path::Index(Box::new(parent.clone()), index as i64));
                }
            }
            SimplePattern::StringLiteral(v) => {
                self.decisions
                    .add(Decision::new(Choice::StringLiteral(v.clone()), parent.clone(), pattern.location.clone()));
            }
            SimplePattern::IntegerLiteral(v) => {
                self.decisions
                    .add(Decision::new(Choice::IntegerLiteral(v.clone()), parent.clone(), pattern.location.clone()));
            }
            SimplePattern::Wildcard => {
                self.decisions
                    .add(Decision::new(Choice::Wildcard, parent.clone(), pattern.location.clone()));
            }
        }
    }
}
