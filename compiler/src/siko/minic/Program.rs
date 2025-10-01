use std::collections::{BTreeMap, BTreeSet};

use crate::siko::minic::Type::Type;

use super::{Constant::StringConstant, Data::Struct, Function::Function};

pub struct Program {
    pub functions: Vec<Function>,
    pub structs: BTreeMap<String, Struct>,
    pub strings: Vec<StringConstant>,
    pub fnPointerTypes: BTreeSet<Type>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: Vec::new(),
            structs: BTreeMap::new(),
            strings: Vec::new(),
            fnPointerTypes: BTreeSet::new(),
        }
    }

    pub fn getStruct(&self, n: &String) -> Struct {
        match self.structs.get(n) {
            Some(s) => s.clone(),
            None => panic!("Struct {} not found in llvm", n),
        }
    }
}
