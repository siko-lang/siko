use std::collections::BTreeSet;

use crate::siko::hir::Type::Type;

use super::{
    Data::{Class, Enum, Field, Variant},
    Function::{InstructionKind, ValueKind, Variable},
    Substitution::{TypeSubstitution, VariableSubstitution},
    TypeVarAllocator::TypeVarAllocator,
    Unification::unify,
};

pub trait Apply {
    fn apply(&self, sub: &TypeSubstitution) -> Self;
}

pub trait ApplyVariable {
    fn applyVar(&self, sub: &VariableSubstitution) -> Self;
}

impl Apply for Type {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        match &self {
            Type::Named(n, args, lifetime) => {
                let newArgs = args.iter().map(|arg| arg.apply(sub)).collect();
                Type::Named(n.clone(), newArgs, lifetime.clone())
            }
            Type::Tuple(args) => {
                let newArgs = args.iter().map(|arg| arg.apply(sub)).collect();
                Type::Tuple(newArgs)
            }
            Type::Function(args, fnResult) => {
                let newArgs = args.iter().map(|arg| arg.apply(sub)).collect();
                let newFnResult = fnResult.apply(sub);
                Type::Function(newArgs, Box::new(newFnResult))
            }
            Type::Var(_) => sub.get(self.clone()),
            Type::Reference(arg, l) => Type::Reference(Box::new(arg.apply(sub)), l.clone()),
            Type::SelfType => self.clone(),
            Type::Never => self.clone(),
        }
    }
}

impl<T: Apply> Apply for Option<T> {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        match self {
            Some(t) => Some(t.apply(sub)),
            None => None,
        }
    }
}

impl<T: Apply> Apply for Vec<T> {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        self.iter().map(|i| i.apply(sub)).collect()
    }
}

impl<T: ApplyVariable> ApplyVariable for Vec<T> {
    fn applyVar(&self, sub: &VariableSubstitution) -> Self {
        self.iter().map(|i| i.applyVar(sub)).collect()
    }
}

impl Apply for Variant {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        let mut v = self.clone();
        v.items = v.items.apply(sub);
        v
    }
}

impl Apply for Enum {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        let mut e = self.clone();
        e.ty = e.ty.apply(sub);
        e.variants = e.variants.apply(sub);
        e
    }
}

impl Apply for Field {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        let mut f = self.clone();
        f.ty = f.ty.apply(sub);
        f
    }
}

impl Apply for Class {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        let mut c = self.clone();
        c.ty = c.ty.apply(sub);
        c.fields = c.fields.apply(sub);
        c
    }
}

impl Apply for Variable {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        //println!("Applying for {}", self.value);
        let mut v = self.clone();
        if sub.forced && v.ty.is_none() {
            panic!("var without type in mono! {}", v);
        }
        v.ty = v.ty.apply(sub);
        v
    }
}

impl Apply for ValueKind {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        match self {
            ValueKind::Arg(n, i) => ValueKind::Arg(n.clone(), *i),
            ValueKind::Value(v) => ValueKind::Value(v.apply(sub)),
        }
    }
}

impl Apply for InstructionKind {
    fn apply(&self, sub: &TypeSubstitution) -> Self {
        match self {
            InstructionKind::FunctionCall(dest, name, args) => InstructionKind::FunctionCall(dest.apply(sub), name.clone(), args.apply(sub)),
            InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                InstructionKind::DynamicFunctionCall(dest.apply(sub), callable.apply(sub), args.apply(sub))
            }
            InstructionKind::ValueRef(dest, value) => InstructionKind::ValueRef(dest.apply(sub), value.clone()),
            InstructionKind::FieldRef(dest, root, field) => InstructionKind::FieldRef(dest.apply(sub), root.apply(sub), field.clone()),
            InstructionKind::TupleIndex(dest, root, index) => InstructionKind::TupleIndex(dest.apply(sub), root.apply(sub), *index),
            InstructionKind::Bind(dest, rhs, mutable) => InstructionKind::Bind(dest.apply(sub), rhs.apply(sub), *mutable),
            InstructionKind::Tuple(dest, args) => InstructionKind::Tuple(dest.apply(sub), args.apply(sub)),
            InstructionKind::StringLiteral(dest, s) => InstructionKind::StringLiteral(dest.apply(sub), s.clone()),
            InstructionKind::IntegerLiteral(dest, n) => InstructionKind::IntegerLiteral(dest.apply(sub), n.clone()),
            InstructionKind::CharLiteral(dest, c) => InstructionKind::CharLiteral(dest.apply(sub), *c),
            InstructionKind::Return(dest, arg) => InstructionKind::Return(dest.apply(sub), arg.apply(sub)),
            InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.apply(sub), arg.apply(sub)),
            InstructionKind::Drop(args) => InstructionKind::Drop(args.clone()),
            InstructionKind::Jump(dest, block_id) => InstructionKind::Jump(dest.apply(sub), *block_id),
            InstructionKind::Assign(name, rhs) => InstructionKind::Assign(name.apply(sub), rhs.apply(sub)),
            InstructionKind::DeclareVar(var) => InstructionKind::DeclareVar(var.apply(sub)),
            InstructionKind::Transform(dest, root, index) => InstructionKind::Transform(dest.apply(sub), root.apply(sub), index.clone()),
            InstructionKind::EnumSwitch(root, cases) => InstructionKind::EnumSwitch(root.apply(sub), cases.clone()),
            InstructionKind::IntegerSwitch(root, cases) => InstructionKind::IntegerSwitch(root.apply(sub), cases.clone()),
            InstructionKind::StringSwitch(root, cases) => InstructionKind::StringSwitch(root.apply(sub), cases.clone()),
        }
    }
}

pub fn instantiateEnum(allocator: &mut TypeVarAllocator, e: &Enum, ty: &Type) -> Enum {
    let vars = e.ty.collectVars(BTreeSet::new());
    let mut sub = TypeSubstitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    let mut e = e.clone();
    e = e.apply(&sub);
    let r = unify(&mut sub, ty, &e.ty);
    assert!(r.is_ok());
    e.apply(&sub)
}

pub fn instantiateEnum2(allocator: &mut TypeVarAllocator, e: &Enum) -> Enum {
    let vars = e.ty.collectVars(BTreeSet::new());
    let mut sub = TypeSubstitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    e.apply(&sub)
}

pub fn instantiateClass(allocator: &mut TypeVarAllocator, c: &Class, ty: &Type) -> Class {
    let vars = c.ty.collectVars(BTreeSet::new());
    let mut sub = TypeSubstitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    let mut e = c.clone();
    e = e.apply(&sub);
    let r = unify(&mut sub, ty, &e.ty);
    assert!(r.is_ok());
    e.apply(&sub)
}

pub fn instantiateType(allocator: &mut TypeVarAllocator, ty: &Type) -> Type {
    let vars = ty.collectVars(BTreeSet::new());
    let mut sub = TypeSubstitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    ty.apply(&sub)
}

impl ApplyVariable for ValueKind {
    fn applyVar(&self, sub: &VariableSubstitution) -> Self {
        match self {
            ValueKind::Arg(n, i) => ValueKind::Arg(n.clone(), *i),
            ValueKind::Value(v) => ValueKind::Value(v.applyVar(sub)),
        }
    }
}

impl ApplyVariable for InstructionKind {
    fn applyVar(&self, sub: &VariableSubstitution) -> Self {
        match self {
            InstructionKind::FunctionCall(dest, name, args) => InstructionKind::FunctionCall(dest.applyVar(sub), name.clone(), args.applyVar(sub)),
            InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                InstructionKind::DynamicFunctionCall(dest.applyVar(sub), callable.applyVar(sub), args.applyVar(sub))
            }
            InstructionKind::ValueRef(dest, value) => InstructionKind::ValueRef(dest.applyVar(sub), value.clone()),
            InstructionKind::FieldRef(dest, root, field) => InstructionKind::FieldRef(dest.applyVar(sub), root.applyVar(sub), field.clone()),
            InstructionKind::TupleIndex(dest, root, index) => InstructionKind::TupleIndex(dest.applyVar(sub), root.applyVar(sub), *index),
            InstructionKind::Bind(dest, rhs, mutable) => InstructionKind::Bind(dest.applyVar(sub), rhs.applyVar(sub), *mutable),
            InstructionKind::Tuple(dest, args) => InstructionKind::Tuple(dest.applyVar(sub), args.applyVar(sub)),
            InstructionKind::StringLiteral(dest, s) => InstructionKind::StringLiteral(dest.applyVar(sub), s.clone()),
            InstructionKind::IntegerLiteral(dest, n) => InstructionKind::IntegerLiteral(dest.applyVar(sub), n.clone()),
            InstructionKind::CharLiteral(dest, c) => InstructionKind::CharLiteral(dest.applyVar(sub), *c),
            InstructionKind::Return(dest, arg) => InstructionKind::Return(dest.applyVar(sub), arg.applyVar(sub)),
            InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.applyVar(sub), arg.applyVar(sub)),
            InstructionKind::Drop(args) => InstructionKind::Drop(args.clone()),
            InstructionKind::Jump(dest, block_id) => InstructionKind::Jump(dest.applyVar(sub), *block_id),
            InstructionKind::Assign(name, rhs) => InstructionKind::Assign(name.applyVar(sub), rhs.applyVar(sub)),
            InstructionKind::DeclareVar(var) => InstructionKind::DeclareVar(var.applyVar(sub)),
            InstructionKind::Transform(dest, root, index) => InstructionKind::Transform(dest.applyVar(sub), root.applyVar(sub), index.clone()),
            InstructionKind::EnumSwitch(root, cases) => InstructionKind::EnumSwitch(root.applyVar(sub), cases.clone()),
            InstructionKind::IntegerSwitch(root, cases) => InstructionKind::IntegerSwitch(root.applyVar(sub), cases.clone()),
            InstructionKind::StringSwitch(root, cases) => InstructionKind::StringSwitch(root.applyVar(sub), cases.clone()),
        }
    }
}

impl ApplyVariable for Variable {
    fn applyVar(&self, sub: &VariableSubstitution) -> Self {
        if self.fixed && !sub.forced {
            self.clone()
        } else {
            sub.get(self.clone())
        }
    }
}
