use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::ExprResult;
use crate::interpreter::Interpreter;
use crate::util::create_none;
use crate::util::create_some;
use crate::util::get_ordering_value;
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
    fn call(&self, _: &Environment, _: Option<ExprId>, _: &NamedFunctionKind, ty: Type) -> Value {
        return Value::new(ValueCore::Map(BTreeMap::new()), ty);
    }
}

pub struct Insert {}

impl ExternFunction for Insert {
    fn call(
        &self,
        environment: &Environment,
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
        environment: &Environment,
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
        environment: &Environment,
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

pub struct Show {}

impl ExternFunction for Show {
    fn call(
        &self,
        environment: &Environment,
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
        environment: &Environment,
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
        environment: &Environment,
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
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let map = environment.get_arg_by_index(0).core.as_map();
        let s = map.len();
        return Value::new(ValueCore::Int(s as i64), ty);
    }
}

pub struct GetKeys {}

impl ExternFunction for GetKeys {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let map = environment.get_arg_by_index(0).core.as_map();
        let keys = map.keys().cloned().collect();
        return Value::new(ValueCore::List(keys), ty);
    }
}
pub struct Update {}

impl ExternFunction for Update {
    fn call2(
        &self,
        environment: &Environment,
        expr_id: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> ExprResult {
        let mut map = environment.get_arg_by_index(0).core.as_map();
        let f = environment.get_arg_by_index(1);
        for (m_k, m_v) in &mut map {
            let tuple_core = ValueCore::Tuple(vec![m_k.clone(), m_v.clone()]);
            let tuple = Value::new(tuple_core, Type::Tuple(vec![]));
            let r = Interpreter::call_func(f.clone(), vec![tuple], expr_id);
            match r {
                ExprResult::Ok(v) => {
                    *m_v = v;
                }
                e => {
                    return e;
                }
            }
        }
        return ExprResult::Ok(Value::new(ValueCore::Map(map), ty));
    }
}

pub struct UpdateS {}

impl ExternFunction for UpdateS {
    fn call2(
        &self,
        environment: &Environment,
        expr_id: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> ExprResult {
        let mut state = environment.get_arg_by_index(0).clone();
        let map2 = environment.get_arg_by_index(1);
        let mut map = map2.core.as_map();
        let f = environment.get_arg_by_index(2);
        for (m_k, m_v) in &mut map {
            let tuple_core = ValueCore::Tuple(vec![m_k.clone(), m_v.clone()]);
            let tuple = Value::new(tuple_core, Type::Tuple(vec![]));
            let r = Interpreter::call_func(f.clone(), vec![state.clone(), tuple], expr_id);
            match r {
                ExprResult::Ok(v) => {
                    let tuple = v.core.as_tuple();
                    state = tuple[0].clone();
                    *m_v = tuple[1].clone();
                }
                e => {
                    return e;
                }
            }
        }
        let map = Value::new(ValueCore::Map(map), map2.ty.clone());
        return ExprResult::Ok(Value::new(ValueCore::Tuple(vec![state, map]), ty));
    }
}

pub struct UpdateValues {}

impl ExternFunction for UpdateValues {
    fn call2(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> ExprResult {
        let mut map = environment.get_arg_by_index(0).core.as_map();
        let old_v = environment.get_arg_by_index(1);
        let new_v = environment.get_arg_by_index(2);
        for (_m_k, m_v) in &mut map {
            let r = Interpreter::call_op_partial_eq(old_v.clone(), m_v.clone());
            if r.core.as_bool() {
                *m_v = new_v.clone();
            }
        }
        return ExprResult::Ok(Value::new(ValueCore::Map(map), ty));
    }
}

pub struct MapPartialEq {}

impl ExternFunction for MapPartialEq {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0);
        let l = l.core.as_map();
        let r = environment.get_arg_by_index(1);
        let r = r.core.as_map();
        if l.len() != r.len() {
            return Interpreter::get_bool_value(false);
        }
        for ((k1, v1), (k2, v2)) in l.iter().zip(r.iter()) {
            let r = Interpreter::call_op_eq(k1.clone(), k2.clone());
            if !r.core.as_bool() {
                return r;
            }
            let r = Interpreter::call_op_eq(v1.clone(), v2.clone());
            if !r.core.as_bool() {
                return r;
            }
        }
        return Interpreter::get_bool_value(true);
    }
}

pub struct MapOrd {}

impl ExternFunction for MapOrd {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0);
        let l = l.core.as_map();
        let r = environment.get_arg_by_index(1);
        let r = r.core.as_map();
        if l.len() != r.len() {
            return get_ordering_value(l.len().cmp(&r.len()));
        }
        for (a, b) in l.iter().zip(r.iter()) {
            let r = Interpreter::call_op_cmp(a.0.clone(), b.0.clone());
            match r.core.as_ordering(0, 1, 2) {
                std::cmp::Ordering::Equal => {
                    let r = Interpreter::call_op_cmp(a.1.clone(), b.1.clone());
                    match r.core.as_ordering(0, 1, 2) {
                        std::cmp::Ordering::Equal => {
                            continue;
                        }
                        _ => {
                            return r;
                        }
                    }
                }
                _ => {
                    return r;
                }
            }
        }
        return get_ordering_value(std::cmp::Ordering::Equal);
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function(MAP_MODULE_NAME, "empty", Box::new(Empty {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "insert", Box::new(Insert {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "remove", Box::new(Remove {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "get", Box::new(Get {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "show", Box::new(Show {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "iter", Box::new(Iter {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "toMap", Box::new(ToMap {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "toMap2", Box::new(ToMap {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "getSize", Box::new(GetSize {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "opEq", Box::new(MapPartialEq {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "cmp", Box::new(MapOrd {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "update", Box::new(Update {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "updateS", Box::new(UpdateS {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "updateValues", Box::new(UpdateValues {}));
    interpreter.add_extern_function(MAP_MODULE_NAME, "getKeys", Box::new(GetKeys {}));
}
