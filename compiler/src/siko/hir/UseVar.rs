use crate::siko::hir::{
    Instruction::{Arguments, InstructionKind, UnresolvedArgument},
    Variable::Variable,
};

pub trait UseVar {
    fn useVars(&self) -> Self;
}

impl UseVar for Variable {
    fn useVars(&self) -> Self {
        self.useVar()
    }
}

impl<T: UseVar> UseVar for Vec<T> {
    fn useVars(&self) -> Self {
        self.iter().map(|x| x.useVars()).collect()
    }
}

impl UseVar for UnresolvedArgument {
    fn useVars(&self) -> UnresolvedArgument {
        match self {
            UnresolvedArgument::Positional(variable) => UnresolvedArgument::Positional(variable.useVar()),
            UnresolvedArgument::Named(name, variable) => UnresolvedArgument::Named(name.clone(), variable.useVar()),
        }
    }
}

impl UseVar for Arguments {
    fn useVars(&self) -> Arguments {
        match self {
            Arguments::Resolved(vars) => Arguments::Resolved(vars.useVars()),
            Arguments::Unresolved(args) => Arguments::Unresolved(args.useVars()),
        }
    }
}

impl UseVar for InstructionKind {
    fn useVars(&self) -> InstructionKind {
        match self {
            InstructionKind::FunctionCall(dest, info) => {
                let mut info = info.clone();
                info.args = info.args.useVars();
                InstructionKind::FunctionCall(dest.clone(), info)
            }
            InstructionKind::Converter(v1, v2) => InstructionKind::Converter(v1.clone(), v2.useVars()),
            InstructionKind::MethodCall(dest, receiver, name, args) => {
                InstructionKind::MethodCall(dest.clone(), receiver.useVar(), name.clone(), args.useVars())
            }
            InstructionKind::DynamicFunctionCall(dest, closure, args) => {
                InstructionKind::DynamicFunctionCall(dest.clone(), closure.useVar(), args.useVars())
            }
            InstructionKind::FieldRef(dest, receiver, infos) => {
                InstructionKind::FieldRef(dest.clone(), receiver.useVar(), infos.clone())
            }
            InstructionKind::Bind(dest, src, mutability) => {
                InstructionKind::Bind(dest.clone(), src.useVar(), mutability.clone())
            }
            InstructionKind::Tuple(dest, args) => InstructionKind::Tuple(dest.clone(), args.useVars()),
            InstructionKind::StringLiteral(v, lit) => InstructionKind::StringLiteral(v.clone(), lit.clone()),
            InstructionKind::IntegerLiteral(v, lit) => InstructionKind::IntegerLiteral(v.clone(), lit.clone()),
            InstructionKind::CharLiteral(v, lit) => InstructionKind::CharLiteral(v.clone(), lit.clone()),
            InstructionKind::Return(v, arg) => InstructionKind::Return(v.clone(), arg.useVar()),
            InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.clone(), arg.useVar()),
            InstructionKind::PtrOf(dest, arg) => InstructionKind::PtrOf(dest.clone(), arg.useVar()),
            InstructionKind::DropPath(p) => InstructionKind::DropPath(p.clone()),
            InstructionKind::DropMetadata(kind) => InstructionKind::DropMetadata(kind.clone()),
            InstructionKind::Drop(dest, arg) => InstructionKind::Drop(dest.clone(), arg.useVar()),
            InstructionKind::Jump(v, blockId) => InstructionKind::Jump(v.clone(), blockId.clone()),
            InstructionKind::Assign(dest, src) => InstructionKind::Assign(dest.clone(), src.useVar()),
            InstructionKind::FieldAssign(dest, rhs, infos) => {
                InstructionKind::FieldAssign(dest.clone(), rhs.useVar(), infos.clone())
            }
            InstructionKind::AddressOfField(dest, rhs, infos) => {
                InstructionKind::AddressOfField(dest.clone(), rhs.useVar(), infos.clone())
            }
            InstructionKind::DeclareVar(v, mutability) => InstructionKind::DeclareVar(v.clone(), mutability.clone()),
            InstructionKind::Transform(dest, arg, index) => {
                InstructionKind::Transform(dest.clone(), arg.useVar(), index.clone())
            }
            InstructionKind::EnumSwitch(arg, cases) => InstructionKind::EnumSwitch(arg.useVar(), cases.clone()),
            InstructionKind::IntegerSwitch(arg, cases) => InstructionKind::IntegerSwitch(arg.useVar(), cases.clone()),
            InstructionKind::BlockStart(id) => InstructionKind::BlockStart(id.clone()),
            InstructionKind::BlockEnd(id) => InstructionKind::BlockEnd(id.clone()),
            InstructionKind::With(v, info) => InstructionKind::With(v.clone(), info.clone()),
            InstructionKind::ReadImplicit(v, index) => InstructionKind::ReadImplicit(v.clone(), index.clone()),
            InstructionKind::WriteImplicit(index, v) => InstructionKind::WriteImplicit(index.clone(), v.useVar()),
            InstructionKind::LoadPtr(dest, src) => InstructionKind::LoadPtr(dest.clone(), src.useVar()),
            InstructionKind::StorePtr(dest, src) => InstructionKind::StorePtr(dest.clone(), src.useVar()),
            InstructionKind::CreateClosure(v, info) => {
                let mut info = info.clone();
                info.closureParams = info.closureParams.iter().map(|p| p.useVar()).collect();
                InstructionKind::CreateClosure(v.clone(), info)
            }
            InstructionKind::ClosureReturn(block_id, variable, return_value) => {
                InstructionKind::ClosureReturn(block_id.clone(), variable.clone(), return_value.useVar())
            }
            InstructionKind::IntegerOp(dest, v1, v2, op) => {
                InstructionKind::IntegerOp(dest.clone(), v1.useVar(), v2.useVar(), op.clone())
            }
            InstructionKind::Yield(v, a) => InstructionKind::Yield(v.clone(), a.useVar()),
        }
    }
}
