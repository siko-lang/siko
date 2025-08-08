use core::panic;
use std::collections::BTreeMap;

use crate::siko::{
    backend::drop::{
        Context::Context,
        DropMetadataStore::DropMetadataStore,
        Path::{InstructionRef, Path, PathSegment},
        Usage::{Usage, UsageKind},
        Util::buildFieldPath,
    },
    hir::{
        Function::{Block, BlockId},
        Instruction::{FieldId, InstructionKind},
        Variable::Variable,
    },
};

pub struct BlockProcessor<'a> {
    receiverPaths: BTreeMap<Variable, Path>,
    dropMetadataStore: &'a mut DropMetadataStore,
}

impl<'a> BlockProcessor<'a> {
    pub fn new(dropMetadataStore: &'a mut DropMetadataStore) -> BlockProcessor<'a> {
        BlockProcessor {
            receiverPaths: BTreeMap::new(),
            dropMetadataStore,
        }
    }

    pub fn process(&mut self, block: &Block, mut context: Context) -> (Context, Vec<BlockId>) {
        // println!("Processing block: {}", block.id);

        // println!("starting context: {}", context);
        // println!("--------------");
        let mut jumpTargets = Vec::new();
        let mut instructionRef = InstructionRef {
            blockId: block.id,
            instructionId: 0,
        };
        for instruction in &block.instructions {
            // println!(
            //     "Processing instruction: {} {} {}",
            //     instruction.kind, instruction.location, instructionRef
            // );
            match &instruction.kind {
                InstructionKind::Jump(_, blockId) => {
                    jumpTargets.push(blockId.clone());
                }
                InstructionKind::EnumSwitch(_, cases) => {
                    // enum switch does not 'use' the variable, transform does
                    for case in cases {
                        jumpTargets.push(case.branch.clone());
                    }
                }
                InstructionKind::IntegerSwitch(var, cases) => {
                    context.useVar(var, instructionRef);
                    for case in cases {
                        jumpTargets.push(case.branch.clone());
                    }
                }
                InstructionKind::StringSwitch(var, cases) => {
                    context.useVar(var, instructionRef);
                    for case in cases {
                        jumpTargets.push(case.branch.clone());
                    }
                }
                kind => {
                    let usageinfo = getUsageInfo(kind.clone());
                    for mut usage in usageinfo.usages {
                        usage.path.instructionRef = instructionRef.clone();
                        context.addUsage(usage);
                    }
                    if let Some(mut assignPath) = usageinfo.assign {
                        assignPath.instructionRef = instructionRef.clone();
                        context.addAssign(assignPath.clone());
                    }
                }
            }
            instructionRef.instructionId += 1;
        }
        let allDropListIds = self.dropMetadataStore.getDropListIds();
        for id in allDropListIds {
            let dropList = self.dropMetadataStore.getDropList(id);
            let rootPath = dropList.getRoot();
            for (name, events) in context.usages.iter() {
                if name.visibleName() == rootPath.root {
                    for path in events.getAllWritePaths() {
                        if path.toSimplePath().sharesPrefixWith(&rootPath) {
                            self.dropMetadataStore.addPath(id, path.clone());
                        }
                    }
                }
            }
        }

        for (name, events) in context.usages.iter() {
            if let Some(declarationList) = self.dropMetadataStore.getDeclarationList(name) {
                for path in events.getAllWritePaths() {
                    declarationList.addPath(path.toSimplePath());
                }
            }
        }

        //println!("Final context: {}", context);
        (context, jumpTargets)
    }
}

struct UsageInfo {
    usages: Vec<Usage>,
    assign: Option<Path>,
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
    //println!("Using variable: {} {}", var.value.visibleName(), ty);
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

fn getUsageInfo(kind: InstructionKind) -> UsageInfo {
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
            let mut path = Path::new(receiver.clone(), dest.location.clone());
            for field in names {
                match &field.name {
                    FieldId::Named(name) => {
                        path = path.add(PathSegment::Named(name.clone()), dest.location.clone());
                    }
                    FieldId::Indexed(index) => {
                        path = path.add(PathSegment::Indexed(*index), dest.location.clone());
                    }
                }
            }
            let kind = if destTy.isReference() || destTy.isPtr() {
                UsageKind::Ref
            } else {
                UsageKind::Move
            };
            UsageInfo::with(vec![Usage { path, kind }], Some(dest.toPath()))
        }
        InstructionKind::FieldAssign(dest, receiver, fields) => UsageInfo::with(
            vec![Usage {
                path: receiver.toPath(),
                kind: UsageKind::Move,
            }],
            Some(buildFieldPath(&dest, &fields)),
        ),
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
        InstructionKind::Ref(_, var) => UsageInfo::with(
            vec![Usage {
                path: var.toPath(),
                kind: UsageKind::Ref,
            }],
            None,
        ),
        InstructionKind::DropListPlaceholder(_) => UsageInfo::empty(),
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
        InstructionKind::StringSwitch(_, _) => UsageInfo::empty(),
    }
}
