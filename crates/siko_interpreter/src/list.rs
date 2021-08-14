use crate::environment::Environment;
use crate::extern_function::ExternFunction;
use crate::interpreter::ExprResult;
use crate::interpreter::Interpreter;
use crate::util::as_json_object_items;
use crate::util::create_json_list;
use crate::util::create_none;
use crate::util::create_some;
use crate::util::get_opt_ordering_value;
use crate::util::get_ordering_value;
use crate::value::Value;
use crate::value::ValueCore;
use siko_constants::LIST_MODULE_NAME;
use siko_ir::expr::ExprId;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;

pub struct Show {}

impl ExternFunction for Show {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let v = environment.get_arg_by_index(0);
        let list = v.core.as_list();
        let mut subs = Vec::new();
        for item in list {
            let s = Interpreter::call_show(item.clone());
            subs.push(s);
        }
        return Value::new(ValueCore::String(format!("[{}]", subs.join(", "))), ty);
    }
}

pub struct ToJson {}

impl ExternFunction for ToJson {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _ty: Type,
    ) -> Value {
        let v = environment.get_arg_by_index(0);
        let list = v.core.as_list();
        let mut subs = Vec::new();
        for item in list {
            let s = Interpreter::call_op_tojson(item.clone());
            subs.push(s);
        }
        return create_json_list(subs);
    }
}

pub struct FromJson {}

impl ExternFunction for FromJson {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let v = environment.get_arg_by_index(0);
        let values = as_json_object_items(v);
        let mut subs = Vec::new();
        let list_type_arg = ty.get_type_args()[0].clone();
        for item in values {
            let s = Interpreter::call_op_fromjson(item.clone(), list_type_arg.clone());
            subs.push(s);
        }
        let core = ValueCore::List(subs);
        return Value::new(core, ty);
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
        let list = environment.get_arg_by_index(0);
        return Value::new(ValueCore::Iterator(Box::new(list.clone())), ty);
    }
}

pub struct ToList {}

impl ExternFunction for ToList {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let iter = environment.get_arg_by_index(0);
        let iter = iter.core.as_iterator();
        let list: Vec<_> = iter.collect();
        return Value::new(ValueCore::List(list), ty);
    }
}

pub struct ListPartialEq {}

impl ExternFunction for ListPartialEq {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0);
        let l = l.core.as_list();
        let r = environment.get_arg_by_index(1);
        let r = r.core.as_list();
        if l.len() != r.len() {
            return Interpreter::get_bool_value(false);
        }
        for (a, b) in l.iter().zip(r.iter()) {
            let r = Interpreter::call_op_eq(a.clone(), b.clone());
            if !r.core.as_bool() {
                return r;
            }
        }
        return Interpreter::get_bool_value(true);
    }
}

pub struct ListPartialOrd {}

impl ExternFunction for ListPartialOrd {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0);
        let l = l.core.as_list();
        let r = environment.get_arg_by_index(1);
        let r = r.core.as_list();
        if l.len() != r.len() {
            return get_opt_ordering_value(l.len().partial_cmp(&r.len()));
        }
        for (a, b) in l.iter().zip(r.iter()) {
            let r = Interpreter::call_op_partial_cmp(a.clone(), b.clone());
            match r.core.as_option(0, 1) {
                Some(v) => match v.core.as_ordering(0, 1, 2) {
                    std::cmp::Ordering::Equal => continue,
                    _ => {
                        return r;
                    }
                },
                None => {
                    return r;
                }
            }
        }
        return get_opt_ordering_value(Some(std::cmp::Ordering::Equal));
    }
}

pub struct ListOrd {}

impl ExternFunction for ListOrd {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let l = environment.get_arg_by_index(0);
        let l = l.core.as_list();
        let r = environment.get_arg_by_index(1);
        let r = r.core.as_list();
        if l.len() != r.len() {
            return get_ordering_value(l.len().cmp(&r.len()));
        }
        for (a, b) in l.iter().zip(r.iter()) {
            let r = Interpreter::call_op_cmp(a.clone(), b.clone());
            match r.core.as_ordering(0, 1, 2) {
                std::cmp::Ordering::Equal => continue,
                _ => {
                    return r;
                }
            }
        }
        return get_ordering_value(std::cmp::Ordering::Equal);
    }
}

pub struct ListAdd {}

impl ExternFunction for ListAdd {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let mut l = environment.get_arg_by_index(0).core.as_list().clone();
        let r = environment.get_arg_by_index(1).core.as_list().clone();
        l.extend(r.into_iter());
        return Value::new(ValueCore::List(l), ty);
    }
}

pub struct AtIndex {}

impl ExternFunction for AtIndex {
    fn call2(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> ExprResult {
        let list = environment.get_arg_by_index(0);
        let index = environment.get_arg_by_index(1).core.as_int();
        let list = list.core.as_list();
        if list.len() <= index as usize {
            println!("PANIC!: atIndex: size: {} index: {}", list.len(), index);
            return ExprResult::Abort;
        } else {
            let value = list[index as usize].clone();
            return ExprResult::Ok(value);
        }
    }
}

pub struct Remove {}

impl ExternFunction for Remove {
    fn call2(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> ExprResult {
        let list = environment.get_arg_by_index(0);
        let list_ty = list.ty.clone();
        let index = environment.get_arg_by_index(1).core.as_int();
        let mut list = list.core.as_list().clone();
        if list.len() <= index as usize {
            println!("PANIC!: remove: size: {} index: {}", list.len(), index);
            return ExprResult::Abort;
        } else {
            let item = list.remove(index as usize);
            let list = Value::new(ValueCore::List(list), list_ty);
            let tuple = Value::new(ValueCore::Tuple(vec![item, list]), ty);
            return ExprResult::Ok(tuple);
        }
    }
}

pub struct GetLength {}

impl ExternFunction for GetLength {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let list = environment.get_arg_by_index(0);
        let len = list.core.as_list().len();
        return Value::new(ValueCore::Int(len as i64), ty);
    }
}

pub struct IsEmpty {}

impl ExternFunction for IsEmpty {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let list = environment.get_arg_by_index(0);
        let empty = list.core.as_list().is_empty();
        return Interpreter::get_bool_value(empty);
    }
}

pub struct Head {}

impl ExternFunction for Head {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let list = environment.get_arg_by_index(0);
        let mut list_type_args = list.ty.get_type_args();
        match list.core.as_list().first() {
            Some(v) => {
                return create_some(v.clone());
            }
            None => {
                return create_none(list_type_args.remove(0));
            }
        }
    }
}

pub struct Tail {}

impl ExternFunction for Tail {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        _: Type,
    ) -> Value {
        let list = environment.get_arg_by_index(0);
        let ty = list.ty.clone();
        let mut list_type_args = list.ty.get_type_args();
        let mut list: Vec<_> = list.core.as_list().clone();
        if list.is_empty() {
            return create_none(list_type_args.remove(0));
        } else {
            let _ = list.remove(0);
            return create_some(Value::new(ValueCore::List(list), ty));
        }
    }
}

pub struct Sort {}

impl ExternFunction for Sort {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let list = environment.get_arg_by_index(0);
        let mut list: Vec<_> = list.core.as_list().clone();
        list.sort();
        return Value::new(ValueCore::List(list), ty);
    }
}

pub struct Dedup {}

impl ExternFunction for Dedup {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let list = environment.get_arg_by_index(0);
        let mut list: Vec<_> = list.core.as_list().clone();
        list.dedup();
        return Value::new(ValueCore::List(list), ty);
    }
}

pub struct Write {}

impl ExternFunction for Write {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let list = environment.get_arg_by_index(0);
        let index = environment.get_arg_by_index(1).core.as_int();
        let item = environment.get_arg_by_index(2).clone();
        let mut list: Vec<_> = list.core.as_list().clone();
        list[index as usize] = item;
        return Value::new(ValueCore::List(list), ty);
    }
}

pub struct Split {}

impl ExternFunction for Split {
    fn call(
        &self,
        environment: &Environment,
        _: Option<ExprId>,
        _: &NamedFunctionKind,
        ty: Type,
    ) -> Value {
        let orig_list = environment.get_arg_by_index(0);
        let index = environment.get_arg_by_index(1).core.as_int();
        let mut list: Vec<_> = orig_list.core.as_list().clone();
        let rest = list.split_off(index as usize);
        let v1 = Value::new(ValueCore::List(list), orig_list.ty.clone());
        let v2 = Value::new(ValueCore::List(rest), orig_list.ty.clone());
        return Value::new(ValueCore::Tuple(vec![v1, v2]), ty);
    }
}

pub fn register_extern_functions(interpreter: &mut Interpreter) {
    interpreter.add_extern_function(LIST_MODULE_NAME, "show", Box::new(Show {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "toJson", Box::new(ToJson {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "fromJson", Box::new(FromJson {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "iter", Box::new(Iter {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "toList", Box::new(ToList {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "opEq", Box::new(ListPartialEq {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "opAdd", Box::new(ListAdd {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "atIndex", Box::new(AtIndex {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "getLength", Box::new(GetLength {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "isEmpty", Box::new(IsEmpty {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "head", Box::new(Head {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "tail", Box::new(Tail {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "sort", Box::new(Sort {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "dedup", Box::new(Dedup {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "partialCmp", Box::new(ListPartialOrd {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "cmp", Box::new(ListOrd {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "write", Box::new(Write {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "split", Box::new(Split {}));
    interpreter.add_extern_function(LIST_MODULE_NAME, "remove", Box::new(Remove {}));
}
