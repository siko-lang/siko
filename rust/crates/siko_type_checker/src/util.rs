use crate::common::FunctionTypeInfo;
use crate::error::TypecheckError;
use siko_ir::program::Program;
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
