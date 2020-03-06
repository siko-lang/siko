use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::value::Value;
use crate::value::ValueCore;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct ReadTextFile {}

impl ExternFunction for ReadTextFile {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let path = environment.get_arg_by_index(0).core.as_string();
        let content = std::fs::read(&path).expect("ReadTextFile failed");
        let content = String::from_utf8_lossy(&content).to_string();
        return Value::new(ValueCore::String(content), ty);
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function("Hack", "readTextFile", Box::new(ReadTextFile {}));
}
