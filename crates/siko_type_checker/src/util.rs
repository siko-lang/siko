use crate::common::FunctionTypeInfo;
use crate::error::TypecheckError;
use siko_ir::program::Program;
use siko_ir::type_signature::TypeSignature;
use siko_ir::type_signature::TypeSignatureId;
use siko_ir::type_var_generator::TypeVarGenerator;
use siko_ir::types::ResolverContext;
use siko_ir::types::Type;
use siko_location_info::location_id::LocationId;
use siko_util::format_list;

pub fn create_general_function_type(
    func_args: &mut Vec<Type>,
    arg_count: usize,
    type_var_generator: &mut TypeVarGenerator,
) -> (Type, Type) {
    if arg_count > 0 {
        let from_ty = type_var_generator.get_new_type_var();
        func_args.push(from_ty.clone());
        let (to_ty, result) =
            create_general_function_type(func_args, arg_count - 1, type_var_generator);
        let func_ty = Type::Function(Box::new(from_ty), Box::new(to_ty));
        (func_ty, result)
    } else {
        let v = type_var_generator.get_new_type_var();
        (v.clone(), v)
    }
}

pub fn create_general_function_type_info(
    arg_count: usize,
    type_var_generator: &mut TypeVarGenerator,
) -> FunctionTypeInfo {
    let mut func_args = Vec::new();
    let (function_type, result_type) =
        create_general_function_type(&mut func_args, arg_count, type_var_generator);
    FunctionTypeInfo {
        displayed_name: format!("<general>"),
        args: func_args,
        typed: false,
        result: result_type,
        function_type: function_type,
        body: None,
    }
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

pub fn function_argument_mismatch(
    program: &Program,
    func_type: &Type,
    args: Vec<Type>,
    location: LocationId,
    errors: &mut Vec<TypecheckError>,
) {
    let mut context = ResolverContext::new(program);
    let function_type_string = func_type.get_resolved_type_string_with_context(&mut context);
    let arg_type_strings: Vec<_> = args
        .iter()
        .map(|arg| arg.get_resolved_type_string_with_context(&mut context))
        .collect();
    let arguments = format_list(&arg_type_strings[..]);
    let err = TypecheckError::FunctionArgumentMismatch(location, arguments, function_type_string);
    errors.push(err);
}
