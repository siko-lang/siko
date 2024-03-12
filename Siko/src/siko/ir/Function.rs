use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

pub struct Param {
    pub name: String,
    pub ty: Type,
}

pub enum Instruction {}

pub struct Block {
    pub instructions: Vec<Instruction>,
}
pub struct Body {
    pub blocks: Vec<Block>,
}
pub struct Function {
    pub name: QualifiedName,
    pub params: Vec<Param>,
    pub result: Type,
    pub body: Option<Body>,
}
