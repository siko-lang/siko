use std::{collections::BTreeMap, fmt};

use crate::siko::{hir::Variable::Variable, resolver::matchcompiler::DataPath::DataPath};

#[derive(Clone)]
pub struct CompileContext {
    values: BTreeMap<DataPath, Variable>,
}

impl CompileContext {
    pub fn new() -> CompileContext {
        CompileContext {
            values: BTreeMap::new(),
        }
    }

    pub fn add(&self, path: DataPath, value: Variable) -> CompileContext {
        let mut c = self.clone();
        c.values.insert(path, value);
        c
    }

    pub fn get(&self, path: &DataPath) -> Variable {
        match self.values.get(path) {
            Some(id) => id.clone(),
            None => {
                panic!("not found value for path in compile context {}", path)
            }
        }
    }
}

impl fmt::Display for CompileContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CompileContext {{ ")?;
        for (key, value) in &self.values {
            write!(f, "{}: {}, ", key, value)?;
        }
        write!(f, "}}")
    }
}
