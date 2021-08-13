use crate::interpreter::Interpreter;
use crate::value::Value;
use crate::value::ValueCore;
use siko_constants::OPTION_TYPE_NAME;
use siko_constants::ORDERING_TYPE_NAME;
use siko_ir::function::NamedFunctionKind;
use siko_ir::types::Type;
use std::cmp::Ordering;

pub fn get_instance_name_from_kind(kind: &NamedFunctionKind) -> &str {
    if let NamedFunctionKind::InstanceMember(Some(s)) = kind {
        s.as_ref()
    } else {
        unreachable!()
    }
}

pub fn get_opt_ordering_value(ordering: Option<Ordering>) -> Value {
    match ordering {
        Some(ordering) => {
            let value = get_ordering_value(ordering);
            return create_some(value);
        }
        None => {
            let value = create_ordering(0);
            return create_none(value.ty);
        }
    }
}

pub fn create_some(value: Value) -> Value {
    let cache = Interpreter::get_typedef_id_cache();
    let concrete_type = Type::Named(
        OPTION_TYPE_NAME.to_string(),
        cache.option_id,
        vec![value.ty.clone()],
    );
    let core = ValueCore::Variant(
        cache.option_id,
        cache.option_variants.get_index("Some"),
        vec![value],
    );
    let some_value = Value::new(core, concrete_type);
    some_value
}

pub fn create_none(value_ty: Type) -> Value {
    let cache = Interpreter::get_typedef_id_cache();
    let concrete_type = Type::Named(
        OPTION_TYPE_NAME.to_string(),
        cache.option_id,
        vec![value_ty],
    );
    let core = ValueCore::Variant(
        cache.option_id,
        cache.option_variants.get_index("None"),
        vec![],
    );
    let none_value = Value::new(core, concrete_type);
    none_value
}

pub fn create_ordering(index: usize) -> Value {
    let cache = Interpreter::get_typedef_id_cache();
    let concrete_type = Type::Named(ORDERING_TYPE_NAME.to_string(), cache.ordering_id, vec![]);
    let core = ValueCore::Variant(cache.ordering_id, index, vec![]);
    let value = Value::new(core, concrete_type);
    value
}

pub fn get_ordering_value(ordering: Ordering) -> Value {
    let cache = Interpreter::get_typedef_id_cache();
    match ordering {
        Ordering::Less => create_ordering(cache.ordering_variants.get_index("Less")),
        Ordering::Equal => create_ordering(cache.ordering_variants.get_index("Equal")),
        Ordering::Greater => create_ordering(cache.ordering_variants.get_index("Greater")),
    }
}

pub fn create_json_string(value: Value) -> Value {
    let cache = Interpreter::get_typedef_id_cache();
    let concrete_type = Type::Named("Json".to_string(), cache.json_id, vec![]);
    let core = ValueCore::Variant(
        cache.json_id,
        cache.json_variants.get_index("JsonString"),
        vec![value],
    );
    let js_value = Value::new(core, concrete_type);
    js_value
}

pub fn create_json_object(value: Value) -> Value {
    let cache = Interpreter::get_typedef_id_cache();
    let concrete_type = Type::Named("Json".to_string(), cache.json_id, vec![]);
    let core = ValueCore::Variant(
        cache.json_id,
        cache.json_variants.get_index("JsonObject"),
        vec![value],
    );
    let js_value = Value::new(core, concrete_type);
    js_value
}

pub fn create_json_list(value: Value) -> Value {
    let cache = Interpreter::get_typedef_id_cache();
    let concrete_type = Type::Named("Json".to_string(), cache.json_id, vec![]);
    let core = ValueCore::Variant(
        cache.json_id,
        cache.json_variants.get_index("JsonList"),
        vec![value],
    );
    let js_value = Value::new(core, concrete_type);
    js_value
}

pub fn create_json_object_item(name: Value, value: Value) -> Value {
    let cache = Interpreter::get_typedef_id_cache();
    let concrete_type = Type::Named("Json".to_string(), cache.json_object_item_id, vec![]);
    let core = ValueCore::Record(cache.json_object_item_id, vec![name, value]);
    let js_value = Value::new(core, concrete_type);
    js_value
}
