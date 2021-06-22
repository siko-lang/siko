mod mir;
mod mir_loader;
mod scc;
mod groups;
mod init_data;

use mir::*;
use mir_loader::*;
use groups::*;
use init_data::*;

fn check_type(ty: &str, mir_program: &Program) -> Type {
    if let Some(_) = mir_program.adts.get(ty) {
        Type::Adt(ty.to_string())
    } else {
        if let Some(_) = mir_program.records.get(ty) {
            Type::Record(ty.to_string())
        } else {
            if ty == "!" {
                Type::Never
            } else {
                panic!("{} is not an adt not a record!", ty);
            }
        }
    }
}

fn check_types(mir_program: &Program) {
    for (_, f) in &mir_program.functions {
        for arg in &f.args {
            check_type(&arg, &mir_program);
        }
        check_type(&f.result, &mir_program);
        match &f.kind {
            FunctionKind::Normal(exprs) => {
                for e in exprs {
                    check_type(&e.ty, &mir_program);
                }
            }
            _ => {}
        }
    }
}

fn process_program(mir_program: Program) {
    check_types(&mir_program);
    let data_groups = collect_data_groups(&mir_program);
    let function_groups = collect_function_groups(&mir_program);
    println!(
        "data_groups: {}, function_groups: {}",
        data_groups.len(),
        function_groups.len()
    );
    init_data(&mir_program, &data_groups);
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    for arg in args {
        println!("Parsing json from {}", arg);
        match load_mir(arg) {
            Ok(mir_program) => {
                println!("MIR loaded");
                process_program(mir_program);
            }
            Err(e) => {
                println!("Failed to parse {:?}", e);
            }
        };
    }
}
