use std::collections::BTreeMap;

use crate::siko::hir::{
    BodyBuilder::BodyBuilder,
    Function::Function,
    Instruction::InstructionKind,
    Variable::{Variable, VariableName},
};

pub fn simplifyFunction(f: &Function) -> Option<Function> {
    let mut simplifier = VarSimplifier::new(f);
    simplifier.process()
}

pub struct VarSimplifier<'a> {
    function: &'a Function,
    useCounts: BTreeMap<VariableName, usize>,
    assignCounts: BTreeMap<VariableName, usize>,
    assignments: BTreeMap<VariableName, Variable>,
    simplifiedVars: BTreeMap<VariableName, Variable>,
}

impl<'a> VarSimplifier<'a> {
    pub fn new(f: &'a Function) -> VarSimplifier<'a> {
        VarSimplifier {
            function: f,
            useCounts: BTreeMap::new(),
            assignCounts: BTreeMap::new(),
            assignments: BTreeMap::new(),
            simplifiedVars: BTreeMap::new(),
        }
    }

    fn replace(&mut self, old: VariableName) -> Option<Variable> {
        let mut currentName = old.clone();
        let mut current = None;
        while let Some(new) = self.simplifiedVars.get(&currentName) {
            current = Some(new.clone());
            currentName = new.name();
        }
        return current;
    }

    fn addAssign(&mut self, var: Variable) {
        *self.assignCounts.entry(var.name()).or_insert(0) += 1;
    }

    fn addUse(&mut self, var: Variable) {
        *self.useCounts.entry(var.name()).or_insert(0) += 1;
    }

    fn process(&mut self) -> Option<Function> {
        if self.function.body.is_none() {
            return None;
        }

        // println!("VarSimplifier processing function: {}", self.function.name);
        // println!("{}", self.function);

        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);

        let allBlockIds = bodyBuilder.getAllBlockIds();
        for blockId in &allBlockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::DeclareVar(_, _) = &instruction.kind {
                        builder.step();
                        continue;
                    }
                    let mut allVars = instruction.kind.collectVariables();
                    if let Some(resultVar) = instruction.kind.getResultVar() {
                        allVars.retain(|v| v != &resultVar);
                        self.addAssign(resultVar);
                    }
                    for var in allVars {
                        self.addUse(var);
                    }
                    match instruction.kind {
                        InstructionKind::Assign(dest, src) => {
                            self.assignments.insert(src.name(), dest.clone());
                        }
                        _ => {}
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }

        for (src, useCount) in &self.useCounts {
            if useCount == &1 {
                if let Some(dest) = self.assignments.get(src) {
                    if self.assignCounts.get(&dest.name()) == Some(&1) {
                        if src.isArg() || dest.isArg() {
                            continue;
                        }
                        self.simplifiedVars.insert(src.clone(), dest.clone());
                    } else {
                        //println!("Variable {} is assigned more than once, cannot simplify", dest);
                    }
                } else {
                    // println!(
                    //     "Variable {} is used only once but has no assignment, cannot simplify",
                    //     src
                    // );
                }
            } else {
                //println!("Variable {} is used {} times, cannot simplify", src, useCount);
            }
        }

        if self.simplifiedVars.is_empty() {
            return None; // No variables to simplify
        }

        // for (src, dest) in &self.simplifiedVars {
        //     println!("Replacing {} with {}", src, dest);
        // }

        for blockId in &allBlockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::DeclareVar(var, _) = &instruction.kind {
                        if self.simplifiedVars.contains_key(&var.name()) {
                            //println!("Removing declaration of variable {}", var);
                            builder.removeInstruction();
                            continue;
                        }
                    }
                    let allVars = instruction.kind.collectVariables();
                    let mut kind = instruction.kind.clone();
                    for var in &allVars {
                        if let Some(dest) = self.replace(var.name()) {
                            //println!("Replacing {} with {}", var.name(), dest);
                            kind = kind.replaceVar(var.clone(), dest.useVar());
                        }
                    }
                    if let InstructionKind::Assign(dest, src) = &kind {
                        if dest.name() == src.name() {
                            //println!("Removing self-assignment of variable {}", dest);
                            builder.removeInstruction();
                            continue;
                        }
                    }
                    // if kind != instruction.kind {
                    //     println!("Replacing instruction {} with {}", instruction.kind, kind);
                    // }
                    builder.replaceInstruction(kind, instruction.location.clone());
                    builder.step();
                } else {
                    break;
                }
            }
        }
        let mut f = self.function.clone();
        f.body = Some(bodyBuilder.build());
        Some(f)
    }
}
