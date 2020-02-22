use crate::util::get_module_name;
use siko_constants::MIR_FUNCTION_TRAIT_NAME;
use siko_constants::MIR_INTERNAL_MODULE_NAME;
use siko_mir::data::TypeDef;
use siko_mir::program::Program;
use siko_mir::types::Modifier;
use siko_mir::types::Type;

pub fn ir_type_to_rust_type(ty: &Type, program: &Program) -> String {
    match ty {
        Type::Function(from, to) => {
            let from = ir_type_to_rust_type(from, program);
            let to = ir_type_to_rust_type(to, program);
            format!(
                "Box<dyn crate::{}::{}<{}, {}>>",
                MIR_INTERNAL_MODULE_NAME, MIR_FUNCTION_TRAIT_NAME, from, to
            )
        }
        Type::Named(m, id) => {
            let typedef = program.typedefs.get(id);
            let (module_name, name) = match typedef {
                TypeDef::Adt(adt) => (get_module_name(&adt.module), adt.name.clone()),
                TypeDef::Record(record) => (get_module_name(&record.module), record.name.clone()),
            };
            match m {
                Modifier::Ref => format!("&crate::{}::{}", module_name, name),
                Modifier::Boxed => format!("Box<crate::{}::{}>", module_name, name),
                Modifier::Owned => format!("crate::{}::{}", module_name, name),
                Modifier::Var(_) => format!("crate::{}::{}", module_name, name), // TODO
            }
        }
        Type::Closure(ty) => {
            let closure = program.get_closure_type(ty);
            format!(
                "crate::{}::{}",
                MIR_INTERNAL_MODULE_NAME,
                closure.get_name()
            )
        }
        Type::Never => format!("!"),
    }
}
