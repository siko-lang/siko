use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::ExprResult;
use crate::interpreter::Interpreter;
use crate::util::create_none;
use crate::util::create_some;
use crate::value::Value;
use crate::value::ValueCore;
use siko_constants::MAP_MODULE_NAME;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;
use std::collections::BTreeMap;
use std::rc::Rc;

pub struct Empty {}

impl ExternFunction for Empty {
    fn call(
        &self,
        _: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        return Value::new(ValueCore::Map(BTreeMap::new()), ty);
    }
}

pub struct Insert {}

impl ExternFunction for Insert {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let mut first_arg = environment.get_arg_by_index(0).clone();
        let mut map_type_args = first_arg.ty.get_type_args();
        let mut map = first_arg.core.as_map();
        let key = environment.get_arg_by_index(1).clone();
        let value = environment.get_arg_by_index(2).clone();
        let res = map.insert(key, value);
        let v = match res {
            Some(v) => create_some(v),
            None => create_none(map_type_args.remove(1)),
        };
        first_arg.core = Rc::new(ValueCore::Map(map));
        let tuple = Value::new(ValueCore::Tuple(vec![first_arg, v]), ty);
        return tuple;
    }
}

pub struct Remove {}

impl ExternFunction for Remove {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let mut first_arg = environment.get_arg_by_index(0).clone();
        let mut map_type_args = first_arg.ty.get_type_args();
        let mut map = first_arg.core.as_map();
        let key = environment.get_arg_by_index(1).clone();
        let res = map.remove(&key);
        let v = match res {
            Some(v) => create_some(v),
            None => create_none(map_type_args.remove(1)),
        };
        first_arg.core = Rc::new(ValueCore::Map(map));
        let tuple = Value::new(ValueCore::Tuple(vec![first_arg, v]), ty);
        return tuple;
    }
}

pub struct Get {}

impl ExternFunction for Get {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let first_arg = environment.get_arg_by_index(0);
        let mut map_type_args = first_arg.ty.get_type_args();
        let map = first_arg.core.as_map();
        let key = environment.get_arg_by_index(1);
        let res = map.get(&key);
        let v = match res {
            Some(v) => create_some(v.clone()),
            None => create_none(map_type_args.remove(1)),
        };
        return v;
    }
}

pub struct Alter {}

impl ExternFunction for Alter {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        result_ty: Type,
    ) -> Value {
        let func = environment.get_arg_by_index(0).clone();
        let key = environment.get_arg_by_index(1).clone();
        let map_arg = environment.get_arg_by_index(2).clone();
        let map_ty = map_arg.ty.clone();
        let mut map_type_args = map_arg.ty.get_type_args();
        let value_type = map_type_args.remove(1);
        let mut map = map_arg.core.as_map();
        match map.get(&key) {
            Some(v) => {
                let item = create_some(v.clone());
                let new = if let ExprResult::Ok(v) = Interpreter::call_func(func, vec![item], None)
                {
                    v
                } else {
                    unreachable!()
                };
                match new.core.as_option(0, 1) {
                    Some(v) => {
                        let res = map.insert(key, v);
                        let map = Value::new(ValueCore::Map(map), map_ty);
                        let removed_item = match res {
                            Some(v) => create_some(v),
                            None => create_none(value_type),
                        };
                        return Value::new(ValueCore::Tuple(vec![map, removed_item]), result_ty);
                    }
                    None => {
                        let removed_item = map.remove(&key);
                        let map = Value::new(ValueCore::Map(map), map_ty);
                        let removed_item =
                            create_some(removed_item.expect("removed item is empty"));
                        return Value::new(ValueCore::Tuple(vec![map, removed_item]), result_ty);
                    }
                }
            }
            None => {
                let empty = create_none(value_type.clone());
                let new = if let ExprResult::Ok(v) = Interpreter::call_func(func, vec![empty], None)
                {
                    v
                } else {
                    unreachable!()
                };
                match new.core.as_option(0, 1) {
                    Some(v) => {
                        map.insert(key, v);
                        let map = Value::new(ValueCore::Map(map), map_ty);
                        let removed_item = create_none(value_type);
                        return Value::new(ValueCore::Tuple(vec![map, removed_item]), result_ty);
                    }
                    None => {
                        let map = Value::new(ValueCore::Map(map), map_ty);
                        let removed_item = create_none(value_type);
                        return Value::new(ValueCore::Tuple(vec![map, removed_item]), result_ty);
                    }
                }
            }
        }
    }
}

pub struct Show {}

impl ExternFunction for Show {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let map = environment.get_arg_by_index(0).core.as_map();
        let mut subs = Vec::new();
        for (k, v) in map {
            let k_s = Interpreter::call_show(k);
            let v_s = Interpreter::call_show(v);
            subs.push(format!("{}:{}", k_s, v_s));
        }
        return Value::new(ValueCore::String(format!("{{{}}}", subs.join(", "))), ty);
    }
}

pub struct Iter {}

impl ExternFunction for Iter {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let map = environment.get_arg_by_index(0);
        return Value::new(ValueCore::Iterator(Box::new(map.clone())), ty);
    }
}

pub struct ToMap {}

impl ExternFunction for ToMap {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let iter = environment.get_arg_by_index(0);
        let iter = iter.core.as_iterator();
        let map: BTreeMap<_, _> = iter
            .map(|v| {
                let mut ts = v.core.as_tuple();
                (ts.remove(0), ts.remove(0))
            })
            .collect();
        return Value::new(ValueCore::Map(map), ty);
    }
}

pub struct GetSize {}

impl ExternFunction for GetSize {
    fn call(
        &self,
        environment: &mut Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let map = environment.get_arg_by_index(0).core.as_map();
        let s = map.len();
        return Value::new(ValueCore::Int(s as i64), ty);
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function(MAP_MODULE_NAME, "empty", Box::new(Empty {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "insert", Box::new(Insert {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "remove", Box::new(Remove {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "get", Box::new(Get {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "alter", Box::new(Alter {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "show", Box::new(Show {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "iter", Box::new(Iter {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "toMap", Box::new(ToMap {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "getSize", Box::new(GetSize {}));
}
