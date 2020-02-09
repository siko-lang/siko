use siko_mir::data::TypeDef;
use siko_mir::data::TypeDefId;
use siko_mir::program::Program;
use siko_mir::types::Type;
use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rewrite {
    Variant(TypeDefId, usize, usize),
    RecordField(TypeDefId, usize),
}

fn check_type(
    typedef_id: TypeDefId,
    program: &Program,
    checked_types: &mut BTreeSet<TypeDefId>,
    cycle: Vec<TypeDefId>,
    rewrites: &mut BTreeSet<Rewrite>,
) {
    checked_types.insert(typedef_id);
    let typedef = program.typedefs.get(&typedef_id);
    match typedef {
        TypeDef::Adt(adt) => {
            for (variant_index, variant) in adt.variants.iter().enumerate() {
                for (item_index, item) in variant.items.iter().enumerate() {
                    if let Some(id) = item.get_typedef_id_opt() {
                        if cycle.contains(&id) {
                            let rewrite = Rewrite::Variant(typedef_id, variant_index, item_index);
                            rewrites.insert(rewrite);
                        } else {
                            let mut cycle = cycle.clone();
                            cycle.push(id);
                            check_type(id, program, checked_types, cycle, rewrites);
                        }
                    }
                }
            }
        }
        TypeDef::Record(record) => {
            for (index, field) in record.fields.iter().enumerate() {
                if let Some(id) = field.ty.get_typedef_id_opt() {
                    if cycle.contains(&id) {
                        let rewrite = Rewrite::RecordField(typedef_id, index);
                        rewrites.insert(rewrite);
                    } else {
                        let mut cycle = cycle.clone();
                        cycle.push(id);
                        check_type(id, program, checked_types, cycle, rewrites);
                    }
                }
            }
        }
    }
}

pub fn check_recursive_data_types(program: &mut Program) {
    let mut rewrites = BTreeSet::new();
    let mut checked_types = BTreeSet::new();
    for (id, _) in program.typedefs.items.iter() {
        if checked_types.insert(*id) {
            let mut cycle = Vec::new();
            cycle.push(*id);
            check_type(*id, program, &mut checked_types, cycle, &mut rewrites);
        }
    }
    for rewrite in rewrites {
        match rewrite {
            Rewrite::Variant(id, variant_index, item_index) => {
                let adt = program.typedefs.get_mut(&id).get_mut_adt();
                let variant = &mut adt.variants[variant_index];
                let item_type = &mut variant.items[item_index];
                *item_type = Type::Boxed(Box::new(item_type.clone()));
            }
            Rewrite::RecordField(id, field_index) => {
                let record = program.typedefs.get_mut(&id).get_mut_record();
                let field = &mut record.fields[field_index];
                field.ty = Type::Boxed(Box::new(field.ty.clone()));
            }
        }
    }
}
