use crate::type_processor::process_stored_type;
use crate::type_processor::process_type;
use siko_constants::MIR_INTERNAL_MODULE_NAME;
use siko_ir::data::TypeDef as IrTypeDef;
use siko_ir::program::Program as IrProgram;
use siko_ir::types::Type as IrType;
use siko_mir::data::Adt as MirAdt;
use siko_mir::data::ExternalDataKind;
use siko_mir::data::Record as MirRecord;
use siko_mir::data::RecordField as MirRecordField;
use siko_mir::data::RecordKind as MirRecordKind;
use siko_mir::data::TypeDef as MirTypeDef;
use siko_mir::data::TypeDefId as MirTypeDefId;
use siko_mir::data::Variant as MirVariant;
use siko_mir::data::VariantItem;
use siko_mir::program::Program as MirProgram;
use siko_mir::types::Type as MirType;
use siko_util::Counter;
use std::collections::BTreeMap;

pub struct TypeDefStore {
    typedefs: BTreeMap<IrType, MirTypeDefId>,
    counter: Counter,
}

impl TypeDefStore {
    pub fn new() -> TypeDefStore {
        TypeDefStore {
            typedefs: BTreeMap::new(),
            counter: Counter::new(),
        }
    }

    pub fn add_tuple(
        &mut self,
        ty: IrType,
        field_types: Vec<MirType>,
        mir_program: &mut MirProgram,
    ) -> (String, MirTypeDefId) {
        let mut newly_added = false;
        let mir_typedef_id = *self.typedefs.entry(ty.clone()).or_insert_with(|| {
            newly_added = true;
            mir_program.typedefs.get_id()
        });
        let name = format!("Tuple{}", mir_typedef_id.id);
        if newly_added {
            let mut fields = Vec::new();
            for (index, field_ty) in field_types.into_iter().enumerate() {
                let field_ty = process_stored_type(field_ty, mir_program);
                let field_ty = field_ty.with_var_modifier(self.counter.next());
                let mir_field = MirRecordField {
                    name: format!("field_{}", index),
                    ty: field_ty,
                };
                fields.push(mir_field);
            }
            let mir_record = MirRecord {
                id: mir_typedef_id,
                module: format!("{}", MIR_INTERNAL_MODULE_NAME),
                name: name.clone(),
                modifier_args: Vec::new(),
                fields: fields,
                kind: MirRecordKind::Normal,
            };
            let mir_typedef = MirTypeDef::Record(mir_record);
            mir_program.typedefs.add_item(mir_typedef_id, mir_typedef);
        }
        (name, mir_typedef_id)
    }

    pub fn add_type(
        &mut self,
        ty: IrType,
        ir_program: &IrProgram,
        mir_program: &mut MirProgram,
    ) -> MirTypeDefId {
        let mut newly_added = false;
        let mir_typedef_id = *self.typedefs.entry(ty.clone()).or_insert_with(|| {
            newly_added = true;
            mir_program.typedefs.get_id()
        });
        if newly_added {
            match &ty {
                IrType::Named(_, ir_typedef_id, args) => {
                    let ir_typdef = ir_program.typedefs.get(ir_typedef_id);
                    match ir_typdef {
                        IrTypeDef::Adt(_) => {
                            let mut adt_type_info = ir_program
                                .adt_type_info_map
                                .get(ir_typedef_id)
                                .expect("Adt type info not found")
                                .clone();
                            let mut unifier = ir_program.get_unifier();
                            let r = unifier.unify(&adt_type_info.adt_type, &ty);
                            assert!(r.is_ok());
                            adt_type_info.apply(&unifier);
                            let ir_adt = ir_program.typedefs.get(ir_typedef_id).get_adt();
                            let mut variants = Vec::new();
                            for (index, variant) in adt_type_info.variant_types.iter().enumerate() {
                                let mut mir_item_types = Vec::new();
                                for (item_ty, _) in &variant.item_types {
                                    let mir_item_ty =
                                        process_type(item_ty, self, ir_program, mir_program);
                                    let mir_item_ty = process_stored_type(mir_item_ty, mir_program);
                                    let mir_item_ty =
                                        mir_item_ty.with_var_modifier(self.counter.next());
                                    let variant_item = VariantItem { ty: mir_item_ty };
                                    mir_item_types.push(variant_item);
                                }
                                let mir_variant = MirVariant {
                                    name: ir_adt.variants[index].name.clone(),
                                    items: mir_item_types,
                                };
                                variants.push(mir_variant);
                            }
                            let mir_adt = MirAdt {
                                id: mir_typedef_id,
                                name: format!("{}_{}", ir_adt.name, mir_typedef_id.id),
                                module: ir_adt.module.clone(),
                                modifier_args: Vec::new(),
                                variants: variants,
                            };
                            let mir_typedef = MirTypeDef::Adt(mir_adt);
                            mir_program.typedefs.add_item(mir_typedef_id, mir_typedef);
                        }
                        IrTypeDef::Record(_) => {
                            let mut record_type_info = ir_program
                                .record_type_info_map
                                .get(ir_typedef_id)
                                .expect("Record type info not found")
                                .clone();
                            let mut unifier = ir_program.get_unifier();
                            let r = unifier.unify(&record_type_info.record_type, &ty);
                            assert!(r.is_ok());
                            record_type_info.apply(&unifier);
                            let ir_record = ir_program.typedefs.get(ir_typedef_id).get_record();
                            let mut fields = Vec::new();
                            for (index, (field_ty, _)) in
                                record_type_info.field_types.iter().enumerate()
                            {
                                let mir_field_ty =
                                    process_type(field_ty, self, ir_program, mir_program);
                                let mir_field_ty = process_stored_type(mir_field_ty, mir_program);
                                let mir_field_ty =
                                    mir_field_ty.with_var_modifier(self.counter.next());
                                let mir_field = MirRecordField {
                                    name: ir_record.fields[index].name.clone(),
                                    ty: mir_field_ty,
                                };
                                fields.push(mir_field);
                            }
                            let (name, kind) = if ir_record.external {
                                let name = if args.is_empty() {
                                    format!("{}", ir_record.name)
                                } else {
                                    format!("{}_{}", ir_record.name, mir_typedef_id.id)
                                };
                                let args: Vec<_> = args
                                    .iter()
                                    .map(|arg| process_type(arg, self, ir_program, mir_program))
                                    .collect();
                                let data_kind =
                                    match (ir_record.module.as_ref(), ir_record.name.as_ref()) {
                                        ("Int", "Int") => ExternalDataKind::Int,
                                        ("Float", "Float") => ExternalDataKind::Float,
                                        ("String", "String") => ExternalDataKind::String,
                                        ("Char", "Char") => ExternalDataKind::Char,
                                        ("Map", "Map") => ExternalDataKind::Map,
                                        ("List", "List") => ExternalDataKind::List,
                                        ("Iterator", "Iterator") => ExternalDataKind::Iterator,
                                        _ => panic!(
                                            "{}/{} not implemented data kind",
                                            ir_record.module, ir_record.name
                                        ),
                                    };
                                (name, MirRecordKind::External(data_kind, args))
                            } else {
                                let name = format!("{}_{}", ir_record.name, mir_typedef_id.id);
                                (name, MirRecordKind::Normal)
                            };
                            let mir_record = MirRecord {
                                id: mir_typedef_id,
                                name: name,
                                module: ir_record.module.clone(),
                                modifier_args: Vec::new(),
                                fields: fields,
                                kind: kind,
                            };
                            let mir_typedef = MirTypeDef::Record(mir_record);
                            mir_program.typedefs.add_item(mir_typedef_id, mir_typedef);
                        }
                    }
                }
                _ => unreachable!(),
            };
        }
        mir_typedef_id
    }
}
