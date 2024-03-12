use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

#[derive(Debug)]
pub enum Parameter {
    Named(String, Type, bool),
    SelfParam(bool),
}

#[derive(Debug)]
pub enum Instruction {}

#[derive(Debug)]
pub struct Block {
    pub instructions: Vec<Instruction>,
}
#[derive(Debug)]
pub struct Body {
    pub blocks: Vec<Block>,
}

#[derive(Debug)]
pub struct Function {
    pub name: QualifiedName,
    pub params: Vec<Parameter>,
    pub result: Type,
    pub body: Option<Body>,
}

impl Function {
    pub fn new(name: QualifiedName, params: Vec<Parameter>, result: Type) -> Function {
        Function {
            name: name,
            params: params,
            result: result,
            body: None,
        }
    }
}
