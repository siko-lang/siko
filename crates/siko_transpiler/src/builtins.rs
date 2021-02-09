use crate::types::ir_type_to_rust_type;
use crate::util::get_ord_type_from_optional_ord;
use crate::util::Indent;
use siko_mir::data::RecordKind;
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
        "{}{} {{ value : std::rc::Rc::new(value) }}",
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
        "parse" => {
            write!(
                output_file,
                "{}match arg0.value.parse::<i64>() {{ Ok(v) => {{ {}::Some ( crate::source::Int::Int {{ value : v }} ) }}, Err(_) => {{ {}::None }} }}\n",
                indent, result_ty_str, result_ty_str
            )?;
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
    arg_types: Vec<String>,
) -> Result<()> {
    indent.inc();
    match original_name {
        "opAdd" => {
            write!(
                output_file,
                "{}let value = format!(\"{{}}{{}}\", *arg0.value, *arg1.value);\n",
                indent
            )?;
            write!(
                output_file,
                "{}{} {{ value : std::rc::Rc::new(value) }}",
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
        "split" => {
            write!(
                output_file,
                "{}let value: Vec<_> = arg0.value.split(&*arg1.value).map(|s| std::rc::Rc::new({} {{ value : std::rc::Rc::new(s.to_string()) }} )).collect();\n",
                indent, arg_types[0]
            )?;
            write!(
                output_file,
                "{}{} {{ value : std::rc::Rc::new(value) }}",
                indent, result_ty_str
            )?;
        }
        "replace" => {
            write!(
                output_file,
                "{}let value = arg0.value.replace(&*arg1.value, &*arg2.value);\n",
                indent
            )?;
            write!(
                output_file,
                "{}{} {{ value : std::rc::Rc::new(value) }}",
                indent, result_ty_str
            )?;
        }
        "chars" => {
            write!(
                output_file,
                "{}let value: Vec<_> = arg0.value.chars().map(|c| std::rc::Rc::new(crate::source::Char::Char {{ value : c }} )).collect();\n",
                indent
            )?;
            write!(
                output_file,
                "{}{} {{ value : std::rc::Rc::new(value) }}",
                indent, result_ty_str
            )?;
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
    arg_types: Vec<String>,
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
            let arg0_type: &String = &arg_types[0];
            write!(
                output_file,
                "{}map_insert!(arg0, arg1, arg2, {}, {}, {})",
                indent, option_ty, result_ty_str, arg0_type
            )?;
        }
        "remove" => {
            let result_id = result_ty.get_typedef_id();
            let tuple_record = program.typedefs.get(&result_id).get_record();
            let option_ty = ir_type_to_rust_type(&tuple_record.fields[1].ty, program);
            let arg0_type: &String = &arg_types[0];
            write!(
                output_file,
                "{}map_remove!(arg0, arg1, {}, {}, {})",
                indent, option_ty, result_ty_str, arg0_type
            )?;
        }
        "get" => {
            write!(
                output_file,
                "{}map_get!(arg0, arg1, {})",
                indent, result_ty_str
            )?;
        }
        "getSize" => {
            indent.inc();
            write!(
                output_file,
                "{}{} {{ value : arg0.value.len() as i64 }}\n",
                indent, result_ty_str
            )?;
            indent.dec();
        }
        "iter" => {
            let map_type = &arg_types[0];

            let iter_id = result_ty.get_typedef_id();
            let iter_record = program.typedefs.get(&iter_id).get_record();
            let iter_arg = if let RecordKind::External(_, args) = &iter_record.kind {
                ir_type_to_rust_type(&args[0], program)
            } else {
                unreachable!();
            };

            let base_map_type: Vec<_> = map_type.split("::").collect();
            let map_iter_name = format!("{}_Iter", base_map_type[2]);
            let iter_trait_name = result_ty_str.replace(
                "crate::source::Iterator::",
                "crate::source::Iterator::Trait_",
            );

            write!(output_file, "{}#[derive(Clone)]\n", indent)?;
            write!(output_file, "{}pub struct {} {{\n", indent, map_iter_name)?;
            indent.inc();
            write!(output_file, "{}pub value: Vec<{}>,\n", indent, iter_arg)?;
            write!(output_file, "{}pub index : usize,\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;

            write!(
                output_file,
                "{}impl {} for {} {{\n",
                indent, iter_trait_name, map_iter_name
            )?;
            indent.inc();
            write!(
                output_file,
                "{}fn next(&mut self) -> Option<{}> {{\n",
                indent, iter_arg
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
                indent, map_iter_name
            )?;
            indent.inc();
            write!(output_file, "{}value: crate::UnpackRC::unpack(arg0.value).into_iter().map(|(k,v)|{{ {} {{ _siko_field_0: crate::UnpackRC::unpack(k), _siko_field_1: crate::UnpackRC::unpack(v) }} }}).collect(),\n", indent, iter_arg)?;
            write!(output_file, "{}index: 0,\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}),\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
        }
        "toMap" => {
            write!(output_file, "{}let mut arg0 = arg0;\n", indent)?;
            write!(
                output_file,
                "{}let mut value = std::collections::BTreeMap::new();\n",
                indent
            )?;
            write!(output_file, "{}loop {{\n", indent)?;
            indent.inc();
            write!(
                output_file,
                "{}if let Some(v) = arg0.value.next() {{\n",
                indent
            )?;
            indent.inc();
            write!(
                output_file,
                "{}value.insert(std::rc::Rc::new(v._siko_field_0), std::rc::Rc::new(v._siko_field_1));\n",
                indent
            )?;
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
                "{}{} {{ value: std::rc::Rc::new(value) }}",
                indent, result_ty_str
            )?;
        }
        "show" => {
            write!(
                output_file,
                "{}let subs: Vec<_> = (*arg0.value).clone().into_iter().map(|(k, v)| format!(\"{{}}:{{}}\", k, v)).collect();\n",
                indent
            )?;
            write!(
                output_file,
                "{}{} {{ value : std::rc::Rc::new(format!(\"{{{{ {{}} }}}}\", subs.join(\", \"))) }}",
                indent, result_ty_str
            )?;
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
        _ => panic!("Map/{} not implemented", original_name),
    }
    indent.dec();
    Ok(())
}

fn generate_list_builtins(
    _function: &Function,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
    original_name: &str,
    result_ty: &Type,
    result_ty_str: &str,
    arg_type_types: Vec<Type>,
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
                "{}{} {{ value : std::rc::Rc::new(format!(\"[{{}}]\", subs.join(\", \"))) }}",
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
            write!(output_file, "{}value.push(std::rc::Rc::new(v));\n", indent)?;
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
                "{}{} {{ value: std::rc::Rc::new(value) }}",
                indent, result_ty_str
            )?;
        }
        "iter" => {
            let list_type = &arg_types[0];

            let id = arg_type_types[0].get_typedef_id();
            let list_record = program.typedefs.get(&id).get_record();
            let list_arg_type = if let RecordKind::External(_, args) = &list_record.kind {
                let value_ty = ir_type_to_rust_type(&args[0], program);
                value_ty
            } else {
                unreachable!()
            };

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
                "{}pub value: std::rc::Rc<Vec<std::rc::Rc<{}>>>,\n",
                indent, list_arg_type
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
                "{}fn next(&mut self) -> Option<{}> {{\n",
                indent, list_arg_type
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
                "{}let v = (*self.value[self.index]).clone();\n",
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
        "cmp" => {
            return generate_cmp_builtin_body(output_file, program, indent, result_ty_str);
        }
        "partialCmp" => {
            return generate_partial_cmp_builtin_body(
                output_file,
                program,
                indent,
                result_ty,
                result_ty_str,
            );
        }
        "isEmpty" => {
            write!(output_file, "{}if arg0.value.is_empty() {{\n", indent)?;
            indent.inc();
            write!(output_file, "{}{}::True\n", indent, result_ty_str)?;
            indent.dec();
            write!(output_file, "{}}} else {{\n", indent)?;
            indent.inc();
            write!(output_file, "{}{}::False\n", indent, result_ty_str)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
        }
        "atIndex" => {
            write!(output_file, "{}let index = arg1.value as usize;\n", indent)?;
            write!(output_file, "{}(*arg0.value[index]).clone()\n", indent)?;
        }
        "split" => {
            let list_type = ir_type_to_rust_type(&arg_type_types[0], program);
            write!(output_file, "{}let mut v1 = (*arg0.value).clone();\n", indent)?;
            write!(output_file, "{}let index = arg1.value as usize;\n", indent)?;
            write!(output_file, "{}let v2 :Vec<_>= v1.split_off(index);\n", indent)?;
            write!(
                output_file,
                "{}let v1 = {} {{ value : std::rc::Rc::new(v1) }};\n",
                indent, list_type
            )?;
            write!(
                output_file,
                "{}let v2 = {} {{ value : std::rc::Rc::new(v2) }};\n",
                indent, list_type
            )?;
            write!(
                output_file,
                "{} {} {{ _siko_field_0 : v1, _siko_field_1: v2 }}\n",
                indent, result_ty_str
            )?;
        }
        "tail" => {
            let list_type = ir_type_to_rust_type(&arg_type_types[0], program);

            write!(output_file, "{}match arg0.value.is_empty() {{\n", indent)?;
            write!(
                output_file,
                "{}true => {{ {}::None }}\n",
                indent, result_ty_str
            )?;
            write!(
                output_file,
                "{}false => {{ let mut v = (*arg0.value).clone(); v.remove(0); {}::Some({} {{ value : std::rc::Rc::new(v) }}) }}\n",
                indent, result_ty_str, list_type
            )?;
            write!(output_file, "{}}}\n", indent)?;
        }
        "opAdd" => {
            write!(output_file, "{}let mut r = Vec::new();\n", indent)?;
            write!(
                output_file,
                "{}r.extend(arg0.value.iter().cloned());\n",
                indent
            )?;
            write!(
                output_file,
                "{}r.extend(arg1.value.iter().cloned());\n",
                indent
            )?;
            write!(
                output_file,
                "{} {} {{ value : std::rc::Rc::new(r) }}\n",
                indent, result_ty_str
            )?;
        }
        "dedup" => {
            write!(output_file, "{}let mut r = Vec::new();\n", indent)?;
            write!(
                output_file,
                "{}r.extend(arg0.value.iter().cloned());\n",
                indent
            )?;
            write!(output_file, "{}r.dedup();\n", indent)?;
            write!(
                output_file,
                "{} {} {{ value : std::rc::Rc::new(r) }}\n",
                indent, result_ty_str
            )?;
        }
        "write" => {
            write!(output_file, "{}let mut r = Vec::new();\n", indent)?;
            write!(
                output_file,
                "{}r.extend(arg0.value.iter().cloned());\n",
                indent
            )?;
            write!(
                output_file,
                "{}r[arg1.value as usize] = std::rc::Rc::new(arg2);\n",
                indent
            )?;
            write!(
                output_file,
                "{} {} {{ value : std::rc::Rc::new(r) }}\n",
                indent, result_ty_str
            )?;
        }
        "sort" => {
            write!(output_file, "{}let mut r = Vec::new();\n", indent)?;
            write!(
                output_file,
                "{}r.extend(arg0.value.iter().cloned());\n",
                indent
            )?;
            write!(output_file, "{}r.sort();\n", indent)?;
            write!(
                output_file,
                "{} {} {{ value : std::rc::Rc::new(r) }}\n",
                indent, result_ty_str
            )?;
        }
        "getLength" => {
            indent.inc();
            write!(
                output_file,
                "{}{} {{ value : arg0.value.len() as i64 }}\n",
                indent, result_ty_str
            )?;
            indent.dec();
        }
        _ => panic!("List/{} not implemented", original_name),
    }
    indent.dec();
    Ok(())
}

fn generate_char_builtins(
    _function: &Function,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
    original_name: &str,
    result_ty: &Type,
    result_ty_str: &str,
    _: Vec<String>,
) -> Result<()> {
    indent.inc();
    match original_name {
        "show" => {
            write!(
                output_file,
                "{}{} {{ value : std::rc::Rc::new(format!(\"{{}}\", arg0.value)) }}",
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
        "isUppercase" => {
            write!(
                output_file,
                "{} match (arg0.value.is_uppercase()) {{ true => {}::True, false => {}::False }}",
                indent, result_ty_str, result_ty_str,
            )?;
        }
        _ => panic!("Char/{} not implemented", original_name),
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
    arg_type_types: Vec<Type>,
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
                arg_types,
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
                arg_types,
            );
        }
        "List" => {
            return generate_list_builtins(
                function,
                output_file,
                program,
                indent,
                original_name,
                result_ty,
                result_ty_str,
                arg_type_types,
                arg_types,
            );
        }
        "Char" => {
            return generate_char_builtins(
                function,
                output_file,
                program,
                indent,
                original_name,
                result_ty,
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
                    let arg = if let RecordKind::External(_, args) = &iter.kind {
                        let arg = &args[0];
                        ir_type_to_rust_type(arg, program)
                    } else {
                        unreachable!();
                    };

                    let iter_trait_name = format!("Trait_{}", iter.name);

                    write!(
                        output_file,
                        "{}impl {} for {} {{\n",
                        indent, iter_trait_name, struct_name
                    )?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}fn next(&mut self) -> Option<{}> {{\n",
                        indent, arg
                    )?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}if let Some(value) = self.arg1.value.next() {{\n",
                        indent
                    )?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}Some(self.arg0.clone().call(value))\n",
                        indent
                    )?;
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
                    let arg = if let RecordKind::External(_, args) = &iter.kind {
                        let arg = &args[0];
                        ir_type_to_rust_type(arg, program)
                    } else {
                        unreachable!();
                    };

                    let closure_return_ty = arg_type_types[0].get_result_type(1);
                    let closure_return_ty = ir_type_to_rust_type(&closure_return_ty, program);
                    write!(
                        output_file,
                        "{}impl {} for {} {{\n",
                        indent, iter_trait_name, struct_name
                    )?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}fn next(&mut self) -> Option<{}> {{\n",
                        indent, arg
                    )?;
                    indent.inc();
                    write!(output_file, "{}loop {{\n", indent)?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}if let Some(value) = self.arg1.value.next() {{\n",
                        indent
                    )?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}match self.arg0.clone().call(value.clone()) {{\n",
                        indent,
                    )?;
                    indent.inc();
                    write!(
                        output_file,
                        "{}{}::True => {{ return Some(value); }},\n",
                        indent, closure_return_ty
                    )?;
                    write!(
                        output_file,
                        "{}{}::False => {{ continue; }},\n",
                        indent, closure_return_ty
                    )?;
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;
                    indent.dec();
                    write!(output_file, "{}}} else {{\n", indent)?;
                    indent.inc();
                    write!(output_file, "{}return None;\n", indent)?;
                    indent.dec();
                    write!(output_file, "{}}}\n", indent)?;
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
                ("Iterator", "forEach") => {
                    write!(output_file, "{}let mut arg0 = arg0;\n", indent)?;
                    write!(output_file, "{}let mut arg1 = arg1;\n", indent)?;
                    write!(output_file, "{}loop {{  match arg1.value.next() {{ Some(v) => {{ arg0.clone().call(v); }}, None => {{ break; }}  }} }}\n", indent)?;
                    write!(output_file, "{}{} {{ }}\n", indent, result_ty_str)?;
                }
                ("Iterator", "fold") => {
                    write!(output_file, "{}let mut arg0 = arg0;\n", indent)?;
                    write!(output_file, "{}let mut arg1 = arg1;\n", indent)?;
                    write!(output_file, "{}let mut arg2 = arg2;\n", indent)?;
                    write!(output_file, "{}loop {{  match arg2.value.next() {{ Some(v) => {{ let mut partial = arg0.clone().call(arg1.clone()); arg1 = partial.call(v); }}, None => {{ break; }}  }} }}\n", indent)?;
                    write!(output_file, "{}arg1\n", indent)?;
                }
                ("Hack", "readTextFile") => {
                    write!(
                        output_file,
                        "let content = std::fs::read(&*arg0.value).expect(\"ReadTextFile failed\");"
                    )?;
                    write!(
                        output_file,
                        "let content = String::from_utf8_lossy(&content).to_string();"
                    )?;
                    write!(
                        output_file,
                        "{} {{ value : std::rc::Rc::new(content) }}",
                        result_ty_str
                    )?;
                }
                ("Hack", "writeTextFile") => {
                    write!(
                        output_file,
                        "let content = std::fs::write(&*arg0.value, &*arg1.value).expect(\"WriteTextFile failed\");"
                    )?;
                    write!(output_file, "{} {{ }}", result_ty_str)?;
                }
                ("Hack", "getArgs") => {
                    write!(
                        output_file,
                        "let args: Vec<String> = std::env::args().collect();"
                    )?;
                    write!(output_file, "let args: Vec<_> = args.into_iter().map(|arg| std::rc::Rc::new({} {{ value : std::rc::Rc::new(arg) }})).collect();", "crate::source::String::String")?;
                    write!(
                        output_file,
                        "{} {{ value : std::rc::Rc::new(args) }}",
                        result_ty_str
                    )?;
                }
                ("Std.Util.Basic", "abort") => {
                    write!(output_file, "panic!(\"abort called\");")?;
                }
                _ => panic!("{}/{} not implemented", function.module, function.name),
            }
        }
    }
    indent.dec();
    Ok(())
}
