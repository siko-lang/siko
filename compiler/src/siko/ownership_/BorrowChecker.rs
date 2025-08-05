use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    cfg::{
        Builder::Builder,
        CFG::{Key, CFG},
    },
    hir::{
        Function::{BlockId, Function, InstructionKind},
        Lifetime::Lifetime,
    },
    location::{Location::Location, Report::Entry},
};

use super::Path::Path;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Value {
    name: String,
    block: BlockId,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.block)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct BorrowContext {
    liveValues: BTreeSet<Value>,
    deadValues: BTreeMap<Path, Location>,
    refs: BTreeMap<Lifetime, Path>,
}

impl BorrowContext {
    pub fn new() -> BorrowContext {
        BorrowContext {
            liveValues: BTreeSet::new(),
            deadValues: BTreeMap::new(),
            refs: BTreeMap::new(),
        }
    }
}

impl Display for BorrowContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "liveValues: [")?;
        for (index, v) in self.liveValues.iter().enumerate() {
            if index == 0 {
                write!(f, "{}", v)?;
            } else {
                write!(f, ", {}", v)?;
            }
        }
        write!(f, "]")?;
        write!(f, " deadValues: [")?;
        for (index, v) in self.deadValues.iter().enumerate() {
            if index == 0 {
                write!(f, "{}", v.0)?;
            } else {
                write!(f, ", {}", v.0)?;
            }
        }
        write!(f, "]")?;
        write!(f, " refs: [")?;
        for (index, (l, p)) in self.refs.iter().enumerate() {
            if index == 0 {
                write!(f, "{}:{}", l, p)?;
            } else {
                write!(f, ", {}:{}", l, p)?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

pub struct BorrowChecker<'a> {
    function: &'a Function,
    cfg: CFG,
    visited: BTreeSet<Key>,
}

impl<'a> BorrowChecker<'a> {
    pub fn new(function: &'a Function) -> BorrowChecker<'a> {
        let mut cfgBuilder = Builder::new(function.name.toString(), function.result.clone());
        cfgBuilder.build(function);
        BorrowChecker {
            function: function,
            cfg: cfgBuilder.getCFG(),
            visited: BTreeSet::new(),
        }
    }

    pub fn check(&mut self) {
        let sources = self.cfg.getSources();
        for s in sources {
            self.processNode(s, BorrowContext::new());
        }

        self.cfg.printDot();
    }

    fn processNode(&mut self, key: Key, mut context: BorrowContext) {
        println!("Processing node {}", key);
        if self.visited.contains(&key) {
            return;
        }
        self.visited.insert(key.clone());
        let node = self.cfg.getNode(&key);
        let outgoings = node.outgoing.clone();

        match key {
            Key::DropKey(id, _) => {
                let i = self.function.getInstruction(id);
                let block = id.getBlockById();
                context.deadValues.extend(
                    context
                        .liveValues
                        .iter()
                        .cloned()
                        .filter(|v| v.block == block)
                        .map(|v| (Path::WholePath(v.name), i.location.clone())),
                );
            }
            Key::Instruction(instruction_id) => {
                let i = self
                    .function
                    .body
                    .as_ref()
                    .expect("no body")
                    .getInstruction(instruction_id);
                match &i.kind {
                    InstructionKind::Bind(name, _) => {
                        context.liveValues.insert(Value {
                            name: name.clone(),
                            block: i.id.getBlockById(),
                        });
                    }
                    InstructionKind::Ref(_) => {
                        let path = node.usage.clone().unwrap();
                        let ty = i.ty.as_ref().unwrap();
                        let lifetimes = ty.collectLifetimes();
                        let refLifetime = lifetimes[0];
                        println!("Path {} {} {}", path, ty, refLifetime);
                        context.refs.insert(refLifetime, path);
                    }
                    // InstructionKind::ValueRef(_, _, _) => {
                    //     if let Some(usage) = &node.usage {
                    //         let ty = i.ty.as_ref().unwrap();
                    //         let lifetimes = ty.collectLifetimes();
                    //         for l in lifetimes {
                    //             match context.refs.get(&l) {
                    //                 Some(path) => {
                    //                     if let Some(_) = context.deadValues.get(path) {
                    //                         let mut entries = Vec::new();
                    //                         entries.push(Entry::new(None, i.location.clone()));
                    //                         // let report = Report::build("reference to moved/dead value".to_string(), entries);
                    //                         // report.print();
                    //                     }
                    //                 }
                    //                 None => {}
                    //             }
                    //         }
                    //         if let Some(loc) = context.deadValues.get(usage) {
                    //             let mut entries = Vec::new();
                    //             entries.push(Entry::new(Some("It was moved here".to_string()), loc.clone()));
                    //             entries.push(Entry::new(Some("Trying to move again here".to_string()), i.location.clone()));
                    //             // let report = Report::build(
                    //             //     "trying to move already moved value".to_string(),
                    //             //     entries,
                    //             // );
                    //             // report.print();
                    //         }
                    //         context.deadValues.insert(usage.clone(), i.location.clone());
                    //     }
                    // }
                    _ => {}
                }
            }
            Key::LoopEnd(_) => {}
            Key::LoopStart(_) => {}
            Key::If(_) => {}
            Key::End => {}
        }

        println!("Context: {}", context);

        self.cfg.setExtra(&key, format!("{}", context));

        for outgoing in outgoings {
            let edge = self.cfg.getEdge(outgoing);
            self.processNode(edge.to.clone(), context.clone());
        }

        // let usage = match &node.usage {
        //     Some(path) => Some(Usage {
        //         id: key.clone(),
        //         path: path.clone(),
        //     }),
        //     None => None,
        // };
        // let updatedUsages = self.processUsages(usage, node.incoming.clone(), key.clone());
        // if updatedUsages {
        //     let node = self.cfg.getNode(&key);
        //     let outgoings = node.outgoing.clone();
        //     for outgoing in outgoings {
        //         let edge = self.cfg.getEdge(outgoing);
        //         self.processNode(edge.to.clone());
        //     }
        // }
    }
}
