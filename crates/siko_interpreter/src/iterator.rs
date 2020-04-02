use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::Interpreter;
use crate::value::Value;
use crate::value::ValueCore;
use siko_constants::ITERATOR_MODULE_NAME;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct Map {}

impl ExternFunction for Map {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let func = environment.get_arg_by_index(0).clone();
        let iterator = environment.get_arg_by_index(1).clone();
        return Value::new(
            ValueCore::IteratorMap(Box::new(iterator), Box::new(func)),
            ty,
        );
    }
}

pub struct Filter {}

impl ExternFunction for Filter {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let func = environment.get_arg_by_index(0).clone();
        let iterator = environment.get_arg_by_index(1).clone();
        return Value::new(
            ValueCore::IteratorFilter(Box::new(iterator), Box::new(func)),
            ty,
        );
    }
}

pub struct Fold {}

impl ExternFunction for Fold {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let func = environment.get_arg_by_index(0).clone();
        let initial = environment.get_arg_by_index(1).clone();
        let iterator = environment.get_arg_by_index(2).clone();
        let iterator = iterator.core.as_iterator();
        let value = iterator.fold(initial, move |acc, x| {
            Interpreter::call_func(func.clone(), vec![acc.clone(), x.clone()], None)
        });
        value
    }
}

pub struct ForEach {}

impl ExternFunction for ForEach {
    fn call(
        &self,
        environment: &mut Environment,
        expr_id: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let func = environment.get_arg_by_index(0);
        let iterator = environment.get_arg_by_index(1).core.as_iterator();
        for elem in iterator {
            Interpreter::call_func(func.clone(), vec![elem], expr_id);
        }
        return Value::new(ValueCore::Tuple(vec![]), ty);
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function(ITERATOR_MODULE_NAME, "map", Box::new(Map {}));
    interpreter.add_extern_function(ITERATOR_MODULE_NAME, "filter", Box::new(Filter {}));
    interpreter.add_extern_function(ITERATOR_MODULE_NAME, "fold", Box::new(Fold {}));
    interpreter.add_extern_function(ITERATOR_MODULE_NAME, "forEach", Box::new(ForEach {}));
}
