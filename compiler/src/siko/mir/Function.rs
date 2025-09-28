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

pub struct ExternInfo {
    pub name: String,
    pub headerName: Option<String>,
}

pub enum ExternKind {
    C(ExternInfo),
    Builtin,
}

pub enum FunctionKind {
    UserDefined(Vec<Block>),
    StructCtor,
    VariantCtor(i64),
    Extern(ExternKind),
}

pub struct Function {
    pub name: String,
    pub fullName: String,
    pub args: Vec<Param>,
    pub result: Type,
    pub kind: FunctionKind,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            FunctionKind::StructCtor => {
                write!(
                    f,
                    "Function: {}\nArguments: ({}) -> {}\nStructCtor",
                    self.name,
                    self.args
                        .iter()
                        .map(|arg| format!("{}", arg))
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.result,
                )
            }
            FunctionKind::VariantCtor(_) => {
                write!(
                    f,
                    "Function: {}\nArguments: ({}) -> {}\nVariantCtor",
                    self.name,
                    self.args
                        .iter()
                        .map(|arg| format!("{}", arg))
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.result,
                )
            }
            FunctionKind::UserDefined(blocks) => {
                write!(
                    f,
                    "Function: {}\nArguments: ({}) -> {}\nBlocks:\n{}",
                    self.name,
                    self.args
                        .iter()
                        .map(|arg| format!("{}", arg))
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.result,
                    blocks
                        .iter()
                        .map(|block| format!("{}", block))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            FunctionKind::Extern(_) => {
                write!(
                    f,
                    "Function: {}\nArguments: ({}) -> {}\nExtern",
                    self.name,
                    self.args
                        .iter()
                        .map(|arg| format!("{}", arg))
                        .collect::<Vec<_>>()
                        .join(", "),
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

#[derive(Clone, PartialEq)]
pub struct EnumCase {
    pub index: Option<u32>,
    pub branch: String,
}

impl std::fmt::Debug for EnumCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.index {
            Some(v) => {
                write!(f, "({}, {})", v, self.branch)
            }
            None => {
                write!(f, "(<default>, {})", self.branch)
            }
        }
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

#[derive(Clone, PartialEq)]
pub enum IntegerOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    LessThan,
}

pub enum Instruction {
    Declare(Variable),
    GetFieldRef(Variable, Variable, i32),
    Reference(Variable, Variable),
    Call(Option<Variable>, String, Vec<Variable>),
    SetField(Variable, Variable, Vec<i32>),
    Return(Variable),
    Memcpy(Variable, Variable), //src -> dest
    IntegerLiteral(Variable, String),
    StringLiteral(Variable, String),
    EnumSwitch(Variable, Vec<EnumCase>),
    IntegerSwitch(Variable, Vec<IntegerCase>),
    Transform(Variable, Variable, u32),
    Jump(String),
    AddressOfField(Variable, Variable, i32),
    LoadPtr(Variable, Variable),
    StorePtr(Variable, Variable),
    IntegerOp(Variable, Variable, Variable, IntegerOp),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Declare(var) => write!(f, "Declare({})", var),
            Instruction::GetFieldRef(var1, var2, index) => {
                write!(f, "GetFieldRef({}, {}, {})", var1, var2, index)
            }
            Instruction::Reference(var1, var2) => {
                write!(f, "Reference({}, {})", var1, var2)
            }
            Instruction::Call(var, func_name, args) => match var {
                Some(v) => write!(
                    f,
                    "Call({}, {}, [{}])",
                    v,
                    func_name,
                    args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", ")
                ),
                None => write!(
                    f,
                    "Call({}, [{}])",
                    func_name,
                    args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", ")
                ),
            },
            Instruction::SetField(dest, src, indices) => {
                write!(f, "SetField({}, {}, {:?})", dest, src, indices)
            }
            Instruction::Return(value) => write!(f, "Return({})", value),
            Instruction::Memcpy(var1, var2) => {
                write!(f, "Memcpy({} => {})", var1, var2)
            }
            Instruction::IntegerLiteral(var, literal) => {
                write!(f, "IntegerLiteral({}, {})", var, literal)
            }
            Instruction::StringLiteral(var, literal) => {
                write!(f, "StringLiteral({}, {})", var, literal)
            }
            Instruction::EnumSwitch(root, cases) => {
                write!(f, "EnumSwitch({}, {:?})", root, cases)
            }
            Instruction::IntegerSwitch(root, cases) => {
                write!(f, "IntegerSwitch({}, {:?})", root, cases)
            }
            Instruction::Jump(label) => write!(f, "Jump({})", label),
            Instruction::Transform(dest, src, ty) => {
                write!(f, "Transform({}, {}, {})", dest, src, ty)
            }
            Instruction::AddressOfField(dest, root, field) => {
                write!(f, "AddressOfField({}, {}, {})", dest, root, field)
            }
            Instruction::LoadPtr(dest, src) => {
                write!(f, "LoadPtr({}, {})", dest, src)
            }
            Instruction::StorePtr(dest, src) => {
                write!(f, "StorePtr({}, {})", dest, src)
            }
            Instruction::IntegerOp(dest, left, right, op) => {
                let op_str = match op {
                    IntegerOp::Add => "+",
                    IntegerOp::Sub => "-",
                    IntegerOp::Mul => "*",
                    IntegerOp::Div => "/",
                    IntegerOp::Mod => "%",
                    IntegerOp::Eq => "==",
                    IntegerOp::LessThan => "<",
                };
                write!(f, "IntegerOp({}, {} {} {})", dest, left, op_str, right)
            }
        }
    }
}
