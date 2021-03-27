use crate::common::FunctionTypeInfo;
use crate::error::TypecheckError;
use crate::type_info_provider::TypeInfoProvider;
use crate::type_store::TypeStore;
use crate::util::create_general_function_type_info;
use siko_ir::expr::Expr;
use siko_ir::expr::ExprId;
use siko_ir::pattern::Pattern;
use siko_ir::pattern::PatternId;
use siko_ir::program::Program;
use siko_ir::types::Type;
use siko_ir::unifier::Unifier;
use siko_ir::walker::Visitor;
use siko_location_info::location_id::LocationId;

pub struct ClassConstraintChecker<'a> {
    program: &'a mut Program,
    type_store: &'a mut TypeStore,
    errors: &'a mut Vec<TypecheckError>,
    type_info_provider: &'a mut TypeInfoProvider,
}

impl<'a> ClassConstraintChecker<'a> {
    pub fn new(
        program: &'a mut Program,
        type_store: &'a mut TypeStore,
        errors: &'a mut Vec<TypecheckError>,
        type_info_provider: &'a mut TypeInfoProvider,
    ) -> ClassConstraintChecker<'a> {
        ClassConstraintChecker {
            program: program,
            type_store: type_store,
            errors: errors,
            type_info_provider: type_info_provider,
        }
    }

    fn unify(
        &mut self,
        ty1: &Type,
        ty2: &Type,
        location: LocationId,
        func_type_info: Option<&mut FunctionTypeInfo>,
    ) {
        let mut unifier = Unifier::new(self.type_info_provider.type_var_generator.clone());
        let mut failed = false;
        let r = unifier.unify(ty1, ty2);
        assert!(r.is_ok());
        let constraints = unifier.get_constraints();
        for constraint in &constraints {
            let mut unifiers = Vec::new();
            if !self.program.instance_resolver.check_instance(
                constraint.class_id,
                &constraint.ty,
                location,
                &mut unifiers,
            ) {
                failed = true;
                break;
            }
        }
        if failed {
            println!("unification failed {} {}", ty1, ty2);
            let ty_str1 = ty1.get_resolved_type_string(self.program);
            let ty_str2 = ty2.get_resolved_type_string(self.program);
            let err = TypecheckError::TypeMismatch(location, ty_str1, ty_str2);
            self.errors.push(err);
        } else {
            if let Some(func_type_info) = func_type_info {
                func_type_info.apply(&unifier);
            }
        }
    }

    fn check_function_call(
        &mut self,
        args: &Vec<ExprId>,
        function_type: &Type,
        location: LocationId,
        expr_id: ExprId,
    ) {
        let mut func_type_info = create_general_function_type_info(
            args.len(),
            &mut self.type_info_provider.type_var_generator,
        );
        let mut unifier = self.program.get_unifier();
        let r = unifier.unify(&func_type_info.function_type, &function_type);
        assert!(r.is_ok());
        func_type_info.apply(&unifier);
        for (index, arg) in args.iter().enumerate() {
            let arg_type = &func_type_info.args[index].clone();
            let expr_ty = self.type_store.get_expr_type(arg).clone();
            self.unify(arg_type, &expr_ty, location, Some(&mut func_type_info));
        }
        let expr_ty = self.type_store.get_expr_type(&expr_id).clone();
        let result_ty = func_type_info.function_type.get_result_type(args.len());
        self.unify(&result_ty, &expr_ty, location, Some(&mut func_type_info));
    }
}

impl<'a> Visitor for ClassConstraintChecker<'a> {
    fn get_program(&self) -> &Program {
        &self.program
    }

    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr) {
        let location = self.program.exprs.get(&expr_id).location_id;
        match expr {
            Expr::ClassFunctionCall(class_member_id, args) => {
                let class_member_type = self
                    .type_info_provider
                    .get_class_member_type(class_member_id);
                self.check_function_call(args, &class_member_type, location, expr_id);
            }
            Expr::DynamicFunctionCall(func_expr_id, args) => {
                let function_type = self.type_store.get_expr_type(func_expr_id).clone();
                self.check_function_call(args, &function_type, location, expr_id);
            }
            Expr::Formatter(_, args) => {
                for arg in args {
                    let show_type = self.program.get_show_type();
                    let expr_ty = self.type_store.get_expr_type(arg).clone();
                    let location = self.program.exprs.get(&arg).location_id;
                    self.unify(&show_type, &expr_ty, location, None);
                }
            }
            Expr::StaticFunctionCall(id, args) => {
                let func_type_info = self
                    .type_info_provider
                    .function_type_info_store
                    .get(id)
                    .clone()
                    .remove_fixed_types();
                self.check_function_call(args, &func_type_info.function_type, location, expr_id);
            }
            _ => {}
        }
    }

    fn visit_pattern(&mut self, _: PatternId, _: &Pattern) {}
}
