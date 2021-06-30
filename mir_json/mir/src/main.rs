mod groups;
mod inference;
mod init_data;
mod mir;
mod mir_loader;
mod scc;

use groups::*;
use inference::*;
use init_data::*;
use mir::*;
use mir_loader::*;

fn check_type(ty: &str, mir_program: &Program) -> Type {
    if ty == "!" {
        Type::Never
    } else {
        match mir_program.data.get(ty).unwrap() {
            Data::Adt(_) => Type::Adt(ty.to_string()),
            Data::Record(_) => Type::Record(ty.to_string()),
        }
    }
}

fn check_types(mir_program: &Program) {
    for (_, f) in &mir_program.functions {
        for arg in &f.args {
            check_type(&arg.ty, &mir_program);
        }
        check_type(&f.result.ty, &mir_program);
        match &f.kind {
            FunctionKind::Normal(exprs) => {
                for e in exprs {
                    check_type(&e.ty.ty, &mir_program);
                }
            }
            _ => {}
        }
    }
}

fn process_program(mut mir_program: Program) -> Program {
    check_types(&mir_program);
    let data_groups = collect_data_groups(&mir_program);
    let function_groups = collect_function_groups(&mir_program);
    println!(
        "data_groups: {}, function_groups: {}",
        data_groups.len(),
        function_groups.len()
    );
    let data_arg_counts = init_data(&mut mir_program, &data_groups);
    inference(function_groups, &mir_program);
    /*
    for (_, f) in &mut mir_program.functions {
        fill_type(&mut f.result, &data_arg_counts);
        for arg in &mut f.args {
            fill_type(arg, &data_arg_counts);
        }
        match &mut f.kind {
            FunctionKind::Normal(exprs) => for e in exprs.iter_mut() {},
            _ => {}
        }
    }
    */
    mir_program
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    for arg in args {
        println!("Parsing json from {}", arg);
        match load_mir(arg) {
            Ok(mir_program) => {
                println!("MIR loaded");
                let mir_program = process_program(mir_program);
                println!("Done!");
                //std::thread::sleep(std::time::Duration::from_secs(60));
            }
            Err(e) => {
                println!("Failed to parse {:?}", e);
            }
        };
    }
}
