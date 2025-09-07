use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::siko::{
    hir::{
        Instruction::{SyntaxBlockId, SyntaxBlockIdSegment},
        Variable::Variable,
        VariableAllocator::VariableAllocator,
    },
    location::Location::Location,
};

#[derive(Clone, Debug)]

pub struct CaptureInfo {
    varAllocator: VariableAllocator,
    captures: Rc<RefCell<BTreeMap<Variable, Variable>>>,
}

impl CaptureInfo {
    pub fn new(varAllocator: VariableAllocator) -> CaptureInfo {
        CaptureInfo {
            varAllocator,
            captures: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub fn resolveCapture(&self, var: Variable) -> Variable {
        let mut captures = self.captures.borrow_mut();
        let entry = captures
            .entry(var.clone())
            .or_insert_with(|| self.varAllocator.allocate(Location::empty()));
        return entry.clone();
    }

    pub fn get(&self) -> BTreeMap<Variable, Variable> {
        self.captures.borrow().clone()
    }
}

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    values: BTreeMap<String, Variable>,
    parent: Option<&'a Environment<'a>>,
    mutables: BTreeSet<String>,
    syntaxBlockId: SyntaxBlockId,
    captures: CaptureInfo,
    lambdaLevel: u32,
}

impl<'a> Environment<'a> {
    pub fn new(varAllocator: VariableAllocator) -> Environment<'a> {
        Environment {
            values: BTreeMap::new(),
            parent: None,
            mutables: BTreeSet::new(),
            syntaxBlockId: SyntaxBlockId::new(),
            captures: CaptureInfo::new(varAllocator),
            lambdaLevel: 0,
        }
    }

    pub fn child(parent: &'a Environment<'a>, syntaxBlockIdItem: SyntaxBlockIdSegment) -> Environment<'a> {
        Environment {
            values: BTreeMap::new(),
            parent: Some(parent),
            mutables: BTreeSet::new(),
            syntaxBlockId: parent.syntaxBlockId.add(syntaxBlockIdItem),
            captures: CaptureInfo::new(parent.captures.varAllocator.clone()),
            lambdaLevel: parent.lambdaLevel,
        }
    }

    pub fn lambdaEnv(parent: &'a Environment<'a>, syntaxBlockIdItem: SyntaxBlockIdSegment) -> Environment<'a> {
        Environment {
            values: BTreeMap::new(),
            parent: Some(parent),
            mutables: BTreeSet::new(),
            syntaxBlockId: parent.syntaxBlockId.add(syntaxBlockIdItem),
            captures: CaptureInfo::new(parent.captures.varAllocator.clone()),
            lambdaLevel: parent.lambdaLevel + 1,
        }
    }

    pub fn addArg(&mut self, arg: Variable, mutable: bool) {
        let name = arg.name().to_string();
        self.values.insert(arg.name().to_string(), arg);
        if mutable {
            self.mutables.insert(name);
        }
    }

    pub fn addValue(&mut self, old: String, new: Variable) {
        //println!("Added value {}", new);
        self.values.insert(old.clone(), new);
    }

    pub fn resolve(&self, value: &String) -> Option<Variable> {
        match self.values.get(value) {
            Some(v) => Some(v.clone()),
            None => {
                if let Some(parent) = self.parent {
                    if parent.lambdaLevel < self.lambdaLevel {
                        if let Some(v) = parent.resolve(value) {
                            let v = self.captures.resolveCapture(v.clone());
                            return Some(v);
                        }
                    }
                    return parent.resolve(value);
                } else {
                    None
                }
            }
        }
    }

    pub fn values(&self) -> &BTreeMap<String, Variable> {
        &self.values
    }

    pub fn isMutable(&self, name: &String) -> bool {
        self.mutables.contains(name)
    }

    pub fn getSyntaxBlockId(&self) -> SyntaxBlockId {
        self.syntaxBlockId.clone()
    }

    pub fn dump(&self) {
        println!("Environment dump:");
        for (name, var) in &self.values {
            println!("  {}: {}", name, var);
        }
        if let Some(parent) = self.parent {
            println!("Parent environment:");
            parent.dump();
        } else {
            println!("No parent environment.");
        }
    }

    pub fn captures(&self) -> BTreeMap<Variable, Variable> {
        self.captures.get()
    }
}
