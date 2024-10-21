use std::collections::BTreeMap;

use super::{Data::Struct, Function::Function};

pub struct Program {
    pub functions: Vec<Function>,
    pub structs: BTreeMap<String, Struct>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: Vec::new(),
            structs: BTreeMap::new(),
        }
    }

    pub fn getStruct(&self, n: &String) -> Struct {
        match self.structs.get(n) {
            Some(s) => s.clone(),
            None => panic!("Struct {} not found in llvm", n),
        }
    }
}
