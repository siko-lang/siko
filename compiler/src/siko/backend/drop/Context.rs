use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    backend::drop::{
        Event::{Collision, Event, EventSeries},
        Path::{InstructionRef, Path},
        Usage::{Usage, UsageKind},
    },
    hir::Variable::{Variable, VariableName},
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    pub liveData: BTreeSet<Path>,
    pub usages: BTreeMap<VariableName, EventSeries>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            liveData: BTreeSet::new(),
            usages: BTreeMap::new(),
        }
    }

    pub fn addLive(&mut self, data: Path) {
        self.liveData.insert(data);
    }

    pub fn addAssign(&mut self, path: Path) {
        //println!("Adding assign path: {}", path);
        self.usages
            .entry(path.root.name.clone())
            .or_insert_with(EventSeries::new)
            .push(Event::Assign(path));
    }

    pub fn addUsage(&mut self, usage: Usage) {
        self.usages
            .entry(usage.path.root.name.clone())
            .or_insert_with(EventSeries::new)
            .push(Event::Usage(usage));
    }

    pub fn useVar(&mut self, var: &Variable, instructionRef: InstructionRef) {
        let ty = var.getType();
        //println!("Using variable: {} {}", var.value.visibleName(), ty);
        if ty.isReference() || ty.isPtr() {
            self.addUsage(Usage {
                path: Path::new(var.clone(), var.location.clone()).setInstructionRef(instructionRef),
                kind: UsageKind::Ref,
            });
        } else {
            self.addUsage(Usage {
                path: Path::new(var.clone(), var.location.clone()).setInstructionRef(instructionRef),
                kind: UsageKind::Move,
            });
        }
    }

    pub fn validate(&self) -> Vec<Collision> {
        let mut collisions = Vec::new();

        for (_, usages) in &self.usages {
            //println!("Validating usages for variable: {} {} usage(s)", var_name, usages.len());
            collisions.extend(usages.validate());
        }

        collisions
    }

    pub fn compress(&self) -> Context {
        let mut compressed = Context::new();
        for path in &self.liveData {
            compressed.addLive(path.clone());
        }
        for (var_name, series) in &self.usages {
            compressed.usages.insert(var_name.clone(), series.compress());
        }
        compressed
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context: [")?;
        for (i, path) in self.liveData.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", path)?;
        }
        if !self.liveData.is_empty() {
            write!(f, ", ")?;
        }
        write!(f, "Usages: [")?;
        for (var_name, series) in &self.usages {
            write!(f, "\nVariable: {} {}", var_name, series)?;
        }
        write!(f, "]")
    }
}
