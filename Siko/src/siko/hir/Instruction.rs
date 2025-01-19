use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::{hir::Function::Variable, location::Location::Location, qualifiedname::QualifiedName};

use super::Function::BlockId;
use super::Type::Type;

#[derive(Clone, PartialEq)]
pub enum JumpDirection {
    Forward,
    Backward,
}

impl Display for JumpDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JumpDirection::Forward => write!(f, "forward"),
            JumpDirection::Backward => write!(f, "backward"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct FieldInfo {
    pub name: String,
    pub location: Location,
    pub ty: Option<Type>,
}

impl Display for FieldInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "f/{}: {}", self.name, ty)
        } else {
            write!(f, "f/{}", self.name)
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct BlockInfo {
    pub id: String,
}

impl Display for BlockInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Clone, PartialEq)]
pub struct EnumCase {
    pub index: u32,
    pub branch: BlockId,
}

impl std::fmt::Debug for EnumCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.index, self.branch)
    }
}

#[derive(Clone, PartialEq)]
pub struct IntegerCase {
    pub value: Option<String>,
    pub branch: BlockId,
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
pub struct StringCase {
    pub value: Option<String>,
    pub branch: BlockId,
}

impl std::fmt::Debug for StringCase {
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
pub enum InstructionKind {
    FunctionCall(Variable, QualifiedName, Vec<Variable>),
    MethodCall(Variable, Variable, String, Vec<Variable>),
    DynamicFunctionCall(Variable, Variable, Vec<Variable>),
    ValueRef(Variable, Variable),
    FieldRef(Variable, Variable, String),
    TupleIndex(Variable, Variable, i32),
    Bind(Variable, Variable, bool), //mutable
    Tuple(Variable, Vec<Variable>),
    StringLiteral(Variable, String),
    IntegerLiteral(Variable, String),
    CharLiteral(Variable, char),
    Return(Variable, Variable),
    Ref(Variable, Variable),
    Drop(Variable, Variable),
    Jump(Variable, BlockId, JumpDirection),
    Assign(Variable, Variable),
    FieldAssign(Variable, Variable, Vec<FieldInfo>),
    DeclareVar(Variable),
    Transform(Variable, Variable, u32),
    EnumSwitch(Variable, Vec<EnumCase>),
    IntegerSwitch(Variable, Vec<IntegerCase>),
    StringSwitch(Variable, Vec<StringCase>),
    BlockStart(BlockInfo),
    BlockEnd(BlockInfo),
}

impl Display for InstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dump())
    }
}

impl Debug for InstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dump())
    }
}

impl InstructionKind {
    pub fn getResultVar(&self) -> Option<Variable> {
        match self {
            InstructionKind::FunctionCall(v, _, _) => Some(v.clone()),
            InstructionKind::MethodCall(v, _, _, _) => Some(v.clone()),
            InstructionKind::DynamicFunctionCall(v, _, _) => Some(v.clone()),
            InstructionKind::ValueRef(v, _) => Some(v.clone()),
            InstructionKind::FieldRef(v, _, _) => Some(v.clone()),
            InstructionKind::TupleIndex(v, _, _) => Some(v.clone()),
            InstructionKind::Bind(v, _, _) => Some(v.clone()),
            InstructionKind::Tuple(v, _) => Some(v.clone()),
            InstructionKind::StringLiteral(v, _) => Some(v.clone()),
            InstructionKind::IntegerLiteral(v, _) => Some(v.clone()),
            InstructionKind::CharLiteral(v, _) => Some(v.clone()),
            InstructionKind::Return(v, _) => Some(v.clone()),
            InstructionKind::Ref(v, _) => Some(v.clone()),
            InstructionKind::Drop(_, _) => None,
            InstructionKind::Jump(v, _, _) => Some(v.clone()),
            InstructionKind::Assign(_, _) => None,
            InstructionKind::FieldAssign(_, _, _) => None,
            InstructionKind::DeclareVar(v) => Some(v.clone()),
            InstructionKind::Transform(v, _, _) => Some(v.clone()),
            InstructionKind::EnumSwitch(_, _) => None,
            InstructionKind::IntegerSwitch(_, _) => None,
            InstructionKind::StringSwitch(_, _) => None,
            InstructionKind::BlockStart(_) => None,
            InstructionKind::BlockEnd(_) => None,
        }
    }

    pub fn collectVariables(&self) -> Vec<Variable> {
        match self {
            InstructionKind::FunctionCall(var, _, args) => {
                let mut vars = vec![var.clone()];
                vars.extend(args.clone());
                vars
            }
            InstructionKind::MethodCall(var, obj, _, args) => {
                let mut vars = vec![var.clone(), obj.clone()];
                vars.extend(args.clone());
                vars
            }
            InstructionKind::DynamicFunctionCall(var, func, args) => {
                let mut vars = vec![var.clone(), func.clone()];
                vars.extend(args.clone());
                vars
            }
            InstructionKind::ValueRef(var, target) => vec![var.clone(), target.clone()],
            InstructionKind::FieldRef(var, target, _) => vec![var.clone(), target.clone()],
            InstructionKind::TupleIndex(var, target, _) => vec![var.clone(), target.clone()],
            InstructionKind::Bind(var, value, _) => vec![var.clone(), value.clone()],
            InstructionKind::Tuple(var, elements) => {
                let mut vars = vec![var.clone()];
                vars.extend(elements.clone());
                vars
            }
            InstructionKind::StringLiteral(var, _) => vec![var.clone()],
            InstructionKind::IntegerLiteral(var, _) => vec![var.clone()],
            InstructionKind::CharLiteral(var, _) => vec![var.clone()],
            InstructionKind::Return(var, value) => vec![var.clone(), value.clone()],
            InstructionKind::Ref(var, target) => vec![var.clone(), target.clone()],
            InstructionKind::Drop(_, _) => vec![],
            InstructionKind::Jump(var, _, _) => vec![var.clone()],
            InstructionKind::Assign(var, value) => vec![var.clone(), value.clone()],
            InstructionKind::FieldAssign(var, value, _) => vec![var.clone(), value.clone()],
            InstructionKind::DeclareVar(var) => vec![var.clone()],
            InstructionKind::Transform(var, target, _) => vec![var.clone(), target.clone()],
            InstructionKind::EnumSwitch(var, _) => {
                vec![var.clone()]
            }
            InstructionKind::IntegerSwitch(var, _) => {
                vec![var.clone()]
            }
            InstructionKind::StringSwitch(var, _) => {
                vec![var.clone()]
            }
            InstructionKind::BlockStart(_) => Vec::new(),
            InstructionKind::BlockEnd(_) => Vec::new(),
        }
    }

    pub fn dump(&self) -> String {
        match self {
            InstructionKind::FunctionCall(dest, name, args) => format!("{} = call({}({:?}))", dest, name, args),
            InstructionKind::MethodCall(dest, receiver, name, args) => {
                format!("{} = methodcall({}.{}({:?}))", dest, receiver, name, args)
            }
            InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                format!("{} = DYN_CALL({}, {:?})", dest, callable, args)
            }
            InstructionKind::ValueRef(dest, v) => format!("{} = {}", dest, v),
            InstructionKind::FieldRef(dest, v, name) => format!("{} = ({}).{}", dest, v, name),
            InstructionKind::TupleIndex(dest, v, idx) => format!("{} = {}.t{}", dest, v, idx),
            InstructionKind::Bind(v, rhs, mutable) => {
                if *mutable {
                    format!("mut ${} = {}", v, rhs)
                } else {
                    format!("${} = {}", v, rhs)
                }
            }
            InstructionKind::Tuple(dest, args) => format!("{} = tuple({:?})", dest, args),
            InstructionKind::StringLiteral(dest, v) => format!("{} = s:[{}]", dest, v),
            InstructionKind::IntegerLiteral(dest, v) => format!("{} = i:[{}]", dest, v),
            InstructionKind::CharLiteral(dest, v) => format!("{} = c:[{}]", dest, v),
            InstructionKind::Return(dest, id) => format!("{} = return({})", dest, id),
            InstructionKind::Ref(dest, id) => format!("{} = &({})", dest, id),
            InstructionKind::Drop(dest, value) => {
                format!("drop({}/{})", dest, value)
            }
            InstructionKind::Jump(dest, id, direction) => {
                format!("{} = jump({}, {})", dest, id, direction)
            }
            InstructionKind::Assign(v, arg) => format!("assign({}, {})", v, arg),
            InstructionKind::FieldAssign(v, arg, fields) => {
                let fields = fields
                    .iter()
                    .map(|info| info.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fieldassign({}, {}, {})", v, arg, fields)
            }
            InstructionKind::DeclareVar(v) => format!("declare({})", v),
            InstructionKind::Transform(dest, arg, index) => format!("{} = transform({}, {})", dest, arg, index),
            InstructionKind::EnumSwitch(root, cases) => format!("enumswitch({}, {:?})", root, cases),
            InstructionKind::IntegerSwitch(root, cases) => format!("integerswitch({}, {:?})", root, cases),
            InstructionKind::StringSwitch(root, cases) => format!("stringswitch({}, {:?})", root, cases),
            InstructionKind::BlockStart(info) => format!("blockstart({})", info),
            InstructionKind::BlockEnd(info) => format!("blockend({})", info),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tag {
    ImplicitRef(u32),
    ImplicitClone(u32),
    Assign(u32),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TagKind {
    ImplicitRef,
    Assign,
}

impl Tag {
    pub fn getId(&self) -> u32 {
        match self {
            Tag::ImplicitRef(id) => *id,
            Tag::ImplicitClone(id) => *id,
            Tag::Assign(id) => *id,
        }
    }

    pub fn isKind(&self, kind: TagKind) -> bool {
        match self {
            Tag::ImplicitRef(_) => kind == TagKind::ImplicitRef,
            Tag::ImplicitClone(_) => kind == TagKind::ImplicitRef,
            Tag::Assign(_) => kind == TagKind::Assign,
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::ImplicitRef(id) => write!(f, "implicit_ref_{}", id),
            Tag::ImplicitClone(id) => write!(f, "implicit_clone_{}", id),
            Tag::Assign(id) => write!(f, "assign_{}", id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub implicit: bool,
    pub kind: InstructionKind,
    pub location: Location,
    pub tags: Vec<Tag>,
}

impl Instruction {
    pub fn dump(&self) {
        println!("    {}", self);
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind.dump())?;
        Ok(())
    }
}
