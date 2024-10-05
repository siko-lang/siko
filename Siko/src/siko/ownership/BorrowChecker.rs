use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    cfg::{
        Builder::Builder,
        CFG::{Key, CFG},
    },
    ir::Function::Function,
};

#[derive(Clone)]
struct BorrowContext {
    liveValues: BTreeSet<String>,
    deadValues: BTreeSet<String>,
}

impl BorrowContext {
    pub fn new() -> BorrowContext {
        BorrowContext {
            liveValues: BTreeSet::new(),
            deadValues: BTreeSet::new(),
        }
    }

    pub fn merge(&mut self, other: BorrowContext) -> bool {
        let before = self.liveValues.len() + self.deadValues.len();
        self.liveValues.extend(other.liveValues);
        self.deadValues.extend(other.deadValues);
        let after = self.liveValues.len() + self.deadValues.len();
        before != after
    }
}

impl Display for BorrowContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "live: [")?;
        for (index, l) in self.liveValues.iter().enumerate() {
            if index == 0 {
                write!(f, "{}", l)?;
            } else {
                write!(f, ", {}", l)?;
            }
        }
        write!(f, "]")?;
        write!(f, "dead: [")?;
        for (index, l) in self.deadValues.iter().enumerate() {
            if index == 0 {
                write!(f, "{}", l)?;
            } else {
                write!(f, ", {}", l)?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

pub struct BorrowChecker<'a> {
    function: &'a Function,
    cfg: CFG,
    contexts: BTreeMap<Key, BorrowContext>,
}

impl<'a> BorrowChecker<'a> {
    pub fn new(function: &'a Function) -> BorrowChecker<'a> {
        let mut cfgBuilder = Builder::new(function.name.toString(), function.result.clone());
        cfgBuilder.build(function);
        BorrowChecker {
            function: function,
            cfg: cfgBuilder.getCFG(),
            contexts: BTreeMap::new(),
        }
    }

    pub fn check(&mut self) {
        for (key, _) in &self.cfg.nodes {
            self.contexts.insert(key.clone(), BorrowContext::new());
        }
        let sources = self.cfg.getSources();
        for s in sources {
            self.processNode(s);
        }

        self.cfg.printDot();
    }

    fn getContext(&self, key: &Key) -> BorrowContext {
        self.contexts
            .get(key)
            .expect("not borrow context found")
            .clone()
    }

    fn processNode(&mut self, key: Key) {
        println!("Processing node {}", key);
        let mut context = self.getContext(&key);
        let node = self.cfg.getNode(&key);
        let outgoings = node.outgoing.clone();
        let mut updated = false;
        for incoming in &node.incoming {
            let e = self.cfg.getEdge(*incoming);
            let incomingContext = self.getContext(&e.from);
            if context.merge(incomingContext) {
                updated = true;
            }
        }

        match key {
            Key::DropKey(instruction_id, _) => {}
            Key::Instruction(instruction_id) => {
                let i = self
                    .function
                    .body
                    .as_ref()
                    .expect("no body")
                    .getInstruction(instruction_id);
            }
            Key::LoopEnd(instruction_id) => {}
            Key::LoopStart(instruction_id) => {}
            Key::If(instruction_id) => {}
            Key::End => {}
        }

        println!("Context: {}", context);
        if updated {
            self.cfg.setExtra(&key, format!("{}", context));

            for outgoing in outgoings {
                let edge = self.cfg.getEdge(outgoing);
                self.processNode(edge.to.clone());
            }
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
