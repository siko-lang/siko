use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::value::Value;
use crate::value::ValueCore;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct Assert {}

impl ExternFunction for Assert {
    fn call(
        &self,
        environment: &Environment,
        current_expr: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let v = environment.get_arg_by_index(0).core.as_bool();
        if !v {
            Interpreter::call_abort(current_expr.expect("No current expr"));
        }
        return Value::new(ValueCore::Tuple(vec![]), ty);
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function("Std.Util", "assert", Box::new(Assert {}));
}
