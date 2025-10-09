use std::{collections::BTreeMap, fmt::Display};

use crate::siko::{
    backend::{
        drop::Event::{Collision, Event, EventSeries},
        path::{
            Path::Path,
            Usage::{Usage, UsageKind},
        },
    },
    hir::{
        BlockBuilder::InstructionRef,
        Variable::{Variable, VariableName},
    },
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    pub baseEvents: BTreeMap<VariableName, Vec<Event>>,
    pub usages: BTreeMap<VariableName, EventSeries>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            baseEvents: BTreeMap::new(),
            usages: BTreeMap::new(),
        }
    }

    pub fn addAssign(&mut self, path: Path) {
        //println!("Adding assign path: {}", path);
        self.usages
            .entry(path.root.name())
            .or_insert_with(EventSeries::new)
            .push(Event::Assign(path));
    }

    pub fn addUsage(&mut self, usage: Usage) {
        self.usages
            .entry(usage.path.root.name())
            .or_insert_with(EventSeries::new)
            .push(Event::Usage(usage));
    }

    pub fn useVar(&mut self, var: &Variable, instructionRef: InstructionRef) {
        let ty = var.getType();
        //println!("Using variable: {} {}", var.value.visibleName(), ty);
        if ty.isReference() || ty.isPtr() {
            self.addUsage(Usage {
                path: Path::new(var.clone(), var.location().clone()).setInstructionRef(instructionRef),
                kind: UsageKind::Ref,
            });
        } else {
            self.addUsage(Usage {
                path: Path::new(var.clone(), var.location().clone()).setInstructionRef(instructionRef),
                kind: UsageKind::Move,
            });
        }
    }

    pub fn validate(&self) -> (Vec<Collision>, BTreeMap<VariableName, Vec<Event>>) {
        //println!("Validating context");
        let mut collisions = Vec::new();
        let mut baseEvents = self.baseEvents.clone();
        for (name, usages) in &self.usages {
            //println!("Validating usages for variable: {} {} usage(s)", name, usages.len());
            let origBaseEvents = self.baseEvents.get(name).cloned().unwrap_or_else(Vec::new);
            //let trace = name.to_string() == "tmp4";
            let trace = false;
            let (cs, updatedBaseEvents) = usages.validate(&origBaseEvents, trace);
            collisions.extend(cs);
            baseEvents.insert(name.clone(), updatedBaseEvents);
        }

        (collisions, baseEvents)
    }

    pub fn compress(&self) -> Context {
        let mut compressed = Context::new();
        for (var_name, series) in &self.usages {
            compressed.usages.insert(var_name.clone(), series.compress());
        }
        compressed
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context: [")?;
        write!(f, "Usages: [")?;
        for (var_name, series) in &self.usages {
            write!(f, "\nVariable: {} {}", var_name, series)?;
        }
        write!(f, "]")
    }
}
