use core::panic;
use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    hir::{
        ConstraintContext::ConstraintContext,
        Data::{Enum, Field, Struct, Variant},
        Function::{Block, Body, Function, FunctionKind, Parameter},
        Instruction::{FieldInfo, Instruction, InstructionKind},
        Program::Program,
        Type::Type,
        Variable::Variable,
        VariableAllocator::VariableAllocator,
    },
    qualifiedname::QualifiedName,
};

fn fieldNameForIndex(index: usize) -> String {
    format!("f{}", index)
}

fn getTuple(ty: &Type) -> QualifiedName {
    QualifiedName::Module("siko".to_string())
        .add(format!("Tuple_{}", ty))
        .monomorphized("".to_string())
}

struct Context {
    tuples: BTreeSet<Type>,
}

impl Context {
    fn new() -> Context {
        Context {
            tuples: BTreeSet::new(),
        }
    }
}

trait RemoveTuples {
    fn removeTuples(&self, ctx: &mut Context) -> Self;
}

impl RemoveTuples for Type {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        match self {
            Type::Tuple(_) => {
                ctx.tuples.insert(self.clone());
                Type::Named(getTuple(self), Vec::new())
            }
            ty => ty.clone(),
        }
    }
}

impl RemoveTuples for Parameter {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        match self {
            Parameter::Named(n, ty, mutable) => Parameter::Named(n.clone(), ty.removeTuples(ctx), *mutable),
            Parameter::SelfParam(mutable, ty) => Parameter::SelfParam(*mutable, ty.removeTuples(ctx)),
        }
    }
}

impl<T: RemoveTuples> RemoveTuples for Option<T> {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        match self {
            Some(t) => Some(t.removeTuples(ctx)),
            None => None,
        }
    }
}

impl<T: RemoveTuples> RemoveTuples for Vec<T> {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        self.iter().map(|item| item.removeTuples(ctx)).collect()
    }
}

impl<K: Clone + Ord, V: RemoveTuples> RemoveTuples for BTreeMap<K, V> {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        self.iter().map(|(k, v)| (k.clone(), v.removeTuples(ctx))).collect()
    }
}

impl RemoveTuples for Body {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let blocks = self.blocks.removeTuples(ctx);
        Body {
            blocks: blocks,
            varTypes: BTreeMap::new(),
            varAllocator: self.varAllocator.clone(),
        }
    }
}

impl RemoveTuples for Block {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let instructions = self.instructions.removeTuples(ctx);
        Block {
            id: self.id,
            instructions: instructions,
        }
    }
}

impl RemoveTuples for Instruction {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut result = self.clone();
        result.kind = result.kind.removeTuples(ctx);
        result
    }
}

impl RemoveTuples for Variable {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut result = self.clone();
        result.ty = result.ty.removeTuples(ctx);
        result
    }
}

impl RemoveTuples for FieldInfo {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut result = self.clone();
        result.ty = result.ty.removeTuples(ctx);
        result
    }
}

impl RemoveTuples for InstructionKind {
    fn removeTuples(&self, ctx: &mut Context) -> InstructionKind {
        match self {
            InstructionKind::Tuple(dest, args) => InstructionKind::FunctionCall(
                dest.removeTuples(ctx),
                getTuple(&dest.getType()),
                args.removeTuples(ctx),
            ),
            InstructionKind::Converter(dest, source) => {
                InstructionKind::Converter(dest.removeTuples(ctx), source.removeTuples(ctx))
            }
            InstructionKind::Transform(dest, root, index) => {
                InstructionKind::Transform(dest.removeTuples(ctx), root.removeTuples(ctx), *index)
            }
            InstructionKind::FunctionCall(dest, name, args) => {
                InstructionKind::FunctionCall(dest.removeTuples(ctx), name.clone(), args.removeTuples(ctx))
            }
            InstructionKind::MethodCall(_, _, _, _) => {
                unreachable!("method call in remove tuples!")
            }
            InstructionKind::DynamicFunctionCall(dest, root, args) => InstructionKind::DynamicFunctionCall(
                dest.removeTuples(ctx),
                root.removeTuples(ctx),
                args.removeTuples(ctx),
            ),
            InstructionKind::FieldRef(dest, receiver, field) => {
                InstructionKind::FieldRef(dest.removeTuples(ctx), receiver.removeTuples(ctx), field.clone())
            }
            InstructionKind::TupleIndex(dest, root, index) => {
                let info = FieldInfo {
                    name: fieldNameForIndex(*index as usize),
                    ty: Some(root.getType().removeTuples(ctx)),
                    location: dest.location.clone(),
                };
                InstructionKind::FieldRef(dest.removeTuples(ctx), root.removeTuples(ctx), vec![info])
            }
            InstructionKind::Bind(_, _, _) => {
                panic!("Bind instruction found in RemoveTuples, this should not happen");
            }
            InstructionKind::StringLiteral(dest, lit) => {
                InstructionKind::StringLiteral(dest.removeTuples(ctx), lit.clone())
            }
            InstructionKind::IntegerLiteral(dest, lit) => {
                InstructionKind::IntegerLiteral(dest.removeTuples(ctx), lit.clone())
            }
            InstructionKind::CharLiteral(dest, lit) => InstructionKind::CharLiteral(dest.removeTuples(ctx), *lit),
            InstructionKind::Return(dest, arg) => {
                InstructionKind::Return(dest.removeTuples(ctx), arg.removeTuples(ctx))
            }
            InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.removeTuples(ctx), arg.removeTuples(ctx)),
            InstructionKind::Drop(_, _) => unreachable!("drop in remove tuples!"),
            InstructionKind::Jump(dest, id, direction) => {
                InstructionKind::Jump(dest.removeTuples(ctx), *id, direction.clone())
            }
            InstructionKind::Assign(lhs, rhs) => InstructionKind::Assign(lhs.removeTuples(ctx), rhs.removeTuples(ctx)),
            InstructionKind::FieldAssign(lhs, rhs, fields) => {
                InstructionKind::FieldAssign(lhs.clone(), rhs.removeTuples(ctx), fields.removeTuples(ctx))
            }
            InstructionKind::DeclareVar(var, mutability) => {
                InstructionKind::DeclareVar(var.removeTuples(ctx), mutability.clone())
            }
            InstructionKind::EnumSwitch(root, cases) => {
                InstructionKind::EnumSwitch(root.removeTuples(ctx), cases.clone())
            }
            InstructionKind::IntegerSwitch(root, cases) => {
                InstructionKind::IntegerSwitch(root.removeTuples(ctx), cases.clone())
            }
            InstructionKind::StringSwitch(root, cases) => {
                InstructionKind::StringSwitch(root.removeTuples(ctx), cases.clone())
            }
            InstructionKind::BlockStart(info) => InstructionKind::BlockStart(info.clone()),
            InstructionKind::BlockEnd(info) => InstructionKind::BlockEnd(info.clone()),
        }
    }
}

impl RemoveTuples for Function {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut f = self.clone();
        f.params = f.params.removeTuples(ctx);
        f.result = f.result.removeTuples(ctx);
        f.body = f.body.removeTuples(ctx);
        f
    }
}

impl RemoveTuples for Field {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut f = self.clone();
        f.ty = f.ty.removeTuples(ctx);
        f
    }
}

impl RemoveTuples for Struct {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut c = self.clone();
        c.ty = c.ty.removeTuples(ctx);
        c.fields = c.fields.removeTuples(ctx);
        c
    }
}

impl RemoveTuples for Variant {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut v = self.clone();
        let item = Type::Tuple(v.items.clone()).removeTuples(ctx);
        v.items = vec![item];
        v
    }
}

impl RemoveTuples for Enum {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut e = self.clone();
        e.ty = e.ty.removeTuples(ctx);
        e.variants = e.variants.removeTuples(ctx);
        e
    }
}

pub fn removeTuples(program: &Program) -> Program {
    let mut result = Program::new();
    let mut ctx = Context::new();
    result.functions = program.functions.removeTuples(&mut ctx);
    result.structs = program.structs.removeTuples(&mut ctx);
    result.enums = program.enums.removeTuples(&mut ctx);
    for tuple in ctx.tuples {
        let name = getTuple(&tuple);
        if let Type::Tuple(args) = tuple {
            let mut fields = Vec::new();
            let mut params = Vec::new();
            for (index, arg) in args.iter().enumerate() {
                let name = fieldNameForIndex(index);
                let field = Field {
                    name: name.clone(),
                    ty: arg.clone(),
                };
                fields.push(field);
                let param = Parameter::Named(name, arg.clone(), false);
                params.push(param);
            }
            let tupleStruct = Struct {
                name: name.clone(),
                ty: Type::Named(name.clone(), Vec::new()),
                fields: fields,
                methods: Vec::new(),
                ownership_info: None,
            };
            let unitFn = Function {
                name: name.clone(),
                params: params,
                result: Type::Named(name.clone(), Vec::new()),
                body: None,
                constraintContext: ConstraintContext::new(),
                kind: FunctionKind::StructCtor,
            };
            result.structs.insert(name.clone(), tupleStruct);
            result.functions.insert(name.clone(), unitFn);
        } else {
            unreachable!()
        }
    }
    result
}
