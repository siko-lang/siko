use crate::types::ir_type_to_rust_type;
use siko_mir::program::Program;
use siko_mir::types::Type;
use std::fmt;

pub struct Indent {
    indent: usize,
}

impl Indent {
    pub fn new() -> Indent {
        Indent { indent: 0 }
    }

    pub fn inc(&mut self) {
        self.indent += 4;
    }

    pub fn dec(&mut self) {
        self.indent -= 4;
    }
}

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.indent {
            write!(f, " ")?
        }
        Ok(())
    }
}

pub fn get_module_name(name: &str) -> String {
    name.replace(".", "_")
}

pub fn arg_name(index: usize) -> String {
    format!("arg{}", index)
}

pub fn get_ord_type_from_optional_ord(ty: &Type, program: &Program) -> String {
    let id = ty.get_typedef_id();
    let adt_opt = program.typedefs.get(&id).get_adt();
    let mut ord_ty = None;
    for v in &adt_opt.variants {
        if v.name == "Some" {
            ord_ty = Some(v.items[0].ty.clone());
        }
    }
    let ord_ty = ord_ty.expect("Ord ty not found");
    let ord_ty_str = ir_type_to_rust_type(&ord_ty, program);
    ord_ty_str
}
