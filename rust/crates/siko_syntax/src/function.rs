use crate::class::Constraint;
use crate::expr::ExprId;
use crate::types::TypeSignatureId;
use siko_location_info::location_id::LocationId;

#[derive(Debug, Clone)]
pub enum FunctionBody {
    Expr(ExprId),
    Extern,
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub id: FunctionTypeId,
    pub name: String,
    pub type_args: Vec<(String, LocationId)>,
    pub constraints: Vec<Constraint>,
    pub full_type_signature_id: TypeSignatureId,
    pub type_signature_id: TypeSignatureId,
    pub location_id: LocationId,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub id: FunctionId,
    pub name: String,
    pub args: Vec<(String, LocationId)>,
    pub body: FunctionBody,
    pub location_id: LocationId,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct FunctionId {
    pub id: usize,
}

impl From<usize> for FunctionId {
    fn from(id: usize) -> FunctionId {
        FunctionId { id: id }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct FunctionTypeId {
    pub id: usize,
}

impl From<usize> for FunctionTypeId {
    fn from(id: usize) -> FunctionTypeId {
        FunctionTypeId { id: id }
    }
}
