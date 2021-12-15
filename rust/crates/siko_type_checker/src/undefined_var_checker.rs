use crate::error::TypecheckError;
use crate::type_store::TypeStore;
use siko_ir::expr::Expr;
use siko_ir::expr::ExprId;
use siko_ir::pattern::Pattern;
use siko_ir::pattern::PatternId;
use siko_ir::program::Program;
use siko_ir::types::Type;
use siko_ir::walker::Visitor;
use siko_location_info::location_id::LocationId;

pub struct UndefinedVarChecker<'a> {
    program: &'a Program,
    type_store: &'a mut TypeStore,
    errors: &'a mut Vec<TypecheckError>,
    func_args: Vec<usize>,
    undef_vars: Vec<usize>,
}

impl<'a> UndefinedVarChecker<'a> {
    pub fn new(
        program: &'a Program,
        type_store: &'a mut TypeStore,
        errors: &'a mut Vec<TypecheckError>,
        func_args: Vec<usize>,
    ) -> UndefinedVarChecker<'a> {
        UndefinedVarChecker {
            program: program,
            type_store: type_store,
            errors: errors,
            func_args: func_args,
            undef_vars: Vec::new(),
        }
    }

    fn check_type(&mut self, ty: &Type, location: LocationId) {
        let mut args = Vec::new();
        ty.collect_type_args(&mut args, self.program);
        for arg in args {
            if !self.func_args.contains(&arg) && !self.undef_vars.contains(&arg) {
                let err = TypecheckError::TypeAnnotationNeeded(location);
                self.errors.push(err);
                self.undef_vars.push(arg);
            }
        }
    }
}

impl<'a> Visitor for UndefinedVarChecker<'a> {
    fn get_program(&self) -> &Program {
        &self.program
    }

    fn visit_expr(&mut self, expr_id: ExprId, _: &Expr) {
        let ty = self.type_store.get_expr_type(&expr_id).clone();
        let location = self.program.exprs.get(&expr_id).location_id;
        self.check_type(&ty, location);
    }

    fn visit_pattern(&mut self, pattern_id: PatternId, _: &Pattern) {
        let ty = self.type_store.get_pattern_type(&pattern_id).clone();
        let location = self.program.patterns.get(&pattern_id).location_id;
        self.check_type(&ty, location);
    }
}
