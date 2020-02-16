use crate::backend_passes::box_convert::box_convert_pass;
use crate::backend_passes::convert_args_to_closures::convert_args_to_closures;
use crate::backend_passes::insert_clone::insert_clone_pass;
use crate::backend_passes::process_data_types::process_data_types;
use crate::backend_passes::process_static_calls::process_static_calls_pass;
use siko_mir::function::FunctionInfo;
use siko_mir::program::Program;

pub fn run_passes(program: &mut Program) {
    process_data_types(program);
    let mut bodies = Vec::new();
    for (_, function) in program.functions.items.iter() {
        if let FunctionInfo::Normal(body) = function.info {
            bodies.push(body);
        }
    }
    for body in &bodies {
        box_convert_pass(body, program);
    }
    for body in &bodies {
        process_static_calls_pass(body, program);
    }
    for body in &bodies {
        insert_clone_pass(body, program);
    }
    convert_args_to_closures(program);
}
