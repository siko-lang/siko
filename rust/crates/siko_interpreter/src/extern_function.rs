use crate::environment::Environment;
use crate::interpreter::ExprResult;
use crate::value::Value;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub trait ExternFunction {
    fn call(&self, _: &Environment, _: Option<ExprId>, _: &NamedFunctionKind, _: Type) -> Value {
        unimplemented!()
    }

    fn call2(
        &self,
        environment: &Environment,
        current_expr: Option<ExprId>,
        kind: &NamedFunctionKind,
        ty: Type,
    ) -> ExprResult {
        ExprResult::Ok(self.call(environment, current_expr, kind, ty))
    }
}