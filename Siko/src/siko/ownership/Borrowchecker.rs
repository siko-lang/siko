use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    iter::zip,
    ops::AddAssign,
};

use crate::siko::cfg::CFG::{Key, CFG};

use super::Path::Path;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Usage {
    id: Key,
    path: Path,
}

impl Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.id, self.path)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct UsageSet {
    usages: BTreeSet<Usage>,
}

impl UsageSet {
    fn new() -> UsageSet {
        UsageSet {
            usages: BTreeSet::new(),
        }
    }

    fn add(&mut self, usage: Usage) {
        self.usages.insert(usage);
    }
}

impl AddAssign for UsageSet {
    fn add_assign(&mut self, mut rhs: UsageSet) {
        self.usages.append(&mut rhs.usages);
    }
}

impl Display for UsageSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut usages = Vec::new();
        for u in &self.usages {
            usages.push(format!("{}", u));
        }
        write!(f, "{}", usages.join(", "))
    }
}

pub struct Borrowchecker {
    cfg: CFG,
    usages: BTreeMap<Key, UsageSet>,
    borrows: BTreeSet<Key>,
}

impl Borrowchecker {
    pub fn new(cfg: CFG) -> Borrowchecker {
        Borrowchecker {
            cfg: cfg,
            usages: BTreeMap::new(),
            borrows: BTreeSet::new(),
        }
    }

    pub fn check(&mut self) {
        let sources = self.cfg.getSources();
        for s in sources {
            self.processNode(s);
        }
    }

    fn invalidates(&self, current: &Path, other: &Path) -> bool {
        if current.getValue() != other.getValue() {
            return false;
        }
        match (current, other) {
            (Path::WholePath(_), Path::WholePath(_)) => true,
            (Path::WholePath(_), Path::PartialPath(_, _)) => true,
            (Path::PartialPath(_, _), Path::WholePath(_)) => true,
            (Path::PartialPath(_, currentFields), Path::PartialPath(_, otherFields)) => {
                for (a, b) in zip(currentFields, otherFields) {
                    if a != b {
                        return false;
                    }
                }
                true
            }
        }
    }

    fn invalidate(&mut self, usage: &Usage, usages: &UsageSet) {
        for prevUsage in &usages.usages {
            if self.invalidates(&usage.path, &prevUsage.path) {
                self.borrows.insert(prevUsage.id.clone());
            }
        }
    }

    fn processUsages(&mut self, usage: Option<Usage>, incomings: Vec<u64>, key: Key) -> bool {
        let mut usages = UsageSet::new();
        for incoming in incomings {
            let edge = self.cfg.getEdge(incoming);
            if let Some(prevUsage) = self.usages.get(&edge.from) {
                usages += prevUsage.clone();
            }
        }
        if let Some(usage) = usage {
            self.invalidate(&usage, &usages);
            usages.add(usage);
        }
        if let Some(oldUsages) = self.usages.get(&key) {
            if oldUsages == &usages {
                return false;
            }
        }
        self.usages.insert(key, usages);
        true
    }

    fn processNode(&mut self, key: Key) {
        let node = self.cfg.getNode(&key);
        let usage = match &node.usage {
            Some(path) => Some(Usage {
                id: key.clone(),
                path: path.clone(),
            }),
            None => None,
        };
        let updatedUsages = self.processUsages(usage, node.incoming.clone(), key.clone());
        if updatedUsages {
            let node = self.cfg.getNode(&key);
            let outgoings = node.outgoing.clone();
            for outgoing in outgoings {
                let edge = self.cfg.getEdge(outgoing);
                self.processNode(edge.to.clone());
            }
        }
    }

    pub fn update(&mut self) {
        for b in &self.borrows {
            self.cfg.setColor(&b, "#cf03fc".to_string());
        }
    }

    pub fn cfg(self) -> CFG {
        self.cfg
    }
}

//     def update(self):
//         borrows = set()
//         for b in self.borrows:
//             if isinstance(b, CFG.InstructionKey):
//                 self.fn.body.getInstruction(b.id).borrow = True
//                 borrows.add(b.id)
//         for c in self.cancelled_drops:
//             self.fn.body.getInstruction(c.id).cancelled = True
//         for (key, node) in self.cfg.nodes.items():
//             #print("key %s, usage %s/%s" % (key, node.usage, type(node.usage)))
//             #print("all usages: %s" % self.usages[key])
//             if isinstance(key, CFG.InstructionKey):
//                 witnessed_usages = self.usages[key]
//                 moves = set()
//                 for witnessed_usage in witnessed_usages.usages:
//                     if witnessed_usage.id.id not in borrows:
//                         moves.add(witnessed_usage.path)
//                 instruction = self.fn.body.getInstruction(key.id)
//                 instruction.moves = moves
//                 if node.usage is not None:
//                     instruction.usage = node.usage

//     def printUsages(self):
//         for (id, usage) in self.usages.items():
//             if usage.len() > 0:
//                 print("   Usages for %s" % id)
//                 print("   %s" % usage)

// def checkFn(fn):
//     #print("Checking %s" % fn.name)
//     #fn.body.dump()
//     cfgbuilder = CFGBuilder.CFGBuilder()
//     cfg = cfgbuilder.build(fn)
//     borrowchecker = Borrowchecker(cfg, fn)
//     borrowchecker.check()
//     borrowchecker.update()
//     # borrowchecker.printUsages()
//     for b in borrowchecker.borrows:
//         cfg.getNode(b).color = "#cf03fc"
//     for c in borrowchecker.cancelled_drops:
//         fn.body.getInstruction(c.id).cancelled = True
//         cfg.getNode(c).color = "#ff99ff"
//     cfg.printDot()

// def cleanDrops(program):
//     for (name, fn) in program.functions.items():
//         for b in fn.body.blocks:
//             for (index, i) in enumerate(b.instructions):
//                 if isinstance(i, Instruction.DropVar):
//                     if i.cancelled:
//                         nop = Instruction.Nop()
//                         nop.id = i.id
//                         b.instructions[index] = nop
//             while True:
//                 if isinstance(b.instructions[-1], Instruction.Nop):
//                     b.instructions.pop()
//                 else:
//                     break

// def processProgram(program):
//     for (name, fn) in program.functions.items():
//         checkFn(fn)
//     cleanDrops(program)
