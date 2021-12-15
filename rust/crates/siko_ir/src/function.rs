use crate::class::ClassMemberId;
use crate::data::TypeDefId;
use crate::expr::ExprId;
use crate::type_signature::TypeSignatureId;
use crate::types::Type;
use siko_location_info::location_id::LocationId;
use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct FunctionId {
    pub id: usize,
}

impl fmt::Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "func#{}", self.id)
    }
}

impl From<usize> for FunctionId {
    fn from(id: usize) -> FunctionId {
        FunctionId { id: id }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NamedFunctionKind {
    Free,
    DefaultClassMember(ClassMemberId),
    InstanceMember(Option<String>),
    ExternClassImpl(String, Type),
}

#[derive(Debug, Clone)]
pub struct NamedFunctionInfo {
    pub body: Option<ExprId>,
    pub module: String,
    pub name: String,
    pub type_signature: Option<TypeSignatureId>,
    pub location_id: LocationId,
    pub kind: NamedFunctionKind,
}

impl fmt::Display for NamedFunctionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.module, self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LambdaInfo {
    pub body: ExprId,
    pub module: String,
    pub host_info: String,
    pub host_function: FunctionId,
    pub index: usize,
    pub location_id: LocationId,
}

impl fmt::Display for LambdaInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/lambda#{}", self.host_info, self.index)
    }
}

#[derive(Debug, Clone)]
pub struct RecordConstructorInfo {
    pub type_id: TypeDefId,
}

impl fmt::Display for RecordConstructorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.type_id)
    }
}

#[derive(Debug, Clone)]
pub struct VariantConstructorInfo {
    pub type_id: TypeDefId,
    pub index: usize,
}

impl fmt::Display for VariantConstructorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.type_id, self.index)
    }
}

#[derive(Debug, Clone)]
pub enum FunctionInfo {
    Lambda(LambdaInfo),
    NamedFunction(NamedFunctionInfo),
    RecordConstructor(RecordConstructorInfo),
    VariantConstructor(VariantConstructorInfo),
}

impl fmt::Display for FunctionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionInfo::Lambda(i) => write!(f, "lambda{}", i),
            FunctionInfo::NamedFunction(i) => write!(f, "{}", i),
            FunctionInfo::RecordConstructor(i) => write!(f, "{}", i),
            FunctionInfo::VariantConstructor(i) => write!(f, "{}", i),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub id: FunctionId,
    pub arg_locations: Vec<LocationId>,
    pub arg_count: usize,
    pub info: FunctionInfo,
    pub inline: bool,
}

impl Function {
    pub fn get_lambda_host(&self) -> Option<FunctionId> {
        match &self.info {
            FunctionInfo::Lambda(i) => Some(i.host_function),
            _ => None,
        }
    }

    pub fn get_body(&self) -> Option<ExprId> {
        match &self.info {
            FunctionInfo::Lambda(i) => Some(i.body),
            FunctionInfo::NamedFunction(i) => i.body.clone(),
            FunctionInfo::RecordConstructor(_) => None,
            FunctionInfo::VariantConstructor(_) => None,
        }
    }

    pub fn is_typed(&self) -> bool {
        match &self.info {
            FunctionInfo::Lambda(_) => false,
            FunctionInfo::NamedFunction(i) => i.type_signature.is_some(),
            FunctionInfo::RecordConstructor(_) => true,
            FunctionInfo::VariantConstructor(_) => true,
        }
    }
}
