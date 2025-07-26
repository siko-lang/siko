use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::{Function::Function, Instruction::InstructionKind, Variable::VariableName};

pub struct SingleUseVariables<'a> {
    function: &'a Function,
    uses: BTreeMap<VariableName, u32>,
    receivers: BTreeSet<VariableName>,
}

impl<'a> SingleUseVariables<'a> {
    pub fn new(f: &'a Function) -> SingleUseVariables<'a> {
        SingleUseVariables {
            function: f,
            uses: BTreeMap::new(),
            receivers: BTreeSet::new(),
        }
    }

    pub fn process(&mut self) -> SingleUseVariableInfo {
        if let Some(body) = &self.function.body {
            let blockIds = body.getAllBlockIds();
            for blockId in blockIds {
                let block = body.getBlockById(blockId);
                for i in block.instructions.iter() {
                    if let InstructionKind::FieldRef(_, receiver, _) = &i.kind {
                        self.receivers.insert(receiver.value.clone());
                    }
                    let mut allVariables = i.kind.collectVariables();
                    if let Some(result) = i.kind.getResultVar() {
                        allVariables.retain(|var| var != &result);
                    }
                    for var in allVariables {
                        let count = self.uses.entry(var.value.clone()).or_insert(0);
                        *count += 1;
                    }
                }
            }
        }

        // for (var, count) in self.uses.iter() {
        //     println!(
        //         "Variable {} is used {} times in {}",
        //         var.visibleName(),
        //         count,
        //         self.function.name
        //     );
        // }
        let singleUseVars: BTreeSet<VariableName> = self
            .uses
            .iter()
            .filter_map(|(var, &count)| if count == 1 { Some(var.clone()) } else { None })
            .collect();
        SingleUseVariableInfo {
            singleUseVars,
            receivers: self.receivers.clone(),
        }
    }
}

pub struct SingleUseVariableInfo {
    singleUseVars: BTreeSet<VariableName>,
    receivers: BTreeSet<VariableName>,
}

impl SingleUseVariableInfo {
    pub fn isSingleUse(&self, var: &VariableName) -> bool {
        self.singleUseVars.contains(var)
    }
    pub fn isReceiver(&self, var: &VariableName) -> bool {
        self.receivers.contains(var)
    }
}
