use siko_ir::expr::ExprId;
use siko_ir::expr::FunctionArgumentRef;
use siko_ir::function::FunctionId;
use siko_ir::pattern::PatternId;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub enum NamedRef {
    FunctionArg(FunctionArgumentRef),
    ExprValue(ExprId, PatternId),
}

pub struct Environment<'a> {
    variables: BTreeMap<String, NamedRef>,
    parent: Option<&'a Environment<'a>>,
    level: usize,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            variables: BTreeMap::new(),
            parent: None,
            level: 0,
        }
    }

    pub fn add_expr_value(&mut self, var: String, id: ExprId, pattern_id: PatternId) {
        self.variables
            .insert(var, NamedRef::ExprValue(id, pattern_id));
    }

    pub fn add_arg(&mut self, var: String, function_id: FunctionId, index: usize) {
        self.variables.insert(
            var,
            NamedRef::FunctionArg(FunctionArgumentRef::new(false, function_id, index)),
        );
    }

    pub fn get_ref(&self, var: &str) -> Option<(NamedRef, usize)> {
        if let Some(named_ref) = self.variables.get(var) {
            return Some((named_ref.clone(), self.level));
        } else {
            if let Some(parent) = self.parent {
                parent.get_ref(var)
            } else {
                None
            }
        }
    }

    pub fn child(parent: &'a Environment<'a>) -> Environment<'a> {
        Environment {
            variables: BTreeMap::new(),
            parent: Some(parent),
            level: parent.level + 1,
        }
    }

    pub fn level(&self) -> usize {
        self.level
    }
}
