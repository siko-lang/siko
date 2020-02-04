#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct TypeSignatureId {
    pub id: usize,
}

impl From<usize> for TypeSignatureId {
    fn from(id: usize) -> TypeSignatureId {
        TypeSignatureId { id: id }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TypeSignature {
    TypeArg(String),
    Named(String, Vec<TypeSignatureId>),
    Variant(String, Vec<TypeSignatureId>),
    Tuple(Vec<TypeSignatureId>),
    Function(TypeSignatureId, TypeSignatureId),
    Ref(TypeSignatureId),
    Wildcard,
}
