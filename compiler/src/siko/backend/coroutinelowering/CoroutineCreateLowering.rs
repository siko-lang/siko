use crate::siko::{
    backend::coroutinelowering::Utils::{getLoweredCoroutineName, getStateMachineEnumName},
    hir::{
        BodyBuilder::BodyBuilder,
        Function::Function,
        Instruction::{CallInfo, InstructionKind},
        Type::Type,
    },
    qualifiedname::QualifiedName,
};

pub fn lowerCoroutineCreate(f: &Function) -> Function {
    if f.body.is_none() {
        return f.clone();
    }
    let mut bodyBuilder = BodyBuilder::cloneFunction(f);
    // println!("Lowering coroutine create for function: {}", f.name);
    // println!("Function: {}", f);
    let allBlockIds = bodyBuilder.getAllBlockIds();
    for blockId in allBlockIds {
        let mut builder = bodyBuilder.iterator(blockId);
        loop {
            if let Some(instr) = builder.getInstruction() {
                match &instr.kind {
                    InstructionKind::FunctionCall(dest, info) => {
                        if info.coroutineSpawn {
                            let entryPointName =
                                QualifiedName::CoroutineStateMachineVariant(Box::new(info.name.clone()), 0 as u32);
                            let stateMachineTy = Type::Named(getStateMachineEnumName(&info.name), vec![]);
                            let variantCtorVar =
                                bodyBuilder.createTempValueWithType(instr.location.clone(), stateMachineTy);
                            let variantCtorCallInfo = CallInfo {
                                name: entryPointName.clone(),
                                args: info.args.clone(),
                                context: info.context.clone(),
                                coroutineSpawn: false,
                                instanceRefs: Vec::new(),
                            };
                            let variantCtorCall =
                                InstructionKind::FunctionCall(variantCtorVar.clone(), variantCtorCallInfo);
                            builder.addInstruction(variantCtorCall, instr.location.clone());
                            builder.step();
                            let coroutineName = getLoweredCoroutineName(&dest.getType());
                            let variantName = QualifiedName::CoroutineInstance(
                                Box::new(coroutineName),
                                Box::new(QualifiedName::CoroutineStateMachineEnum(Box::new(info.name.clone()))),
                            );
                            let info = CallInfo::new(variantName, vec![variantCtorVar.useVar()]);
                            let newCall = InstructionKind::FunctionCall(dest.clone(), info);
                            builder.replaceInstruction(newCall, instr.location.clone());
                        }
                    }
                    _ => {}
                }
                builder.step();
            } else {
                break;
            }
        }
    }
    // println!("Lowered coroutine create for function: {}", f.name);
    // println!("Function: {}", f);
    let mut f = f.clone();
    f.body = Some(bodyBuilder.build());
    f
}
