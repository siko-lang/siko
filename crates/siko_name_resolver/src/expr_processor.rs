use crate::environment::Environment;
use crate::environment::NamedRef;
use crate::error::ResolverError;
use crate::import::ImportedItemInfo;
use crate::import::Namespace;
use crate::item::DataMember;
use crate::item::Item;
use crate::lambda_helper::LambdaHelper;
use crate::module::Module;
use crate::type_arg_resolver::TypeArgResolver;
use crate::type_processor::process_type_signature;
use siko_constants::BuiltinOperator;
use siko_ir::class::ClassMemberId as IrClassMemberId;
use siko_ir::data::TypeDefId;
use siko_ir::expr::Case as IrCase;
use siko_ir::expr::Expr as IrExpr;
use siko_ir::expr::ExprId as IrExprId;
use siko_ir::expr::FieldAccessInfo;
use siko_ir::expr::RecordFieldValueExpr;
use siko_ir::expr::RecordUpdateInfo;
use siko_ir::function::Function as IrFunction;
use siko_ir::function::FunctionId as IrFunctionId;
use siko_ir::function::FunctionInfo;
use siko_ir::function::LambdaInfo;
use siko_ir::pattern::BindGroup;
use siko_ir::pattern::Pattern as IrPattern;
use siko_ir::pattern::PatternId as IrPatternId;
use siko_ir::pattern::RangeKind as IrRangeKind;
use siko_ir::program::Program as IrProgram;
use siko_location_info::item::ItemInfo;
use siko_location_info::location_id::LocationId;
use siko_syntax::expr::Expr;
use siko_syntax::expr::ExprId;
use siko_syntax::pattern::Pattern;
use siko_syntax::pattern::PatternId;
use siko_syntax::pattern::RangeKind;
use siko_syntax::program::Program;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

enum PathResolveResult {
    VariableRef(IrExprId),
    FunctionRef(IrFunctionId),
    ClassMemberRef(IrClassMemberId),
}

fn resolve_item_path(
    path: &str,
    module: &Module,
    environment: &Environment,
    lambda_helper: LambdaHelper,
    program: &Program,
    ir_program: &mut IrProgram,
    id: ExprId,
    errors: &mut Vec<ResolverError>,
    location_id: LocationId,
) -> PathResolveResult {
    if let Some((named_ref, level)) = environment.get_ref(path) {
        let ir_expr = lambda_helper.process_named_ref(named_ref.clone(), level);
        let ir_expr_id = add_expr(ir_expr, id, ir_program, program);
        return PathResolveResult::VariableRef(ir_expr_id);
    }
    if let Some(items) = module.imported_items.get(path) {
        match ImportedItemInfo::resolve_ambiguity(items, Namespace::Value) {
            None => {
                let err = ResolverError::AmbiguousName(path.to_string(), location_id);
                errors.push(err);
                let ir_expr = IrExpr::Tuple(vec![]);
                let ir_expr_id = add_expr(ir_expr, id, ir_program, program);
                return PathResolveResult::VariableRef(ir_expr_id);
            }
            Some(item) => match item.item {
                Item::Function(_, ir_function_id) => {
                    return PathResolveResult::FunctionRef(ir_function_id);
                }
                Item::Record(_, ir_typedef_id) => {
                    let ir_record = ir_program.typedefs.get(&ir_typedef_id).get_record();
                    return PathResolveResult::FunctionRef(ir_record.constructor);
                }
                Item::Variant(_, _, ir_typedef_id, index) => {
                    let ir_adt = ir_program.typedefs.get(&ir_typedef_id).get_adt();
                    return PathResolveResult::FunctionRef(ir_adt.variants[index].constructor);
                }
                Item::ClassMember(_, _, ir_class_member_id) => {
                    return PathResolveResult::ClassMemberRef(ir_class_member_id);
                }
                _ => {}
            },
        }
    }
    let err = ResolverError::UnknownFunction(path.to_string(), location_id);
    errors.push(err);
    let ir_expr = IrExpr::Tuple(vec![]);
    let ir_expr_id = add_expr(ir_expr, id, ir_program, program);
    return PathResolveResult::VariableRef(ir_expr_id);
}

fn add_expr(
    ir_expr: IrExpr,
    ast_id: ExprId,
    ir_program: &mut IrProgram,
    program: &Program,
) -> IrExprId {
    let expr_id = ir_program.exprs.get_id();
    let location_id = program.exprs.get(&ast_id).location_id;
    let expr_info = ItemInfo::new(ir_expr, location_id);
    ir_program.exprs.add_item(expr_id, expr_info);
    expr_id
}

fn process_field_access(
    module: &Module,
    errors: &mut Vec<ResolverError>,
    name: String,
    location_id: LocationId,
) -> Vec<FieldAccessInfo> {
    let mut accesses = Vec::new();
    match module.imported_members.get(&name) {
        Some(members) => {
            for member in members {
                match &member.member {
                    DataMember::Variant(..) => {}
                    DataMember::RecordField(record_field) => {
                        let access = FieldAccessInfo {
                            record_id: record_field.ir_typedef_id,
                            index: record_field.index,
                        };
                        accesses.push(access);
                    }
                }
            }
        }
        None => {
            let err = ResolverError::UnknownFieldName(name.clone(), location_id);
            errors.push(err);
        }
    }
    accesses
}

fn resolve_pattern_type_constructor(
    name: &String,
    module: &Module,
    errors: &mut Vec<ResolverError>,
    location_id: LocationId,
    ids: Vec<IrPatternId>,
    irrefutable: bool,
) -> IrPattern {
    if let Some(items) = module.imported_items.get(name) {
        match ImportedItemInfo::resolve_ambiguity(items, Namespace::Value) {
            None => {
                let err = ResolverError::AmbiguousName(name.to_string(), location_id);
                errors.push(err);
                return IrPattern::Wildcard;
            }
            Some(item) => match item.item {
                Item::Function(_, _) => unreachable!(),
                Item::Record(_, ir_typedef_id) => {
                    return IrPattern::Record(ir_typedef_id, ids);
                }
                Item::Variant(_, _, ir_typedef_id, index) => {
                    if irrefutable {
                        let err = ResolverError::NotIrrefutablePattern(location_id);
                        errors.push(err);
                        return IrPattern::Wildcard;
                    } else {
                        return IrPattern::Variant(ir_typedef_id, index, ids);
                    }
                }
                _ => {}
            },
        }
    };
    let err = ResolverError::UnknownTypeName(name.to_string(), location_id);
    errors.push(err);
    return IrPattern::Wildcard;
}

fn resolve_record_type(
    name: &String,
    module: &Module,
    errors: &mut Vec<ResolverError>,
    location_id: LocationId,
) -> Option<TypeDefId> {
    if let Some(items) = module.imported_items.get(name) {
        match ImportedItemInfo::resolve_ambiguity(items, Namespace::Type) {
            None => {
                let err = ResolverError::AmbiguousName(name.to_string(), location_id);
                errors.push(err);
                return None;
            }
            Some(item) => match item.item {
                Item::Function(_, _) => unreachable!(),
                Item::Record(_, ir_typedef_id) => {
                    return Some(ir_typedef_id);
                }
                Item::Variant(..) => {
                    let err = ResolverError::NotRecordType(name.clone(), location_id);
                    errors.push(err);
                    return None;
                }
                _ => {}
            },
        }
    };
    let err = ResolverError::UnknownTypeName(name.to_string(), location_id);
    errors.push(err);
    return None;
}

fn process_pattern(
    case_expr_id: IrExprId,
    pattern_id: PatternId,
    program: &Program,
    ir_program: &mut IrProgram,
    module: &Module,
    environment: &mut Environment,
    bindings: &mut BTreeMap<String, Vec<LocationId>>,
    errors: &mut Vec<ResolverError>,
    lambda_helper: LambdaHelper,
    irrefutable: bool,
    type_arg_resolver: &mut TypeArgResolver,
) -> IrPatternId {
    let ir_pattern_id = ir_program.patterns.get_id();
    let info = program.patterns.get(&pattern_id);
    let (pattern, location_id) = (&info.item, info.location_id);
    let ir_pattern = match pattern {
        Pattern::Binding(name) => {
            let locations = bindings.entry(name.clone()).or_insert_with(|| Vec::new());
            locations.push(location_id);
            environment.add_expr_value(name.clone(), case_expr_id, ir_pattern_id);
            IrPattern::Binding(name.clone())
        }
        Pattern::Or(_) => unreachable!(),
        Pattern::Tuple(patterns) => {
            let ids: Vec<_> = patterns
                .iter()
                .map(|id| {
                    process_pattern(
                        case_expr_id,
                        *id,
                        program,
                        ir_program,
                        module,
                        environment,
                        bindings,
                        errors,
                        lambda_helper.clone(),
                        irrefutable,
                        type_arg_resolver,
                    )
                })
                .collect();
            IrPattern::Tuple(ids)
        }
        Pattern::Constructor(name, patterns) => {
            let ids: Vec<_> = patterns
                .iter()
                .map(|id| {
                    process_pattern(
                        case_expr_id,
                        *id,
                        program,
                        ir_program,
                        module,
                        environment,
                        bindings,
                        errors,
                        lambda_helper.clone(),
                        irrefutable,
                        type_arg_resolver,
                    )
                })
                .collect();
            resolve_pattern_type_constructor(name, module, errors, location_id, ids, irrefutable)
        }
        Pattern::Guarded(pattern_id, guard_expr_id) => {
            let ir_pattern_id = process_pattern(
                case_expr_id,
                *pattern_id,
                program,
                ir_program,
                module,
                environment,
                bindings,
                errors,
                lambda_helper.clone(),
                irrefutable,
                type_arg_resolver,
            );
            let ir_guard_expr_id = process_expr(
                *guard_expr_id,
                program,
                module,
                environment,
                ir_program,
                errors,
                lambda_helper.clone(),
                type_arg_resolver,
            );
            IrPattern::Guarded(ir_pattern_id, ir_guard_expr_id)
        }
        Pattern::Wildcard => IrPattern::Wildcard,
        Pattern::IntegerLiteral(v) => {
            if irrefutable {
                let err = ResolverError::NotIrrefutablePattern(location_id);
                errors.push(err);
                IrPattern::Wildcard
            } else {
                IrPattern::IntegerLiteral(*v)
            }
        }
        Pattern::CharLiteral(v) => {
            if irrefutable {
                let err = ResolverError::NotIrrefutablePattern(location_id);
                errors.push(err);
                IrPattern::Wildcard
            } else {
                IrPattern::CharLiteral(*v)
            }
        }
        Pattern::StringLiteral(v) => {
            if irrefutable {
                let err = ResolverError::NotIrrefutablePattern(location_id);
                errors.push(err);
                IrPattern::Wildcard
            } else {
                IrPattern::StringLiteral(v.clone())
            }
        }
        Pattern::Typed(pattern_id, type_signature_id) => {
            let ir_pattern_id = process_pattern(
                case_expr_id,
                *pattern_id,
                program,
                ir_program,
                module,
                environment,
                bindings,
                errors,
                lambda_helper.clone(),
                irrefutable,
                type_arg_resolver,
            );

            let result = process_type_signature(
                &type_signature_id,
                program,
                ir_program,
                module,
                type_arg_resolver,
                errors,
            );

            match result {
                Some(i) => IrPattern::Typed(ir_pattern_id, i),
                None => IrPattern::Wildcard,
            }
        }
        Pattern::Record(name, items) => {
            if let Some(ir_type_id) = resolve_record_type(name, module, errors, location_id) {
                let record = ir_program.typedefs.get(&ir_type_id).get_record().clone();
                let mut unused_fields = BTreeSet::new();
                let mut initialized_twice = BTreeSet::new();
                for f in &record.fields {
                    unused_fields.insert(f.name.clone());
                }
                let mut ir_items: Vec<_> = items
                    .iter()
                    .map(|i| {
                        let mut field_index = None;
                        for (index, f) in record.fields.iter().enumerate() {
                            if f.name == i.name {
                                field_index = Some(index);
                                if !unused_fields.remove(&f.name) {
                                    initialized_twice.insert(f.name.clone());
                                }
                            }
                        }
                        let field_index = match field_index {
                            None => {
                                let err = ResolverError::NoSuchField(
                                    record.name.clone(),
                                    i.name.clone(),
                                    i.location_id,
                                );
                                errors.push(err);
                                0
                            }
                            Some(i) => i,
                        };
                        let ir_pattern_id = process_pattern(
                            case_expr_id,
                            i.value,
                            program,
                            ir_program,
                            module,
                            environment,
                            bindings,
                            errors,
                            lambda_helper.clone(),
                            irrefutable,
                            type_arg_resolver,
                        );
                        (field_index, ir_pattern_id)
                    })
                    .collect();
                if !unused_fields.is_empty() {
                    let err = ResolverError::MissingFields(
                        unused_fields.into_iter().collect(),
                        location_id,
                    );
                    errors.push(err);
                }
                if !initialized_twice.is_empty() {
                    let err = ResolverError::FieldsInitializedMultipleTimes(
                        initialized_twice.into_iter().collect(),
                        location_id,
                    );
                    errors.push(err);
                }
                ir_items.sort_by(|a, b| a.0.cmp(&b.0));
                let ir_items: Vec<_> = ir_items.into_iter().map(|i| i.1).collect();
                IrPattern::Record(ir_type_id, ir_items)
            } else {
                IrPattern::Wildcard
            }
        }
        Pattern::CharRange(start, end, kind) => {
            if irrefutable {
                let err = ResolverError::NotIrrefutablePattern(location_id);
                errors.push(err);
                IrPattern::Wildcard
            } else {
                let ir_kind = match kind {
                    RangeKind::Exclusive => IrRangeKind::Exclusive,
                    RangeKind::Inclusive => IrRangeKind::Inclusive,
                };
                IrPattern::CharRange(*start, *end, ir_kind)
            }
        }
    };
    let ir_pattern_info = ItemInfo {
        item: ir_pattern,
        location_id: location_id,
    };
    ir_program.patterns.add_item(ir_pattern_id, ir_pattern_info);
    ir_pattern_id
}

pub fn process_expr(
    id: ExprId,
    program: &Program,
    module: &Module,
    environment: &mut Environment,
    ir_program: &mut IrProgram,
    errors: &mut Vec<ResolverError>,
    lambda_helper: LambdaHelper,
    type_arg_resolver: &mut TypeArgResolver,
) -> IrExprId {
    let expr = &program.exprs.get(&id).item;
    let location_id = program.exprs.get(&id).location_id;
    //println!("Processing expr {} {}", id, expr);
    match expr {
        Expr::Lambda(args, lambda_body) => {
            let ir_lambda_id = ir_program.functions.get_id();
            let mut arg_names = BTreeSet::new();
            let mut conflicting_names: BTreeSet<String> = BTreeSet::new();
            let mut environment = Environment::child(environment);
            for (index, arg) in args.iter().enumerate() {
                if !arg_names.insert(arg.0.clone()) {
                    conflicting_names.insert(arg.0.clone());
                }
                environment.add_arg(arg.0.clone(), ir_lambda_id, index);
            }
            if !conflicting_names.is_empty() {
                let err = ResolverError::LambdaArgumentConflict(
                    conflicting_names.into_iter().collect(),
                    location_id.clone(),
                );
                errors.push(err);
            }

            let local_lambda_helper = LambdaHelper::new(
                environment.level(),
                lambda_helper.host_function_name(),
                lambda_helper.clone_counter(),
                ir_lambda_id,
                lambda_helper.host_function(),
                Some(lambda_helper),
            );

            let ir_lambda_body = process_expr(
                *lambda_body,
                program,
                module,
                &mut environment,
                ir_program,
                errors,
                local_lambda_helper.clone(),
                type_arg_resolver,
            );

            let lambda_info = LambdaInfo {
                body: ir_lambda_body,
                module: module.name.clone(),
                host_info: local_lambda_helper.host_function_name(),
                host_function: local_lambda_helper.host_function(),
                index: local_lambda_helper.get_lambda_index(),
                location_id: location_id,
            };

            let captures = local_lambda_helper.captures();

            let arg_locations: Vec<_> = args.iter().map(|arg| arg.1).collect();

            let ir_function = IrFunction {
                id: ir_lambda_id,
                arg_count: arg_locations.len() + captures.len(),
                arg_locations: arg_locations,
                info: FunctionInfo::Lambda(lambda_info),
            };
            ir_program.functions.add_item(ir_lambda_id, ir_function);

            let captured_lambda_args: Vec<_> = captures
                .into_iter()
                .map(|expr| add_expr(expr, id, ir_program, program))
                .collect();
            let ir_expr = IrExpr::StaticFunctionCall(ir_lambda_id, captured_lambda_args);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::FunctionCall(id_expr_id, args) => {
            let ir_args: Vec<IrExprId> = args
                .iter()
                .map(|id| {
                    process_expr(
                        *id,
                        program,
                        module,
                        environment,
                        ir_program,
                        errors,
                        lambda_helper.clone(),
                        type_arg_resolver,
                    )
                })
                .collect();
            let id_expr = &program.exprs.get(id_expr_id).item;
            if let Expr::Path(path) = id_expr {
                match resolve_item_path(
                    path,
                    module,
                    environment,
                    lambda_helper,
                    program,
                    ir_program,
                    id,
                    errors,
                    location_id,
                ) {
                    PathResolveResult::FunctionRef(n) => {
                        let ir_expr = IrExpr::StaticFunctionCall(n, ir_args);
                        return add_expr(ir_expr, id, ir_program, program);
                    }
                    PathResolveResult::VariableRef(ir_id_expr_id) => {
                        let ir_expr = IrExpr::DynamicFunctionCall(ir_id_expr_id, ir_args);
                        return add_expr(ir_expr, id, ir_program, program);
                    }
                    PathResolveResult::ClassMemberRef(n) => {
                        let ir_expr = IrExpr::ClassFunctionCall(n, ir_args);
                        return add_expr(ir_expr, id, ir_program, program);
                    }
                }
            } else {
                if let Expr::Builtin(op) = id_expr {
                    if *op == BuiltinOperator::PipeForward {
                        assert_eq!(ir_args.len(), 2);
                        let left = ir_args[0];
                        let right = ir_args[1];
                        let ir_expr = IrExpr::DynamicFunctionCall(right, vec![left]);
                        return add_expr(ir_expr, id, ir_program, program);
                    } else {
                        let path = op.get_function_name();
                        match resolve_item_path(
                            &path,
                            module,
                            environment,
                            lambda_helper.clone(),
                            program,
                            ir_program,
                            id,
                            errors,
                            location_id,
                        ) {
                            PathResolveResult::ClassMemberRef(n) => {
                                let ir_expr = IrExpr::ClassFunctionCall(n, ir_args);
                                return add_expr(ir_expr, id, ir_program, program);
                            }
                            PathResolveResult::FunctionRef(n) => {
                                let ir_expr = IrExpr::StaticFunctionCall(n, ir_args);
                                return add_expr(ir_expr, id, ir_program, program);
                            }
                            _ => {
                                if errors.is_empty() {
                                    panic!("Couldn't handle builtin function {}", path.clone(),);
                                } else {
                                    let ir_expr = IrExpr::Tuple(Vec::new());
                                    return add_expr(ir_expr, id, ir_program, program);
                                }
                            }
                        }
                    }
                } else {
                    let id_expr = process_expr(
                        *id_expr_id,
                        program,
                        module,
                        environment,
                        ir_program,
                        errors,
                        lambda_helper.clone(),
                        type_arg_resolver,
                    );
                    let ir_expr = IrExpr::DynamicFunctionCall(id_expr, ir_args);
                    return add_expr(ir_expr, id, ir_program, program);
                }
            }
        }
        Expr::Builtin(_) => panic!("Builtinop reached!"),
        Expr::If(cond, true_branch, false_branch) => {
            let mut cond_env = Environment::child(environment);
            let mut true_env = Environment::child(environment);
            let mut false_env = Environment::child(environment);
            let ir_cond = process_expr(
                *cond,
                program,
                module,
                &mut cond_env,
                ir_program,
                errors,
                lambda_helper.clone(),
                type_arg_resolver,
            );
            let ir_true_branch = process_expr(
                *true_branch,
                program,
                module,
                &mut true_env,
                ir_program,
                errors,
                lambda_helper.clone(),
                type_arg_resolver,
            );
            let ir_false_branch = process_expr(
                *false_branch,
                program,
                module,
                &mut false_env,
                ir_program,
                errors,
                lambda_helper.clone(),
                type_arg_resolver,
            );
            let ir_expr = IrExpr::If(ir_cond, ir_true_branch, ir_false_branch);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::Tuple(items) => {
            let ir_items: Vec<IrExprId> = items
                .iter()
                .map(|id| {
                    process_expr(
                        *id,
                        program,
                        module,
                        environment,
                        ir_program,
                        errors,
                        lambda_helper.clone(),
                        type_arg_resolver,
                    )
                })
                .collect();
            let ir_expr = IrExpr::Tuple(ir_items);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::List(items) => {
            let ir_items: Vec<IrExprId> = items
                .iter()
                .map(|id| {
                    process_expr(
                        *id,
                        program,
                        module,
                        environment,
                        ir_program,
                        errors,
                        lambda_helper.clone(),
                        type_arg_resolver,
                    )
                })
                .collect();
            let ir_expr = IrExpr::List(ir_items);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::Path(path) => {
            match resolve_item_path(
                path,
                module,
                environment,
                lambda_helper,
                program,
                ir_program,
                id,
                errors,
                location_id,
            ) {
                PathResolveResult::FunctionRef(n) => {
                    let ir_expr = IrExpr::StaticFunctionCall(n, vec![]);
                    return add_expr(ir_expr, id, ir_program, program);
                }
                PathResolveResult::VariableRef(ir_expr_id) => {
                    return ir_expr_id;
                }
                PathResolveResult::ClassMemberRef(n) => {
                    let ir_expr = IrExpr::ClassFunctionCall(n, vec![]);
                    return add_expr(ir_expr, id, ir_program, program);
                }
            }
        }
        Expr::IntegerLiteral(v) => {
            let ir_expr = IrExpr::IntegerLiteral(v.clone());
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::FloatLiteral(v) => {
            let ir_expr = IrExpr::FloatLiteral(v.clone());
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::CharLiteral(v) => {
            let ir_expr = IrExpr::CharLiteral(v.clone());
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::StringLiteral(v) => {
            let ir_expr = IrExpr::StringLiteral(v.clone());
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::Do(items) => {
            let ir_items: Vec<IrExprId> = items
                .iter()
                .map(|id| {
                    process_expr(
                        *id,
                        program,
                        module,
                        environment,
                        ir_program,
                        errors,
                        lambda_helper.clone(),
                        type_arg_resolver,
                    )
                })
                .collect();
            let ir_expr = IrExpr::Do(ir_items);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::Bind(pattern_id, expr_id) => {
            let ir_expr_id = process_expr(
                *expr_id,
                program,
                module,
                environment,
                ir_program,
                errors,
                lambda_helper.clone(),
                type_arg_resolver,
            );
            let mut bindings = BTreeMap::new();
            let ir_pattern_id = process_pattern(
                ir_expr_id,
                *pattern_id,
                program,
                ir_program,
                module,
                environment,
                &mut bindings,
                errors,
                lambda_helper.clone(),
                true,
                type_arg_resolver,
            );
            for binding in bindings {
                if binding.1.len() > 1 {
                    let err =
                        ResolverError::PatternBindConflict(binding.0.clone(), binding.1.clone());
                    errors.push(err);
                }
            }
            let ir_expr = IrExpr::Bind(ir_pattern_id, ir_expr_id);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::FieldAccess(name, expr_id) => {
            let ir_expr_id = process_expr(
                *expr_id,
                program,
                module,
                environment,
                ir_program,
                errors,
                lambda_helper,
                type_arg_resolver,
            );
            let accesses = process_field_access(module, errors, name.to_string(), location_id);
            let ir_expr = IrExpr::FieldAccess(accesses, ir_expr_id);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::TupleFieldAccess(index, expr_id) => {
            let ir_expr_id = process_expr(
                *expr_id,
                program,
                module,
                environment,
                ir_program,
                errors,
                lambda_helper,
                type_arg_resolver,
            );
            let ir_expr = IrExpr::TupleFieldAccess(*index, ir_expr_id);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::Formatter(fmt, items) => {
            let ir_items: Vec<IrExprId> = items
                .iter()
                .map(|id| {
                    process_expr(
                        *id,
                        program,
                        module,
                        environment,
                        ir_program,
                        errors,
                        lambda_helper.clone(),
                        type_arg_resolver,
                    )
                })
                .collect();
            let ir_expr = IrExpr::Formatter(fmt.clone(), ir_items);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::CaseOf(body_id, cases) => {
            let ir_body_id = process_expr(
                *body_id,
                program,
                module,
                environment,
                ir_program,
                errors,
                lambda_helper.clone(),
                type_arg_resolver,
            );
            let mut ir_cases = Vec::new();
            let mut bind_groups = Vec::new();
            for case in cases {
                if let Pattern::Or(sub_patterns) = &program.patterns.get(&case.pattern_id).item {
                    let mut all_bindings = BTreeMap::new();
                    for sub_pattern in sub_patterns {
                        let mut case_environment = Environment::child(environment);
                        let mut bindings = BTreeMap::new();
                        let pattern_id = process_pattern(
                            ir_body_id,
                            *sub_pattern,
                            program,
                            ir_program,
                            module,
                            &mut case_environment,
                            &mut bindings,
                            errors,
                            lambda_helper.clone(),
                            false,
                            type_arg_resolver,
                        );
                        for binding in bindings {
                            if binding.1.len() > 1 {
                                let err = ResolverError::PatternBindConflict(
                                    binding.0.clone(),
                                    binding.1.clone(),
                                );
                                errors.push(err);
                            } else {
                                let binding_data = all_bindings
                                    .entry(binding.0.clone())
                                    .or_insert_with(|| Vec::new());
                                let binding_ref = case_environment
                                    .get_ref(&binding.0)
                                    .expect("Binding not found in env");
                                if let NamedRef::ExprValue(_, pattern_id) = binding_ref.0 {
                                    binding_data.push((binding.1[0], pattern_id));
                                } else {
                                    panic!("Pattern binding does not refer to a pattern")
                                }
                            }
                        }
                        let ir_case_body_id = process_expr(
                            case.body,
                            program,
                            module,
                            &mut case_environment,
                            ir_program,
                            errors,
                            lambda_helper.clone(),
                            type_arg_resolver,
                        );
                        let ir_case = IrCase {
                            pattern_id: pattern_id,
                            body: ir_case_body_id,
                        };
                        ir_cases.push(ir_case);
                    }
                    if errors.is_empty() {
                        for binding in all_bindings {
                            if binding.1.len() != sub_patterns.len() {
                                let err = ResolverError::PatternBindNotPresent(
                                    binding.0.clone(),
                                    binding.1[0].0.clone(),
                                );
                                errors.push(err);
                            } else {
                                let bind_group = BindGroup {
                                    patterns: binding.1.iter().map(|(_, id)| *id).collect(),
                                };
                                bind_groups.push(bind_group);
                            }
                        }
                    }
                } else {
                    let mut case_environment = Environment::child(environment);
                    let mut bindings = BTreeMap::new();
                    let pattern_id = process_pattern(
                        ir_body_id,
                        case.pattern_id,
                        program,
                        ir_program,
                        module,
                        &mut case_environment,
                        &mut bindings,
                        errors,
                        lambda_helper.clone(),
                        false,
                        type_arg_resolver,
                    );
                    for binding in bindings {
                        if binding.1.len() > 1 {
                            let err = ResolverError::PatternBindConflict(
                                binding.0.clone(),
                                binding.1.clone(),
                            );
                            errors.push(err);
                        }
                    }
                    let ir_case_body_id = process_expr(
                        case.body,
                        program,
                        module,
                        &mut case_environment,
                        ir_program,
                        errors,
                        lambda_helper.clone(),
                        type_arg_resolver,
                    );
                    let ir_case = IrCase {
                        pattern_id: pattern_id,
                        body: ir_case_body_id,
                    };
                    ir_cases.push(ir_case);
                }
            }
            let ir_expr = IrExpr::CaseOf(ir_body_id, ir_cases, bind_groups);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::RecordInitialization(name, items) => {
            if let Some(ir_type_id) = resolve_record_type(name, module, errors, location_id) {
                let record = ir_program.typedefs.get(&ir_type_id).get_record().clone();
                let mut unused_fields = BTreeSet::new();
                let mut initialized_twice = BTreeSet::new();
                for f in &record.fields {
                    unused_fields.insert(f.name.clone());
                }
                let ir_items: Vec<_> = items
                    .iter()
                    .map(|i| {
                        let mut field_index = None;
                        for (index, f) in record.fields.iter().enumerate() {
                            if f.name == i.field_name {
                                field_index = Some(index);
                                if !unused_fields.remove(&f.name) {
                                    initialized_twice.insert(f.name.clone());
                                }
                            }
                        }
                        let field_index = match field_index {
                            None => {
                                let err = ResolverError::NoSuchField(
                                    record.name.clone(),
                                    i.field_name.clone(),
                                    i.location_id,
                                );
                                errors.push(err);
                                0
                            }
                            Some(i) => i,
                        };
                        let ir_body_id = process_expr(
                            i.body,
                            program,
                            module,
                            environment,
                            ir_program,
                            errors,
                            lambda_helper.clone(),
                            type_arg_resolver,
                        );
                        let value_expr = RecordFieldValueExpr {
                            expr_id: ir_body_id,
                            index: field_index,
                        };
                        value_expr
                    })
                    .collect();
                if !unused_fields.is_empty() {
                    let err = ResolverError::MissingFields(
                        unused_fields.into_iter().collect(),
                        location_id,
                    );
                    errors.push(err);
                }
                if !initialized_twice.is_empty() {
                    let err = ResolverError::FieldsInitializedMultipleTimes(
                        initialized_twice.into_iter().collect(),
                        location_id,
                    );
                    errors.push(err);
                }
                let ir_expr = IrExpr::RecordInitialization(ir_type_id, ir_items);
                return add_expr(ir_expr, id, ir_program, program);
            } else {
                let ir_expr = IrExpr::Tuple(vec![]);
                return add_expr(ir_expr, id, ir_program, program);
            }
        }
        Expr::RecordUpdate(name, items) => {
            let record_expr_id = match resolve_item_path(
                name,
                module,
                environment,
                lambda_helper.clone(),
                program,
                ir_program,
                id,
                errors,
                location_id,
            ) {
                PathResolveResult::FunctionRef(n) => {
                    let ir_expr = IrExpr::StaticFunctionCall(n, vec![]);
                    add_expr(ir_expr, id, ir_program, program)
                }
                PathResolveResult::VariableRef(ir_expr_id) => ir_expr_id,
                PathResolveResult::ClassMemberRef(n) => {
                    let ir_expr = IrExpr::ClassFunctionCall(n, vec![]);
                    add_expr(ir_expr, id, ir_program, program)
                }
            };
            let mut potential_type_ids = BTreeSet::new();
            let mut access_list = Vec::new();
            let mut names = BTreeSet::new();
            let mut initialized_twice = BTreeSet::new();
            let mut field_exprs = Vec::new();
            for (index, item) in items.iter().enumerate() {
                if !names.insert(item.field_name.clone()) {
                    initialized_twice.insert(item.field_name.clone());
                }
                let accesses =
                    process_field_access(module, errors, item.field_name.clone(), item.location_id);
                if index == 0 {
                    for access in &accesses {
                        potential_type_ids.insert(access.record_id);
                    }
                } else {
                    let current: BTreeSet<_> =
                        accesses.iter().map(|access| access.record_id).collect();
                    potential_type_ids =
                        potential_type_ids.intersection(&current).cloned().collect();
                }
                access_list.push(accesses);
                let ir_body_id = process_expr(
                    item.body,
                    program,
                    module,
                    environment,
                    ir_program,
                    errors,
                    lambda_helper.clone(),
                    type_arg_resolver,
                );
                field_exprs.push(ir_body_id);
            }
            if potential_type_ids.is_empty() {
                let names: Vec<_> = names.into_iter().collect();
                let err = ResolverError::NoRecordFoundWithFields(names, location_id);
                errors.push(err);
            }
            if !initialized_twice.is_empty() {
                let err = ResolverError::FieldsInitializedMultipleTimes(
                    initialized_twice.into_iter().collect(),
                    location_id,
                );
                errors.push(err);
            }
            let mut updates: Vec<RecordUpdateInfo> = Vec::new();
            for potential_type_id in potential_type_ids {
                let mut update_items: Vec<RecordFieldValueExpr> = Vec::new();
                for (index, accesses) in access_list.iter().enumerate() {
                    for access in accesses {
                        if access.record_id == potential_type_id {
                            update_items.push(RecordFieldValueExpr {
                                expr_id: field_exprs[index],
                                index: access.index,
                            });
                        }
                    }
                }
                updates.push(RecordUpdateInfo {
                    record_id: potential_type_id,
                    items: update_items,
                });
            }
            let ir_expr = IrExpr::RecordUpdate(record_expr_id, updates);
            return add_expr(ir_expr, id, ir_program, program);
        }
        Expr::Return(inner_expr_id) => {
            let ir_inner_expr_id = process_expr(
                *inner_expr_id,
                program,
                module,
                environment,
                ir_program,
                errors,
                lambda_helper.clone(),
                type_arg_resolver,
            );
            let ir_expr = IrExpr::Return(ir_inner_expr_id);
            return add_expr(ir_expr, id, ir_program, program);
        }
    }
}
