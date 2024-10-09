use super::Function::Function;

pub struct Program {
    pub functions: Vec<Function>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: Vec::new(),
        }
    }
}
