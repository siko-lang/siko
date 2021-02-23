use crate::types::ir_type_to_rust_type;
use crate::util::arg_name;
use crate::util::get_module_name;
use crate::util::Indent;
use siko_constants::MIR_FUNCTION_TRAIT_NAME;
use siko_mir::program::Program;
use siko_mir::types::Closure;
use siko_mir::types::DynamicCallTrait;
use siko_mir::types::PartialFunctionCall;
use siko_mir::types::Type;
use std::io::Result;
use std::io::Write;

fn write_partial_function_call_def(
    output_file: &mut dyn Write,
    partial_function_call: &PartialFunctionCall,
    indent: &mut Indent,
    program: &Program,
) -> Result<()> {
    write!(
        output_file,
        "{}pub struct {} {{\n",
        indent,
        partial_function_call.get_name()
    )?;
    indent.inc();
    for (index, field) in partial_function_call.fields.iter().enumerate() {
        write!(
            output_file,
            "{} pub {}: Option<{}>,\n",
            indent,
            arg_name(index),
            ir_type_to_rust_type(&field.ty, program)
        )?;
    }
    indent.dec();
    write!(output_file, "{}}}\n", indent)?;
    Ok(())
}

fn write_partial_function_call_clone_impl(
    output_file: &mut dyn Write,
    partial_function_call: &PartialFunctionCall,
    indent: &mut Indent,
) -> Result<()> {
    write!(
        output_file,
        "{}impl Clone for {} {{\n",
        indent,
        partial_function_call.get_name()
    )?;
    indent.inc();
    write!(
        output_file,
        "{}fn clone(&self) -> {} {{\n",
        indent,
        partial_function_call.get_name()
    )?;
    indent.inc();
    write!(
        output_file,
        "{}{} {{\n",
        indent,
        partial_function_call.get_name()
    )?;
    indent.inc();
    for index in 0..partial_function_call.fields.len() {
        write!(
            output_file,
            "{}{}: self.{}.clone(),\n",
            indent,
            arg_name(index),
            arg_name(index),
        )?;
    }
    indent.dec();
    write!(output_file, "{}}}\n", indent)?;
    indent.dec();
    write!(output_file, "{}}}\n", indent)?;
    indent.dec();
    write!(output_file, "{}}}\n", indent)?;
    Ok(())
}

fn write_dyn_trait_impl_save_arg(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    partial_function_call: &PartialFunctionCall,
    from: &Type,
    to: &Type,
    field_index: usize,
    program: &Program,
) -> Result<()> {
    write!(
        output_file,
        "{}impl {}<{}, {}> for {} {{\n",
        indent,
        MIR_FUNCTION_TRAIT_NAME,
        ir_type_to_rust_type(from, program),
        ir_type_to_rust_type(to, program),
        partial_function_call.get_name()
    )?;
    {
        indent.inc();
        write!(
            output_file,
            "{}fn call(&mut self, arg0: {}) -> {} {{\n",
            indent,
            ir_type_to_rust_type(from, program),
            ir_type_to_rust_type(to, program),
        )?;
        {
            indent.inc();
            write!(
                output_file,
                "{}let value = {} {{\n",
                indent,
                partial_function_call.get_name(),
            )?;
            {
                indent.inc();
                for index in 0..partial_function_call.fields.len() {
                    if index == field_index {
                        write!(
                            output_file,
                            "{}{}: Some(arg0),\n",
                            indent,
                            arg_name(field_index)
                        )?;
                    } else {
                        write!(
                            output_file,
                            "{}{}: self.{}.take(),\n",
                            indent,
                            arg_name(index),
                            arg_name(index)
                        )?;
                    }
                }
                indent.dec();
            }
            write!(output_file, "{}}};\n", indent,)?;
            write!(
                output_file,
                "{}{} {{ value : Box::new(value) }}\n",
                indent,
                ir_type_to_rust_type(to, program),
            )?;
            indent.dec();
        }
        write!(output_file, "{}}}\n", indent)?;

        write!(
            output_file,
            "{}fn call_ro(&self, arg0: {}) -> {} {{\n",
            indent,
            ir_type_to_rust_type(from, program),
            ir_type_to_rust_type(to, program),
        )?;
        {
            indent.inc();
            write!(
                output_file,
                "{}let value = {} {{\n",
                indent,
                partial_function_call.get_name(),
            )?;
            {
                indent.inc();
                for index in 0..partial_function_call.fields.len() {
                    if index == field_index {
                        write!(
                            output_file,
                            "{}{}: Some(arg0),\n",
                            indent,
                            arg_name(field_index)
                        )?;
                    } else {
                        write!(
                            output_file,
                            "{}{}: self.{}.clone().take(),\n",
                            indent,
                            arg_name(index),
                            arg_name(index)
                        )?;
                    }
                }
                indent.dec();
            }
            write!(output_file, "{}}};\n", indent,)?;
            write!(
                output_file,
                "{}{} {{ value : Box::new(value) }}\n",
                indent,
                ir_type_to_rust_type(to, program),
            )?;
            indent.dec();
        }
        write!(output_file, "{}}}\n", indent)?;
        write!(
            output_file,
            "{}fn box_clone(&self) -> Box<dyn {}<{},{}>> {{\n",
            indent,
            MIR_FUNCTION_TRAIT_NAME,
            ir_type_to_rust_type(from, program),
            ir_type_to_rust_type(to, program),
        )?;
        {
            indent.inc();
            write!(output_file, "{}Box::new(self.clone())\n", indent)?;
            indent.dec();
        }
        write!(output_file, "{}}}\n", indent)?;
        indent.dec();
    }
    write!(output_file, "{}}}\n", indent)?;
    Ok(())
}

fn write_dyn_trait_impl_real_call(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    partial_function_call: &PartialFunctionCall,
    from: &Type,
    to: &Type,
    program: &Program,
) -> Result<()> {
    write!(
        output_file,
        "{}impl {}<{}, {}> for {} {{\n",
        indent,
        MIR_FUNCTION_TRAIT_NAME,
        ir_type_to_rust_type(from, program),
        ir_type_to_rust_type(to, program),
        partial_function_call.get_name()
    )?;
    {
        indent.inc();
        write!(
            output_file,
            "{}fn call(&mut self, arg0: {}) -> {} {{\n",
            indent,
            ir_type_to_rust_type(from, program),
            ir_type_to_rust_type(to, program),
        )?;
        {
            indent.inc();
            let function = program.functions.get(&partial_function_call.function);
            write!(
                output_file,
                "{}crate::source::{}::{}(",
                indent,
                get_module_name(&function.module),
                function.name
            )?;
            for index in 0..partial_function_call.fields.len() {
                write!(
                    output_file,
                    "self.{}.take().expect(\"Missing arg\"), ",
                    arg_name(index)
                )?;
            }
            write!(output_file, "arg0)\n")?;
            indent.dec();
        }
        write!(output_file, "{}}}\n", indent)?;
        write!(
            output_file,
            "{}fn call_ro(&self, arg0: {}) -> {} {{\n",
            indent,
            ir_type_to_rust_type(from, program),
            ir_type_to_rust_type(to, program),
        )?;
        {
            indent.inc();
            let function = program.functions.get(&partial_function_call.function);
            write!(
                output_file,
                "{}crate::source::{}::{}(",
                indent,
                get_module_name(&function.module),
                function.name
            )?;
            for index in 0..partial_function_call.fields.len() {
                write!(
                    output_file,
                    "self.{}.clone().take().expect(\"Missing arg\"), ",
                    arg_name(index)
                )?;
            }
            write!(output_file, "arg0)\n")?;
            indent.dec();
        }
        write!(output_file, "{}}}\n", indent)?;
        write!(
            output_file,
            "{}fn box_clone(&self) -> Box<dyn {}<{},{}>> {{\n",
            indent,
            MIR_FUNCTION_TRAIT_NAME,
            ir_type_to_rust_type(from, program),
            ir_type_to_rust_type(to, program),
        )?;
        {
            indent.inc();
            write!(output_file, "{}Box::new(self.clone())\n", indent)?;
            indent.dec();
        }
        write!(output_file, "{}}}\n", indent)?;
        indent.dec();
    }
    write!(output_file, "{}}}\n", indent)?;
    Ok(())
}

fn write_function_trait(output_file: &mut dyn Write, indent: &mut Indent) -> Result<()> {
    write!(
        output_file,
        "{}pub trait {}<A, B> {{\n",
        indent, MIR_FUNCTION_TRAIT_NAME
    )?;
    indent.inc();
    write!(output_file, "{}fn call(&mut self, a: A) -> B;\n", indent)?;
    write!(output_file, "{}fn call_ro(&self, a: A) -> B;\n", indent)?;
    write!(
        output_file,
        "{}fn box_clone(&self) -> Box<dyn Function<A,B>>;\n",
        indent
    )?;
    indent.dec();
    write!(output_file, "{}}}\n", indent)?;
    Ok(())
}

fn write_closure_def(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    closure: &Closure,
    program: &Program,
) -> Result<()> {
    write!(output_file, "{}pub struct {} {{\n", indent, closure.name)?;
    let fn_name = ir_type_to_rust_type(&closure.ty, program);
    indent.inc();
    write!(output_file, "{}pub value: {}\n", indent, fn_name,)?;
    indent.dec();
    write!(output_file, "{}}}\n", indent)?;
    Ok(())
}

fn write_closure_clone_impl(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    closure: &Closure,
) -> Result<()> {
    write!(
        output_file,
        "{}impl Clone for {} {{\n",
        indent, closure.name
    )?;
    indent.inc();
    write!(
        output_file,
        "{}fn clone(&self) -> {} {{\n",
        indent, closure.name
    )?;
    indent.inc();
    write!(output_file, "{}{} {{\n", indent, closure.name)?;
    indent.inc();
    write!(output_file, "{}value: self.value.box_clone(),\n", indent)?;
    indent.dec();
    write!(output_file, "{}}}\n", indent)?;
    indent.dec();
    write!(output_file, "{}}}\n", indent)?;
    indent.dec();
    write!(output_file, "{}}}\n", indent)?;
    Ok(())
}

fn write_closure_impl(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    closure: &Closure,
    program: &Program,
) -> Result<()> {
    write!(output_file, "{}impl {} {{\n", indent, closure.name)?;
    indent.inc();
    {
        write!(
            output_file,
            "{}pub fn call(&mut self, arg0: {}) -> {} {{\n",
            indent,
            ir_type_to_rust_type(&closure.from_ty, program),
            ir_type_to_rust_type(&closure.to_ty, program)
        )?;
        {
            indent.inc();
            write!(output_file, "{}self.value.call(arg0)\n", indent)?;
            indent.dec();
        }
        write!(output_file, "{}}}\n", indent)?;
        write!(
            output_file,
            "{}pub fn call_ro(&self, arg0: {}) -> {} {{\n",
            indent,
            ir_type_to_rust_type(&closure.from_ty, program),
            ir_type_to_rust_type(&closure.to_ty, program)
        )?;
        {
            indent.inc();
            write!(output_file, "{}self.value.call_ro(arg0)\n", indent)?;
            indent.dec();
        }
        write!(output_file, "{}}}\n", indent)?;
        write!(output_file, "{}}}\n", indent)?;
        indent.dec();
    }
    Ok(())
}

pub fn write_internal_defs(
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
) -> Result<()> {
    for (_, partial_function_call) in program.partial_function_calls.items.iter() {
        write_partial_function_call_def(output_file, partial_function_call, indent, program)?;
        write_partial_function_call_clone_impl(output_file, partial_function_call, indent)?;

        for dyn_trait in &partial_function_call.traits {
            match dyn_trait {
                DynamicCallTrait::ArgSave {
                    from,
                    to,
                    field_index,
                } => {
                    write_dyn_trait_impl_save_arg(
                        output_file,
                        indent,
                        partial_function_call,
                        from,
                        to,
                        *field_index,
                        program,
                    )?;
                }
                DynamicCallTrait::RealCall { from, to } => {
                    write_dyn_trait_impl_real_call(
                        output_file,
                        indent,
                        partial_function_call,
                        from,
                        to,
                        program,
                    )?;
                }
            }
        }
    }

    write_function_trait(output_file, indent)?;

    for (_, closure) in &program.closures {
        write_closure_def(output_file, indent, closure, program)?;
        write_closure_impl(output_file, indent, closure, program)?;
        write_closure_clone_impl(output_file, indent, closure)?;
    }
    Ok(())
}
