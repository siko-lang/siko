use crate::type_processor::process_stored_type;
use siko_mir::program::Program;
use siko_mir::types::Type;

pub fn convert_args_to_closures(program: &mut Program) {
    for (id, function) in program.functions.items.clone() {
        if function.arg_count == 0 {
            let new_type = process_stored_type(function.function_type.clone(), program);
            program.functions.get_mut(&id).function_type = new_type;
        } else {
            if let Type::Function(from, to) = &function.function_type {
                let from = process_stored_type(*from.clone(), program);
                let to = process_stored_type(*to.clone(), program);
                let new_type = Type::Function(Box::new(from), Box::new(to));
                program.functions.get_mut(&id).function_type = new_type;
            }
        }
    }
}
