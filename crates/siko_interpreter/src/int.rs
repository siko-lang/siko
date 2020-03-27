use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::util::create_none;
use crate::util::create_some;
use crate::util::get_opt_ordering_value;
use crate::util::get_ordering_value;
use crate::value::Value;
use crate::value::ValueCore;
use siko_constants::INT_MODULE_NAME;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct IntAdd {}

impl ExternFunction for IntAdd {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_int();
        let r = environment.get_arg_by_index(1).core.as_int();
        return Value::new(ValueCore::Int(l + r), ty);
    }
}

pub struct IntSub {}

impl ExternFunction for IntSub {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_int();
        let r = environment.get_arg_by_index(1).core.as_int();
        return Value::new(ValueCore::Int(l - r), ty);
    }
}

pub struct IntMul {}

impl ExternFunction for IntMul {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_int();
        let r = environment.get_arg_by_index(1).core.as_int();
        return Value::new(ValueCore::Int(l * r), ty);
    }
}

pub struct IntDiv {}

impl ExternFunction for IntDiv {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_int();
        let r = environment.get_arg_by_index(1).core.as_int();
        return Value::new(ValueCore::Int(l / r), ty);
    }
}

pub struct IntPartialEq {}

impl ExternFunction for IntPartialEq {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_int();
        let r = environment.get_arg_by_index(1).core.as_int();
        return Interpreter::get_bool_value(l == r);
    }
}

pub struct IntPartialOrd {}

impl ExternFunction for IntPartialOrd {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_int();
        let r = environment.get_arg_by_index(1).core.as_int();
        let ord = l.partial_cmp(&r);
        return get_opt_ordering_value(ord);
    }
}

pub struct IntOrd {}

impl ExternFunction for IntOrd {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0).core.as_int();
        let r = environment.get_arg_by_index(1).core.as_int();
        let ord = l.cmp(&r);
        return get_ordering_value(ord);
    }
}

pub struct IntShow {}

impl ExternFunction for IntShow {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let value = environment.get_arg_by_index(0).core.as_int();
        return Value::new(ValueCore::String(value.to_string()), ty);
    }
}

pub struct Parse {}

impl ExternFunction for Parse {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let value = environment.get_arg_by_index(0).core.as_string();
        let mut ty_args = ty.get_type_args();
        let int_ty = ty_args.remove(0);
        match value.parse::<i64>() {
            Ok(v) => create_some(Value::new(ValueCore::Int(v), int_ty.clone())),
            Err(_) => create_none(int_ty.clone()),
        }
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function(INT_MODULE_NAME, "opAdd", Box::new(IntAdd {}));
    interpreter.add_extern_function(INT_MODULE_NAME, "opSub", Box::new(IntSub {}));
    interpreter.add_extern_function(INT_MODULE_NAME, "opMul", Box::new(IntMul {}));
    interpreter.add_extern_function(INT_MODULE_NAME, "opDiv", Box::new(IntDiv {}));
    interpreter.add_extern_function(INT_MODULE_NAME, "opEq", Box::new(IntPartialEq {}));
    interpreter.add_extern_function(INT_MODULE_NAME, "partialCmp", Box::new(IntPartialOrd {}));
    interpreter.add_extern_function(INT_MODULE_NAME, "cmp", Box::new(IntOrd {}));
    interpreter.add_extern_function(INT_MODULE_NAME, "show", Box::new(IntShow {}));
    interpreter.add_extern_function(INT_MODULE_NAME, "parse", Box::new(Parse {}));
}
