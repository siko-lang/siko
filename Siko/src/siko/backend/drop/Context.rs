use std::collections::BTreeMap;

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
    pub liveData: Vec<Path>,
    pub usages: BTreeMap<VariableName, EventSeries>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            liveData: Vec::new(),
            usages: BTreeMap::new(),
        }
    }

    pub fn addLive(&mut self, data: Path) {
        self.liveData.push(data);
    }

    pub fn addAssign(&mut self, path: Path) {
        self.usages
            .entry(path.root.value.clone())
            .or_insert_with(EventSeries::new)
            .push(Event::Assign(path));
    }

    pub fn addUsage(&mut self, usage: Usage) {
        self.usages
            .entry(usage.path.root.value.clone())
            .or_insert_with(EventSeries::new)
            .push(Event::Usage(usage));
    }

    pub fn useVar(&mut self, var: &Variable, instructionRef: InstructionRef) {
        let ty = var.getType();
        //  println!("Using variable: {} {}", var.value.visibleName(), ty);
        if ty.isReference() {
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
