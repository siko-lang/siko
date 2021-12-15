use crate::module::Module;
use crate::util::Indent;
use siko_constants::MIR_INTERNAL_MODULE_NAME;
use siko_mir::data::TypeDef;
use siko_mir::program::Program;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Result;
use std::io::Write;

struct RustProgram {
    modules: BTreeMap<String, Module>,
}

impl RustProgram {
    fn new() -> RustProgram {
        RustProgram {
            modules: BTreeMap::new(),
        }
    }

    fn get_module(&mut self, module_name: String) -> &mut Module {
        let module = self
            .modules
            .entry(module_name.clone())
            .or_insert_with(|| Module::new(module_name.clone()));
        module
    }

    fn write(&self, output_file: &mut dyn Write, program: &Program) -> Result<()> {
        let mut indent = Indent::new();
        for (_, module) in &self.modules {
            if !module.internal {
                module.write(output_file, program, &mut indent)?;
            }
        }
        for (_, module) in &self.modules {
            if module.internal {
                module.write(output_file, program, &mut indent)?;
            }
        }
        Ok(())
    }
}

pub struct Transpiler {}

impl Transpiler {
    pub fn process(program: &Program, target_file: &str) -> Result<()> {
        let filename = format!("{}", target_file);
        let mut output_file = File::create(filename)?;
        write!(output_file, "#![allow(non_snake_case)]\n")?;
        write!(output_file, "#![allow(non_camel_case_types)]\n")?;
        write!(output_file, "#![allow(unused_variables)]\n")?;
        write!(output_file, "#![allow(dead_code)]\n")?;
        write!(output_file, "#![allow(unused_parens)]\n\n")?;
        write!(output_file, "#![allow(unused_braces)]\n\n")?;
        write!(output_file, "#![allow(unused_macros)]\n\n")?;
        write!(output_file, "#![allow(redundant_semicolons)]\n\n")?;
        write!(output_file, "#![allow(unreachable_code)]\n\n")?;
        write!(output_file, "#![allow(non_shorthand_field_patterns)]\n\n")?;
        write!(output_file, "#![allow(unused_mut)]\n\n")?;
        write!(output_file, "#![allow(unused_assignments)]\n\n")?;
        write!(output_file, "#![allow(unreachable_patterns)]\n\n")?;
        let mut rust_program = RustProgram::new();
        rust_program.get_module(MIR_INTERNAL_MODULE_NAME.to_string());
        for (id, function) in program.functions.items.iter() {
            let module = rust_program.get_module(function.module.clone());
            module.functions.insert(*id);
        }
        for (id, typedef) in program.typedefs.items.iter() {
            match typedef {
                TypeDef::Adt(adt) => {
                    let module = rust_program.get_module(adt.module.clone());
                    module.typedefs.insert(*id);
                }
                TypeDef::Record(record) => {
                    let module = rust_program.get_module(record.module.clone());
                    module.typedefs.insert(*id);
                }
            }
        }
        rust_program.write(&mut output_file, program)?;
        Ok(())
    }
}
