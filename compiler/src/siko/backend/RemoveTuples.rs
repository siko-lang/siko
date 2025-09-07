use core::panic;
use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    hir::{
        Block::Block,
        Body::Body,
        ConstraintContext::ConstraintContext,
        Data::{Enum, Field, Struct, Variant},
        Function::{Attributes, Function, FunctionKind, Parameter},
        Instruction::{
            CallInfo, ClosureCreateInfo, FieldId, FieldInfo, ImplicitHandler, Instruction, InstructionKind,
            WithContext, WithInfo,
        },
        Program::Program,
        Type::Type,
        Variable::Variable,
    },
    location::Location::Location,
    qualifiedname::QualifiedName,
};

fn fieldNameForIndex(index: usize) -> String {
    format!("f{}", index)
}

fn getTuple(ty: &Type) -> QualifiedName {
    let sikoModuleName = "siko";
    if let Type::Named(name, _) = ty {
        if name.module().toString() == sikoModuleName && name.getShortName().starts_with("Tuple_") {
            return name.clone();
        }
    }
    QualifiedName::Module(sikoModuleName.to_string()).add(format!("Tuple_{}", ty))
}

pub fn getUnitTypeName() -> Type {
    Type::Named(getTuple(&Type::Tuple(Vec::new())), Vec::new())
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
            Type::Tuple(args) => {
                for arg in args {
                    arg.removeTuples(ctx);
                }
                ctx.tuples.insert(self.clone());
                Type::Named(getTuple(self), Vec::new())
            }
            Type::Reference(ty) => ty.removeTuples(ctx).asRef(),
            Type::Ptr(ty) => Type::Ptr(Box::new(ty.removeTuples(ctx))),
            Type::Function(args, r) => {
                let args = args.removeTuples(ctx);
                let r = r.removeTuples(ctx);
                Type::Function(args, Box::new(r))
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
            varAllocator: self.varAllocator.clone(),
        }
    }
}

impl RemoveTuples for Block {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let instructions = self.getInstructions().removeTuples(ctx);
        Block::newWith(self.getId(), instructions)
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
        let result = self.clone();
        result.setType(result.getType().removeTuples(ctx));
        result
    }
}

impl RemoveTuples for FieldId {
    fn removeTuples(&self, _: &mut Context) -> Self {
        match self {
            FieldId::Named(name) => FieldId::Named(name.clone()),
            FieldId::Indexed(index) => FieldId::Named(fieldNameForIndex(*index as usize)),
        }
    }
}

impl RemoveTuples for FieldInfo {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut result = self.clone();
        result.name = result.name.removeTuples(ctx);
        result.ty = result.ty.removeTuples(ctx);
        result
    }
}

impl RemoveTuples for ImplicitHandler {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut result = self.clone();
        result.var = result.var.removeTuples(ctx);
        result
    }
}

impl RemoveTuples for WithContext {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        match self {
            WithContext::EffectHandler(handler) => WithContext::EffectHandler(handler.clone()),
            WithContext::Implicit(handler) => WithContext::Implicit(handler.removeTuples(ctx)),
        }
    }
}

impl RemoveTuples for WithInfo {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut result = self.clone();
        result.contexts = result.contexts.removeTuples(ctx);
        result
    }
}

impl RemoveTuples for CallInfo {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut result = self.clone();
        result.args = result.args.removeTuples(ctx);
        result
    }
}

impl RemoveTuples for InstructionKind {
    fn removeTuples(&self, ctx: &mut Context) -> InstructionKind {
        match self {
            InstructionKind::Tuple(dest, args) => {
                let tupleName = getTuple(&dest.getType());
                InstructionKind::FunctionCall(dest.removeTuples(ctx), CallInfo::new(tupleName, args.removeTuples(ctx)))
            }
            InstructionKind::Converter(dest, source) => {
                InstructionKind::Converter(dest.removeTuples(ctx), source.removeTuples(ctx))
            }
            InstructionKind::Transform(dest, root, info) => {
                InstructionKind::Transform(dest.removeTuples(ctx), root.removeTuples(ctx), info.clone())
            }
            InstructionKind::FunctionCall(dest, info) => {
                InstructionKind::FunctionCall(dest.removeTuples(ctx), info.removeTuples(ctx))
            }
            InstructionKind::MethodCall(_, _, _, _) => {
                unreachable!("method call in remove tuples!")
            }
            InstructionKind::DynamicFunctionCall(dest, root, args) => InstructionKind::DynamicFunctionCall(
                dest.removeTuples(ctx),
                root.removeTuples(ctx),
                args.removeTuples(ctx),
            ),
            InstructionKind::FieldRef(dest, receiver, field) => InstructionKind::FieldRef(
                dest.removeTuples(ctx),
                receiver.removeTuples(ctx),
                field.removeTuples(ctx),
            ),
            InstructionKind::Bind(_, _, _) => {
                panic!("Bind instruction found in RemoveTuples, this should not happen");
            }
            InstructionKind::StringLiteral(dest, lit) => {
                InstructionKind::StringLiteral(dest.removeTuples(ctx), lit.clone())
            }
            InstructionKind::IntegerLiteral(dest, lit) => {
                InstructionKind::IntegerLiteral(dest.removeTuples(ctx), lit.clone())
            }
            InstructionKind::CharLiteral(dest, lit) => {
                InstructionKind::CharLiteral(dest.removeTuples(ctx), lit.clone())
            }
            InstructionKind::Return(dest, arg) => {
                InstructionKind::Return(dest.removeTuples(ctx), arg.removeTuples(ctx))
            }
            InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.removeTuples(ctx), arg.removeTuples(ctx)),
            InstructionKind::PtrOf(dest, arg) => InstructionKind::PtrOf(dest.removeTuples(ctx), arg.removeTuples(ctx)),
            InstructionKind::DropPath(id) => {
                panic!(
                    "DropListPlaceholder found in RemoveTuples, this should not happen: {}",
                    id
                );
            }
            InstructionKind::DropMetadata(id) => {
                panic!("DropMetadata found in RemoveTuples, this should not happen: {}", id);
            }
            InstructionKind::Drop(_, _) => {
                unreachable!("drop in remove tuples!")
            }
            InstructionKind::Jump(dest, id) => InstructionKind::Jump(dest.removeTuples(ctx), *id),
            InstructionKind::Assign(lhs, rhs) => InstructionKind::Assign(lhs.removeTuples(ctx), rhs.removeTuples(ctx)),
            InstructionKind::FieldAssign(lhs, rhs, fields) => {
                InstructionKind::FieldAssign(lhs.clone(), rhs.removeTuples(ctx), fields.removeTuples(ctx))
            }
            InstructionKind::AddressOfField(dest, receiver, fields) => InstructionKind::AddressOfField(
                dest.removeTuples(ctx),
                receiver.removeTuples(ctx),
                fields.removeTuples(ctx),
            ),
            InstructionKind::DeclareVar(var, mutability) => {
                InstructionKind::DeclareVar(var.removeTuples(ctx), mutability.clone())
            }
            InstructionKind::EnumSwitch(root, cases) => {
                InstructionKind::EnumSwitch(root.removeTuples(ctx), cases.clone())
            }
            InstructionKind::IntegerSwitch(root, cases) => {
                InstructionKind::IntegerSwitch(root.removeTuples(ctx), cases.clone())
            }
            InstructionKind::BlockStart(info) => InstructionKind::BlockStart(info.clone()),
            InstructionKind::BlockEnd(info) => InstructionKind::BlockEnd(info.clone()),
            InstructionKind::With(v, info) => InstructionKind::With(v.clone(), info.clone()),
            InstructionKind::ReadImplicit(var, name) => {
                InstructionKind::ReadImplicit(var.removeTuples(ctx), name.clone())
            }
            InstructionKind::WriteImplicit(index, var) => {
                InstructionKind::WriteImplicit(index.clone(), var.removeTuples(ctx))
            }
            InstructionKind::LoadPtr(dest, src) => {
                InstructionKind::LoadPtr(dest.removeTuples(ctx), src.removeTuples(ctx))
            }
            InstructionKind::StorePtr(dest, src) => {
                InstructionKind::StorePtr(dest.removeTuples(ctx), src.removeTuples(ctx))
            }
            InstructionKind::CreateClosure(v, info) => {
                InstructionKind::CreateClosure(v.removeTuples(ctx), info.removeTuples(ctx))
            }
            InstructionKind::ClosureReturn(_, _, _) => {
                panic!("ClosureReturn found in RemoveTuples, this should not happen");
            }
        }
    }
}

impl RemoveTuples for ClosureCreateInfo {
    fn removeTuples(&self, ctx: &mut Context) -> Self {
        let mut info = self.clone();
        info.closureParams = info.closureParams.removeTuples(ctx);
        info
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
                let argType = if arg.isTuple() {
                    Type::Named(getTuple(arg), Vec::new())
                } else {
                    arg.clone()
                };
                let name = fieldNameForIndex(index);
                let field = Field {
                    name: name.clone(),
                    ty: argType.clone(),
                };
                fields.push(field);
                let param = Parameter::Named(name, argType, false);
                params.push(param);
            }
            let tupleStruct = Struct {
                name: name.clone(),
                location: Location::empty(),
                ty: Type::Named(name.clone(), Vec::new()),
                fields: fields,
                methods: Vec::new(),
            };
            let unitFn = Function {
                name: name.clone(),
                params: params,
                result: Type::Named(name.clone(), Vec::new()),
                body: None,
                constraintContext: ConstraintContext::new(),
                kind: FunctionKind::StructCtor,
                attributes: Attributes::new(),
            };
            result.structs.insert(name.clone(), tupleStruct);
            result.functions.insert(name.clone(), unitFn);
        } else {
            unreachable!()
        }
    }
    result
}
