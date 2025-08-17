use crate::siko::hir::Type::Type;

use super::{
    ConstraintContext::{Constraint, ConstraintContext},
    Data::{Enum, Field, Struct, Variant},
    Instruction::{FieldInfo, InstructionKind},
    Substitution::Substitution,
    Trait::{AssociatedType, Instance, MemberInfo, Trait},
    Variable::Variable,
};

pub trait Apply {
    fn apply(self, sub: &Substitution) -> Self;
}

impl Apply for Type {
    fn apply(self, sub: &Substitution) -> Self {
        match self {
            Type::Named(n, args) => {
                let newArgs = args.into_iter().map(|arg| arg.apply(sub)).collect();
                Type::Named(n.clone(), newArgs)
            }
            Type::Tuple(args) => {
                let newArgs = args.into_iter().map(|arg| arg.apply(sub)).collect();
                Type::Tuple(newArgs)
            }
            Type::Function(args, fnResult) => {
                let newArgs = args.into_iter().map(|arg| arg.apply(sub)).collect();
                let newFnResult = fnResult.apply(sub);
                Type::Function(newArgs, Box::new(newFnResult))
            }
            Type::Var(v) => sub.get(Type::Var(v)),
            Type::Reference(arg, l) => Type::Reference(Box::new(arg.apply(sub)), l.clone()),
            Type::Ptr(arg) => Type::Ptr(Box::new(arg.apply(sub))),
            Type::SelfType => Type::SelfType,
            Type::Never(v) => Type::Never(v),
        }
    }
}

impl<T: Apply> Apply for Option<T> {
    fn apply(self, sub: &Substitution) -> Self {
        match self {
            Some(t) => Some(t.apply(sub)),
            None => None,
        }
    }
}

impl<T: Apply> Apply for Vec<T> {
    fn apply(self, sub: &Substitution) -> Self {
        self.into_iter().map(|i| i.apply(sub)).collect()
    }
}

impl Apply for Variant {
    fn apply(mut self, sub: &Substitution) -> Self {
        self.items = self.items.apply(sub);
        self
    }
}

impl Apply for Enum {
    fn apply(mut self, sub: &Substitution) -> Self {
        self.ty = self.ty.apply(sub);
        self.variants = self.variants.apply(sub);
        self
    }
}

impl Apply for Field {
    fn apply(mut self, sub: &Substitution) -> Self {
        self.ty = self.ty.apply(sub);
        self
    }
}

impl Apply for Struct {
    fn apply(mut self, sub: &Substitution) -> Self {
        self.ty = self.ty.apply(sub);
        self.fields = self.fields.apply(sub);
        self
    }
}

impl Apply for Trait {
    fn apply(mut self, sub: &Substitution) -> Self {
        //println!("Applying for {}", self.value);
        self.params = self.params.apply(sub);
        self.constraint = self.constraint.apply(sub);
        self.members = self.members.apply(sub);
        self
    }
}

impl Apply for AssociatedType {
    fn apply(mut self, sub: &Substitution) -> Self {
        //println!("Applying for {}", self.value);
        self.ty = self.ty.apply(sub);
        self
    }
}

impl Apply for MemberInfo {
    fn apply(mut self, sub: &Substitution) -> Self {
        //println!("Applying for {}", self.value);
        self.result = self.result.apply(sub);
        self
    }
}

impl Apply for Instance {
    fn apply(mut self, sub: &Substitution) -> Self {
        //println!("Applying for {}", self.value);
        self.types = self.types.apply(sub);
        self.associatedTypes = self.associatedTypes.apply(sub);
        self.members = self.members.apply(sub);
        self
    }
}

impl Apply for Variable {
    fn apply(mut self, sub: &Substitution) -> Self {
        //println!("Applying for {}", self.value);
        self.ty = self.ty.apply(sub);
        self
    }
}

impl Apply for Constraint {
    fn apply(mut self, sub: &Substitution) -> Self {
        //println!("Applying for {}", self.value);
        self.args = self.args.apply(sub);
        self.associatedTypes = self.associatedTypes.apply(sub);
        self
    }
}

impl Apply for ConstraintContext {
    fn apply(mut self, sub: &Substitution) -> Self {
        //println!("Applying for {}", self.value);
        self.typeParameters = self.typeParameters.apply(sub);
        self.constraints = self.constraints.apply(sub);
        self
    }
}

impl Apply for FieldInfo {
    fn apply(mut self, sub: &Substitution) -> Self {
        //println!("Applying for {}", self.value);
        self.ty = self.ty.apply(sub);
        self
    }
}

impl Apply for InstructionKind {
    fn apply(self, sub: &Substitution) -> Self {
        match self {
            InstructionKind::FunctionCall(dest, name, args) => {
                InstructionKind::FunctionCall(dest.apply(sub), name.clone(), args.apply(sub))
            }
            InstructionKind::Converter(dest, source) => InstructionKind::Converter(dest.apply(sub), source.apply(sub)),
            InstructionKind::MethodCall(dest, receiver, name, args) => {
                InstructionKind::MethodCall(dest.apply(sub), receiver.apply(sub), name.clone(), args.apply(sub))
            }
            InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                InstructionKind::DynamicFunctionCall(dest.apply(sub), callable.apply(sub), args.apply(sub))
            }
            InstructionKind::FieldRef(dest, root, field) => {
                InstructionKind::FieldRef(dest.apply(sub), root.apply(sub), field.clone())
            }
            InstructionKind::Bind(dest, rhs, mutable) => {
                InstructionKind::Bind(dest.apply(sub), rhs.apply(sub), mutable)
            }
            InstructionKind::Tuple(dest, args) => InstructionKind::Tuple(dest.apply(sub), args.apply(sub)),
            InstructionKind::StringLiteral(dest, s) => InstructionKind::StringLiteral(dest.apply(sub), s.clone()),
            InstructionKind::IntegerLiteral(dest, n) => InstructionKind::IntegerLiteral(dest.apply(sub), n.clone()),
            InstructionKind::CharLiteral(dest, c) => InstructionKind::CharLiteral(dest.apply(sub), c),
            InstructionKind::Return(dest, arg) => InstructionKind::Return(dest.apply(sub), arg.apply(sub)),
            InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.apply(sub), arg.apply(sub)),
            InstructionKind::DropPath(id) => InstructionKind::DropPath(id.clone()),
            InstructionKind::DropMetadata(id) => InstructionKind::DropMetadata(id.clone()),
            InstructionKind::Drop(dest, drop) => InstructionKind::Drop(dest.apply(sub), drop.apply(sub)),
            InstructionKind::Jump(dest, targetBlockId) => InstructionKind::Jump(dest.apply(sub), targetBlockId),
            InstructionKind::Assign(name, rhs) => InstructionKind::Assign(name.apply(sub), rhs.apply(sub)),
            InstructionKind::FieldAssign(name, rhs, fields) => {
                InstructionKind::FieldAssign(name.apply(sub), rhs.apply(sub), fields.apply(sub))
            }
            InstructionKind::AddressOfField(var, receiver, fields) => {
                InstructionKind::AddressOfField(var.apply(sub), receiver.apply(sub), fields.apply(sub))
            }
            InstructionKind::DeclareVar(var, mutability) => {
                InstructionKind::DeclareVar(var.apply(sub), mutability.clone())
            }
            InstructionKind::Transform(dest, root, index) => {
                InstructionKind::Transform(dest.apply(sub), root.apply(sub), index.clone())
            }
            InstructionKind::EnumSwitch(root, cases) => InstructionKind::EnumSwitch(root.apply(sub), cases.clone()),
            InstructionKind::IntegerSwitch(root, cases) => {
                InstructionKind::IntegerSwitch(root.apply(sub), cases.clone())
            }
            InstructionKind::BlockStart(info) => InstructionKind::BlockStart(info.clone()),
            InstructionKind::BlockEnd(info) => InstructionKind::BlockEnd(info.clone()),
            InstructionKind::With(v, handlers, blockId, syntaxBlockId) => {
                InstructionKind::With(v.apply(sub), handlers.clone(), blockId.clone(), syntaxBlockId.clone())
            }
            InstructionKind::GetImplicit(var, name) => InstructionKind::GetImplicit(var.apply(sub), name.clone()),
        }
    }
}
