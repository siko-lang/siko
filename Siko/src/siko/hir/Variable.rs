use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::location::Location::Location;

use super::Type::Type;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableName {
    Transform(u32),
    MatchVar(u32),
    MatchValue(u32),
    LoopVar(u32),
    LoopFinalValue(u32),
    FunctionResult(u32),
    BlockValue(u32),
    ImplicitRef(u32),
    ImplicitClone(u32),
    ImplicitConvert(u32),
    ImplicitResult(u32),
    ImplicitSelf(u32),
    ImplicitDeref(u32),
    Ref(u32),
    FieldRef(u32),
    Unit(u32),
    Tuple(u32),
    TupleIndex(u32),
    Jump(u32),
    Literal(u32),
    Ret(u32),
    Call(u32),
    Local(String, u32),
    Arg(String),
    DropVar(u32),
    AutoDropResult(u32),
    DropImplicitCloneRef(u32),
    DropImplicitClone(u32),
    LocalArg(u32),
}

impl VariableName {
    pub fn visibleName(&self) -> String {
        match self {
            VariableName::Transform(i) => format!("transform{}", i),
            VariableName::MatchVar(i) => format!("matchVar{}", i),
            VariableName::MatchValue(i) => format!("matchValue{}", i),
            VariableName::LoopVar(i) => format!("loopVar{}", i),
            VariableName::LoopFinalValue(i) => format!("loopFinalValue{}", i),
            VariableName::FunctionResult(i) => format!("functionResult{}", i),
            VariableName::BlockValue(i) => format!("blockValue{}", i),
            VariableName::ImplicitRef(i) => format!("implicitRef{}", i),
            VariableName::ImplicitClone(i) => format!("implicitClone{}", i),
            VariableName::ImplicitConvert(i) => format!("implicitConvert{}", i),
            VariableName::ImplicitResult(i) => format!("implicitResult{}", i),
            VariableName::ImplicitSelf(i) => format!("implicitSelf{}", i),
            VariableName::ImplicitDeref(i) => format!("implicitDeref{}", i),
            VariableName::Ref(i) => format!("ref{}", i),
            VariableName::FieldRef(i) => format!("fieldRef{}", i),
            VariableName::Unit(i) => format!("unit{}", i),
            VariableName::Tuple(i) => format!("tuple{}", i),
            VariableName::TupleIndex(i) => format!("tupleIndex{}", i),
            VariableName::Jump(i) => format!("jump{}", i),
            VariableName::Literal(i) => format!("lit{}", i),
            VariableName::Ret(i) => format!("ret{}", i),
            VariableName::Call(i) => format!("call{}", i),
            VariableName::Local(n, _) => n.clone(),
            VariableName::Arg(n) => n.clone(),
            VariableName::DropVar(i) => format!("dropVar{}", i),
            VariableName::AutoDropResult(i) => format!("autoDropResult{}", i),
            VariableName::DropImplicitCloneRef(i) => format!("implicitCloneRef{}", i),
            VariableName::DropImplicitClone(i) => format!("implicitClone{}", i),
            VariableName::LocalArg(i) => format!("localArg{}", i),
        }
    }
}

impl Display for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Transform(i) => write!(f, "transform{}", i),
            VariableName::MatchVar(i) => write!(f, "matchVar{}", i),
            VariableName::MatchValue(i) => write!(f, "matchValue{}", i),
            VariableName::LoopVar(i) => write!(f, "loopVar{}", i),
            VariableName::LoopFinalValue(i) => write!(f, "loopFinalValue{}", i),
            VariableName::FunctionResult(i) => write!(f, "functionResult{}", i),
            VariableName::BlockValue(i) => write!(f, "blockValue{}", i),
            VariableName::ImplicitRef(i) => write!(f, "implicitRef{}", i),
            VariableName::ImplicitClone(i) => write!(f, "implicitClone{}", i),
            VariableName::ImplicitConvert(i) => write!(f, "implicitConvert{}", i),
            VariableName::ImplicitResult(i) => write!(f, "implicitResult{}", i),
            VariableName::ImplicitSelf(i) => write!(f, "implicitSelf{}", i),
            VariableName::ImplicitDeref(i) => write!(f, "implicitDeref{}", i),
            VariableName::Ref(i) => write!(f, "ref{}", i),
            VariableName::FieldRef(i) => write!(f, "fieldRef{}", i),
            VariableName::Unit(i) => write!(f, "unit{}", i),
            VariableName::Tuple(i) => write!(f, "tuple{}", i),
            VariableName::TupleIndex(i) => write!(f, "tupleIndex{}", i),
            VariableName::Jump(i) => write!(f, "jump{}", i),
            VariableName::Literal(i) => write!(f, "lit{}", i),
            VariableName::Ret(i) => write!(f, "ret{}", i),
            VariableName::Call(i) => write!(f, "call{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
            VariableName::DropVar(i) => write!(f, "dropVar{}", i),
            VariableName::AutoDropResult(i) => write!(f, "autoDropResult{}", i),
            VariableName::DropImplicitCloneRef(i) => write!(f, "implicitCloneRef{}", i),
            VariableName::DropImplicitClone(i) => write!(f, "implicitClone{}", i),
            VariableName::LocalArg(i) => write!(f, "localArg{}", i),
        }
    }
}

impl Debug for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Transform(i) => write!(f, "transform{}", i),
            VariableName::MatchVar(i) => write!(f, "matchVar{}", i),
            VariableName::MatchValue(i) => write!(f, "matchValue{}", i),
            VariableName::LoopVar(i) => write!(f, "loopVar{}", i),
            VariableName::LoopFinalValue(i) => write!(f, "loopFinalValue{}", i),
            VariableName::FunctionResult(i) => write!(f, "functionResult{}", i),
            VariableName::BlockValue(i) => write!(f, "blockValue{}", i),
            VariableName::ImplicitRef(i) => write!(f, "implicitRef{}", i),
            VariableName::ImplicitClone(i) => write!(f, "implicitClone{}", i),
            VariableName::ImplicitConvert(i) => write!(f, "implicitConvert{}", i),
            VariableName::ImplicitResult(i) => write!(f, "implicitResult{}", i),
            VariableName::ImplicitSelf(i) => write!(f, "implicitSelf{}", i),
            VariableName::ImplicitDeref(i) => write!(f, "implicitDeref{}", i),
            VariableName::Ref(i) => write!(f, "ref{}", i),
            VariableName::FieldRef(i) => write!(f, "fieldRef{}", i),
            VariableName::Unit(i) => write!(f, "unit{}", i),
            VariableName::Tuple(i) => write!(f, "tuple{}", i),
            VariableName::TupleIndex(i) => write!(f, "tupleIndex{}", i),
            VariableName::Jump(i) => write!(f, "jump{}", i),
            VariableName::Literal(i) => write!(f, "lit{}", i),
            VariableName::Ret(i) => write!(f, "ret{}", i),
            VariableName::Call(i) => write!(f, "call{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
            VariableName::DropVar(i) => write!(f, "dropVar{}", i),
            VariableName::AutoDropResult(i) => write!(f, "autoDropResult{}", i),
            VariableName::DropImplicitCloneRef(i) => write!(f, "implicitCloneRef{}", i),
            VariableName::DropImplicitClone(i) => write!(f, "implicitClone{}", i),
            VariableName::LocalArg(i) => write!(f, "localArg{}", i),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub value: VariableName,
    pub location: Location,
    pub ty: Option<Type>,
    pub index: u32,
}

impl Variable {
    pub fn getType(&self) -> &Type {
        match &self.ty {
            Some(ty) => ty,
            None => panic!("No type found for var {}", self.value),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "${}/{}: {}", self.value, self.index, ty)
        } else {
            write!(f, "${}/{}", self.value, self.index)
        }
    }
}

impl std::fmt::Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
