use crate::error::ResolverError;
use crate::import::ImportedItemInfo;
use crate::import::Namespace;
use crate::item::Item;
use crate::module::Module;
use crate::type_arg_resolver::TypeArgResolver;
use siko_ir::class::ClassId;
use siko_ir::program::Program as IrProgram;
use siko_ir::type_signature::TypeSignature as IrTypeSignature;
use siko_ir::type_signature::TypeSignatureId as IrTypeSignatureId;
use siko_location_info::item::ItemInfo;
use siko_location_info::location_id::LocationId;
use siko_syntax::program::Program;
use siko_syntax::types::TypeSignature as AstTypeSignature;
use siko_syntax::types::TypeSignatureId;
use std::collections::BTreeMap;

fn process_named_type(
    name: &str,
    named_args: &Vec<TypeSignatureId>,
    location_id: LocationId,
    program: &Program,
    ir_program: &mut IrProgram,
    module: &Module,
    type_arg_resolver: &mut TypeArgResolver,
    errors: &mut Vec<ResolverError>,
) -> Option<IrTypeSignatureId> {
    let ir_type_signature = match module.imported_items.get(name) {
        Some(items) => match ImportedItemInfo::resolve_ambiguity(items, Namespace::Type) {
            None => {
                println!("WTF {:?}", items);
                let error = ResolverError::AmbiguousName(name.to_string(), location_id);
                errors.push(error);
                return None;
            }
            Some(item) => {
                let mut named_arg_ids = Vec::new();
                for named_arg in named_args {
                    match process_type_signature(
                        named_arg,
                        program,
                        ir_program,
                        module,
                        type_arg_resolver,
                        errors,
                    ) {
                        Some(id) => {
                            named_arg_ids.push(id);
                        }
                        None => {
                            return None;
                        }
                    }
                }
                match item.item {
                    Item::Adt(_, ir_typedef_id) => {
                        let ir_adt = ir_program.typedefs.get(&ir_typedef_id).get_adt();
                        if ir_adt.type_args.len() != named_arg_ids.len() {
                            let err = ResolverError::IncorrectTypeArgumentCount(
                                name.to_string(),
                                ir_adt.type_args.len(),
                                named_arg_ids.len(),
                                location_id,
                            );
                            errors.push(err);
                            return None;
                        }
                        IrTypeSignature::Named(ir_adt.name.clone(), ir_typedef_id, named_arg_ids)
                    }
                    Item::Record(_, ir_typedef_id) => {
                        let ir_record = ir_program.typedefs.get(&ir_typedef_id).get_record();

                        if ir_record.type_args.len() != named_arg_ids.len() {
                            let err = ResolverError::IncorrectTypeArgumentCount(
                                name.to_string(),
                                ir_record.type_args.len(),
                                named_arg_ids.len(),
                                location_id,
                            );
                            errors.push(err);
                            return None;
                        }
                        IrTypeSignature::Named(ir_record.name.clone(), ir_typedef_id, named_arg_ids)
                    }
                    Item::Function(..)
                    | Item::Variant(..)
                    | Item::ClassMember(..)
                    | Item::Class(..) => {
                        let err = ResolverError::NameNotType(name.to_string(), location_id);
                        errors.push(err);
                        return None;
                    }
                }
            }
        },
        None => {
            let error = ResolverError::UnknownTypeName(name.to_string(), location_id);
            errors.push(error);
            return None;
        }
    };
    let id = ir_program.type_signatures.get_id();
    let type_info = ItemInfo::new(ir_type_signature, location_id);
    ir_program.type_signatures.add_item(id, type_info);
    return Some(id);
}

pub fn subtitute_type_signature(
    source: &IrTypeSignatureId,
    from: usize,
    to: &IrTypeSignatureId,
    ir_program: &mut IrProgram,
) -> IrTypeSignatureId {
    let info = ir_program.type_signatures.get(source).clone();
    let new_signature = match &info.item {
        IrTypeSignature::Function(old_from, old_to) => {
            let new_from = subtitute_type_signature(old_from, from, to, ir_program);
            let new_to = subtitute_type_signature(old_to, from, to, ir_program);
            IrTypeSignature::Function(new_from, new_to)
        }
        IrTypeSignature::Wildcard => {
            return *source;
        }
        IrTypeSignature::Named(name, type_id, items) => {
            let new_items: Vec<_> = items
                .iter()
                .map(|item| subtitute_type_signature(item, from, to, ir_program))
                .collect();
            IrTypeSignature::Named(name.clone(), *type_id, new_items)
        }
        IrTypeSignature::TypeArgument(index, _, _) => {
            if *index == from {
                return *to;
            } else {
                return *source;
            }
        }
        IrTypeSignature::Tuple(items) => {
            let new_items: Vec<_> = items
                .iter()
                .map(|item| subtitute_type_signature(item, from, to, ir_program))
                .collect();
            IrTypeSignature::Tuple(new_items)
        }
        IrTypeSignature::Variant(_, _) => unreachable!(),
        IrTypeSignature::Ref(item) => {
            IrTypeSignature::Ref(subtitute_type_signature(item, from, to, ir_program))
        }
    };
    let id = ir_program.type_signatures.get_id();
    let type_info = ItemInfo::new(new_signature, info.location_id);
    ir_program.type_signatures.add_item(id, type_info);
    id
}

pub fn collect_type_args(
    type_signature_id: &TypeSignatureId,
    program: &Program,
    type_args: &mut BTreeMap<String, LocationId>,
) {
    let info = program.type_signatures.get(type_signature_id);
    match &info.item {
        AstTypeSignature::Function(from, to) => {
            collect_type_args(from, program, type_args);
            collect_type_args(to, program, type_args);
        }
        AstTypeSignature::Named(_, items) => {
            for item in items {
                collect_type_args(item, program, type_args);
            }
        }
        AstTypeSignature::Tuple(items) => {
            for item in items {
                collect_type_args(item, program, type_args);
            }
        }
        AstTypeSignature::TypeArg(name) => {
            type_args.insert(name.clone(), info.location_id);
        }
        AstTypeSignature::Variant(_, items) => {
            for item in items {
                collect_type_args(item, program, type_args);
            }
        }
        AstTypeSignature::Wildcard => {}
        AstTypeSignature::Ref(item) => {
            collect_type_args(item, program, type_args);
        }
    }
}

pub fn process_class_type_signature(
    type_signature_id: &TypeSignatureId,
    program: &Program,
    ir_program: &mut IrProgram,
    type_arg_resolver: &mut TypeArgResolver,
    errors: &mut Vec<ResolverError>,
    class_id: ClassId,
) -> Option<(IrTypeSignatureId, String)> {
    let info = program.type_signatures.get(type_signature_id);
    match &info.item {
        AstTypeSignature::TypeArg(name) => {
            let constraints = vec![class_id];
            let index =
                type_arg_resolver.add_explicit(name.clone(), constraints.clone(), info.location_id);
            let ir_type_signature = IrTypeSignature::TypeArgument(index, name.clone(), constraints);
            let id = ir_program.type_signatures.get_id();
            let type_info = ItemInfo::new(ir_type_signature, info.location_id);
            ir_program.type_signatures.add_item(id, type_info);
            Some((id, name.clone()))
        }
        _ => {
            let err = ResolverError::InvalidClassArgument(info.location_id);
            errors.push(err);
            None
        }
    }
}

pub fn process_type_signature(
    type_signature_id: &TypeSignatureId,
    program: &Program,
    ir_program: &mut IrProgram,
    module: &Module,
    type_arg_resolver: &mut TypeArgResolver,
    errors: &mut Vec<ResolverError>,
) -> Option<IrTypeSignatureId> {
    let info = program.type_signatures.get(type_signature_id);
    let type_signature = &info.item;
    let location_id = info.location_id;
    let ir_type_signature = match type_signature {
        AstTypeSignature::TypeArg(name) => {
            if let Some(info) = type_arg_resolver.resolve_arg(name) {
                IrTypeSignature::TypeArgument(info.index, name.clone(), info.constraints)
            } else {
                let error = ResolverError::UnknownTypeArg(name.clone(), location_id);
                errors.push(error);
                return None;
            }
        }
        AstTypeSignature::Variant(name, items) => {
            let mut item_ids = Vec::new();
            for item in items {
                match process_type_signature(
                    item,
                    program,
                    ir_program,
                    module,
                    type_arg_resolver,
                    errors,
                ) {
                    Some(id) => {
                        item_ids.push(id);
                    }
                    None => {
                        return None;
                    }
                }
            }
            IrTypeSignature::Variant(name.clone(), item_ids)
        }
        AstTypeSignature::Named(name, named_args) => {
            return process_named_type(
                name,
                named_args,
                location_id,
                program,
                ir_program,
                module,
                type_arg_resolver,
                errors,
            );
        }
        AstTypeSignature::Tuple(items) => {
            let mut item_ids = Vec::new();
            for item in items {
                match process_type_signature(
                    item,
                    program,
                    ir_program,
                    module,
                    type_arg_resolver,
                    errors,
                ) {
                    Some(id) => {
                        item_ids.push(id);
                    }
                    None => {}
                }
            }
            IrTypeSignature::Tuple(item_ids)
        }
        AstTypeSignature::Function(from, to) => {
            let ir_from = match process_type_signature(
                from,
                program,
                ir_program,
                module,
                type_arg_resolver,
                errors,
            ) {
                Some(id) => id,
                None => ir_program.type_signatures.get_id(),
            };
            let ir_to = match process_type_signature(
                to,
                program,
                ir_program,
                module,
                type_arg_resolver,
                errors,
            ) {
                Some(id) => id,
                None => ir_program.type_signatures.get_id(),
            };
            IrTypeSignature::Function(ir_from, ir_to)
        }
        AstTypeSignature::Wildcard => IrTypeSignature::Wildcard,
        AstTypeSignature::Ref(item) => {
            let ir_item = process_type_signature(
                item,
                program,
                ir_program,
                module,
                type_arg_resolver,
                errors,
            );
            if let Some(ir_item) = ir_item {
                IrTypeSignature::Ref(ir_item)
            } else {
                return None;
            }
        }
    };
    let id = ir_program.type_signatures.get_id();
    let type_info = ItemInfo::new(ir_type_signature, location_id);
    ir_program.type_signatures.add_item(id, type_info);
    return Some(id);
}
