use crate::siko::hir::{
    Instruction::{
        Arguments, CallInfo, ClosureCreateInfo, ImplicitHandler, InstructionKind, UnresolvedArgument, WithContext,
        WithInfo,
    },
    Variable::Variable,
};

pub trait ReplaceVar {
    fn replaceVar(&self, from: &Variable, to: Variable) -> Self;
}

impl ReplaceVar for Variable {
    fn replaceVar(&self, from: &Variable, to: Variable) -> Variable {
        if self == from {
            to
        } else {
            self.clone()
        }
    }
}

impl<T: ReplaceVar> ReplaceVar for Vec<T> {
    fn replaceVar(&self, from: &Variable, to: Variable) -> Vec<T> {
        self.iter().map(|item| item.replaceVar(from, to.clone())).collect()
    }
}

impl ReplaceVar for UnresolvedArgument {
    fn replaceVar(&self, from: &Variable, to: Variable) -> UnresolvedArgument {
        match self {
            UnresolvedArgument::Positional(variable) => UnresolvedArgument::Positional(variable.replaceVar(from, to)),
            UnresolvedArgument::Named(name, location, variable) => {
                UnresolvedArgument::Named(name.clone(), location.clone(), variable.replaceVar(from, to))
            }
        }
    }
}

impl ReplaceVar for Arguments {
    fn replaceVar(&self, from: &Variable, to: Variable) -> Arguments {
        match self {
            Arguments::Resolved(vars) => Arguments::Resolved(vars.replaceVar(from, to)),
            Arguments::Unresolved(args) => Arguments::Unresolved(args.replaceVar(from, to)),
        }
    }
}

impl ReplaceVar for CallInfo {
    fn replaceVar(&self, from: &Variable, to: Variable) -> CallInfo {
        let mut info = self.clone();
        info.args = info.args.replaceVar(from, to);
        info
    }
}

impl ReplaceVar for ImplicitHandler {
    fn replaceVar(&self, from: &Variable, to: Variable) -> ImplicitHandler {
        let mut handler = self.clone();
        handler.var = handler.var.replaceVar(from, to);
        handler
    }
}

impl ReplaceVar for WithContext {
    fn replaceVar(&self, from: &Variable, to: Variable) -> WithContext {
        match self {
            WithContext::EffectHandler(name) => WithContext::EffectHandler(name.clone()),
            WithContext::Implicit(handler) => WithContext::Implicit(handler.replaceVar(from, to)),
        }
    }
}

impl ReplaceVar for ClosureCreateInfo {
    fn replaceVar(&self, from: &Variable, to: Variable) -> ClosureCreateInfo {
        let mut info = self.clone();
        info.closureParams = info.closureParams.replaceVar(from, to);
        info
    }
}

impl ReplaceVar for WithInfo {
    fn replaceVar(&self, from: &Variable, to: Variable) -> WithInfo {
        let mut info = self.clone();
        info.contexts = info.contexts.replaceVar(from, to);
        info
    }
}

impl ReplaceVar for InstructionKind {
    fn replaceVar(&self, from: &Variable, to: Variable) -> InstructionKind {
        match self {
            InstructionKind::FunctionCall(var, info) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_info = info.replaceVar(&from, to);
                InstructionKind::FunctionCall(new_var, new_info)
            }
            InstructionKind::Converter(var, source) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_source = source.replaceVar(&from, to);
                InstructionKind::Converter(new_var, new_source)
            }
            InstructionKind::MethodCall(var, obj, name, args) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_obj = obj.replaceVar(&from, to.clone());
                let new_args = args.replaceVar(from, to);
                InstructionKind::MethodCall(new_var, new_obj, name.clone(), new_args)
            }
            InstructionKind::DynamicFunctionCall(var, func, args) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_func = func.replaceVar(&from, to.clone());
                let new_args = args.replaceVar(from, to);
                InstructionKind::DynamicFunctionCall(new_var, new_func, new_args)
            }
            InstructionKind::FieldRef(var, target, name) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_target = target.replaceVar(&from, to);
                InstructionKind::FieldRef(new_var, new_target, name.clone())
            }
            InstructionKind::Bind(var, value, mutable) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_value = value.replaceVar(&from, to);
                InstructionKind::Bind(new_var, new_value, *mutable)
            }
            InstructionKind::Tuple(var, elements) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_elements = elements.replaceVar(from, to);
                InstructionKind::Tuple(new_var, new_elements)
            }
            InstructionKind::StringLiteral(var, value) => {
                let new_var = var.replaceVar(&from, to.clone());
                InstructionKind::StringLiteral(new_var, value.clone())
            }
            InstructionKind::IntegerLiteral(var, value) => {
                let new_var = var.replaceVar(&from, to.clone());
                InstructionKind::IntegerLiteral(new_var, value.clone())
            }
            InstructionKind::CharLiteral(var, value) => {
                let new_var = var.replaceVar(&from, to.clone());
                InstructionKind::CharLiteral(new_var, value.clone())
            }
            InstructionKind::Return(var, value) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_value = value.replaceVar(&from, to);
                InstructionKind::Return(new_var, new_value)
            }
            InstructionKind::Ref(var, target) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_target = target.replaceVar(&from, to);
                InstructionKind::Ref(new_var, new_target)
            }
            InstructionKind::PtrOf(var, target) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_target = target.replaceVar(&from, to);
                InstructionKind::PtrOf(new_var, new_target)
            }
            InstructionKind::DropPath(_) => self.clone(),
            InstructionKind::DropMetadata(_) => self.clone(),
            InstructionKind::Drop(_, _) => self.clone(),
            InstructionKind::Jump(var, id) => {
                let new_var = var.replaceVar(&from, to.clone());
                InstructionKind::Jump(new_var, id.clone())
            }
            InstructionKind::Assign(var, arg) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_arg = arg.replaceVar(&from, to);
                InstructionKind::Assign(new_var, new_arg)
            }
            InstructionKind::FieldAssign(var, arg, fields) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_arg = arg.replaceVar(&from, to);
                InstructionKind::FieldAssign(new_var, new_arg, fields.clone())
            }
            InstructionKind::AddressOfField(var, target, fields) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_target = target.replaceVar(&from, to);
                InstructionKind::AddressOfField(new_var, new_target, fields.clone())
            }
            InstructionKind::DeclareVar(var, mutability) => {
                let new_var = var.replaceVar(&from, to);
                InstructionKind::DeclareVar(new_var, mutability.clone())
            }
            InstructionKind::Transform(var, arg, info) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_arg = arg.replaceVar(&from, to);
                InstructionKind::Transform(new_var, new_arg, info.clone())
            }
            InstructionKind::EnumSwitch(root, cases) => {
                let new_root = root.replaceVar(&from, to);
                InstructionKind::EnumSwitch(new_root, cases.clone())
            }
            InstructionKind::IntegerSwitch(root, cases) => {
                let new_root = root.replaceVar(&from, to);
                InstructionKind::IntegerSwitch(new_root, cases.clone())
            }
            InstructionKind::BlockStart(info) => InstructionKind::BlockStart(info.clone()),
            InstructionKind::BlockEnd(info) => InstructionKind::BlockEnd(info.clone()),
            InstructionKind::With(v, info) => {
                let info = info.replaceVar(&from, to.clone());
                InstructionKind::With(v.replaceVar(&from, to), info)
            }
            InstructionKind::ReadImplicit(var, name) => {
                InstructionKind::ReadImplicit(var.replaceVar(&from, to.clone()), name.clone())
            }
            InstructionKind::WriteImplicit(name, var) => {
                InstructionKind::WriteImplicit(name.clone(), var.replaceVar(&from, to.clone()))
            }
            InstructionKind::LoadPtr(var, target) => {
                InstructionKind::LoadPtr(var.replaceVar(&from, to.clone()), target.replaceVar(&from, to))
            }
            InstructionKind::StorePtr(var, target) => {
                InstructionKind::StorePtr(var.replaceVar(&from, to.clone()), target.replaceVar(&from, to))
            }
            InstructionKind::CreateClosure(var, info) => {
                let info = info.replaceVar(&from, to.clone());
                InstructionKind::CreateClosure(var.replaceVar(&from, to.clone()), info)
            }
            InstructionKind::ClosureReturn(block_id, variable, return_value) => {
                InstructionKind::ClosureReturn(block_id.clone(), variable.clone(), return_value.clone())
            }
            InstructionKind::IntegerOp(var, v1, v2, op) => {
                let new_var = var.replaceVar(&from, to.clone());
                let new_v1 = v1.replaceVar(&from, to.clone());
                let new_v2 = v2.replaceVar(&from, to);
                InstructionKind::IntegerOp(new_var, new_v1, new_v2, op.clone())
            }
            InstructionKind::Yield(v, a) => {
                let new_v = v.replaceVar(&from, to.clone());
                let new_a = a.replaceVar(&from, to);
                InstructionKind::Yield(new_v, new_a)
            }
        }
    }
}
