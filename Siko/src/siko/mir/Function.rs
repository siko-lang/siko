use std::fmt;

use super::Type::Type;

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

pub struct Function {
    pub name: String,
    pub args: Vec<Param>,
    pub result: Type,
    pub blocks: Vec<Block>,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function: {}\nArguments: ({}) -> {}\nBlocks:\n{}",
            self.name,
            self.args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", "),
            self.result,
            self.blocks.iter().map(|block| format!("{}", block)).collect::<Vec<_>>().join("\n")
        )
    }
}

pub struct Block {
    pub id: String,
    pub instructions: Vec<Instruction>,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block {}:\n{}",
            self.id,
            self.instructions
                .iter()
                .map(|instr| format!("  {}", instr))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub ty: Type,
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub enum Value {
    Void,
    Numeric(String),
    Var(Variable),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Void => write!(f, "Void"),
            Value::Numeric(num) => write!(f, "Numeric({})", num),
            Value::Var(var) => write!(f, "Var({})", var),
        }
    }
}

pub enum Instruction {
    Declare(Variable),
    GetFieldRef(Variable, Variable, i32),
    Reference(Variable, Variable),
    Call(Variable, String, Vec<Variable>),
    Assign(Variable, Value),
    Return(Value),
    Memcpy(Variable, Variable),
    IntegerLiteral(Variable, String),
    Jump(String),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Declare(var) => write!(f, "Declare({})", var),
            Instruction::GetFieldRef(var1, var2, index) => write!(f, "GetFieldRef({}, {}, {})", var1, var2, index),
            Instruction::Reference(var1, var2) => write!(f, "Reference({}, {})", var1, var2),
            Instruction::Call(var, func_name, args) => write!(
                f,
                "Call({}, {}, [{}])",
                var,
                func_name,
                args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", ")
            ),
            Instruction::Assign(var, value) => write!(f, "Assign({}, {})", var, value),
            Instruction::Return(value) => write!(f, "Return({})", value),
            Instruction::Memcpy(var1, var2) => write!(f, "Memcpy({}, {})", var1, var2),
            Instruction::IntegerLiteral(var, literal) => write!(f, "IntegerLiteral({}, {})", var, literal),
            Instruction::Jump(label) => write!(f, "Jump({})", label),
        }
    }
}
