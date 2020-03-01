use crate::environment::Environment;
use crate::error::Error;
use crate::error::ResolverError;
use crate::export_processor::process_exports;
use crate::expr_processor::process_expr;
use crate::import::ImportedItemInfo;
use crate::import::Namespace;
use crate::import_processor::process_imports;
use crate::item::DataMember;
use crate::item::Item;
use crate::item::RecordField;
use crate::item::Variant;
use crate::lambda_arg_shifter::LambdaArgShifter;
use crate::lambda_helper::LambdaHelper;
use crate::module::Module;
use crate::type_arg_resolver::TypeArgResolver;
use crate::type_processor::collect_type_args;
use crate::type_processor::process_class_type_signature;
use crate::type_processor::process_type_signature;
use crate::type_processor::subtitute_type_signature;
use siko_constants::get_auto_derivable_classes;
use siko_constants::STD_OPS_MODULE_NAME;
use siko_ir::class::Class as IrClass;
use siko_ir::class::ClassId as IrClassId;
use siko_ir::class::ClassMember as IrClassMember;
use siko_ir::class::Instance as IrInstance;
use siko_ir::class::InstanceMember as IrInstanceMember;
use siko_ir::data::Adt;
use siko_ir::data::DerivedClass as IrDerivedClass;
use siko_ir::data::Record;
use siko_ir::data::RecordField as IrRecordField;
use siko_ir::data::TypeDef;
use siko_ir::data::TypeDefId;
use siko_ir::data::Variant as IrVariant;
use siko_ir::data::VariantItem;
use siko_ir::function::Function as IrFunction;
use siko_ir::function::FunctionId as IrFunctionId;
use siko_ir::function::FunctionInfo;
use siko_ir::function::NamedFunctionInfo;
use siko_ir::function::NamedFunctionKind;
use siko_ir::function::RecordConstructorInfo;
use siko_ir::function::VariantConstructorInfo;
use siko_ir::program::Program as IrProgram;
use siko_ir::type_signature::TypeSignature;
use siko_ir::type_signature::TypeSignatureId;
use siko_ir::type_var_generator::TypeVarGenerator;
use siko_ir::walker::walk_expr;
use siko_location_info::location_id::LocationId;
use siko_syntax::class::ClassId as AstClassId;
use siko_syntax::class::Instance as AstInstance;
use siko_syntax::data::AdtId;
use siko_syntax::data::DerivedClass as AstDerivedClass;
use siko_syntax::data::RecordId;
use siko_syntax::function::Function as AstFunction;
use siko_syntax::function::FunctionBody as AstFunctionBody;
use siko_syntax::function::FunctionId as AstFunctionId;
use siko_syntax::function::FunctionType as AstFunctionType;
use siko_syntax::function::FunctionTypeId as AstFunctionTypeId;
use siko_syntax::module::Module as AstModule;
use siko_syntax::program::Program;
use siko_util::RcCounter;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

fn check_function_and_function_type_consistency(
    functions: &BTreeMap<String, Vec<AstFunctionId>>,
    function_types: &BTreeMap<String, Vec<AstFunctionTypeId>>,
) -> (
    Vec<AstFunctionId>,
    Vec<AstFunctionTypeId>,
    BTreeMap<String, Vec<AstFunctionId>>,
    BTreeMap<String, Vec<AstFunctionTypeId>>,
) {
    let function_name_set: BTreeSet<_> = functions.keys().collect();
    let function_type_name_set: BTreeSet<_> = function_types.keys().collect();
    let mut functions_without_types: Vec<AstFunctionId> = Vec::new();
    let mut function_types_without_functions: Vec<AstFunctionTypeId> = Vec::new();
    for n in function_name_set.difference(&function_type_name_set) {
        let ids = functions.get(*n).unwrap();
        functions_without_types.extend(ids);
    }
    for n in function_type_name_set.difference(&function_name_set) {
        let ids = function_types.get(*n).unwrap();
        function_types_without_functions.extend(ids);
    }
    let mut conflicting_functions = functions.clone();
    for (name, ids) in functions {
        if ids.len() == 1 {
            conflicting_functions.remove(name);
        }
    }
    let mut conflicting_function_types = function_types.clone();
    for (name, ids) in function_types {
        if ids.len() == 1 {
            conflicting_function_types.remove(name);
        }
    }
    (
        functions_without_types,
        function_types_without_functions,
        conflicting_functions,
        conflicting_function_types,
    )
}

#[derive(Debug)]
pub struct Resolver {
    modules: BTreeMap<String, Module>,
    type_var_generator: TypeVarGenerator,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            modules: BTreeMap::new(),
            type_var_generator: TypeVarGenerator::new(RcCounter::new()),
        }
    }

    fn register_module(
        &mut self,
        ast_module: &AstModule,
        modules: &mut BTreeMap<String, Vec<Module>>,
    ) {
        let module = Module::new(
            ast_module.id,
            ast_module.name.clone(),
            ast_module.location_id,
        );

        let mods = modules
            .entry(ast_module.name.clone())
            .or_insert_with(Vec::new);
        mods.push(module);
    }

    fn process_module_conflicts(
        &mut self,
        modules: BTreeMap<String, Vec<Module>>,
    ) -> Result<(), Error> {
        let mut errors = Vec::new();
        let mut module_conflicts = BTreeMap::new();

        for (name, modules) in modules.iter() {
            if modules.len() > 1 {
                let ids = modules.iter().map(|m| m.location_id).collect();
                module_conflicts.insert(name.clone(), ids);
            }
        }

        if !module_conflicts.is_empty() {
            let e = ResolverError::ModuleConflict(module_conflicts);
            errors.push(e);
        }

        if errors.is_empty() {
            for (name, mods) in modules {
                let modules: Vec<Module> = mods;
                self.modules
                    .insert(name, modules.into_iter().next().expect("Empty module set"));
            }
            Ok(())
        } else {
            return Err(Error::resolve_err(errors));
        }
    }

    fn process_items_and_types(
        &mut self,
        program: &Program,
        errors: &mut Vec<ResolverError>,
        ir_program: &mut IrProgram,
    ) {
        for (_, module) in &mut self.modules {
            let ast_module = program.modules.get(&module.id);
            for record_id in &ast_module.records {
                let record = program.records.get(record_id);
                let ir_typedef_id = ir_program.typedefs.get_id();
                let ir_ctor_id = ir_program.functions.get_id();
                let record_ctor_info = RecordConstructorInfo {
                    type_id: ir_typedef_id,
                };
                let arg_locations: Vec<_> = record
                    .fields
                    .iter()
                    .map(|field_id| program.record_fields.get(field_id).location_id)
                    .collect();
                let ir_ctor_function = IrFunction {
                    id: ir_ctor_id,
                    arg_count: arg_locations.len(),
                    arg_locations: arg_locations,
                    info: FunctionInfo::RecordConstructor(record_ctor_info),
                };
                ir_program.functions.add_item(ir_ctor_id, ir_ctor_function);

                let ir_record = Record {
                    name: record.name.clone(),
                    module: ast_module.name.clone(),
                    id: ir_typedef_id,
                    type_args: (0..record.type_args.len()).collect(),
                    fields: Vec::new(),
                    constructor: ir_ctor_id,
                    location_id: record.location_id,
                    external: record.external,
                    derived_classes: Vec::new(),
                };

                let typedef = TypeDef::Record(ir_record);
                ir_program.typedefs.add_item(ir_typedef_id, typedef);
                let item = Item::Record(*record_id, ir_typedef_id);
                module.add_item(record.name.clone(), item);
                for (index, field_id) in record.fields.iter().enumerate() {
                    let field = program.record_fields.get(field_id);
                    let record_field = RecordField {
                        field_id: field.id,
                        record_id: *record_id,
                        ir_typedef_id: ir_typedef_id,
                        index: index,
                    };
                    let member = DataMember::RecordField(record_field);
                    module.add_member(field.name.clone(), member);
                }
            }
            for adt_id in &ast_module.adts {
                let adt = program.adts.get(adt_id);
                let ir_typedef_id = ir_program.typedefs.get_id();
                let ir_adt = Adt {
                    name: adt.name.clone(),
                    module: ast_module.name.clone(),
                    id: ir_typedef_id,
                    type_args: (0..adt.type_args.len()).collect(),
                    variants: Vec::new(),
                    derived_classes: Vec::new(),
                };
                let typedef = TypeDef::Adt(ir_adt);
                ir_program.typedefs.add_item(ir_typedef_id, typedef);
                let item = Item::Adt(*adt_id, ir_typedef_id);
                module.add_item(adt.name.clone(), item);
                for (index, variant_id) in adt.variants.iter().enumerate() {
                    let ast_variant = program.variants.get(variant_id);
                    let item = Item::Variant(*adt_id, *variant_id, ir_typedef_id, index);
                    module.add_item(ast_variant.name.clone(), item);
                    let variant = Variant {
                        variant_id: *variant_id,
                        adt_id: *adt_id,
                    };
                    let member = DataMember::Variant(variant);
                    module.add_member(ast_variant.name.clone(), member);
                }
            }
            for (_, function_ids) in &ast_module.functions {
                for function_id in function_ids {
                    let function = program.functions.get(function_id);
                    let ir_function_id = ir_program.functions.get_id();
                    let item = Item::Function(function.id, ir_function_id);
                    module.add_item(function.name.clone(), item);
                }
            }
            for class_id in &ast_module.classes {
                let ir_class_id = ir_program.classes.get_id();
                let class = program.classes.get(class_id);
                let item = Item::Class(class.id, ir_class_id);
                module.add_item(class.name.clone(), item);
                let mut members = BTreeMap::new();
                for (_, member_function_type_ids) in &class.member_function_types {
                    for member_function_type_id in member_function_type_ids {
                        let ir_class_member_id = ir_program.class_members.get_id();
                        let class_member = program.function_types.get(member_function_type_id);
                        let item = Item::ClassMember(
                            class.id,
                            *member_function_type_id,
                            ir_class_member_id,
                        );
                        module.add_item(class_member.name.clone(), item);
                        members.insert(class_member.name.clone(), ir_class_member_id);
                    }
                }
                let auto_derivable_classes: Vec<_> = get_auto_derivable_classes();
                let auto_derivable = module.name == STD_OPS_MODULE_NAME
                    && auto_derivable_classes.contains(&class.name.as_ref());
                let ir_class = IrClass {
                    id: ir_class_id,
                    name: class.name.clone(),
                    module: module.name.clone(),
                    type_signature: None,
                    constraints: Vec::new(),
                    members: members,
                    location_id: class.location_id,
                    auto_derivable: auto_derivable,
                };
                ir_program.classes.add_item(ir_class_id, ir_class);
            }
        }

        for (_, module) in &self.modules {
            for (name, items) in &module.items {
                if items.len() > 1 {
                    let mut locations = Vec::new();
                    let mut adt_found = false;
                    let mut variant_found = false;
                    for item in items {
                        match item {
                            Item::Function(id, _) => {
                                let function = program.functions.get(id);
                                locations.push(function.location_id);
                            }
                            Item::Record(id, _) => {
                                let record = program.records.get(id);
                                locations.push(record.location_id);
                            }
                            Item::Adt(id, _) => {
                                let adt = program.adts.get(id);
                                locations.push(adt.location_id);
                                adt_found = true;
                            }
                            Item::Variant(_, id, _, _) => {
                                let variant = program.variants.get(id);
                                locations.push(variant.location_id);
                                variant_found = true;
                            }
                            Item::Class(id, _) => {
                                let class = program.classes.get(id);
                                locations.push(class.location_id);
                            }
                            Item::ClassMember(_, id, _) => {
                                let class_member = program.function_types.get(id);
                                locations.push(class_member.location_id);
                            }
                        }
                    }
                    if items.len() == 2 && adt_found && variant_found {
                        continue;
                    }
                    let err = ResolverError::InternalModuleConflicts(
                        module.name.clone(),
                        name.clone(),
                        locations,
                    );
                    errors.push(err);
                }
            }
        }

        for (_, record) in &program.records.items {
            let mut field_names = BTreeSet::new();
            for field_id in &record.fields {
                let field = program.record_fields.get(field_id);
                if !field_names.insert(field.name.clone()) {
                    let err = ResolverError::RecordFieldNotUnique(
                        record.name.clone(),
                        field.name.clone(),
                        record.location_id,
                    );
                    errors.push(err);
                }
            }
        }

        for (_, adt) in &program.adts.items {
            let mut variant_names = BTreeSet::new();
            for variant_id in &adt.variants {
                let variant = program.variants.get(variant_id);
                if !variant_names.insert(variant.name.clone()) {
                    let err = ResolverError::VariantNotUnique(
                        adt.name.clone(),
                        variant.name.clone(),
                        adt.location_id,
                    );
                    errors.push(err);
                }
            }
        }
    }

    fn process_function_type(
        &self,
        function_type: &AstFunctionType,
        module: &Module,
        program: &Program,
        ir_program: &mut IrProgram,
        errors: &mut Vec<ResolverError>,
    ) -> (Option<TypeSignatureId>, TypeArgResolver) {
        let mut type_arg_resolver = TypeArgResolver::new(self.type_var_generator.clone());

        for (type_arg, location_id) in function_type.type_args.iter() {
            type_arg_resolver.add_explicit(type_arg.clone(), Vec::new(), *location_id);
        }

        for constraint in &function_type.constraints {
            if let Some(ir_class_id) = self.lookup_class(
                &constraint.class_name,
                constraint.location_id,
                module,
                errors,
            ) {
                type_arg_resolver.add_constraint(&constraint.arg, ir_class_id);
            }
        }

        let result = process_type_signature(
            &function_type.type_signature_id,
            program,
            ir_program,
            module,
            &mut type_arg_resolver,
            errors,
        );

        let unused_args = type_arg_resolver.collect_unused_args();

        for (unused_arg, location_id) in unused_args {
            let err = ResolverError::UnusedTypeArgument(unused_arg, location_id);
            errors.push(err);
        }

        (result, type_arg_resolver)
    }

    fn process_function(
        &self,
        program: &Program,
        ir_program: &mut IrProgram,
        function: &AstFunction,
        ir_function_id: IrFunctionId,
        module: &Module,
        errors: &mut Vec<ResolverError>,
        type_signature_id: Option<TypeSignatureId>,
        type_arg_resolver: &mut TypeArgResolver,
        kind: NamedFunctionKind,
    ) {
        let mut body = None;

        if let AstFunctionBody::Expr(id) = function.body {
            let mut environment = Environment::new();
            let mut arg_names = BTreeSet::new();
            let mut conflicting_names = BTreeSet::new();
            for (index, arg) in function.args.iter().enumerate() {
                if !arg_names.insert(arg.0.clone()) {
                    conflicting_names.insert(arg.0.clone());
                }
                environment.add_arg(arg.0.clone(), ir_function_id, index);
            }
            if !conflicting_names.is_empty() {
                let err = ResolverError::ArgumentConflict(
                    conflicting_names.into_iter().collect(),
                    function.location_id.clone(),
                );
                errors.push(err);
            }
            let host_function = format!("{}/{}", module.name, function.name);
            let lambda_helper = LambdaHelper::new(
                0,
                host_function,
                LambdaHelper::new_counter(),
                ir_function_id,
                ir_function_id,
                None,
            );
            let body_id = process_expr(
                id,
                program,
                module,
                &mut environment,
                ir_program,
                errors,
                lambda_helper,
                type_arg_resolver,
                None,
            );
            body = Some(body_id);
        }

        let named_info = NamedFunctionInfo {
            body: body,
            name: function.name.clone(),
            module: module.name.clone(),
            type_signature: type_signature_id,
            location_id: function.location_id,
            kind: kind,
        };

        let arg_locations: Vec<_> = function.args.iter().map(|arg| arg.1).collect();

        let ir_function = IrFunction {
            id: ir_function_id,
            arg_count: arg_locations.len(),
            arg_locations: arg_locations,
            info: FunctionInfo::NamedFunction(named_info),
        };
        ir_program.functions.add_item(ir_function_id, ir_function);
    }

    fn process_adt(
        &self,
        program: &Program,
        ir_program: &mut IrProgram,
        adt_id: &AdtId,
        ir_typedef_id: TypeDefId,
        module: &Module,
        errors: &mut Vec<ResolverError>,
    ) {
        let adt = program.adts.get(adt_id);
        let mut type_signature_ids = Vec::new();
        for variant_id in &adt.variants {
            let variant = program.variants.get(&variant_id);
            type_signature_ids.push(variant.type_signature_id);
        }

        let mut type_arg_resolver = TypeArgResolver::new(self.type_var_generator.clone());
        {
            let ir_adt = ir_program.typedefs.get_mut(&ir_typedef_id).get_mut_adt();
            ir_adt.type_args.clear();
            for (type_arg, location_id) in adt.type_args.iter() {
                let index =
                    type_arg_resolver.add_explicit(type_arg.clone(), Vec::new(), *location_id);
                ir_adt.type_args.push(index);
            }
        }

        let result: Vec<_> = type_signature_ids
            .iter()
            .map(|id| {
                process_type_signature(
                    id,
                    program,
                    ir_program,
                    module,
                    &mut type_arg_resolver,
                    errors,
                )
            })
            .collect();

        let unused_args = type_arg_resolver.collect_unused_args();

        for (unused_arg, location_id) in unused_args {
            let err = ResolverError::UnusedTypeArgument(unused_arg, location_id);
            errors.push(err);
        }

        if errors.is_empty() {
            let mut ir_variants = Vec::new();
            for (index, _) in adt.variants.iter().enumerate() {
                let ir_typesignature_id = result[index].expect("type signature missing");
                let signature = ir_program
                    .type_signatures
                    .get(&ir_typesignature_id)
                    .item
                    .clone();
                if let TypeSignature::Variant(name, items) = signature {
                    let items: Vec<_> = items
                        .iter()
                        .map(|i| VariantItem {
                            type_signature_id: *i,
                        })
                        .collect();
                    let ir_ctor_id = ir_program.functions.get_id();
                    let variant_ctor_info = VariantConstructorInfo {
                        type_id: ir_typedef_id,
                        index: index,
                    };
                    let arg_locations: Vec<_> = items
                        .iter()
                        .map(|item| {
                            ir_program
                                .type_signatures
                                .get(&item.type_signature_id)
                                .location_id
                        })
                        .collect();
                    let ir_ctor_function = IrFunction {
                        id: ir_ctor_id,
                        arg_count: arg_locations.len(),
                        arg_locations: arg_locations,
                        info: FunctionInfo::VariantConstructor(variant_ctor_info),
                    };
                    ir_program.functions.add_item(ir_ctor_id, ir_ctor_function);

                    let ir_variant = IrVariant {
                        name: name.clone(),
                        items: items,
                        type_signature_id: ir_typesignature_id,
                        constructor: ir_ctor_id,
                    };

                    ir_variants.push(ir_variant);
                } else {
                    panic!("Expected variant, found {:?}", signature);
                }
            }

            let ir_adt = ir_program.typedefs.get_mut(&ir_typedef_id).get_mut_adt();
            ir_adt.variants = ir_variants;
            ir_adt.derived_classes =
                self.process_derived_classes(&adt.derived_classes, module, errors);
        }
    }

    fn process_record(
        &self,
        program: &Program,
        ir_program: &mut IrProgram,
        record_id: &RecordId,
        ir_typedef_id: TypeDefId,
        module: &Module,
        errors: &mut Vec<ResolverError>,
    ) {
        let record = program.records.get(record_id);
        let mut type_signature_ids = Vec::new();
        for field_id in &record.fields {
            let field = program.record_fields.get(field_id);
            type_signature_ids.push(field.type_signature_id);
        }

        let mut type_arg_resolver = TypeArgResolver::new(self.type_var_generator.clone());
        {
            let ir_record = ir_program.typedefs.get_mut(&ir_typedef_id).get_mut_record();
            ir_record.type_args.clear();
            for (type_arg, location_id) in record.type_args.iter() {
                let index =
                    type_arg_resolver.add_explicit(type_arg.clone(), Vec::new(), *location_id);
                ir_record.type_args.push(index);
            }
        }

        let result: Vec<_> = type_signature_ids
            .iter()
            .map(|id| {
                process_type_signature(
                    id,
                    program,
                    ir_program,
                    module,
                    &mut type_arg_resolver,
                    errors,
                )
            })
            .collect();

        if !record.external {
            let unused_args = type_arg_resolver.collect_unused_args();
            for (unused_arg, location_id) in unused_args {
                let err = ResolverError::UnusedTypeArgument(unused_arg, location_id);
                errors.push(err);
            }
        }

        if errors.is_empty() {
            let mut ir_fields = Vec::new();
            for (index, field_id) in record.fields.iter().enumerate() {
                let field = program.record_fields.get(field_id);
                let ir_typesignature_id = result[index].expect("type signature missing");
                let ir_field = IrRecordField {
                    name: field.name.clone(),
                    type_signature_id: ir_typesignature_id,
                };
                ir_fields.push(ir_field);
            }

            let ir_record = ir_program.typedefs.get_mut(&ir_typedef_id).get_mut_record();
            ir_record.fields = ir_fields;
            ir_record.derived_classes =
                self.process_derived_classes(&record.derived_classes, module, errors);
        }
    }

    fn process_derived_classes(
        &self,
        derived_classes: &Vec<AstDerivedClass>,
        module: &Module,
        errors: &mut Vec<ResolverError>,
    ) -> Vec<IrDerivedClass> {
        let mut ir_derived_classes = Vec::new();
        for derived_class in derived_classes {
            if let Some(ir_class_id) = self.lookup_class(
                &derived_class.name,
                derived_class.location_id,
                module,
                errors,
            ) {
                ir_derived_classes.push(IrDerivedClass {
                    class_id: ir_class_id,
                    location_id: derived_class.location_id,
                });
            }
        }
        ir_derived_classes
    }

    fn lookup_class(
        &self,
        class_name: &String,
        location_id: LocationId,
        module: &Module,
        errors: &mut Vec<ResolverError>,
    ) -> Option<IrClassId> {
        match module.imported_items.get(class_name) {
            Some(items) => match ImportedItemInfo::resolve_ambiguity(items, Namespace::Type) {
                None => {
                    let err = ResolverError::AmbiguousName(class_name.clone(), location_id);
                    errors.push(err);
                    return None;
                }
                Some(item) => match item.item {
                    Item::Class(_, ir_class_id) => {
                        return Some(ir_class_id);
                    }
                    _ => {
                        let err = ResolverError::NotAClassName(class_name.clone(), location_id);
                        errors.push(err);
                    }
                },
            },
            None => {
                let err = ResolverError::NotAClassName(class_name.clone(), location_id);
                errors.push(err);
            }
        }
        None
    }

    fn process_class(
        &self,
        program: &Program,
        ir_program: &mut IrProgram,
        class_id: &AstClassId,
        ir_class_id: &IrClassId,
        module: &Module,
        errors: &mut Vec<ResolverError>,
    ) {
        let class = program.classes.get(class_id);

        let mut type_arg_resolver = TypeArgResolver::new(self.type_var_generator.clone());

        let (class_type_signature_id, class_arg) = if let Some(class_type_signature_id) =
            process_class_type_signature(
                &class.arg,
                program,
                ir_program,
                &mut type_arg_resolver,
                errors,
                *ir_class_id,
            ) {
            class_type_signature_id
        } else {
            return;
        };

        let mut ir_constraints = Vec::new();

        for constraint in &class.constraints {
            if let Some(ir_class_id) = self.lookup_class(
                &constraint.class_name,
                constraint.location_id,
                module,
                errors,
            ) {
                ir_constraints.push(ir_class_id);
                if !type_arg_resolver.add_constraint(&constraint.arg, ir_class_id) {
                    let err = ResolverError::InvalidArgumentInTypeClassConstraint(
                        constraint.arg.clone(),
                        constraint.location_id,
                    );
                    errors.push(err);
                }
            }
        }

        let ir_class = ir_program.classes.get_mut(ir_class_id);
        ir_class.constraints = ir_constraints;
        ir_class.type_signature = Some(class_type_signature_id);

        let (functions_without_types, _, conflicting_functions, _) =
            check_function_and_function_type_consistency(
                &class.member_functions,
                &class.member_function_types,
            );

        for id in functions_without_types {
            let function = program.functions.get(&id);
            let err = ResolverError::DefaultClassMemberWithoutType(
                class.name.clone(),
                function.name.clone(),
                function.location_id,
            );
            errors.push(err);
        }

        for (name, ids) in conflicting_functions {
            let locations: Vec<_> = ids
                .iter()
                .map(|id| program.functions.get(id).location_id)
                .collect();
            let err = ResolverError::ConflictingDefaultClassMember(
                class.name.clone(),
                name.clone(),
                locations,
            );
            errors.push(err);
        }

        let ir_class = ir_program.classes.get(ir_class_id).clone();
        for (name, function_type_ids) in &class.member_function_types {
            assert_eq!(function_type_ids.len(), 1);
            let function_type_id = &function_type_ids[0];
            let class_member = program.function_types.get(function_type_id);
            let ir_class_member_id = *ir_class.members.get(name).expect("Class member not found");
            let signature_type_args: BTreeSet<_> =
                class_member.type_args.iter().map(|i| i.0.clone()).collect();
            if !signature_type_args.contains(&class_arg) {
                let err = ResolverError::ClassMemberTypeArgMissing(
                    class_member.name.clone(),
                    class_arg.clone(),
                    class_member.location_id,
                );
                errors.push(err);
                continue;
            }

            let mut type_arg_resolver = type_arg_resolver.clone();

            for (type_arg, location) in &class_member.type_args {
                if *type_arg == class_arg {
                    continue;
                }
                type_arg_resolver.add_explicit(type_arg.clone(), vec![], *location);
            }

            for constraint in &class_member.constraints {
                if let Some(ir_class_id) = self.lookup_class(
                    &constraint.class_name,
                    constraint.location_id,
                    module,
                    errors,
                ) {
                    type_arg_resolver.add_constraint(&constraint.arg, ir_class_id);
                }
            }

            let result = process_type_signature(
                &class_member.type_signature_id,
                program,
                ir_program,
                module,
                &mut type_arg_resolver,
                errors,
            );

            if errors.is_empty() {
                let default_implementation =
                    if let Some(impls) = class.member_functions.get(&class_member.name) {
                        assert_eq!(impls.len(), 1);
                        let impl_id = impls[0];
                        let ir_function_id = ir_program.functions.get_id();
                        let function = &program.functions.get(&impl_id);
                        self.process_function(
                            program,
                            ir_program,
                            &function,
                            ir_function_id,
                            module,
                            errors,
                            result,
                            &mut type_arg_resolver,
                            NamedFunctionKind::DefaultClassMember(ir_class_member_id),
                        );
                        Some(ir_function_id)
                    } else {
                        None
                    };
                let ir_class_member = IrClassMember {
                    id: ir_class_member_id,
                    class_id: *ir_class_id,
                    name: class_member.name.clone(),
                    class_type_signature: class_type_signature_id,
                    type_signature: result.expect("Type signature not found"),
                    default_implementation: default_implementation,
                    location_id: class_member.location_id,
                };
                ir_program
                    .class_members
                    .add_item(ir_class_member_id, ir_class_member);
            }
        }
    }

    fn get_instance_member_type_signature(
        &self,
        ir_program: &mut IrProgram,
        ir_class_member: &IrClassMember,
        instance_type_signature: TypeSignatureId,
    ) -> TypeSignatureId {
        let item = &ir_program
            .type_signatures
            .get(&ir_class_member.class_type_signature)
            .item;
        match item {
            TypeSignature::TypeArgument(index, _, _) => subtitute_type_signature(
                &ir_class_member.type_signature,
                *index,
                &instance_type_signature,
                ir_program,
            ),
            _ => panic!("Invalid class type signature"),
        }
    }

    fn process_instance(
        &self,
        instance: &AstInstance,
        program: &Program,
        ir_program: &mut IrProgram,
        module: &Module,
        errors: &mut Vec<ResolverError>,
    ) {
        let mut type_arg_resolver = TypeArgResolver::new(self.type_var_generator.clone());

        let mut type_args = BTreeMap::new();

        collect_type_args(&instance.type_signature_id, program, &mut type_args);

        for (type_arg, location_id) in type_args {
            type_arg_resolver.add_explicit(type_arg, Vec::new(), location_id);
        }

        for constraint in &instance.constraints {
            if let Some(ir_class_id) = self.lookup_class(
                &constraint.class_name,
                constraint.location_id,
                module,
                errors,
            ) {
                if !type_arg_resolver.add_constraint(&constraint.arg, ir_class_id) {
                    let err = ResolverError::InvalidTypeArgInInstanceConstraint(
                        constraint.arg.clone(),
                        constraint.location_id,
                    );
                    errors.push(err);
                }
            }
        }

        let ir_class_id =
            match self.lookup_class(&instance.class_name, instance.location_id, module, errors) {
                Some(ir_class_id) => ir_class_id,
                None => {
                    return;
                }
            };
        let result = process_type_signature(
            &instance.type_signature_id,
            program,
            ir_program,
            module,
            &mut type_arg_resolver,
            errors,
        );
        let ir_class = ir_program.classes.get(&ir_class_id).clone();

        let (
            _,
            function_types_without_functions,
            conflicting_functions,
            conflicting_function_types,
        ) = check_function_and_function_type_consistency(
            &instance.member_functions,
            &instance.member_function_types,
        );

        for id in function_types_without_functions {
            let function_type = program.function_types.get(&id);
            let err = ResolverError::InstanceMemberWithoutImplementation(
                function_type.name.clone(),
                function_type.location_id,
            );
            errors.push(err);
        }

        for (name, ids) in conflicting_functions {
            let locations: Vec<_> = ids
                .iter()
                .map(|id| program.functions.get(id).location_id)
                .collect();
            let err = ResolverError::ConflictingInstanceMemberFunction(name.clone(), locations);
            errors.push(err);
        }

        for (name, ids) in conflicting_function_types {
            let locations: Vec<_> = ids
                .iter()
                .map(|id| program.function_types.get(id).location_id)
                .collect();
            let err = ResolverError::ConflictingFunctionTypesInInstance(name.clone(), locations);
            errors.push(err);
        }

        if !errors.is_empty() {
            return;
        }

        if let Some(instance_type_signature) = result {
            let id = ir_program.instances.get_id();

            let mut members = BTreeMap::new();
            let mut implemented_members = BTreeSet::new();

            for (_, member_function_ids) in &instance.member_functions {
                assert_eq!(member_function_ids.len(), 1);
                let member_function_id = &member_function_ids[0];
                let function = program.functions.get(member_function_id);
                let member_name = &function.name;
                if let Some(class_member_id) = ir_class.members.get(member_name) {
                    let ir_function_id = ir_program.functions.get_id();
                    let ir_class_member = ir_program.class_members.get(class_member_id).clone();
                    let member_function_type_signature_id = if let Some(function_type) =
                        instance.member_function_types.get(member_name)
                    {
                        // TODO: the instance impl has its own type signature, compare it with the class member
                        unimplemented!()
                    } else {
                        self.get_instance_member_type_signature(
                            ir_program,
                            &ir_class_member,
                            instance_type_signature,
                        )
                    };
                    let ir_instance_member = IrInstanceMember {
                        type_signature: member_function_type_signature_id,
                        function_id: ir_function_id,
                    };
                    members.insert(member_name.clone(), ir_instance_member);
                    self.process_function(
                        program,
                        ir_program,
                        &function,
                        ir_function_id,
                        module,
                        errors,
                        Some(member_function_type_signature_id),
                        &mut type_arg_resolver,
                        NamedFunctionKind::InstanceMember(instance.name.clone()),
                    );
                } else {
                    let err = ResolverError::NotAClassMember(
                        member_name.clone(),
                        instance.class_name.clone(),
                        function.location_id,
                    );
                    errors.push(err);
                }
                implemented_members.insert(member_name.clone());
            }

            for (class_member, ir_class_member_id) in &ir_class.members {
                if !implemented_members.contains(class_member) {
                    let ir_class_member = ir_program.class_members.get(ir_class_member_id).clone();
                    if ir_class_member.default_implementation.is_none() {
                        let err = ResolverError::MissingClassMemberInInstance(
                            class_member.clone(),
                            instance.class_name.clone(),
                            instance.location_id,
                        );
                        errors.push(err);
                    }
                }
            }

            let ir_instance = IrInstance {
                id: id,
                name: instance.name.clone(),
                class_id: ir_class_id,
                type_signature: instance_type_signature,
                members: members,
                location_id: instance.location_id,
            };

            ir_program.instances.add_item(id, ir_instance);
        }
    }

    pub fn resolve(&mut self, program: &Program) -> Result<IrProgram, Error> {
        let mut errors = Vec::new();

        let mut modules = BTreeMap::new();

        for ast_module in program.modules.items.values() {
            self.register_module(ast_module, &mut modules);
        }

        self.process_module_conflicts(modules)?;

        let mut ir_program = IrProgram::new(self.type_var_generator.clone());

        self.process_items_and_types(program, &mut errors, &mut ir_program);

        if !errors.is_empty() {
            return Err(Error::resolve_err(errors));
        }

        process_exports(&mut self.modules, program, &mut errors);

        if !errors.is_empty() {
            return Err(Error::resolve_err(errors));
        }

        process_imports(&mut self.modules, program, &mut errors);

        if !errors.is_empty() {
            return Err(Error::resolve_err(errors));
        }

        for (_, module) in &self.modules {
            for (_, items) in &module.items {
                for item in items {
                    match item {
                        Item::Adt(ast_adt_id, ir_adt_id) => self.process_adt(
                            program,
                            &mut ir_program,
                            ast_adt_id,
                            *ir_adt_id,
                            module,
                            &mut errors,
                        ),
                        Item::Record(ast_record_id, ir_record_id) => self.process_record(
                            program,
                            &mut ir_program,
                            ast_record_id,
                            *ir_record_id,
                            module,
                            &mut errors,
                        ),
                        Item::Class(ast_class_id, ir_class_id) => self.process_class(
                            program,
                            &mut ir_program,
                            ast_class_id,
                            ir_class_id,
                            module,
                            &mut errors,
                        ),
                        _ => {}
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(Error::resolve_err(errors));
        }

        for (_, module) in &self.modules {
            let ast_module = program.modules.get(&module.id);
            let mut named_instances = BTreeSet::new();
            for instance_id in &ast_module.instances {
                let instance = program.instances.get(&instance_id);
                if let Some(name) = &instance.name {
                    if !named_instances.insert(name) {
                        let err = ResolverError::NamedInstancedNotUnique(
                            module.name.clone(),
                            name.clone(),
                            instance.location_id,
                        );
                        errors.push(err);
                    }
                }
                self.process_instance(instance, program, &mut ir_program, module, &mut errors);
            }
        }

        if !errors.is_empty() {
            return Err(Error::resolve_err(errors));
        }

        for (_, module) in &self.modules {
            let ast_module = program.modules.get(&module.id);
            let (_, function_types_without_functions, _, conflicting_function_types) =
                check_function_and_function_type_consistency(
                    &ast_module.functions,
                    &ast_module.function_types,
                );

            for id in function_types_without_functions {
                let function_type = program.function_types.get(&id);
                let err = ResolverError::FunctionTypeWithoutImplementationInModule(
                    module.name.clone(),
                    function_type.name.clone(),
                    function_type.location_id,
                );
                errors.push(err);
            }

            for (name, ids) in conflicting_function_types {
                let locations: Vec<_> = ids
                    .iter()
                    .map(|id| program.function_types.get(id).location_id)
                    .collect();
                let err = ResolverError::ConflictingFunctionTypesInModule(
                    module.name.clone(),
                    name.clone(),
                    locations,
                );
                errors.push(err);
            }
        }

        if !errors.is_empty() {
            return Err(Error::resolve_err(errors));
        }

        for (_, module) in &self.modules {
            for (_, items) in &module.items {
                for item in items {
                    match item {
                        Item::Function(ast_function_id, ir_function_id) => {
                            let function = program.functions.get(ast_function_id);
                            let ast_module = program.modules.get(&module.id);
                            let (type_signature_id, mut type_arg_resolver) =
                                if let Some(function_types) =
                                    ast_module.function_types.get(&function.name)
                                {
                                    assert_eq!(function_types.len(), 1);
                                    let function_type_id = function_types[0];
                                    let function_type =
                                        program.function_types.get(&function_type_id);
                                    self.process_function_type(
                                        function_type,
                                        module,
                                        program,
                                        &mut ir_program,
                                        &mut errors,
                                    )
                                } else {
                                    (None, TypeArgResolver::new(self.type_var_generator.clone()))
                                };
                            self.process_function(
                                program,
                                &mut ir_program,
                                &function,
                                *ir_function_id,
                                module,
                                &mut errors,
                                type_signature_id,
                                &mut type_arg_resolver,
                                NamedFunctionKind::Free,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(Error::resolve_err(errors));
        } else {
            let mut class_names = BTreeMap::new();
            for (class_id, class) in &ir_program.classes.items {
                class_names.insert(class.name.clone(), *class_id);
            }
            ir_program.class_names = class_names;
            let mut named_types = BTreeMap::new();
            for (id, typedef) in &ir_program.typedefs.items {
                match typedef {
                    TypeDef::Adt(adt) => {
                        let types = named_types
                            .entry(adt.module.clone())
                            .or_insert_with(|| BTreeMap::new());
                        types.insert(adt.name.clone(), *id);
                    }
                    TypeDef::Record(record) => {
                        let types = named_types
                            .entry(record.module.clone())
                            .or_insert_with(|| BTreeMap::new());
                        types.insert(record.name.clone(), *id);
                    }
                }
            }
            ir_program.named_types = named_types;
        }

        let mut lambdas = Vec::new();

        for (_, function) in ir_program.functions.items.iter() {
            match &function.info {
                FunctionInfo::Lambda(info) => {
                    let capture_count = function.arg_count - function.arg_locations.len();
                    lambdas.push((info.body, capture_count));
                }
                _ => {}
            }
        }

        for (body, capture_count) in lambdas {
            let mut shifter = LambdaArgShifter::new(&mut ir_program, capture_count);
            walk_expr(&body, &mut shifter);
        }

        Ok(ir_program)
    }
}
