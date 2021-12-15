use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::value::Value;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct And {}

impl ExternFunction for And {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_bool();
        let r = environment.get_arg_by_index(1).core.as_bool();
        return Interpreter::get_bool_value(l && r);
    }
}

pub struct Or {}

impl ExternFunction for Or {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_bool();
        if l {
            return Interpreter::get_bool_value(l);
        } else {
            let r = environment.get_arg_by_index(1).core.as_bool();
            return Interpreter::get_bool_value(r);
        }
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function("Std.Ops", "opAnd", Box::new(And {}));
    interpreter.add_extern_function("Std.Ops", "opOr", Box::new(Or {}));
}
