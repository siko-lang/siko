use crate::value::CallableKind;
use crate::value::Value;
use siko_ir::expr::FunctionArgumentRef;
use siko_ir::pattern::PatternId;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Environment<'a> {
    callable_kind: CallableKind,
    args: Vec<Value>,
    variables: BTreeMap<PatternId, Value>,
    parent: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new(callable_kind: CallableKind, args: Vec<Value>) -> Environment<'a> {
        Environment {
            callable_kind: callable_kind,
            args: args,
            variables: BTreeMap::new(),
            parent: None,
        }
    }

    pub fn add(&mut self, var: PatternId, value: Value) {
        self.variables.insert(var, value);
    }

    pub fn get_value(&self, var: &PatternId) -> Value {
        if let Some(value) = self.variables.get(var) {
            return value.clone();
        } else {
            if let Some(parent) = self.parent {
                parent.get_value(var)
            } else {
                panic!("Value {} not found", var);
            }
        }
    }

    pub fn block_child(parent: &'a Environment<'a>) -> Environment<'a> {
        Environment {
            callable_kind: parent.callable_kind,
            args: parent.args.clone(),
            variables: BTreeMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get_arg(&self, arg_ref: &FunctionArgumentRef) -> Value {
        if let CallableKind::FunctionId(id) = self.callable_kind {
            assert_eq!(id, arg_ref.id);
            assert!(!arg_ref.captured);
            assert!(self.args[arg_ref.index].ty.is_concrete_type());
            return self.args[arg_ref.index].clone();
        }
        if let Some(parent) = self.parent {
            let v = parent.get_arg(arg_ref);
            assert!(v.ty.is_concrete_type());
            return v;
        } else {
            unreachable!()
        }
    }

    pub fn get_arg_by_index(&self, index: usize) -> &Value {
        return &self.args[index];
    }
}
