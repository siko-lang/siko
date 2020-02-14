use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::util::get_opt_ordering_value;
use crate::util::get_ordering_value;
use crate::value::Value;
use crate::value::ValueCore;
use siko_constants::CHAR_MODULE_NAME;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct CharPartialEq {}

impl ExternFunction for CharPartialEq {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_char();
        let r = environment.get_arg_by_index(1).core.as_char();
        return Interpreter::get_bool_value(l == r);
    }
}

pub struct CharPartialOrd {}

impl ExternFunction for CharPartialOrd {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_char();
        let r = environment.get_arg_by_index(1).core.as_char();
        let ord = l.partial_cmp(&r);
        return get_opt_ordering_value(ord);
    }
}

pub struct CharOrd {}

impl ExternFunction for CharOrd {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_char();
        let r = environment.get_arg_by_index(1).core.as_char();
        let ord = l.cmp(&r);
        return get_ordering_value(ord);
    }
}

pub struct CharShow {}

impl ExternFunction for CharShow {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let value = environment.get_arg_by_index(0).core.as_char();
        return Value::new(ValueCore::String(value.to_string()), ty);
    }
}

pub struct CharIsUppercase {}

impl ExternFunction for CharIsUppercase {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let value = environment.get_arg_by_index(0).core.as_char();
        return Interpreter::get_bool_value(value.is_uppercase());
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function(CHAR_MODULE_NAME, "opEq", Box::new(CharPartialEq {}));
    interpreter.add_extern_function(CHAR_MODULE_NAME, "partialCmp", Box::new(CharPartialOrd {}));
    interpreter.add_extern_function(CHAR_MODULE_NAME, "cmp", Box::new(CharOrd {}));
    interpreter.add_extern_function(CHAR_MODULE_NAME, "show", Box::new(CharShow {}));
    interpreter.add_extern_function(CHAR_MODULE_NAME, "isUppercase", Box::new(CharIsUppercase {}));
}
