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

pub enum FunctionKind {
    UserDefined(Vec<Block>),
    ClassCtor,
    VariantCtor(i64),
    Extern,
}

pub struct Function {
    pub name: String,
    pub args: Vec<Param>,
    pub result: Type,
    pub kind: FunctionKind,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            FunctionKind::ClassCtor => {
                write!(
                    f,
                    "Function: {}\nArguments: ({}) -> {}\nClassCtor",
                    self.name,
                    self.args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", "),
                    self.result,
                )
            }
            FunctionKind::VariantCtor(_) => {
                write!(
                    f,
                    "Function: {}\nArguments: ({}) -> {}\nVariantCtor",
                    self.name,
                    self.args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", "),
                    self.result,
                )
            }
            FunctionKind::UserDefined(blocks) => {
                write!(
                    f,
                    "Function: {}\nArguments: ({}) -> {}\nBlocks:\n{}",
                    self.name,
                    self.args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", "),
                    self.result,
                    blocks.iter().map(|block| format!("{}", block)).collect::<Vec<_>>().join("\n")
                )
            }
            FunctionKind::Extern => {
                write!(
                    f,
                    "Function: {}\nArguments: ({}) -> {}\nExtern",
                    self.name,
                    self.args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", "),
                    self.result,
                )
            }
        }
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

#[derive(Clone, PartialEq)]
pub struct EnumCase {
    pub index: u32,
    pub branch: String,
}

impl std::fmt::Debug for EnumCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.index, self.branch)
    }
}
pub struct IntegerCase {
    pub value: Option<String>,
    pub branch: String,
}

impl std::fmt::Debug for IntegerCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(v) => {
                write!(f, "({}, {})", v, self.branch)
            }
            None => {
                write!(f, "(<default>, {})", self.branch)
            }
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
    Memcpy(Variable, Variable), //src -> dest
    Store(Variable, Variable),  //src -> dest
    IntegerLiteral(Variable, String),
    StringLiteral(Variable, String),
    EnumSwitch(Variable, Vec<EnumCase>),
    IntegerSwitch(Variable, Vec<IntegerCase>),
    Transform(Variable, Variable, u32),
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
            Instruction::Memcpy(var1, var2) => write!(f, "Memcpy({} => {})", var1, var2),
            Instruction::Store(var1, var2) => write!(f, "Store({} => {})", var1, var2),
            Instruction::IntegerLiteral(var, literal) => write!(f, "IntegerLiteral({}, {})", var, literal),
            Instruction::StringLiteral(var, literal) => write!(f, "StringLiteral({}, {})", var, literal),
            Instruction::EnumSwitch(root, cases) => write!(f, "enumswitch({}, {:?})", root, cases),
            Instruction::IntegerSwitch(root, cases) => write!(f, "integerswitch({}, {:?})", root, cases),
            Instruction::Jump(label) => write!(f, "Jump({})", label),
            Instruction::Transform(dest, src, ty) => write!(f, "Transform({}, {}, {})", dest, src, ty),
        }
    }
}
