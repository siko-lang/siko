use crate::siko::hir::{
    Instruction::{CallInfo, ClosureCreateInfo, InstructionKind, WithContext, WithInfo},
    Variable::Variable,
};

pub trait CollectVariables {
    fn collectVariables(&self, vars: &mut Vec<Variable>);
}

impl CollectVariables for Variable {
    fn collectVariables(&self, vars: &mut Vec<Variable>) {
        vars.push(self.clone());
    }
}

impl<T: CollectVariables> CollectVariables for Vec<T> {
    fn collectVariables(&self, vars: &mut Vec<Variable>) {
        for item in self {
            item.collectVariables(vars);
        }
    }
}

impl CollectVariables for CallInfo {
    fn collectVariables(&self, vars: &mut Vec<Variable>) {
        self.args.collectVariables(vars);
    }
}

impl CollectVariables for WithContext {
    fn collectVariables(&self, vars: &mut Vec<Variable>) {
        match self {
            WithContext::EffectHandler(_) => {}
            WithContext::Implicit(handler) => {
                handler.var.collectVariables(vars);
            }
        }
    }
}

impl CollectVariables for WithInfo {
    fn collectVariables(&self, vars: &mut Vec<Variable>) {
        self.contexts.collectVariables(vars);
    }
}

impl CollectVariables for ClosureCreateInfo {
    fn collectVariables(&self, vars: &mut Vec<Variable>) {
        self.closureParams.collectVariables(vars);
    }
}

impl CollectVariables for InstructionKind {
    fn collectVariables(&self, vars: &mut Vec<Variable>) {
        match self {
            InstructionKind::FunctionCall(var, info) => {
                var.collectVariables(vars);
                info.collectVariables(vars);
            }
            InstructionKind::Converter(var, target) => {
                var.collectVariables(vars);
                target.collectVariables(vars);
            }
            InstructionKind::MethodCall(var, obj, _, args) => {
                var.collectVariables(vars);
                obj.collectVariables(vars);
                args.collectVariables(vars);
            }
            InstructionKind::DynamicFunctionCall(var, func, args) => {
                var.collectVariables(vars);
                func.collectVariables(vars);
                args.collectVariables(vars);
            }
            InstructionKind::FieldRef(var, target, _) => {
                var.collectVariables(vars);
                target.collectVariables(vars);
            }
            InstructionKind::Bind(var, value, _) => {
                var.collectVariables(vars);
                value.collectVariables(vars);
            }
            InstructionKind::Tuple(var, elements) => {
                var.collectVariables(vars);
                elements.collectVariables(vars);
            }
            InstructionKind::StringLiteral(var, _) => var.collectVariables(vars),
            InstructionKind::IntegerLiteral(var, _) => var.collectVariables(vars),
            InstructionKind::CharLiteral(var, _) => var.collectVariables(vars),
            InstructionKind::Return(var, value) => {
                var.collectVariables(vars);
                value.collectVariables(vars);
            }
            InstructionKind::Ref(var, target) => {
                var.collectVariables(vars);
                target.collectVariables(vars);
            }
            InstructionKind::PtrOf(var, target) => {
                var.collectVariables(vars);
                target.collectVariables(vars);
            }
            InstructionKind::DropPath(_) => {}
            InstructionKind::DropMetadata(_) => {}
            InstructionKind::Drop(_, _) => {}
            InstructionKind::Jump(var, _) => var.collectVariables(vars),
            InstructionKind::Assign(var, value) => {
                var.collectVariables(vars);
                value.collectVariables(vars);
            }
            InstructionKind::FieldAssign(var, value, _) => {
                var.collectVariables(vars);
                value.collectVariables(vars);
            }
            InstructionKind::AddressOfField(var, target, _) => {
                var.collectVariables(vars);
                target.collectVariables(vars);
            }
            InstructionKind::DeclareVar(var, _) => var.collectVariables(vars),
            InstructionKind::Transform(var, target, _) => {
                var.collectVariables(vars);
                target.collectVariables(vars);
            }
            InstructionKind::EnumSwitch(var, _) => {
                var.collectVariables(vars);
            }
            InstructionKind::IntegerSwitch(var, _) => {
                var.collectVariables(vars);
            }
            InstructionKind::BlockStart(_) => {}
            InstructionKind::BlockEnd(_) => {}
            InstructionKind::With(v, info) => {
                v.collectVariables(vars);
                info.collectVariables(vars);
            }
            InstructionKind::ReadImplicit(var, _) => var.collectVariables(vars),
            InstructionKind::WriteImplicit(_, var) => var.collectVariables(vars),
            InstructionKind::LoadPtr(dest, src) => {
                dest.collectVariables(vars);
                src.collectVariables(vars);
            }
            InstructionKind::StorePtr(dest, src) => {
                dest.collectVariables(vars);
                src.collectVariables(vars);
            }
            InstructionKind::CreateClosure(var, info) => {
                var.collectVariables(vars);
                info.collectVariables(vars);
            }
            InstructionKind::ClosureReturn(_, variable, return_value) => {
                variable.collectVariables(vars);
                return_value.collectVariables(vars);
            }
            InstructionKind::IntegerOp(var, v1, v2, _) => {
                var.collectVariables(vars);
                v1.collectVariables(vars);
                v2.collectVariables(vars);
            }
            InstructionKind::Yield(v, a) => {
                v.collectVariables(vars);
                a.collectVariables(vars);
            }
        }
    }
}
