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
        self.structs.get(n).cloned().expect("struct not found")
    }
}
