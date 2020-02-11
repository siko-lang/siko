use crate::expr::write_expr;
use crate::types::ir_type_to_rust_type;
use crate::util::Indent;
use siko_mir::pattern::Pattern;
use siko_mir::pattern::PatternId;
use siko_mir::pattern::RangeKind;
use siko_mir::program::Program;
use std::io::Result;
use std::io::Write;

pub fn write_pattern(
    pattern_id: PatternId,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
) -> Result<()> {
    let pattern = &program.patterns.get(&pattern_id).item;
    match pattern {
        Pattern::Binding(name) => {
            write!(output_file, "{}", name)?;
        }
        Pattern::Record(id, items) => {
            let ty = program.get_pattern_type(&pattern_id);
            let record = program.typedefs.get(id).get_record();
            write!(output_file, "{} {{", ir_type_to_rust_type(ty, program))?;
            for (index, item) in items.iter().enumerate() {
                let field = &record.fields[index];
                write!(output_file, "{}: ", field.name)?;
                write_pattern(*item, output_file, program, indent)?;
                write!(output_file, ", ")?;
            }
            write!(output_file, "}}")?;
        }
        Pattern::Variant(id, index, items) => {
            let ty = program.get_pattern_type(&pattern_id);
            let adt = program.typedefs.get(id).get_adt();
            let variant = &adt.variants[*index];
            write!(
                output_file,
                "{}::{}",
                ir_type_to_rust_type(ty, program),
                variant.name
            )?;
            if !items.is_empty() {
                write!(output_file, "(")?;
                for (index, item) in items.iter().enumerate() {
                    write_pattern(*item, output_file, program, indent)?;
                    if index != items.len() - 1 {
                        write!(output_file, ", ")?;
                    }
                }
                write!(output_file, ")")?;
            }
        }
        Pattern::Guarded(pattern, expr) => {
            write_pattern(*pattern, output_file, program, indent)?;
            write!(output_file, " if {{ match ")?;
            write_expr(*expr, output_file, program, indent)?;
            let ty = program.get_expr_type(expr);
            let ty = ir_type_to_rust_type(ty, program);
            write!(
                output_file,
                "  {{ {}::True => true, {}::False => false }} }}",
                ty, ty
            )?;
        }
        Pattern::Wildcard => {
            write!(output_file, "_")?;
        }
        Pattern::IntegerLiteral(i) => {
            let ty = program.get_pattern_type(&pattern_id);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "{} {{ value: {} }}", ty, i)?;
        }
        Pattern::CharLiteral(i) => {
            let ty = program.get_pattern_type(&pattern_id);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "{} {{ value: '{}' }}", ty, i)?;
        }
        Pattern::CharRange(start, end, kind) => {
            match kind {
                RangeKind::Exclusive => {
                    write!(
                        output_file,
                        "p if std::ops::Range{{ start : '{}', end: '{}' }}",
                        start, end
                    )?;
                }
                RangeKind::Inclusive => {
                    write!(
                        output_file,
                        "p if std::ops::RangeInclusive::new('{}', '{}')",
                        start, end
                    )?;
                }
            }
            write!(output_file, ".contains(&p.value)")?;
        }
        Pattern::StringLiteral(s) => {
            let ty = program.get_pattern_type(&pattern_id);
            let ty = ir_type_to_rust_type(ty, program);
            write!(output_file, "{} {{ value: {} }}", ty, s)?;
        }
    }
    Ok(())
}
