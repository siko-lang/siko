use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::util::get_opt_ordering_value;
use crate::util::get_ordering_value;
use crate::value::Value;
use crate::value::ValueCore;
use siko_constants::STRING_MODULE_NAME;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct StringAdd {}

impl ExternFunction for StringAdd {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_string();
        let r = environment.get_arg_by_index(1).core.as_string();
        return Value::new(ValueCore::String(l + &r), ty);
    }
}

pub struct StringPartialEq {}

impl ExternFunction for StringPartialEq {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_string();
        let r = environment.get_arg_by_index(1).core.as_string();
        return Interpreter::get_bool_value(l == r);
    }
}

pub struct StringPartialOrd {}

impl ExternFunction for StringPartialOrd {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_string();
        let r = environment.get_arg_by_index(1).core.as_string();
        let ord = l.partial_cmp(&r);
        return get_opt_ordering_value(ord);
    }
}

pub struct StringOrd {}

impl ExternFunction for StringOrd {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_string();
        let r = environment.get_arg_by_index(1).core.as_string();
        let ord = l.cmp(&r);
        return get_ordering_value(ord);
    }
}

pub struct StringShow {}

impl ExternFunction for StringShow {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let value = environment.get_arg_by_index(0).core.as_string();
        return Value::new(ValueCore::String(value.to_string()), ty);
    }
}

pub struct StringChars {}

impl ExternFunction for StringChars {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let value = environment.get_arg_by_index(0).core.as_string();
        let chars: Vec<_> = value
            .chars()
            .map(|c| Value::new(ValueCore::Char(c), Interpreter::get_char_type()))
            .collect();
        return Value::new(ValueCore::List(chars), ty);
    }
}

pub struct StringSplit {}

impl ExternFunction for StringSplit {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let input = environment.get_arg_by_index(0).core.as_string();
        let sep = environment.get_arg_by_index(1).core.as_string();
        let output: Vec<_> = input
            .split(&sep)
            .map(|o| {
                Value::new(
                    ValueCore::String(o.to_string()),
                    Interpreter::get_string_type(),
                )
            })
            .collect();
        return Value::new(ValueCore::List(output), ty);
    }
}

pub struct StringReplace {}

impl ExternFunction for StringReplace {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let input = environment.get_arg_by_index(0).core.as_string();
        let from = environment.get_arg_by_index(1).core.as_string();
        let to = environment.get_arg_by_index(2).core.as_string();
        let output = input.replace(&from, &to);
        return Value::new(ValueCore::String(output), ty);
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function(STRING_MODULE_NAME, "opAdd", Box::new(StringAdd {}));
    interpreter.add_extern_function(STRING_MODULE_NAME, "opEq", Box::new(StringPartialEq {}));
    interpreter.add_extern_function(
        STRING_MODULE_NAME,
        "partialCmp",
        Box::new(StringPartialOrd {}),
    );
    interpreter.add_extern_function(STRING_MODULE_NAME, "cmp", Box::new(StringOrd {}));
    interpreter.add_extern_function(STRING_MODULE_NAME, "show", Box::new(StringShow {}));
    interpreter.add_extern_function(STRING_MODULE_NAME, "chars", Box::new(StringChars {}));
    interpreter.add_extern_function(STRING_MODULE_NAME, "split", Box::new(StringSplit {}));
    interpreter.add_extern_function(STRING_MODULE_NAME, "replace", Box::new(StringReplace {}));
}
