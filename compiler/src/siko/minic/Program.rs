use std::collections::BTreeMap;

use super::{Constant::StringConstant, Data::Struct, Function::Function};

pub struct Program {
    pub functions: Vec<Function>,
    pub structs: BTreeMap<String, Struct>,
    pub strings: Vec<StringConstant>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: Vec::new(),
            structs: BTreeMap::new(),
            strings: Vec::new(),
        }
    }

    pub fn getStruct(&self, n: &String) -> Struct {
        match self.structs.get(n) {
            Some(s) => s.clone(),
            None => panic!("Struct {} not found in llvm", n),
        }
    }
}
