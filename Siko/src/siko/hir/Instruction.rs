use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

use super::Function::BlockId;
use super::Type::Type;
use super::Variable::Variable;

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
pub enum FieldId {
    Named(String),
    Indexed(u32),
}

impl FieldId {
    pub fn name(&self) -> String {
        match self {
            FieldId::Named(name) => name.clone(),
            FieldId::Indexed(index) => panic!("indexed field found in FieldId::name() {}", index),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct FieldInfo {
    pub name: FieldId,
    pub location: Location,
    pub ty: Option<Type>,
}

impl Display for FieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldId::Named(name) => write!(f, "{}", name),
            FieldId::Indexed(index) => write!(f, "t{}", index),
        }
    }
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
pub enum Mutability {
    Mutable,
    Immutable,
    ExplicitMutable,
}

impl std::fmt::Display for Mutability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mutability::Mutable => write!(f, "mutable"),
            Mutability::Immutable => write!(f, "immutable"),
            Mutability::ExplicitMutable => write!(f, "explicit mutable"),
        }
    }
}

impl std::fmt::Debug for Mutability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq)]
pub enum InstructionKind {
    FunctionCall(Variable, QualifiedName, Vec<Variable>),
    Converter(Variable, Variable),
    MethodCall(Variable, Variable, String, Vec<Variable>),
    DynamicFunctionCall(Variable, Variable, Vec<Variable>),
    FieldRef(Variable, Variable, Vec<FieldInfo>),
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
    DeclareVar(Variable, Mutability),
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
            InstructionKind::Converter(v, _) => Some(v.clone()),
            InstructionKind::MethodCall(v, _, _, _) => Some(v.clone()),
            InstructionKind::DynamicFunctionCall(v, _, _) => Some(v.clone()),
            InstructionKind::FieldRef(v, _, _) => Some(v.clone()),
            InstructionKind::Bind(v, _, _) => Some(v.clone()),
            InstructionKind::Tuple(v, _) => Some(v.clone()),
            InstructionKind::StringLiteral(v, _) => Some(v.clone()),
            InstructionKind::IntegerLiteral(v, _) => Some(v.clone()),
            InstructionKind::CharLiteral(v, _) => Some(v.clone()),
            InstructionKind::Return(v, _) => Some(v.clone()),
            InstructionKind::Ref(v, _) => Some(v.clone()),
            InstructionKind::Drop(_, _) => None,
            InstructionKind::Jump(v, _, _) => Some(v.clone()),
            InstructionKind::Assign(v, _) => Some(v.clone()),
            InstructionKind::FieldAssign(_, _, _) => None,
            InstructionKind::DeclareVar(v, _) => Some(v.clone()),
            InstructionKind::Transform(v, _, _) => Some(v.clone()),
            InstructionKind::EnumSwitch(_, _) => None,
            InstructionKind::IntegerSwitch(_, _) => None,
            InstructionKind::StringSwitch(_, _) => None,
            InstructionKind::BlockStart(_) => None,
            InstructionKind::BlockEnd(_) => None,
        }
    }

    pub fn replaceVar(&self, from: Variable, to: Variable) -> InstructionKind {
        match self {
            InstructionKind::FunctionCall(var, name, args) => {
                let new_var = var.replace(&from, to.clone());
                let new_args = args.iter().map(|arg| arg.replace(&from, to.clone())).collect();
                InstructionKind::FunctionCall(new_var, name.clone(), new_args)
            }
            InstructionKind::Converter(var, source) => {
                let new_var = var.replace(&from, to.clone());
                let new_source = source.replace(&from, to);
                InstructionKind::Converter(new_var, new_source)
            }
            InstructionKind::MethodCall(var, obj, name, args) => {
                let new_var = var.replace(&from, to.clone());
                let new_obj = obj.replace(&from, to.clone());
                let new_args = args.iter().map(|arg| arg.replace(&from, to.clone())).collect();
                InstructionKind::MethodCall(new_var, new_obj, name.clone(), new_args)
            }
            InstructionKind::DynamicFunctionCall(var, func, args) => {
                let new_var = var.replace(&from, to.clone());
                let new_func = func.replace(&from, to.clone());
                let new_args = args.iter().map(|arg| arg.replace(&from, to.clone())).collect();
                InstructionKind::DynamicFunctionCall(new_var, new_func, new_args)
            }
            InstructionKind::FieldRef(var, target, name) => {
                let new_var = var.replace(&from, to.clone());
                let new_target = target.replace(&from, to);
                InstructionKind::FieldRef(new_var, new_target, name.clone())
            }
            InstructionKind::Bind(var, value, mutable) => {
                let new_var = var.replace(&from, to.clone());
                let new_value = value.replace(&from, to);
                InstructionKind::Bind(new_var, new_value, *mutable)
            }
            InstructionKind::Tuple(var, elements) => {
                let new_var = var.replace(&from, to.clone());
                let new_elements = elements.iter().map(|elem| elem.replace(&from, to.clone())).collect();
                InstructionKind::Tuple(new_var, new_elements)
            }
            InstructionKind::StringLiteral(var, value) => {
                let new_var = var.replace(&from, to.clone());
                InstructionKind::StringLiteral(new_var, value.clone())
            }
            InstructionKind::IntegerLiteral(var, value) => {
                let new_var = var.replace(&from, to.clone());
                InstructionKind::IntegerLiteral(new_var, value.clone())
            }
            InstructionKind::CharLiteral(var, value) => {
                let new_var = var.replace(&from, to.clone());
                InstructionKind::CharLiteral(new_var, *value)
            }
            InstructionKind::Return(var, value) => {
                let new_var = var.replace(&from, to.clone());
                let new_value = value.replace(&from, to);
                InstructionKind::Return(new_var, new_value)
            }
            InstructionKind::Ref(var, target) => {
                let new_var = var.replace(&from, to.clone());
                let new_target = target.replace(&from, to);
                InstructionKind::Ref(new_var, new_target)
            }
            InstructionKind::Drop(_, _) => self.clone(),
            InstructionKind::Jump(var, id, direction) => {
                let new_var = var.replace(&from, to.clone());
                InstructionKind::Jump(new_var, id.clone(), direction.clone())
            }
            InstructionKind::Assign(var, arg) => {
                let new_var = var.replace(&from, to.clone());
                let new_arg = arg.replace(&from, to);
                InstructionKind::Assign(new_var, new_arg)
            }
            InstructionKind::FieldAssign(var, arg, fields) => {
                let new_var = var.replace(&from, to.clone());
                let new_arg = arg.replace(&from, to);
                InstructionKind::FieldAssign(new_var, new_arg, fields.clone())
            }
            InstructionKind::DeclareVar(var, mutability) => {
                let new_var = var.replace(&from, to);
                InstructionKind::DeclareVar(new_var, mutability.clone())
            }
            InstructionKind::Transform(var, arg, index) => {
                let new_var = var.replace(&from, to.clone());
                let new_arg = arg.replace(&from, to);
                InstructionKind::Transform(new_var, new_arg, *index)
            }
            InstructionKind::EnumSwitch(root, cases) => {
                let new_root = root.replace(&from, to);
                InstructionKind::EnumSwitch(new_root, cases.clone())
            }
            InstructionKind::IntegerSwitch(root, cases) => {
                let new_root = root.replace(&from, to);
                InstructionKind::IntegerSwitch(new_root, cases.clone())
            }
            InstructionKind::StringSwitch(root, cases) => {
                let new_root = root.replace(&from, to);
                InstructionKind::StringSwitch(new_root, cases.clone())
            }
            InstructionKind::BlockStart(info) => InstructionKind::BlockStart(info.clone()),
            InstructionKind::BlockEnd(info) => InstructionKind::BlockEnd(info.clone()),
        }
    }

    pub fn collectVariables(&self) -> Vec<Variable> {
        match self {
            InstructionKind::FunctionCall(var, _, args) => {
                let mut vars = vec![var.clone()];
                vars.extend(args.clone());
                vars
            }
            InstructionKind::Converter(var, target) => vec![var.clone(), target.clone()],
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
            InstructionKind::FieldRef(var, target, _) => vec![var.clone(), target.clone()],
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
            InstructionKind::DeclareVar(var, _) => vec![var.clone()],
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
            InstructionKind::Converter(dest, source) => format!("{} = convert({})", dest, source),
            InstructionKind::MethodCall(dest, receiver, name, args) => {
                format!("{} = methodcall({}.{}({:?}))", dest, receiver, name, args)
            }
            InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                format!("{} = DYN_CALL({}, {:?})", dest, callable, args)
            }
            InstructionKind::FieldRef(dest, v, fields) => format!(
                "{} = ({}){}",
                dest,
                v,
                fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(".")
            ),
            InstructionKind::Bind(v, rhs, mutable) => {
                if *mutable {
                    format!("mut {} = {}", v, rhs)
                } else {
                    format!("{} = {}", v, rhs)
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
                let fields = fields.iter().map(|info| info.to_string()).collect::<Vec<_>>().join(".");
                format!("fieldassign({}, {}, {})", v, arg, fields)
            }
            InstructionKind::DeclareVar(v, mutability) => format!("declare({}, {:?})", v, mutability),
            InstructionKind::Transform(dest, arg, index) => format!("{} = transform({}, {})", dest, arg, index),
            InstructionKind::EnumSwitch(root, cases) => format!("enumswitch({}, {:?})", root, cases),
            InstructionKind::IntegerSwitch(root, cases) => format!("integerswitch({}, {:?})", root, cases),
            InstructionKind::StringSwitch(root, cases) => format!("stringswitch({}, {:?})", root, cases),
            InstructionKind::BlockStart(info) => format!("blockstart({})", info),
            InstructionKind::BlockEnd(info) => format!("blockend({})", info),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub implicit: bool,
    pub kind: InstructionKind,
    pub location: Location,
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
