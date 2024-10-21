use std::collections::BTreeMap;

use crate::siko::{
    hir::{
        Data::{Class, Field},
        Function::{Block, Body, Function, FunctionKind, Instruction, InstructionKind, Parameter},
        Program::Program,
        Type::Type,
    },
    qualifiedname::QualifiedName,
    resolver::Resolver::createConstraintContext,
};

fn getUnit() -> QualifiedName {
    QualifiedName::Module("siko".to_string()).add("Unit".to_string())
}

trait RemoveTuples {
    fn removeTuples(&self) -> Self;
}

impl RemoveTuples for Type {
    fn removeTuples(&self) -> Self {
        match self {
            Type::Tuple(_) => Type::Named(getUnit(), Vec::new(), None),
            ty => ty.clone(),
        }
    }
}

impl RemoveTuples for Parameter {
    fn removeTuples(&self) -> Self {
        match self {
            Parameter::Named(n, ty, mutable) => {
                Parameter::Named(n.clone(), ty.removeTuples(), *mutable)
            }
            Parameter::SelfParam(mutable, ty) => Parameter::SelfParam(*mutable, ty.removeTuples()),
        }
    }
}

impl<T: RemoveTuples> RemoveTuples for Option<T> {
    fn removeTuples(&self) -> Self {
        match self {
            Some(t) => Some(t.removeTuples()),
            None => None,
        }
    }
}

impl<T: RemoveTuples> RemoveTuples for Vec<T> {
    fn removeTuples(&self) -> Self {
        self.iter().map(|item| item.removeTuples()).collect()
    }
}

impl<K: Clone + Ord, V: RemoveTuples> RemoveTuples for BTreeMap<K, V> {
    fn removeTuples(&self) -> Self {
        self.iter()
            .map(|(k, v)| (k.clone(), v.removeTuples()))
            .collect()
    }
}

impl RemoveTuples for Body {
    fn removeTuples(&self) -> Self {
        let blocks = self.blocks.removeTuples();
        Body { blocks: blocks }
    }
}

impl RemoveTuples for Block {
    fn removeTuples(&self) -> Self {
        let instructions = self.instructions.removeTuples();
        Block {
            id: self.id,
            instructions: instructions,
        }
    }
}

impl RemoveTuples for Instruction {
    fn removeTuples(&self) -> Self {
        let mut result = self.clone();
        result.ty = result.ty.removeTuples();
        result.kind = result.kind.removeTuples();
        result
    }
}

impl RemoveTuples for InstructionKind {
    fn removeTuples(&self) -> Self {
        match self {
            InstructionKind::Tuple(_) => InstructionKind::FunctionCall(getUnit(), Vec::new()),
            InstructionKind::TupleIndex(_, _) => todo!(),
            _ => self.clone(),
        }
    }
}

impl RemoveTuples for Function {
    fn removeTuples(&self) -> Self {
        let mut f = self.clone();
        f.params = f.params.removeTuples();
        f.result = f.result.removeTuples();
        f.body = f.body.removeTuples();
        f
    }
}

impl RemoveTuples for Field {
    fn removeTuples(&self) -> Self {
        let mut f = self.clone();
        f.ty = f.ty.removeTuples();
        f
    }
}

impl RemoveTuples for Class {
    fn removeTuples(&self) -> Self {
        let mut c = self.clone();
        c.ty = c.ty.removeTuples();
        c.fields = c.fields.removeTuples();
        c
    }
}

pub fn removeTuples(program: &Program) -> Program {
    let mut result = Program::new();
    result.functions = program.functions.removeTuples();
    result.classes = program.classes.removeTuples();
    result.enums = program.enums.clone();
    let unit = Class {
        name: getUnit(),
        ty: Type::Tuple(Vec::new()).removeTuples(),
        fields: Vec::new(),
        methods: Vec::new(),
        lifetime_info: None,
    };
    let unitFn = Function {
        name: getUnit(),
        params: Vec::new(),
        result: Type::Tuple(Vec::new()).removeTuples(),
        body: None,
        constraintContext: createConstraintContext(&None),
        kind: FunctionKind::ClassCtor,
    };
    result.classes.insert(getUnit(), unit);
    result.functions.insert(getUnit(), unitFn);
    result
}
