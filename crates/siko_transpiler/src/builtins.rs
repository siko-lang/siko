use crate::types::ir_type_to_rust_type;
use crate::util::get_ord_type_from_optional_ord;
use crate::util::Indent;
use siko_mir::function::Function;
use siko_mir::program::Program;
use siko_mir::types::Type;
use siko_util::to_first_uppercase;
use std::io::Result;
use std::io::Write;

fn generate_partial_cmp_builtin_body(
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
    result_ty: &Type,
    result_ty_str: &str,
) -> Result<()> {
    let ord_ty_str = get_ord_type_from_optional_ord(result_ty, program);
    write!(
        output_file,
        "{}partial_cmp_body!(arg0, arg1, {}, {})",
        indent, result_ty_str, ord_ty_str
    )?;
    Ok(())
}

fn generate_cmp_builtin_body(
    output_file: &mut dyn Write,
    _program: &Program,
    indent: &mut Indent,
    result_ty_str: &str,
) -> Result<()> {
    write!(
        output_file,
        "{}cmp_body!(arg0, arg1, {})",
        indent, result_ty_str
    )?;
    Ok(())
}

fn generate_show_builtin_body(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    result_ty_str: &str,
) -> Result<()> {
    write!(
        output_file,
        "{}let value = format!(\"{{}}\", arg0.value);\n",
        indent
    )?;
    write!(
        output_file,
        "{}{} {{ value : value }}",
        indent, result_ty_str
    )?;
    Ok(())
}

fn generate_opeq_builtin_body(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    result_ty_str: &str,
) -> Result<()> {
    write!(
        output_file,
        "{}let value = arg0.value == arg1.value;\n",
        indent
    )?;
    write!(
        output_file,
        "{} match value {{ true => {}::True, false => {}::False, }}",
        indent, result_ty_str, result_ty_str
    )?;
    Ok(())
}

fn generate_opdiv_builtin_body(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    result_ty_str: &str,
) -> Result<()> {
    write!(
        output_file,
        "{}let value = arg0.value / arg1.value;\n",
        indent
    )?;
    write!(
        output_file,
        "{}{} {{ value : value }}",
        indent, result_ty_str
    )?;
    Ok(())
}

fn generate_opmul_builtin_body(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    result_ty_str: &str,
) -> Result<()> {
    write!(
        output_file,
        "{}let value = arg0.value * arg1.value;\n",
        indent
    )?;
    write!(
        output_file,
        "{}{} {{ value : value }}",
        indent, result_ty_str
    )?;
    Ok(())
}

fn generate_opsub_builtin_body(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    result_ty_str: &str,
) -> Result<()> {
    write!(
        output_file,
        "{}let value = arg0.value - arg1.value;\n",
        indent
    )?;
    write!(
        output_file,
        "{}{} {{ value : value }}",
        indent, result_ty_str
    )?;
    Ok(())
}

fn generate_opadd_builtin_body(
    output_file: &mut dyn Write,
    indent: &mut Indent,
    result_ty_str: &str,
) -> Result<()> {
    write!(
        output_file,
        "{}let value = arg0.value + arg1.value;\n",
        indent
    )?;
    write!(
        output_file,
        "{}{} {{ value : value }}",
        indent, result_ty_str
    )?;
    Ok(())
}

fn generate_num_builtins(
    module: &str,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
    original_name: &str,
    result_ty: &Type,
    result_ty_str: &str,
) -> Result<()> {
    indent.inc();
    match original_name {
        "opAdd" => {
            generate_opadd_builtin_body(output_file, indent, result_ty_str)?;
        }
        "opSub" => {
            generate_opsub_builtin_body(output_file, indent, result_ty_str)?;
        }
        "opMul" => {
            generate_opmul_builtin_body(output_file, indent, result_ty_str)?;
        }
        "opDiv" => {
            generate_opdiv_builtin_body(output_file, indent, result_ty_str)?;
        }
        "opEq" => {
            generate_opeq_builtin_body(output_file, indent, result_ty_str)?;
        }
        "show" => {
            generate_show_builtin_body(output_file, indent, result_ty_str)?;
        }
        "partialCmp" => {
            generate_partial_cmp_builtin_body(
                output_file,
                program,
                indent,
                result_ty,
                result_ty_str,
            )?;
        }
        "cmp" => {
            generate_cmp_builtin_body(output_file, program, indent, result_ty_str)?;
        }
        _ => panic!("{}/{} not implemented", module, original_name),
    }
    indent.dec();
    Ok(())
}

fn generate_string_builtins(
    module: &str,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
    original_name: &str,
    result_ty: &Type,
    result_ty_str: &str,
) -> Result<()> {
    indent.inc();
    match original_name {
        "opAdd" => {
            write!(
                output_file,
                "{}let value = format!(\"{{}}{{}}\", arg0.value, arg1.value);\n",
                indent
            )?;
            write!(
                output_file,
                "{}{} {{ value : value }}",
                indent, result_ty_str
            )?;
        }
        "opEq" => {
            generate_opeq_builtin_body(output_file, indent, result_ty_str)?;
        }
        "partialCmp" => {
            generate_partial_cmp_builtin_body(
                output_file,
                program,
                indent,
                result_ty,
                result_ty_str,
            )?;
        }
        "cmp" => {
            generate_cmp_builtin_body(output_file, program, indent, result_ty_str)?;
        }
        _ => panic!("{}/{} not implemented", module, original_name),
    }
    indent.dec();
    Ok(())
}

fn generate_map_builtins(
    _function: &Function,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
    original_name: &str,
    result_ty: &Type,
    result_ty_str: &str,
) -> Result<()> {
    indent.inc();
    match original_name {
        "empty" => {
            write!(output_file, "{}map_empty!({})", indent, result_ty_str)?;
        }
        "insert" => {
            let result_id = result_ty.get_typedef_id();
            let tuple_record = program.typedefs.get(&result_id).get_record();
            let option_ty = ir_type_to_rust_type(&tuple_record.fields[1].ty, program);
            write!(
                output_file,
                "{}map_insert!(arg0, arg1, arg2, {}, {})",
                indent, option_ty, result_ty_str
            )?;
        }
        "remove" => {
            let result_id = result_ty.get_typedef_id();
            let tuple_record = program.typedefs.get(&result_id).get_record();
            let option_ty = ir_type_to_rust_type(&tuple_record.fields[1].ty, program);
            write!(
                output_file,
                "{}map_remove!(arg0, arg1, {}, {})",
                indent, option_ty, result_ty_str
            )?;
        }
        "get" => {
            write!(output_file, "{}map_get!(arg0, arg1, {})", indent, result_ty_str)?;
        }
        _ => panic!("Map/{} not implemented", original_name),
    }
    indent.dec();
    Ok(())
}

fn generate_list_builtins(
    _function: &Function,
    output_file: &mut dyn Write,
    _program: &Program,
    indent: &mut Indent,
    original_name: &str,
    result_ty_str: &str,
    arg_types: Vec<String>,
) -> Result<()> {
    indent.inc();
    match original_name {
        "show" => {
            write!(
                output_file,
                "{}let subs: Vec<_> = arg0.value.iter().map(|item| format!(\"{{}}\", item)).collect();\n",
                indent
            )?;
            write!(
                output_file,
                "{}{} {{ value : format!(\"[{{}}]\", subs.join(\", \")) }}",
                indent, result_ty_str
            )?;
        }
        "toList" => {
            write!(output_file, "{}let mut arg0 = arg0;\n", indent)?;
            write!(output_file, "{}let mut value = Vec::new();\n", indent)?;
            write!(output_file, "{}loop {{\n", indent)?;
            indent.inc();
            write!(
                output_file,
                "{}if let Some(v) = arg0.value.next() {{\n",
                indent
            )?;
            indent.inc();
            write!(output_file, "{}value.push(v);\n", indent)?;
            indent.dec();
            write!(output_file, "{}}} else {{\n", indent)?;
            indent.inc();
            write!(output_file, "{}break;\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            write!(
                output_file,
                "{}{} {{ value: value }}",
                indent, result_ty_str
            )?;
        }
        "iter" => {
            let list_type = &arg_types[0];
            let base_list_type: Vec<_> = list_type.split("::").collect();
            let list_iter_name = format!("{}_Iter", base_list_type[2]);
            let iter_trait_name = result_ty_str.replace(
                "crate::source::Iterator::",
                "crate::source::Iterator::Trait_",
            );

            write!(output_file, "{}#[derive(Clone)]\n", indent)?;
            write!(output_file, "{}pub struct {} {{\n", indent, list_iter_name)?;
            indent.inc();
            write!(
                output_file,
                "{}pub value: Vec<crate::source::Int::Int>,\n",
                indent
            )?;
            write!(output_file, "{}pub index: usize,\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;

            write!(
                output_file,
                "{}impl {} for {} {{\n",
                indent, iter_trait_name, list_iter_name
            )?;
            indent.inc();
            write!(
                output_file,
                "{}fn next(&mut self) -> Option<crate::source::Int::Int> {{\n",
                indent
            )?;
            indent.inc();
            write!(
                output_file,
                "{}if self.index >= self.value.len() {{\n",
                indent
            )?;
            indent.inc();
            write!(output_file, "{}None\n", indent)?;
            indent.dec();
            write!(output_file, "{}}} else {{\n", indent)?;
            indent.inc();
            write!(
                output_file,
                "{}let v = self.value[self.index].clone();\n",
                indent
            )?;
            write!(output_file, "{}self.index += 1;\n", indent)?;
            write!(output_file, "{}Some(v)\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;

            write!(
                output_file,
                "{}fn box_clone(&self) -> Box<dyn {}> {{\n",
                indent, iter_trait_name
            )?;
            indent.inc();
            write!(output_file, "{}Box::new(self.clone())\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;

            write!(output_file, "{}{} {{\n", indent, result_ty_str)?;
            indent.inc();
            write!(
                output_file,
                "{}value: Box::new({} {{\n",
                indent, list_iter_name
            )?;
            indent.inc();
            write!(output_file, "{}value: arg0.value.clone(),\n", indent)?;
            write!(output_file, "{}index: 0,\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}),\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
        }
        "opEq" => {
            write!(output_file, "{}if arg0.value.eq(&arg1.value) {{\n", indent)?;
            indent.inc();
            write!(output_file, "{}{}::True\n", indent, result_ty_str)?;
            indent.dec();
            write!(output_file, "{}}} else {{\n", indent)?;
            indent.inc();
            write!(output_file, "{}{}::False\n", indent, result_ty_str)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
        }
        _ => panic!("List/{} not implemented", original_name),
    }
    indent.dec();
    Ok(())
}

pub fn generate_builtin(
    function: &Function,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
    original_name: &str,
    result_ty: &Type,
    result_ty_str: &str,
    arg_types: Vec<String>,
) -> Result<()> {
    match function.module.as_ref() {
        "Int" => {
            return generate_num_builtins(
                function.module.as_ref(),
                output_file,
                program,
                indent,
                original_name,
                result_ty,
                result_ty_str,
            );
        }
        "String" => {
            return generate_string_builtins(
                function.module.as_ref(),
                output_file,
                program,
                indent,
                original_name,
                result_ty,
                result_ty_str,
            );
        }
        "Float" => {
            return generate_num_builtins(
                function.module.as_ref(),
                output_file,
                program,
                indent,
                original_name,
                result_ty,
                result_ty_str,
            );
        }
        "Map" => {
            return generate_map_builtins(
                function,
                output_file,
                program,
                indent,
                original_name,
                result_ty,
                result_ty_str,
            );
        }
        "List" => {
            return generate_list_builtins(
                function,
                output_file,
                program,
                indent,
                original_name,
                result_ty_str,
                arg_types,
            );
        }
        _ => {
            indent.inc();
            match (function.module.as_ref(), original_name) {
                ("Std.Ops", "opAnd") => {
                    write!(
                        output_file,
                        "{} match (arg0, arg1) {{ ({}::True, {}::True) => {}::True,",
                        indent, result_ty_str, result_ty_str, result_ty_str,
                    )?;
                    write!(
                        output_file,
                        "({}::True, {}::False) => {}::False,",
                        result_ty_str, result_ty_str, result_ty_str,
                    )?;
                    write!(
                        output_file,
                        "({}::False, {}::True) => {}::False,",
                        result_ty_str, result_ty_str, result_ty_str,
                    )?;
                    write!(
                        output_file,
                        "({}::False, {}::False) => {}::False, }}",
                        result_ty_str, result_ty_str, result_ty_str,
                    )?;
                }
                ("Std.Ops", "opOr") => {
                    write!(
                        output_file,
                        "{} match (arg0, arg1) {{ ({}::True, {}::True) => {}::True,",
                        indent, result_ty_str, result_ty_str, result_ty_str,
                    )?;
                    write!(
                        output_file,
                        "({}::True, {}::False) => {}::True,",
                        result_ty_str, result_ty_str, result_ty_str,
                    )?;
                    write!(
                        output_file,
                        "({}::False, {}::True) => {}::True,",
                        result_ty_str, result_ty_str, result_ty_str,
                    )?;
                    write!(
                        output_file,
                        "({}::False, {}::False) => {}::False, }}",
                        result_ty_str, result_ty_str, result_ty_str,
                    )?;
                }
                ("Std.Util.Basic", "println") => {
                    write!(output_file, "{}println!(\"{{}}\", arg0);\n", indent)?;
                    write!(output_file, "{}{} {{ }}", indent, result_ty_str)?;
                }
                ("Std.Util.Basic", "print") => {
                    write!(output_file, "{}print!(\"{{}}\", arg0);\n", indent)?;
                    write!(output_file, "{}{} {{ }}", indent, result_ty_str)?;
                }
                ("Std.Util", "assert") => {
                    let panic = "{{ panic!(\"Assertion failed\"); }}";
                    write!(
                        output_file,
                        "{} match arg0 {{ {}::True => {{}}, {}::False => {} }}",
                        indent, arg_types[0], arg_types[0], panic
                    )?;
                    write!(output_file, "{}{} {{ }}", indent, result_ty_str)?;
                }
                ("Iterator", "map") => {
                    let struct_name = to_first_uppercase(function.name.clone());
                    write!(output_file, "{}struct {} {{\n", indent, struct_name)?;
                    indent.inc();
                    for (index, arg_type) in arg_types.iter().enumerate() {
                        write!(output_file, "{}pub arg{}: {},\n", indent, index, arg_type)?;
                    }
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;

                    write!(output_file, "{}impl Clone for {} {{\n", indent, struct_name)?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}fn clone(&self) -> {} {{\n",
                        indent, struct_name
                    )?;
                    indent.inc();
                    write!(output_file, "{}{} {{\n", indent, struct_name)?;
                    indent.inc();
                    write!(output_file, "{}arg0: self.arg0.clone(),\n", indent,)?;
                    write!(output_file, "{}arg1: self.arg1.clone(),\n", indent,)?;
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;

                    let iter_id = result_ty.get_typedef_id();
                    let iter = program.typedefs.get(&iter_id).get_record();
                    let iter_trait_name = format!("Trait_{}", iter.name);

                    write!(
                        output_file,
                        "{}impl {} for {} {{\n",
                        indent, iter_trait_name, struct_name
                    )?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}fn next(&mut self) -> Option<crate::source::Int::Int> {{\n",
                        indent
                    )?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}if let Some(value) = self.arg1.value.next() {{\n",
                        indent
                    )?;
                    indent.inc();
                    write!(output_file, "{}Some(self.arg0.call(value))\n", indent)?;
                    indent.dec();
                    write!(output_file, "{}}} else {{\n", indent)?;
                    indent.inc();
                    write!(output_file, "{}None\n", indent)?;
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;
                    write!(
                        output_file,
                        "{}fn box_clone(&self) -> Box<dyn {}> {{\n",
                        indent, iter_trait_name
                    )?;
                    indent.inc();
                    write!(output_file, "{}Box::new(self.clone())\n", indent)?;
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;

                    write!(output_file, "{}{} {{\n", indent, result_ty_str)?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}value: Box::new({} {{\n",
                        indent, struct_name
                    )?;
                    indent.inc();
                    write!(output_file, "{}arg0: arg0,\n", indent,)?;
                    write!(output_file, "{}arg1: arg1,\n", indent,)?;
                    indent.dec();
                    write!(output_file, "{}}}),\n", indent,)?;
                    indent.dec();
                    write!(output_file, "{}}}", indent)?;
                }
                ("Iterator", "filter") => {
                    write!(output_file, "{}{} }}\n", indent, result_ty_str)?;
                }
                _ => panic!("{}/{} not implemented", function.module, function.name),
            }
        }
    }
    indent.dec();
    Ok(())
}
