use std::fmt::Display;

use crate::siko::{
    backend::drop::{Path::Path, Util::buildFieldPath},
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

fn varToUsage(var: &Variable) -> Usage {
    let ty = var.getType();
    //println!("Using variable: {} {}", var.name.visibleName(), ty);
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

pub fn getUsageInfo(kind: InstructionKind) -> UsageInfo {
    match kind {
        InstructionKind::DeclareVar(_, _) => UsageInfo::empty(),
        InstructionKind::BlockStart(_) => UsageInfo::empty(),
        InstructionKind::BlockEnd(_) => UsageInfo::empty(),
        InstructionKind::FunctionCall(dest, _, args) => {
            UsageInfo::with(args.iter().map(|arg| varToUsage(arg)).collect(), Some(dest.toPath()))
        }
        InstructionKind::Assign(dest, src) => UsageInfo::with(vec![varToUsage(&src)], Some(dest.toPath())),
        InstructionKind::Return(_, arg) => UsageInfo::with(vec![varToUsage(&arg)], None),
        InstructionKind::FieldRef(dest, receiver, names) => {
            let destTy = dest.getType();
            let path = buildFieldPath(&receiver, &names);
            let kind = if destTy.isReference() || destTy.isPtr() {
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
        InstructionKind::DynamicFunctionCall(_, _, _) => {
            panic!("Dynamic function call found in block processor");
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
        InstructionKind::With(_, _, _) => UsageInfo::empty(),
    }
}
