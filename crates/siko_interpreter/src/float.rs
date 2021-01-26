use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::util::get_opt_ordering_value;
use crate::value::Value;
use crate::value::ValueCore;
use siko_constants::FLOAT_MODULE_NAME;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct FloatAdd {}

impl ExternFunction for FloatAdd {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_float();
        let r = environment.get_arg_by_index(1).core.as_float();
        return Value::new(ValueCore::Float(l + r), ty);
    }
}

pub struct FloatSub {}

impl ExternFunction for FloatSub {
    fn call(
        &self,
        environment: & Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_float();
        let r = environment.get_arg_by_index(1).core.as_float();
        return Value::new(ValueCore::Float(l - r), ty);
    }
}

pub struct FloatMul {}

impl ExternFunction for FloatMul {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_float();
        let r = environment.get_arg_by_index(1).core.as_float();
        return Value::new(ValueCore::Float(l * r), ty);
    }
}

pub struct FloatDiv {}

impl ExternFunction for FloatDiv {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_float();
        let r = environment.get_arg_by_index(1).core.as_float();
        return Value::new(ValueCore::Float(l / r), ty);
    }
}

pub struct FloatPartialEq {}

impl ExternFunction for FloatPartialEq {
    fn call(
        &self,
        environment: & Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_float();
        let r = environment.get_arg_by_index(1).core.as_float();
        return Interpreter::get_bool_value(l == r);
    }
}

pub struct FloatPartialOrd {}

impl ExternFunction for FloatPartialOrd {
    fn call(
        &self,
        environment: & Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_float();
        let r = environment.get_arg_by_index(1).core.as_float();
        let ord = l.partial_cmp(&r);
        return get_opt_ordering_value(ord);
    }
}

pub struct FloatShow {}

impl ExternFunction for FloatShow {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let value = environment.get_arg_by_index(0).core.as_float();
        return Value::new(ValueCore::String(value.to_string()), ty);
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function(FLOAT_MODULE_NAME, "opAdd", Box::new(FloatAdd {}));
    interpreter.add_extern_function(FLOAT_MODULE_NAME, "opSub", Box::new(FloatSub {}));
    interpreter.add_extern_function(FLOAT_MODULE_NAME, "opMul", Box::new(FloatMul {}));
    interpreter.add_extern_function(FLOAT_MODULE_NAME, "opDiv", Box::new(FloatDiv {}));
    interpreter.add_extern_function(FLOAT_MODULE_NAME, "opEq", Box::new(FloatPartialEq {}));
    interpreter.add_extern_function(
        FLOAT_MODULE_NAME,
        "partialCmp",
        Box::new(FloatPartialOrd {}),
    );
    interpreter.add_extern_function(FLOAT_MODULE_NAME, "show", Box::new(FloatShow {}));
}
