use crate::types::ir_type_to_rust_type;
use crate::util::Indent;
use siko_mir::data::ExternalDataKind;
use siko_mir::data::RecordKind;
use siko_mir::data::TypeDef;
use siko_mir::data::TypeDefId;
use siko_mir::program::Program;
use std::io::Result;
use std::io::Write;

pub fn write_typedef(
    typedef_id: TypeDefId,
    output_file: &mut dyn Write,
    program: &Program,
    indent: &mut Indent,
) -> Result<()> {
    let typedef = program.typedefs.get(&typedef_id);
    match typedef {
        TypeDef::Adt(adt) => {
            write!(output_file, "{}#[derive(Clone)]\n", indent)?;
            write!(output_file, "{}pub enum {} {{\n", indent, adt.name)?;
            indent.inc();
            for variant in &adt.variants {
                let items = if variant.items.is_empty() {
                    format!("")
                } else {
                    let mut is = Vec::new();
                    for item in &variant.items {
                        let rust_ty = ir_type_to_rust_type(&item.ty, program);
                        is.push(rust_ty);
                    }
                    format!("({})", is.join(", "))
                };
                write!(output_file, "{}{}{},\n", indent, variant.name, items)?;
            }
            indent.dec();
            write!(output_file, "{}}}\n", indent)?;
        }
        TypeDef::Record(record) => {
            if let RecordKind::External(data_kind, args) = &record.kind {
                match data_kind {
                    ExternalDataKind::Int => {
                        write!(output_file, "{}#[derive(Clone)]\n", indent)?;
                        write!(output_file, "{}pub struct Int {{\n", indent)?;
                        indent.inc();
                        write!(output_file, "{}pub value: i64,\n", indent,)?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                    }
                    ExternalDataKind::String => {
                        write!(output_file, "{}#[derive(Clone)]\n", indent)?;
                        write!(output_file, "{}pub struct String {{\n", indent)?;
                        indent.inc();
                        write!(output_file, "{}pub value: std::string::String,\n", indent,)?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                    }
                    ExternalDataKind::Float => {
                        write!(output_file, "{}#[derive(Clone)]\n", indent)?;
                        write!(output_file, "{}pub struct Float {{\n", indent)?;
                        indent.inc();
                        write!(output_file, "{}pub value: f64,\n", indent,)?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                    }
                    ExternalDataKind::Char => {
                        write!(output_file, "{}#[derive(Clone)]\n", indent)?;
                        write!(output_file, "{}pub struct Char {{\n", indent)?;
                        indent.inc();
                        write!(output_file, "{}pub value: char,\n", indent,)?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                    }
                    ExternalDataKind::Map => {
                        write!(
                            output_file,
                            "{}#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]\n",
                            indent
                        )?;
                        write!(output_file, "{}pub struct {} {{\n", indent, record.name)?;
                        indent.inc();
                        let key_ty = ir_type_to_rust_type(&args[0], program);
                        let value_ty = ir_type_to_rust_type(&args[1], program);
                        write!(
                            output_file,
                            "{}pub value: std::collections::BTreeMap<{}, {}>,\n",
                            indent, key_ty, value_ty
                        )?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                    }
                    ExternalDataKind::Iterator => {
                        let elem_ty = ir_type_to_rust_type(&args[0], program);
                        write!(
                            output_file,
                            "{}pub trait Trait_{} {{\n",
                            indent, record.name
                        )?;
                        indent.inc();
                        write!(
                            output_file,
                            "{}fn next(&mut self) -> Option<{}>;\n",
                            indent, elem_ty
                        )?;
                        write!(
                            output_file,
                            "{}fn box_clone(&self) -> Box<dyn Trait_{}>;\n",
                            indent, record.name
                        )?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                        write!(output_file, "{}pub struct {} {{\n", indent, record.name)?;
                        indent.inc();
                        write!(
                            output_file,
                            "{}pub value: Box<dyn Trait_{}>,\n",
                            indent, record.name
                        )?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;

                        write!(output_file, "{}impl Clone for {} {{\n", indent, record.name)?;
                        indent.inc();
                        write!(
                            output_file,
                            "{}fn clone(&self) -> {} {{\n",
                            indent, record.name
                        )?;
                        indent.inc();
                        write!(output_file, "{}{} {{\n", indent, record.name)?;
                        indent.inc();
                        write!(output_file, "{}value: self.value.box_clone(),\n", indent,)?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                    }
                    ExternalDataKind::List => {
                        let elem_ty = ir_type_to_rust_type(&args[0], program);
                        write!(output_file, "{}#[derive(Clone)]\n", indent)?;
                        write!(output_file, "{}pub struct {} {{\n", indent, record.name)?;
                        indent.inc();
                        write!(output_file, "{}pub value: Vec<{}>,\n", indent, elem_ty)?;
                        indent.dec();
                        write!(output_file, "{}}}\n", indent)?;
                    }
                }
            } else {
                write!(output_file, "{}pub struct {} {{\n", indent, record.name)?;
                indent.inc();
                for field in &record.fields {
                    let field_type = ir_type_to_rust_type(&field.ty, program);
                    write!(
                        output_file,
                        "{}pub {}: {},\n",
                        indent, field.name, field_type
                    )?;
                }
                indent.dec();
                write!(output_file, "{}}}\n", indent)?;
                write!(output_file, "{}impl Clone for {} {{\n", indent, record.name)?;
                indent.inc();
                write!(
                    output_file,
                    "{}fn clone(&self) -> {} {{\n",
                    indent, record.name
                )?;
                indent.inc();
                write!(output_file, "{}{} {{\n", indent, record.name)?;
                indent.inc();
                for field in &record.fields {
                    write!(
                        output_file,
                        "{}{}: self.{}.clone(),\n",
                        indent, field.name, field.name
                    )?;
                }
                indent.dec();
                write!(output_file, "{}}}\n", indent)?;
                indent.dec();
                write!(output_file, "{}}}\n", indent)?;
                indent.dec();
                write!(output_file, "{}}}\n", indent)?;
            }
        }
    }
    Ok(())
}
