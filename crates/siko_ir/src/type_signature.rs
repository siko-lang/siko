use crate::class::ClassId;
use crate::data::TypeDefId;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct TypeSignatureId {
    pub id: usize,
}

impl From<usize> for TypeSignatureId {
    fn from(id: usize) -> TypeSignatureId {
        TypeSignatureId { id: id }
    }
}

#[derive(Debug, Clone)]
pub enum TypeSignature {
    Tuple(Vec<TypeSignatureId>),
    Function(TypeSignatureId, TypeSignatureId),
    TypeArgument(usize, String, Vec<ClassId>),
    Named(String, TypeDefId, Vec<TypeSignatureId>),
    Variant(String, Vec<TypeSignatureId>),
    Ref(TypeSignatureId),
    Wildcard,
    Never,
}
