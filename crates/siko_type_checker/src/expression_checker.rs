use crate::error::TypecheckError;
use crate::type_info_provider::TypeInfoProvider;
use crate::type_store::TypeStore;
use siko_ir::data_type_info::AdtTypeInfo;
use siko_ir::expr::Expr;
use siko_ir::expr::ExprId;
use siko_ir::function::FunctionId;
use siko_ir::pattern::Pattern;
use siko_ir::pattern::PatternId;
use siko_ir::program::Program;
use siko_ir::types::Type;
use siko_ir::unifier::Unifier;
use siko_ir::walker::Visitor;
use siko_location_info::location_id::LocationId;
use siko_util::dependency_processor::DependencyGroup;

pub struct ExpressionChecker<'a> {
    program: &'a Program,
    group: &'a DependencyGroup<FunctionId>,
    type_store: &'a mut TypeStore,
    type_info_provider: &'a mut TypeInfoProvider,
    errors: &'a mut Vec<TypecheckError>,
    disambiguations: Vec<(ExprId, usize)>,
}

impl<'a> ExpressionChecker<'a> {
    pub fn new(
        program: &'a Program,
        group: &'a DependencyGroup<FunctionId>,
        type_store: &'a mut TypeStore,
        type_info_provider: &'a mut TypeInfoProvider,
        errors: &'a mut Vec<TypecheckError>,
    ) -> ExpressionChecker<'a> {
        ExpressionChecker {
            program: program,
            group: group,
            type_store: type_store,
            type_info_provider: type_info_provider,
            errors: errors,
            disambiguations: Vec::new(),
        }
    }

    fn unify(&mut self, ty1: &Type, ty2: &Type, location: LocationId) -> Unifier {
        let mut unifier = Unifier::new(self.type_info_provider.type_var_generator.clone());
        if !unifier.unify(ty1, ty2).is_ok() {
            let ty_str1 = ty1.get_resolved_type_string(self.program);
            let ty_str2 = ty2.get_resolved_type_string(self.program);
            let err = TypecheckError::TypeMismatch(location, ty_str1, ty_str2);
            self.errors.push(err);
        } else {
            self.type_store.apply(&unifier);
            for id in &self.group.items {
                let info = self.type_info_provider.function_type_info_store.get_mut(id);
                info.apply(&unifier);
            }
        }
        unifier
    }

    pub fn match_expr_with(&mut self, expr_id: ExprId, ty: &Type) -> Unifier {
        let expr_ty = self.type_store.get_expr_type(&expr_id).clone();
        let location = self.program.exprs.get(&expr_id).location_id;
        self.unify(ty, &expr_ty, location)
    }

    fn match_pattern_with(&mut self, pattern_id: PatternId, ty: &Type) {
        let pattern_ty = self.type_store.get_pattern_type(&pattern_id).clone();
        let location = self.program.patterns.get(&pattern_id).location_id;
        self.unify(ty, &pattern_ty, location);
    }

    fn match_expr_with_pattern(&mut self, expr_id: ExprId, pattern_id: PatternId) {
        let expr_ty = self.type_store.get_expr_type(&expr_id).clone();
        let pattern_ty = self.type_store.get_pattern_type(&pattern_id).clone();
        let location = self.program.patterns.get(&pattern_id).location_id;
        self.unify(&expr_ty, &pattern_ty, location);
    }

    fn match_exprs(&mut self, expr_id1: ExprId, expr_id2: ExprId) {
        let expr_ty1 = self.type_store.get_expr_type(&expr_id1).clone();
        let expr_ty2 = self.type_store.get_expr_type(&expr_id2).clone();
        let location = self.program.exprs.get(&expr_id2).location_id;
        self.unify(&expr_ty1, &expr_ty2, location);
    }

    fn match_patterns(&mut self, pattern_id1: PatternId, pattern_id2: PatternId) {
        let pattern_ty1 = self.type_store.get_pattern_type(&pattern_id1).clone();
        let pattern_ty2 = self.type_store.get_pattern_type(&pattern_id2).clone();
        let location = self.program.patterns.get(&pattern_id2).location_id;
        self.unify(&pattern_ty1, &pattern_ty2, location);
    }

    fn check_function_call(&mut self, expr_id: ExprId, args: &Vec<ExprId>) {
        for (index, arg) in args.iter().enumerate() {
            let func_type_info = self.type_store.get_func_type_for_expr(&expr_id);
            let arg_type = &func_type_info.args[index].clone();
            self.match_expr_with(*arg, &arg_type);
        }
    }

    pub fn get_disambiguations(&self) -> Vec<(ExprId, usize)> {
        self.disambiguations.clone()
    }
}

impl<'a> Visitor for ExpressionChecker<'a> {
    fn get_program(&self) -> &Program {
        &self.program
    }

    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr) {
        //self.expr_processor.create_type_var_for_expr(expr_id);
        //println!("C {} {}", expr_id, expr);
        match expr {
            Expr::ArgRef(arg_ref) => {
                let func_type_info = self
                    .type_info_provider
                    .function_type_info_store
                    .get(&arg_ref.id);
                let arg_ty = func_type_info.args[arg_ref.index].clone();
                self.match_expr_with(expr_id, &arg_ty);
            }
            Expr::Bind(pattern_id, rhs) => {
                self.match_expr_with_pattern(*rhs, *pattern_id);
                let expr_ty = Type::Tuple(Vec::new());
                self.match_expr_with(expr_id, &expr_ty);
            }
            Expr::CaseOf(case_expr, cases, bind_groups) => {
                for bind_group in bind_groups {
                    for patterns in bind_group.patterns.windows(2) {
                        self.match_patterns(patterns[0], patterns[1]);
                    }
                }
                if let Some(first) = cases.first() {
                    self.match_exprs(expr_id, first.body);
                    for case in cases {
                        self.match_expr_with_pattern(*case_expr, case.pattern_id);
                        self.match_exprs(expr_id, case.body);
                    }
                }
            }
            Expr::ClassFunctionCall(_, args) => {
                self.check_function_call(expr_id, args);
            }
            Expr::CharLiteral(_) => {}
            Expr::DynamicFunctionCall(func_expr_id, args) => {
                self.type_store
                    .remove_fixed_types_from_expr_type(&func_expr_id);
                let func_type_info = self.type_store.get_func_type_for_expr(&expr_id).clone();
                self.match_expr_with(*func_expr_id, &func_type_info.function_type);
                self.check_function_call(expr_id, args);
            }
            Expr::Do(items) => {
                let last_expr_id = items[items.len() - 1];
                self.match_exprs(expr_id, last_expr_id);
            }
            Expr::ExprValue(_, pattern_id) => {
                self.match_expr_with_pattern(expr_id, *pattern_id);
            }
            Expr::FieldAccess(infos, receiver_expr_id) => {
                let mut failed = true;
                let receiver_ty = self.type_store.get_expr_type(receiver_expr_id).clone();
                if let Type::Named(_, id, _) = receiver_ty {
                    for (index, info) in infos.iter().enumerate() {
                        if info.record_id == id {
                            let mut record_type_info =
                                self.type_info_provider.get_record_type_info(&id);
                            let mut unifier =
                                Unifier::new(self.type_info_provider.type_var_generator.clone());
                            if unifier
                                .unify(&record_type_info.record_type, &receiver_ty)
                                .is_ok()
                            {
                                record_type_info.apply(&unifier);
                                let field_ty = &record_type_info.field_types[info.index].0;
                                self.match_expr_with(expr_id, field_ty);
                                failed = false;
                                self.disambiguations.push((expr_id, index));
                                break;
                            }
                        }
                    }
                    if failed {
                        let mut all_records = Vec::new();
                        for info in infos {
                            let record = self.program.typedefs.get(&info.record_id).get_record();
                            all_records.push(record.name.clone());
                        }
                        let expected_type = format!("{}", all_records.join(" or "));
                        let found_type = receiver_ty.get_resolved_type_string(self.program);
                        let location = self.program.exprs.get(&receiver_expr_id).location_id;
                        let err = TypecheckError::TypeMismatch(location, expected_type, found_type);
                        self.errors.push(err);
                        return;
                    }
                }
                if failed {
                    let location = self.program.exprs.get(&receiver_expr_id).location_id;
                    let err = TypecheckError::TypeAnnotationNeeded(location);
                    self.errors.push(err);
                }
            }
            Expr::FloatLiteral(_) => {}
            Expr::Formatter(fmt, args) => {
                let subs: Vec<_> = fmt.split("{}").collect();
                if subs.len() != args.len() + 1 {
                    let location = self.program.exprs.get(&expr_id).location_id;
                    let err = TypecheckError::InvalidFormatString(location);
                    self.errors.push(err);
                }
                for arg in args {
                    let show_type = self.program.get_show_type();
                    self.match_expr_with(*arg, &show_type);
                }
            }
            Expr::If(cond, true_branch, false_branch) => {
                let bool_ty = self.program.get_bool_type();
                self.match_expr_with(*cond, &bool_ty);
                self.match_exprs(*true_branch, *false_branch);
                self.match_exprs(expr_id, *true_branch);
            }
            Expr::IntegerLiteral(_) => {}
            Expr::List(items) => {
                if let Some(first) = items.first() {
                    let ty = self.type_store.get_expr_type(first).clone();
                    let ty = self.program.get_list_type(ty);
                    self.match_expr_with(expr_id, &ty);
                    for item in items {
                        self.match_exprs(*first, *item);
                    }
                }
            }
            Expr::StaticFunctionCall(_, args) => {
                self.check_function_call(expr_id, args);
            }
            Expr::StringLiteral(_) => {}
            Expr::RecordInitialization(_, values) => {
                for value in values {
                    let record_type_info = self
                        .type_store
                        .get_record_type_info_for_expr(&expr_id)
                        .clone();
                    let field_type = &record_type_info.field_types[value.index];
                    self.match_expr_with(value.expr_id, &field_type.0);
                }
            }
            Expr::RecordUpdate(receiver_expr_id, record_updates) => {
                let location_id = self.program.exprs.get(&expr_id).location_id;
                let receiver_ty = self.type_store.get_expr_type(receiver_expr_id);
                let real_record_type = match receiver_ty {
                    Type::Named(_, id, _) => Some(*id),
                    Type::Var(_, _) => {
                        if record_updates.len() == 1 {
                            Some(record_updates[0].record_id)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                let mut expected_records = Vec::new();
                let mut matching_update = None;
                for (index, record_update) in record_updates.iter().enumerate() {
                    let record = self
                        .program
                        .typedefs
                        .get(&record_update.record_id)
                        .get_record();
                    expected_records.push(record.name.clone());
                    if let Some(id) = real_record_type {
                        if record_update.record_id == id {
                            matching_update = Some(record_update);
                            self.disambiguations.push((expr_id, index));
                            break;
                        }
                    }
                }
                match matching_update {
                    Some(update) => {
                        let mut record_type_info = self
                            .type_info_provider
                            .get_record_type_info(&update.record_id);
                        let unifier =
                            self.match_expr_with(*receiver_expr_id, &record_type_info.record_type);
                        record_type_info.apply(&unifier);
                        for field_update in &update.items {
                            let field = &record_type_info.field_types[field_update.index];
                            let unifier = self.match_expr_with(field_update.expr_id, &field.0);
                            record_type_info.apply(&unifier);
                        }
                        self.match_expr_with(expr_id, &record_type_info.record_type);
                    }
                    None => {
                        let expected_type = format!("{}", expected_records.join(" or "));
                        let found_type = receiver_ty.get_resolved_type_string(self.program);
                        let err =
                            TypecheckError::TypeMismatch(location_id, expected_type, found_type);
                        self.errors.push(err);
                    }
                }
            }
            Expr::Tuple(items) => {
                let item_types: Vec<_> = items
                    .iter()
                    .map(|item| self.type_store.get_expr_type(item).clone())
                    .collect();
                let tuple_ty = Type::Tuple(item_types);
                self.match_expr_with(expr_id, &tuple_ty);
            }
            Expr::TupleFieldAccess(index, receiver_expr_id) => {
                let receiver_ty = self.type_store.get_expr_type(receiver_expr_id).clone();
                if let Type::Tuple(items) = &receiver_ty {
                    if items.len() > *index {
                        self.match_expr_with(expr_id, &items[*index]);
                        return;
                    }
                } else if let Type::Var(..) = &receiver_ty {
                    let location = self.program.exprs.get(&receiver_expr_id).location_id;
                    let err = TypecheckError::TypeAnnotationNeeded(location);
                    self.errors.push(err);
                    return;
                }
                let expected_type = format!("<tuple with at least {} item(s)>", index + 1);
                let found_type = receiver_ty.get_resolved_type_string(self.program);
                let location = self.program.exprs.get(&receiver_expr_id).location_id;
                let err = TypecheckError::TypeMismatch(location, expected_type, found_type);
                self.errors.push(err);
            }
        }
    }

    fn visit_pattern(&mut self, pattern_id: PatternId, pattern: &Pattern) {
        //println!("C {} {:?}", pattern_id, pattern);
        match pattern {
            Pattern::Binding(_) => {}
            Pattern::Guarded(inner, guard_expr_id) => {
                self.match_patterns(*inner, pattern_id);
                let bool_ty = self.program.get_bool_type();
                self.match_expr_with(*guard_expr_id, &bool_ty);
            }
            Pattern::IntegerLiteral(_) => {}
            Pattern::Record(_, fields) => {
                let record_type_info = self
                    .type_store
                    .get_record_type_info_for_pattern(&pattern_id)
                    .clone();
                for (field, field_type) in fields.iter().zip(record_type_info.field_types.iter()) {
                    self.match_pattern_with(*field, &field_type.0);
                }
            }
            Pattern::StringLiteral(_) => {}
            Pattern::CharLiteral(_) => {}
            Pattern::CharRange(_, _, _) => {}
            Pattern::Tuple(items) => {
                let ty = self.type_store.get_pattern_type(&pattern_id).clone();
                if let Type::Tuple(item_types) = ty {
                    for (item, item_ty) in items.iter().zip(item_types.iter()) {
                        self.match_pattern_with(*item, item_ty);
                    }
                }
            }
            Pattern::Typed(inner, _) => {
                self.match_patterns(*inner, pattern_id);
            }
            Pattern::Variant(_, index, items) => {
                let info: AdtTypeInfo = self
                    .type_store
                    .get_adt_type_info_for_pattern(&pattern_id)
                    .clone();
                let variant_type = &info.variant_types[*index];
                for (item, variant_item) in items.iter().zip(variant_type.item_types.iter()) {
                    self.match_pattern_with(*item, &variant_item.0);
                }
            }
            Pattern::Wildcard => {}
        }
    }
}
