use crate::class::ClassId;
use crate::data::TypeDefId;
use crate::program::Program;
use crate::type_var_generator::TypeVarGenerator;
use crate::types::Type;

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

pub fn process_type_signature(
    type_signature_id: TypeSignatureId,
    program: &Program,
    type_var_generator: &mut TypeVarGenerator,
) -> Type {
    let type_signature = &program.type_signatures.get(&type_signature_id).item;
    match type_signature {
        TypeSignature::Function(from, to) => {
            let from_ty = process_type_signature(*from, program, type_var_generator);
            let to_ty = process_type_signature(*to, program, type_var_generator);
            Type::Function(Box::new(from_ty), Box::new(to_ty))
        }
        TypeSignature::Named(name, id, items) => {
            let items: Vec<_> = items
                .iter()
                .map(|item| process_type_signature(*item, program, type_var_generator))
                .collect();
            Type::Named(name.clone(), *id, items)
        }
        TypeSignature::Tuple(items) => {
            let items: Vec<_> = items
                .iter()
                .map(|item| process_type_signature(*item, program, type_var_generator))
                .collect();
            Type::Tuple(items)
        }
        TypeSignature::TypeArgument(index, name, constraints) => {
            let mut constraints = constraints.clone();
            // unifier assumes that the constraints are sorted!
            constraints.sort();
            Type::FixedTypeArg(name.clone(), *index, constraints)
        }
        TypeSignature::Variant(..) => panic!("Variant should not appear here"),
        TypeSignature::Wildcard => type_var_generator.get_new_type_var(),
        TypeSignature::Never => Type::Never(type_var_generator.get_new_index()),
        TypeSignature::Ref(item) => {
            let ty = process_type_signature(*item, program, type_var_generator);
            Type::Ref(Box::new(ty))
        }
    }
}
