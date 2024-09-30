use std::{collections::BTreeMap, fmt::Debug, fmt::Display};

use crate::siko::ir::{
    Lifetime::Lifetime,
    Type::{formatTypes, Type},
};

#[derive(Clone)]
pub struct DataFlowProfile {
    pub args: Vec<Type>,
    pub result: Type,
    pub deps: BTreeMap<Lifetime, Vec<Lifetime>>,
}

impl DataFlowProfile {
    pub fn new(args: Vec<Type>, result: Type) -> DataFlowProfile {
        DataFlowProfile {
            args: args,
            result: result,
            deps: BTreeMap::new(),
        }
    }
}

impl Debug for DataFlowProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for DataFlowProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut allDeps = Vec::new();
        for (l, deps) in &self.deps {
            let deps: Vec<_> = deps.iter().map(|l| format!("{}", l)).collect();
            let s = format!("{}: {}", l, deps.join(", "));
            allDeps.push(s);
        }
        write!(
            f,
            "{} -> {}: {}",
            formatTypes(&self.args),
            self.result,
            allDeps.join("& ")
        )?;
        Ok(())
    }
}
