use crate::error::TypecheckError;
use crate::type_info_provider::TypeInfoProvider;
use crate::type_store::TypeStore;
use crate::util::create_general_function_type_info;
use crate::util::function_argument_mismatch;
use crate::util::process_type_signature;
use siko_ir::expr::Expr;
use siko_ir::expr::ExprId;
use siko_ir::function::FunctionId;
use siko_ir::pattern::Pattern;
use siko_ir::pattern::PatternId;
use siko_ir::program::Program;
use siko_ir::types::Type;
use siko_ir::walker::Visitor;
use siko_util::dependency_processor::DependencyGroup;

pub struct TypeStoreInitializer<'a> {
    program: &'a Program,
    group: &'a DependencyGroup<FunctionId>,
    type_store: &'a mut TypeStore,
    type_info_provider: &'a mut TypeInfoProvider,
    errors: &'a mut Vec<TypecheckError>,
}

impl<'a> TypeStoreInitializer<'a> {
    pub fn new(
        program: &'a Program,
        group: &'a DependencyGroup<FunctionId>,
        type_store: &'a mut TypeStore,
        type_info_provider: &'a mut TypeInfoProvider,
        errors: &'a mut Vec<TypecheckError>,
    ) -> TypeStoreInitializer<'a> {
        TypeStoreInitializer {
            program: program,
            group: group,
            type_store: type_store,
            type_info_provider: type_info_provider,
            errors: errors,
        }
    }
}

impl<'a> Visitor for TypeStoreInitializer<'a> {
    fn get_program(&self) -> &Program {
        &self.program
    }

    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr) {
        //println!("I {} {}", expr_id, expr);
        match expr {
            Expr::ArgRef(_) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::Bind(_, _) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::CaseOf(..) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::CharLiteral(_) => {
                self.type_store
                    .initialize_expr(expr_id, self.program.get_char_type());
            }
            Expr::ClassFunctionCall(class_member_id, args) => {
                let class_member_type = self
                    .type_info_provider
                    .get_class_member_type(class_member_id);
                let mut func_type_info = create_general_function_type_info(
                    args.len(),
                    &mut self.type_info_provider.type_var_generator,
                );
                let mut unifier = self.program.get_unifier();
                let r = unifier.unify(&func_type_info.function_type, &class_member_type);
                if r.is_err() {
                    let args: Vec<_> = args
                        .iter()
                        .map(|arg| self.type_store.get_expr_type(arg).clone())
                        .collect();
                    let location = self.program.exprs.get(&expr_id).location_id;
                    function_argument_mismatch(
                        self.program,
                        &class_member_type,
                        args,
                        location,
                        self.errors,
                    )
                }
                func_type_info.apply(&unifier);
                self.type_store.initialize_expr_with_func(
                    expr_id,
                    func_type_info.result.clone(),
                    func_type_info,
                );
            }
            Expr::DynamicFunctionCall(_, args) => {
                let func_type_info = create_general_function_type_info(
                    args.len(),
                    &mut self.type_info_provider.type_var_generator,
                );
                self.type_store.initialize_expr_with_func(
                    expr_id,
                    func_type_info.result.clone(),
                    func_type_info,
                );
            }
            Expr::Do(_) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::ExprValue(_, _) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::FieldAccess(..) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::FloatLiteral(_) => {
                self.type_store
                    .initialize_expr(expr_id, self.program.get_float_type());
            }
            Expr::Formatter(..) => {
                self.type_store
                    .initialize_expr(expr_id, self.program.get_string_type());
            }
            Expr::If(..) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::IntegerLiteral(_) => {
                self.type_store
                    .initialize_expr(expr_id, self.program.get_int_type());
            }
            Expr::List(_) => {
                let ty = self.program.get_list_type(
                    self.type_info_provider
                        .type_var_generator
                        .get_new_type_var(),
                );
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::StaticFunctionCall(function_id, args) => {
                let static_func_type_info = self
                    .type_info_provider
                    .get_function_type(function_id, !self.group.items.contains(function_id));
                let mut func_type_info = create_general_function_type_info(
                    args.len(),
                    &mut self.type_info_provider.type_var_generator,
                );
                let mut unifier = self.program.get_unifier();
                let r = unifier.unify(
                    &func_type_info.function_type,
                    &static_func_type_info.function_type,
                );
                if r.is_err() {
                    let args: Vec<_> = args
                        .iter()
                        .map(|arg| self.type_store.get_expr_type(arg).clone())
                        .collect();
                    let location = self.program.exprs.get(&expr_id).location_id;
                    function_argument_mismatch(
                        self.program,
                        &static_func_type_info.function_type,
                        args,
                        location,
                        self.errors,
                    )
                }
                func_type_info.apply(&unifier);
                self.type_store.initialize_expr_with_func(
                    expr_id,
                    func_type_info.result.clone(),
                    func_type_info,
                );
            }
            Expr::StringLiteral(_) => {
                self.type_store
                    .initialize_expr(expr_id, self.program.get_string_type());
            }
            Expr::RecordInitialization(id, _) => {
                let record_type_info = self.type_info_provider.get_record_type_info(id);
                let ty = record_type_info.record_type.clone();
                self.type_store
                    .initialize_expr_with_record_type(expr_id, ty, record_type_info);
            }
            Expr::RecordUpdate(..) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::Tuple(items) => {
                let item_types: Vec<_> = items
                    .iter()
                    .map(|_| {
                        self.type_info_provider
                            .type_var_generator
                            .get_new_type_var()
                    })
                    .collect();
                let tuple_ty = Type::Tuple(item_types);
                self.type_store.initialize_expr(expr_id, tuple_ty);
            }
            Expr::TupleFieldAccess(_, _) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_expr(expr_id, ty);
            }
            Expr::Return(_) => {
                self.type_store.initialize_expr(
                    expr_id,
                    Type::Never(self.type_info_provider.type_var_generator.get_new_index()),
                );
            }
        }
    }

    fn visit_pattern(&mut self, pattern_id: PatternId, pattern: &Pattern) {
        //println!("I {} {:?}", pattern_id, pattern);
        match pattern {
            Pattern::Binding(_) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_pattern(pattern_id, ty);
            }
            Pattern::Guarded(_, _) => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_pattern(pattern_id, ty);
            }
            Pattern::IntegerLiteral(_) => {
                self.type_store
                    .initialize_pattern(pattern_id, self.program.get_int_type());
            }
            Pattern::CharLiteral(_) => {
                self.type_store
                    .initialize_pattern(pattern_id, self.program.get_char_type());
            }
            Pattern::CharRange(_, _, _) => {
                self.type_store
                    .initialize_pattern(pattern_id, self.program.get_char_type());
            }
            Pattern::Record(typedef_id, fields) => {
                let record_type_info = self.type_info_provider.get_record_type_info(typedef_id);
                if record_type_info.field_types.len() != fields.len() {
                    let location = self.program.patterns.get(&pattern_id).location_id;
                    let record = self.program.typedefs.get(typedef_id).get_record();
                    let err = TypecheckError::InvalidRecordPattern(
                        location,
                        record.name.clone(),
                        record_type_info.field_types.len(),
                        fields.len(),
                    );
                    self.errors.push(err);
                }
                let ty = record_type_info.record_type.clone();
                self.type_store.initialize_pattern_with_record_type(
                    pattern_id,
                    ty,
                    record_type_info,
                );
            }
            Pattern::StringLiteral(_) => {
                self.type_store
                    .initialize_pattern(pattern_id, self.program.get_string_type());
            }
            Pattern::Typed(_, type_signature) => {
                let ty = process_type_signature(
                    *type_signature,
                    self.program,
                    &mut self.type_info_provider.type_var_generator,
                );
                self.type_store.initialize_pattern(pattern_id, ty);
            }
            Pattern::Tuple(items) => {
                let item_types: Vec<_> = items
                    .iter()
                    .map(|_| {
                        self.type_info_provider
                            .type_var_generator
                            .get_new_type_var()
                    })
                    .collect();
                let tuple_ty = Type::Tuple(item_types);
                self.type_store.initialize_pattern(pattern_id, tuple_ty);
            }
            Pattern::Variant(typedef_id, index, args) => {
                let adt_type_info = self.type_info_provider.get_adt_type_info(typedef_id);
                let variant = &adt_type_info.variant_types[*index];
                if variant.item_types.len() != args.len() {
                    let location = self.program.patterns.get(&pattern_id).location_id;
                    let adt = self.program.typedefs.get(typedef_id).get_adt();
                    let variant_name = adt.variants[*index].name.clone();
                    let err = TypecheckError::InvalidVariantPattern(
                        location,
                        variant_name,
                        variant.item_types.len(),
                        args.len(),
                    );
                    self.errors.push(err);
                }
                let ty = adt_type_info.adt_type.clone();
                self.type_store
                    .initialize_pattern_with_adt_type(pattern_id, ty, adt_type_info);
            }
            Pattern::Wildcard => {
                let ty = self
                    .type_info_provider
                    .type_var_generator
                    .get_new_type_var();
                self.type_store.initialize_pattern(pattern_id, ty);
            }
        }
    }
}
