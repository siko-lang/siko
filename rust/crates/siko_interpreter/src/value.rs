use crate::interpreter::ExprResult;
use crate::interpreter::Interpreter;
use im_rc::Vector;
use siko_ir::data::TypeDefId;
use siko_ir::function::FunctionId;
use siko_ir::program::Program;
use siko_ir::types::Type;
use siko_ir::unifier::Unifier;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub enum BuiltinCallable {
    Show,
    PartialEq,
    PartialOrd,
    Ord,
    ToJson,
    FromJson,
}

#[derive(Debug, Clone, Copy)]
pub enum CallableKind {
    FunctionId(FunctionId),
    Builtin(BuiltinCallable),
}

#[derive(Debug, Clone)]
pub struct Callable {
    pub kind: CallableKind,
    pub values: Vec<Value>,
    pub unifier: Unifier,
}

#[derive(Debug, Clone)]
pub struct Value {
    pub core: Rc<ValueCore>,
    pub ty: Type,
}

impl Value {
    pub fn new(core: ValueCore, ty: Type) -> Value {
        Value {
            core: Rc::new(core),
            ty: ty,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        let copy = self.clone();
        let other = other.clone();
        let v = Interpreter::call_op_eq(copy, other);
        v.core.as_bool()
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let copy = self.clone();
        let other = other.clone();
        let v = Interpreter::call_op_partial_cmp(copy, other);
        match v.core.as_option(0, 1) {
            Some(v) => Some(v.core.as_ordering(0, 1, 2)),
            None => None,
        }
    }
}

impl Eq for Value {}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        let copy = self.clone();
        let other = other.clone();
        let v = Interpreter::call_op_cmp(copy, other);
        v.core.as_ordering(0, 1, 2)
    }
}

#[derive(Debug, Clone)]
pub enum ValueCore {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Tuple(Vec<Value>),
    Callable(Callable),
    Variant(TypeDefId, usize, Vec<Value>),
    Record(TypeDefId, Vec<Value>),
    List(Vector<Value>),
    Map(BTreeMap<Value, Value>),
    Iterator(Box<Value>),
    IteratorMap(Box<Value>, Box<Value>),
    IteratorFilter(Box<Value>, Box<Value>),
}

impl ValueCore {
    pub fn as_int(&self) -> i64 {
        match self {
            ValueCore::Int(i) => *i,
            _ => unreachable!(),
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            ValueCore::Float(i) => *i,
            _ => unreachable!(),
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            ValueCore::String(i) => i.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            ValueCore::Char(c) => c.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            ValueCore::Variant(_, 0, _) => true,
            ValueCore::Variant(_, 1, _) => false,
            _ => panic!("Trying to use {:?} as bool", self),
        }
    }

    pub fn as_simple_enum_variant(&self) -> (TypeDefId, usize) {
        match self {
            ValueCore::Variant(id, index, _) => (*id, *index),
            _ => unreachable!(),
        }
    }

    pub fn as_option(&self, some_index: usize, none_index: usize) -> Option<Value> {
        match self {
            ValueCore::Variant(_, index, items) if *index == some_index => Some(items[0].clone()),
            ValueCore::Variant(_, index, _) if *index == none_index => None,
            _ => unreachable!(),
        }
    }

    pub fn as_ordering(
        &self,
        less_index: usize,
        equal_index: usize,
        greater_index: usize,
    ) -> Ordering {
        match self {
            ValueCore::Variant(_, index, _) if *index == less_index => Ordering::Less,
            ValueCore::Variant(_, index, _) if *index == equal_index => Ordering::Equal,
            ValueCore::Variant(_, index, _) if *index == greater_index => Ordering::Greater,
            _ => unreachable!(),
        }
    }

    pub fn as_map(&self) -> BTreeMap<Value, Value> {
        match self {
            ValueCore::Map(m) => m.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_list(&self) -> &Vector<Value> {
        match self {
            ValueCore::List(l) => l,
            _ => unreachable!(),
        }
    }

    pub fn as_tuple(&self) -> Vec<Value> {
        match self {
            ValueCore::Tuple(items) => items.clone(),
            _ => unreachable!(),
        }
    }

    pub fn show(&self, program: &Program) -> String {
        match self {
            ValueCore::Variant(id, index, items) => {
                let adt = program.typedefs.get(id).get_adt();
                let variant = &adt.variants[*index];
                let mut item_strings = Vec::new();
                for item in items {
                    let item_str = Interpreter::call_show(item.clone());
                    item_strings.push(format!("({})", item_str));
                }
                if item_strings.is_empty() {
                    format!("{}", variant.name)
                } else {
                    format!("{} {}", variant.name, item_strings.join(" "))
                }
            }
            ValueCore::Record(id, fields) => {
                let record = program.typedefs.get(id).get_record();
                let mut field_strings = Vec::new();
                for (index, field_value) in fields.iter().enumerate() {
                    let field = &record.fields[index];
                    let field_str = Interpreter::call_show(field_value.clone());
                    field_strings.push(format!("{}: {}", field.name, field_str));
                }
                if field_strings.is_empty() {
                    format!("{}", record.name)
                } else {
                    format!("{} {{ {} }}", record.name, field_strings.join(", "))
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn as_iterator(&self) -> Box<dyn Iterator<Item = Value>> {
        match self {
            ValueCore::Iterator(v) => match (*v.core).clone() {
                ValueCore::List(v) => Box::new(v.into_iter()),
                ValueCore::Map(v) => Box::new(v.into_iter().map(|(k, v)| {
                    Value::new(
                        ValueCore::Tuple(vec![k.clone(), v.clone()]),
                        Type::Tuple(vec![k.ty.clone(), v.ty.clone()]),
                    )
                })),
                _ => unreachable!(),
            },
            ValueCore::IteratorMap(v, func) => {
                let func = *func.clone();
                let iterator = v.core.as_iterator();
                let iterator = iterator.map(move |x| {
                    if let ExprResult::Ok(v) = Interpreter::call_func(func.clone(), vec![x], None) {
                        v
                    } else {
                        unreachable!()
                    }
                });

                Box::new(iterator)
            }
            ValueCore::IteratorFilter(v, func) => {
                let func = *func.clone();
                let iterator = v.core.as_iterator();
                let iterator = iterator.filter(move |x| {
                    if let ExprResult::Ok(v) =
                        Interpreter::call_func(func.clone(), vec![x.clone()], None)
                    {
                        v.core.as_bool()
                    } else {
                        unreachable!()
                    }
                });
                Box::new(iterator)
            }
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for ValueCore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueCore::Int(v) => write!(f, "{}", v),
            ValueCore::Float(v) => write!(f, "{}", v),
            ValueCore::String(v) => write!(f, "{}", v),
            ValueCore::Char(v) => write!(f, "{}", v),
            ValueCore::Tuple(vs) => {
                let ss: Vec<_> = vs.iter().map(|v| format!("{}", v.core)).collect();
                write!(f, "({})", ss.join(", "))
            }
            ValueCore::Callable(_) => write!(f, "<closure>"),
            ValueCore::Variant(id, index, vs) => {
                let ss: Vec<_> = vs.iter().map(|v| format!("{}", v.core)).collect();
                write!(f, "V([{}/{}]{})", id, index, ss.join(", "))
            }
            ValueCore::Record(id, vs) => {
                let ss: Vec<_> = vs.iter().map(|v| format!("{}", v.core)).collect();
                write!(f, "R([{}]{})", id, ss.join(", "))
            }
            ValueCore::List(vs) => {
                let ss: Vec<_> = vs.iter().map(|v| format!("{}", v.core)).collect();
                write!(f, "[{}]", ss.join(", "))
            }
            ValueCore::Map(vs) => {
                let ss: Vec<_> = vs
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k.core, v.core))
                    .collect();
                write!(f, "{{{}}}", ss.join(", "))
            }
            ValueCore::Iterator(v) => write!(f, "Iterator({})", v.core),
            ValueCore::IteratorMap(v, func) => write!(f, "IteratorMap({}, {})", v.core, func.core),
            ValueCore::IteratorFilter(v, func) => {
                write!(f, "IteratorFilter({}, {})", v.core, func.core)
            }
        }
    }
}
