use std::fmt::Display;

use crate::siko::{
    backend::drop::{Path::Path, ReferenceStore::ReferenceStore, Util::buildFieldPath},
    hir::{Instruction::InstructionKind, Variable::Variable},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsageKind {
    Move,
    Ref,
}

impl Display for UsageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsageKind::Move => write!(f, "move"),
            UsageKind::Ref => write!(f, "ref"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Usage {
    pub path: Path,
    pub kind: UsageKind,
}

impl Usage {
    pub fn isMove(&self) -> bool {
        self.kind == UsageKind::Move
    }
}

impl Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.kind, self.path)
    }
}

pub struct UsageInfo {
    pub usages: Vec<Usage>,
    pub assign: Option<Path>,
}

impl UsageInfo {
    pub fn empty() -> Self {
        UsageInfo {
            usages: Vec::new(),
            assign: None,
        }
    }

    pub fn with(usages: Vec<Usage>, assign: Option<Path>) -> Self {
        UsageInfo { usages, assign }
    }
}

impl Display for UsageInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let usages: Vec<String> = self.usages.iter().map(|u| format!("{}", u)).collect();
        let assign = if let Some(assign) = &self.assign {
            format!("assign: {}", assign)
        } else {
            "no assign".to_string()
        };
        write!(f, "usages: [{}], {}", usages.join(", "), assign)
    }
}

fn varToUsage(var: &Variable) -> Usage {
    let ty = var.getType();
    //println!("Using variable: {} {}", var.name().visibleName(), ty);
    assert!(var.isUsage());
    if ty.isReference() || ty.isPtr() {
        Usage {
            path: var.toPath(),
            kind: UsageKind::Ref,
        }
    } else {
        Usage {
            path: var.toPath(),
            kind: UsageKind::Move,
        }
    }
}

pub fn getUsageInfo(kind: InstructionKind, referenceStore: &ReferenceStore) -> UsageInfo {
    match kind {
        InstructionKind::DeclareVar(_, _) => UsageInfo::empty(),
        InstructionKind::BlockStart(_) => UsageInfo::empty(),
        InstructionKind::BlockEnd(_) => UsageInfo::empty(),
        InstructionKind::FunctionCall(dest, info) => UsageInfo::with(
            info.args.getVariables().iter().map(|arg| varToUsage(arg)).collect(),
            Some(dest.toPath()),
        ),
        InstructionKind::Assign(dest, src) => UsageInfo::with(vec![varToUsage(&src)], Some(dest.toPath())),
        InstructionKind::Return(_, arg) => UsageInfo::with(vec![varToUsage(&arg)], None),
        InstructionKind::FieldRef(dest, receiver, names) => {
            let destTy = dest.getType();
            let path = buildFieldPath(&receiver, &names);
            let kind = if destTy.isReference() || destTy.isPtr() || referenceStore.isReference(&dest.name()) {
                UsageKind::Ref
            } else {
                UsageKind::Move
            };
            UsageInfo::with(vec![Usage { path, kind }], Some(dest.toPath()))
        }
        InstructionKind::FieldAssign(dest, receiver, fields) => {
            let receiverTy = receiver.getType();
            if receiverTy.isReference() || receiverTy.isPtr() {
                UsageInfo::with(
                    vec![Usage {
                        path: receiver.toPath(),
                        kind: UsageKind::Ref,
                    }],
                    Some(buildFieldPath(&dest, &fields)),
                )
            } else {
                UsageInfo::with(
                    vec![Usage {
                        path: receiver.toPath(),
                        kind: UsageKind::Move,
                    }],
                    Some(buildFieldPath(&dest, &fields)),
                )
            }
        }
        InstructionKind::AddressOfField(_, _, _) => UsageInfo::empty(), // ptr shenanigans are invisible for the dropchecker
        InstructionKind::Tuple(dest, args) => {
            UsageInfo::with(args.iter().map(|arg| varToUsage(arg)).collect(), Some(dest.toPath()))
        }
        InstructionKind::Converter(_, _) => {
            panic!("Converter instruction found in block processor");
        }
        InstructionKind::MethodCall(_, _, _, _) => {
            panic!("Method call instruction found in block processor");
        }
        InstructionKind::DynamicFunctionCall(dest, closure, args) => {
            let mut usages = vec![varToUsage(&closure)];
            usages.extend(args.iter().map(|arg| varToUsage(arg)));
            UsageInfo::with(usages, Some(dest.toPath()))
        }
        InstructionKind::Bind(_, _, _) => {
            panic!("Bind instruction found in block processor");
        }
        InstructionKind::StringLiteral(_, _) => UsageInfo::empty(),
        InstructionKind::IntegerLiteral(_, _) => UsageInfo::empty(),
        InstructionKind::CharLiteral(_, _) => UsageInfo::empty(),
        InstructionKind::Ref(dest, var) => UsageInfo::with(
            vec![Usage {
                path: var.toPath(),
                kind: UsageKind::Ref,
            }],
            Some(dest.toPath()),
        ),
        InstructionKind::PtrOf(_, _) => UsageInfo::empty(),
        InstructionKind::DropPath(_) => UsageInfo::empty(),
        InstructionKind::DropMetadata(_) => UsageInfo::empty(),
        InstructionKind::Drop(_, _) => {
            panic!("Drop instruction found in block processor");
        }
        InstructionKind::Jump(_, _) => {
            UsageInfo::empty() // Jump instructions do not have usages
        }
        InstructionKind::Transform(dest, src, _) => UsageInfo::with(vec![varToUsage(&src)], Some(dest.toPath())),
        InstructionKind::EnumSwitch(_, _) => UsageInfo::empty(),
        InstructionKind::IntegerSwitch(_, _) => UsageInfo::empty(),
        InstructionKind::With(_, _) => UsageInfo::empty(),
        InstructionKind::ReadImplicit(var, _) => UsageInfo::with(Vec::new(), Some(var.toPath())),
        InstructionKind::WriteImplicit(_, var) => UsageInfo::with(vec![varToUsage(&var)], None),
        InstructionKind::LoadPtr(dest, _) => UsageInfo::with(vec![], Some(dest.toPath())),
        InstructionKind::StorePtr(dest, src) => UsageInfo::with(vec![varToUsage(&src)], Some(dest.toPath())),
        InstructionKind::CreateClosure(var, _) => UsageInfo::with(Vec::new(), Some(var.toPath())),
        InstructionKind::ClosureReturn(_, _, _) => {
            panic!("ClosureReturn found in drop checker, this should not happen")
        }
        InstructionKind::IntegerOp(dest, left, right, _) => {
            UsageInfo::with(vec![varToUsage(&left), varToUsage(&right)], Some(dest.toPath()))
        }
        InstructionKind::Yield(dest, arg) => UsageInfo::with(vec![varToUsage(&arg)], Some(dest.toPath())),
        InstructionKind::FunctionPtr(dest, _) => UsageInfo::with(Vec::new(), Some(dest.toPath())),
        InstructionKind::FunctionPtrCall(dest, closure, args) => {
            let mut usages = vec![varToUsage(&closure)];
            usages.extend(args.iter().map(|arg| varToUsage(arg)));
            UsageInfo::with(usages, Some(dest.toPath()))
        }
        InstructionKind::Sizeof(dest, var) => UsageInfo::with(vec![varToUsage(&var)], Some(dest.toPath())),
        InstructionKind::Transmute(dest, var) => UsageInfo::with(vec![varToUsage(&var)], Some(dest.toPath())),
        InstructionKind::CreateUninitializedArray(dest) => UsageInfo::with(Vec::new(), Some(dest.toPath())),
        InstructionKind::ArrayLen(dest, arr) => UsageInfo::with(vec![varToUsage(&arr)], Some(dest.toPath())),
    }
}
