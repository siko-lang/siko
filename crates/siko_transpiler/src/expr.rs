use crate::pattern::write_pattern;
use crate::types::ir_type_to_rust_type;
use crate::util::arg_name;
use crate::util::get_module_name;
use crate::util::Indent;
use siko_constants::MIR_INTERNAL_MODULE_NAME;
use siko_mir::expr::Expr;
use siko_mir::expr::ExprId;
use siko_mir::pattern::Pattern;
use siko_mir::program::Program;
use siko_mir::types::Type;
use std::io::Result;
use std::io::Write;

pub fn write_expr(
    expr_id: ExprId,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
) -> Result<bool> {
    let mut is_statement = false;
    let expr = &program.exprs.get(&expr_id).item;
    match expr {
        Expr::ArgRef(i) => {
            let arg = arg_name(*i);
            write!(output_file, "{}", arg)?;
        }
        Expr::Do(items) => {
            write!(output_file, "{{\n")?;
            indent.inc();
            for (index, item) in items.iter().enumerate() {
                write!(output_file, "{}", indent)?;
                let is_statement = write_expr(*item, output_file, program, indent)?;
                if is_statement {
                    if index == items.len() - 1 {
                        let ty = program.get_expr_type(&expr_id);
                        write!(output_file, "{} {{ }} ", ir_type_to_rust_type(ty, program))?;
                    } else {
                        write!(output_file, "\n")?;
                    }
                } else {
                    if index != items.len() - 1 {
                        write!(output_file, ";\n")?;
                    }
                }
            }
            indent.dec();
            write!(output_file, "\n{}}}", indent)?;
        }
        Expr::RecordInitialization(id, items) => {
            let ty = program.get_expr_type(&expr_id);
            let record = program.typedefs.get(id).get_record();
            write!(output_file, "{} {{", ir_type_to_rust_type(ty, program))?;
            for (item, index) in items {
                let field = &record.fields[*index];
                write!(output_file, "{}: ", field.name)?;
                if let Type::Boxed(_) = field.ty {
                    write!(output_file, "Box::new(")?;
                    write_expr(*item, output_file, program, indent)?;
                    write!(output_file, ")")?;
                } else {
                    write_expr(*item, output_file, program, indent)?;
                };
                write!(output_file, ", ")?;
            }
            write!(output_file, "}}")?;
        }
        Expr::RecordUpdate(receiver, items) => {
            let ty = program.get_expr_type(&expr_id);
            let id = ty.get_typedef_id();
            let record = program.typedefs.get(&id).get_record();
            write!(output_file, "{{ let mut value = ")?;
            indent.inc();
            write_expr(*receiver, output_file, program, indent)?;
            write!(output_file, ";\n")?;
            for (item, index) in items {
                let field = &record.fields[*index];
                write!(output_file, "{}value.{} = ", indent, field.name)?;
                if let Type::Boxed(_) = field.ty {
                    write!(output_file, "Box::new(")?;
                    write_expr(*item, output_file, program, indent)?;
                    write!(output_file, ")")?;
                } else {
                    write_expr(*item, output_file, program, indent)?;
                };
                write!(output_file, ";\n")?;
            }
            write!(output_file, "{}value }}", indent)?;
            indent.dec();
        }
        Expr::Bind(pattern, rhs) => {
            write!(output_file, "let ")?;
            write_pattern(*pattern, output_file, program, indent)?;
            write!(output_file, " = ")?;
            write_expr(*rhs, output_file, program, indent)?;
            write!(output_file, ";")?;
            is_statement = true;
        }
        Expr::ExprValue(_, pattern_id) => {
            let pattern = &program.patterns.get(pattern_id).item;
            if let Pattern::Binding(n) = pattern {
                write!(output_file, "{}", n)?;
            } else {
                unreachable!();
            }
        }
        Expr::PartialFunctionCall(id, args) => {
            let partial_function_call = program.partial_function_calls.get(id);
            let closure = program.get_closure_type(&partial_function_call.closure_type);
            let name = partial_function_call.get_name();
            write!(
                output_file,
                "crate::{}::{} {{ value: Box::new(crate::{}::{} {{",
                get_module_name(MIR_INTERNAL_MODULE_NAME),
                closure.name,
                get_module_name(MIR_INTERNAL_MODULE_NAME),
                name
            )?;
            for index in 0..partial_function_call.fields.len() {
                write!(output_file, "{} : ", arg_name(index))?;
                if index < args.len() {
                    let arg = &args[index];
                    write!(output_file, "Some(")?;
                    write_expr(*arg, output_file, program, indent)?;
                    write!(output_file, ")")?;
                } else {
                    write!(output_file, "None")?;
                }
                if index != partial_function_call.fields.len() - 1 {
                    write!(output_file, ", ")?;
                }
            }
            write!(output_file, "}})}}")?;
        }
        Expr::StaticFunctionCall(id, args) => {
            let function = program.functions.get(id);
            assert_eq!(function.arg_count, args.len());
            let name = format!(
                "crate::{}::{}",
                get_module_name(&function.module),
                function.name
            );
            write!(output_file, "{} (", name)?;
            for (index, arg) in args.iter().enumerate() {
                write_expr(*arg, output_file, program, indent)?;
                if index != args.len() - 1 {
                    write!(output_file, ", ")?;
                }
            }
            write!(output_file, ")")?;
        }
        Expr::IntegerLiteral(i) => {
            let ty = program.get_expr_type(&expr_id);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "{} {{ value: {} }}", ty, i)?;
        }
        Expr::StringLiteral(s) => {
            let ty = program.get_expr_type(&expr_id);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "{} {{ value: \"{}\".to_string() }}", ty, s)?;
        }
        Expr::FloatLiteral(f) => {
            let ty = program.get_expr_type(&expr_id);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "{} {{ value: {:.5} }}", ty, f)?;
        }
        Expr::CharLiteral(c) => {
            let ty = program.get_expr_type(&expr_id);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "{} {{ value: '{}' }}", ty, c)?;
        }
        Expr::Formatter(fmt, args) => {
            let ty = program.get_expr_type(&expr_id);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "{} {{ value : format!(\"{}\"", ty, fmt)?;
            if !args.is_empty() {
                write!(output_file, ",")?;
            }
            for (index, arg) in args.iter().enumerate() {
                write_expr(*arg, output_file, program, indent)?;
                write!(output_file, ".value")?;
                if index != args.len() - 1 {
                    write!(output_file, ",")?;
                }
            }
            write!(output_file, ")}}")?;
        }
        Expr::CaseOf(body, cases) => {
            write!(output_file, "match (")?;
            write_expr(*body, output_file, program, indent)?;
            write!(output_file, ") {{\n")?;
            indent.inc();
            for case in cases {
                write!(output_file, "{}", indent)?;
                write_pattern(case.pattern_id, output_file, program, indent)?;
                write!(output_file, " => {{")?;
                write_expr(case.body, output_file, program, indent)?;
                write!(output_file, "}}\n")?;
            }
            indent.dec();
            write!(output_file, "{}}}", indent)?;
        }
        Expr::If(cond, true_branch, false_branch) => {
            let ty = program.get_expr_type(cond);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "if {{ match (")?;
            write_expr(*cond, output_file, program, indent)?;
            write!(
                output_file,
                ") {{ {}::True => true, {}::False => false, }} }} ",
                ty, ty
            )?;
            write!(output_file, " {{ ")?;
            write_expr(*true_branch, output_file, program, indent)?;
            write!(output_file, " }} ")?;
            write!(output_file, " else {{ ")?;
            write_expr(*false_branch, output_file, program, indent)?;
            write!(output_file, " }} ")?;
        }
        Expr::FieldAccess(index, receiver) => {
            let ty = program.get_expr_type(receiver);
            let id = ty.get_typedef_id();
            let record = program.typedefs.get(&id).get_record();
            let field = &record.fields[*index];
            write_expr(*receiver, output_file, program, indent)?;
            write!(output_file, ".{}", field.name)?;
        }
        Expr::List(items) => {
            let ty = program.get_expr_type(&expr_id);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "{} {{ value: vec![", ty)?;
            for (index, item) in items.iter().enumerate() {
                write_expr(*item, output_file, program, indent)?;
                if index != items.len() - 1 {
                    write!(output_file, ", ")?;
                }
            }
            write!(output_file, "] }}")?;
        }
        Expr::DynamicFunctionCall(receiver, args) => {
            indent.inc();
            write!(output_file, "{{\n{}let mut dyn_fn = ", indent)?;
            write_expr(*receiver, output_file, program, indent)?;
            write!(output_file, ";\n")?;
            for (index, arg) in args.iter().enumerate() {
                if index == args.len() - 1 {
                    write!(output_file, "{}let dyn_fn = dyn_fn.call(", indent)?;
                } else {
                    write!(output_file, "{}let mut dyn_fn = dyn_fn.call(", indent)?;
                }
                write_expr(*arg, output_file, program, indent)?;
                write!(output_file, ");\n")?;
            }
            write!(output_file, "{}dyn_fn\n", indent)?;
            indent.dec();
            write!(output_file, "{}}}", indent)?;
        }
        Expr::Clone(rhs) => {
            write_expr(*rhs, output_file, program, indent)?;
            write!(output_file, ".clone()")?;
        }
        Expr::Deref(rhs) => {
            write!(output_file, "*")?;
            write_expr(*rhs, output_file, program, indent)?;
        }
    }
    Ok(is_statement)
}
