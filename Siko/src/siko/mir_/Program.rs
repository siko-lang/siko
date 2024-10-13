use super::{Data::Class, Function::Function};

pub struct Program {
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: Vec::new(),
            classes: Vec::new(),
        }
    }
}
