use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::interpreter::ExprResult;
use crate::value::Value;
use crate::value::ValueCore;
use siko_ir::expr::{ExprId};
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct Assert {}

impl ExternFunction for Assert {
    fn call2(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> ExprResult {
        let v = environment.get_arg_by_index(0).core.as_bool();
        if !v {
            println!("Assertion failed!");
            return ExprResult::Abort;
        }
        let v= Value::new(ValueCore::Tuple(vec![]), ty);
        ExprResult::Ok(v)
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function("Std.Util", "assert", Box::new(Assert {}));
}
