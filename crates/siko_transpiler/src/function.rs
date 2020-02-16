use crate::builtins::generate_builtin;
use crate::expr::write_expr;
use crate::types::ir_type_to_rust_type;
use crate::util::arg_name;
use crate::util::get_ord_type_from_optional_ord;
use crate::util::Indent;
use siko_constants::EQ_CLASS_NAME;
use siko_mir::function::FunctionId;
use siko_mir::function::FunctionInfo;
use siko_mir::program::Program;
use siko_mir::types::Type;
use std::io::Result;
use std::io::Write;

pub fn write_function(
    function_id: FunctionId,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
) -> Result<()> {
    let function = program.functions.get(&function_id);
    let mut fn_args = Vec::new();
    function.function_type.get_args(&mut fn_args);
    let mut args: Vec<String> = Vec::new();
    let mut arg_types: Vec<String> = Vec::new();
    for i in 0..function.arg_count {
        let arg_ty = ir_type_to_rust_type(&fn_args[i], program);
        let arg_str = format!("{}: {}", arg_name(i), arg_ty);
        arg_types.push(arg_ty);
        args.push(arg_str);
    }
    let args: String = args.join(", ");
    let result_type = function.function_type.get_result_type(function.arg_count);
    let result_ty_str = ir_type_to_rust_type(&result_type, program);
    if let FunctionInfo::ExternClassImpl(class_name, ty, body) = &function.info {
        if class_name == "show" {
            let impl_ty = ir_type_to_rust_type(&ty, program);
            write!(
                output_file,
                "{}impl std::fmt::Display for {} {{\n",
                indent, impl_ty,
            )?;
            indent.inc();
            write!(
                output_file,
                "{}fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{\n",
                indent
            )?;
            indent.inc();
            write!(output_file, "{}let arg0 = self.clone();\n", indent)?;
            write!(output_file, "{}let value = ", indent)?;
            write_expr(*body, output_file, program, indent)?;
            write!(output_file, ";\n")?;
            write!(output_file, "{}write!(f, \"{{}}\", value.value)\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
        } else if class_name == "cmp" {
            let impl_ty = ir_type_to_rust_type(&ty, program);
            write!(
                output_file,
                "{}impl std::cmp::Ord for {} {{\n",
                indent, impl_ty,
            )?;
            indent.inc();
            write!(
                output_file,
                "{}fn cmp(&self, arg1: &{}) -> std::cmp::Ordering {{\n",
                indent, impl_ty
            )?;
            indent.inc();
            write!(output_file, "{}let arg0 = self.clone();\n", indent)?;
            write!(output_file, "{}let arg1 = arg1.clone();\n", indent)?;
            write!(output_file, "{}let value = ", indent)?;
            write_expr(*body, output_file, program, indent)?;
            write!(output_file, ";\n")?;
            write!(output_file, "{}match value {{\n", indent)?;
            indent.inc();
            write!(
                output_file,
                "{} {}::Less => std::cmp::Ordering::Less,\n",
                indent, result_ty_str
            )?;
            write!(
                output_file,
                "{} {}::Equal => std::cmp::Ordering::Equal,\n",
                indent, result_ty_str
            )?;
            write!(
                output_file,
                "{} {}::Greater => std::cmp::Ordering::Greater,\n",
                indent, result_ty_str
            )?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
        } else if class_name == "opEq" {
            let impl_ty = ir_type_to_rust_type(&ty, program);
            write!(
                output_file,
                "{}impl std::cmp::PartialEq for {} {{\n",
                indent, impl_ty,
            )?;
            indent.inc();
            write!(
                output_file,
                "{}fn eq(&self, arg1: &{}) -> bool {{\n",
                indent, impl_ty
            )?;
            indent.inc();
            write!(output_file, "{}let arg0 = self.clone();\n", indent)?;
            write!(output_file, "{}let arg1 = arg1.clone();\n", indent)?;
            write!(output_file, "{}let value = ", indent)?;
            write_expr(*body, output_file, program, indent)?;
            write!(output_file, ";\n")?;
            write!(output_file, "{}match value {{\n", indent)?;
            indent.inc();
            write!(output_file, "{} {}::True => true,\n", indent, result_ty_str)?;
            write!(
                output_file,
                "{} {}::False => false,\n",
                indent, result_ty_str
            )?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
        } else if class_name == "partialCmp" {
            let impl_ty = ir_type_to_rust_type(&ty, program);
            let ord_ty_str = get_ord_type_from_optional_ord(&result_type, program);
            write!(
                output_file,
                "{}impl std::cmp::PartialOrd for {} {{\n",
                indent, impl_ty,
            )?;
            indent.inc();
            write!(
                output_file,
                "{}fn partial_cmp(&self, arg1: &{}) -> Option<std::cmp::Ordering> {{\n",
                indent, impl_ty
            )?;
            indent.inc();
            write!(output_file, "{}let arg0 = self.clone();\n", indent)?;
            write!(output_file, "{}let arg1 = arg1.clone();\n", indent)?;
            write!(output_file, "{}let value = ", indent)?;
            write_expr(*body, output_file, program, indent)?;
            write!(output_file, ";\n")?;
            write!(output_file, "{}match value {{\n", indent)?;
            indent.inc();
            write!(
                output_file,
                "{} {}::Some({}::Greater) => Some(std::cmp::Ordering::Greater),\n",
                indent, result_ty_str, ord_ty_str
            )?;
            write!(
                output_file,
                "{} {}::Some({}::Equal) => Some(std::cmp::Ordering::Equal),\n",
                indent, result_ty_str, ord_ty_str
            )?;
            write!(
                output_file,
                "{} {}::Some({}::Less) => Some(std::cmp::Ordering::Less),\n",
                indent, result_ty_str, ord_ty_str
            )?;
            write!(output_file, "{} {}::None => None,\n", indent, result_ty_str)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
        } else if class_name == EQ_CLASS_NAME {
            let impl_ty = ir_type_to_rust_type(&ty, program);
            write!(
                output_file,
                "{}impl std::cmp::Eq for {} {{ }}\n",
                indent, impl_ty,
            )?;
        } else {
            panic!("Unimplemented extern class impl {}", class_name);
        }
    } else {
        write!(
            output_file,
            "{}pub fn {}({}) -> {} {{\n",
            indent, function.name, args, result_ty_str
        )?;
        match &function.info {
            FunctionInfo::Normal(body) => {
                indent.inc();
                write!(output_file, "{}", indent)?;
                write_expr(*body, output_file, program, indent)?;
                indent.dec();
            }
            FunctionInfo::Extern(original_name) => {
                generate_builtin(
                    function,
                    output_file,
                    program,
                    indent,
                    original_name.as_ref(),
                    &result_type,
                    result_ty_str.as_ref(),
                    arg_types,
                )?;
            }
            FunctionInfo::VariantConstructor(id, index) => {
                let adt = program.typedefs.get(id).get_adt();
                let variant = &adt.variants[*index];
                indent.inc();
                write!(output_file, "{}{}::{}", indent, result_ty_str, variant.name)?;
                if function.arg_count > 0 {
                    let mut args = Vec::new();
                    for i in 0..function.arg_count {
                        let item_type = &variant.items[i];
                        let arg_str = if let Type::Boxed(_) = item_type {
                            format!("Box::new({})", arg_name(i))
                        } else {
                            format!("{}", arg_name(i))
                        };
                        args.push(arg_str);
                    }
                    write!(output_file, "({})", args.join(", "))?;
                }
                indent.dec();
            }
            FunctionInfo::RecordConstructor(id) => {
                let record = program.typedefs.get(id).get_record();
                indent.inc();
                write!(output_file, "{}{}", indent, result_ty_str)?;
                let mut args = Vec::new();
                for (index, field) in record.fields.iter().enumerate() {
                    let arg_str = if let Type::Boxed(_) = field.ty {
                        format!("Box::new({})", arg_name(index))
                    } else {
                        format!("{}", arg_name(index))
                    };
                    let arg_str = format!("{}: {}", field.name, arg_str);
                    args.push(arg_str);
                }
                write!(output_file, "{{ {} }}", args.join(", "))?;
                indent.dec();
            }
            FunctionInfo::ExternClassImpl(..) => unreachable!(),
        }
        write!(output_file, "\n{}}}\n", indent,)?;
    }
    Ok(())
}
