use std::collections::BTreeMap;

use crate::siko::{
    backend::recursivedatahandler::DataGroup::processDataGroups,
    hir::{
        BodyBuilder::BodyBuilder,
        Function::{Function, FunctionKind},
        Instantiation::instantiateEnum,
        Instruction::{CallInfo, FieldId, InstructionKind},
        Program::Program,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
    },
    location::Report::ReportContext,
    qualifiedname::builtins::{getBoxGetFnName, getBoxNewFnName, getBoxReleaseFnName},
};

pub fn process(ctx: &ReportContext, program: Program) -> Program {
    let mut program = processDataGroups(ctx, program);
    let mut functions = BTreeMap::new();
    for (name, f) in &program.functions {
        // Do something with each function
        let processed = processFunction(f, &program);
        functions.insert(name.clone(), processed);
    }
    program.functions = functions;
    program
}

fn processFunction(function: &Function, program: &Program) -> Function {
    if let None = &function.body {
        return function.clone();
    }

    let mut allocator = TypeVarAllocator::new();

    // println!("Processing function: {}", function.name);
    // println!("Function: {}", function);

    let mut bodyBuilder = BodyBuilder::cloneFunction(function);

    let allBlockIds = bodyBuilder.getAllBlockIds();

    let mut transformVars = BTreeMap::new();

    for blockId in allBlockIds {
        let mut builder = bodyBuilder.iterator(blockId);
        loop {
            if let Some(instruction) = builder.getInstruction() {
                //println!("Processing instruction: {}", instruction);
                match &instruction.kind {
                    InstructionKind::FunctionCall(_, info) => {
                        let f = program.getFunction(&info.name).expect("function not found");
                        match f.kind {
                            FunctionKind::VariantCtor(index) => {
                                let mut newKind = instruction.kind.clone();
                                let enumName = f.name.base();
                                let e = program.getEnum(&enumName).expect("enum not found");
                                let variant = &e.variants[index as usize];
                                //println!("Calling {}.{} => {}", f.name, index, formatTypes(&variant.items));
                                for (arg, ty) in info.args.iter().zip(&variant.items) {
                                    let argType = arg.getType();
                                    if argType == *ty {
                                        continue;
                                    }
                                    if ty.isBoxed(&argType) {
                                        //println!("Boxing argument: {} of type {}", arg, argType);
                                        let boxedVar = bodyBuilder
                                            .createTempValueWithType(instruction.location.clone(), ty.clone());
                                        let boxCall = InstructionKind::FunctionCall(
                                            boxedVar.clone(),
                                            CallInfo::new(getBoxNewFnName(), vec![arg.clone()]),
                                        );
                                        newKind = newKind.replaceVar(arg.clone(), boxedVar);
                                        builder.addInstruction(boxCall, instruction.location.clone());
                                        builder.step();
                                    }
                                }
                                builder.replaceInstruction(newKind, instruction.location.clone());
                            }
                            _ => {}
                        }
                    }
                    InstructionKind::Transform(dest, source, index) => {
                        let sourceType = source.getType().unpackRef();
                        let enumName = sourceType.getName().expect("not named type");
                        let e = program.getEnum(&enumName).expect("enum not found");
                        let e = instantiateEnum(&mut allocator, &e, &sourceType);
                        let variant = &e.variants[*index as usize];
                        let variantType = Type::Tuple(variant.items.clone());
                        let destType = dest.getType().unpackRef();
                        if destType != variantType {
                            //println!("Transforming {} from {} to {}", dest, sourceType, variantType);
                            let newDest = dest.clone();
                            if source.getType().isReference() {
                                newDest.setType(variantType.asRef());
                            } else {
                                newDest.setType(variantType.clone());
                            }
                            let newKind = InstructionKind::Transform(newDest, source.clone(), *index);
                            builder.replaceInstruction(newKind, instruction.location.clone());
                            transformVars.insert(dest.clone(), variantType);
                        }
                    }
                    InstructionKind::FieldRef(dest, source, fields) => {
                        if let Some(variantTypes) = transformVars.get(source) {
                            assert_eq!(fields.len(), 1);
                            let fieldInfo = &fields[0];
                            let index = match &fieldInfo.name {
                                FieldId::Indexed(index) => index,
                                _ => {
                                    panic!("Field reference with non-indexed field: {}", fieldInfo.name);
                                }
                            };
                            // println!(
                            //     "Transforming field reference: {} from {} to {} index {}",
                            //     dest, source, variantTypes, index
                            // );
                            let isRef = source.getType().isReference();
                            let newSource = source.clone();
                            if isRef {
                                newSource.setType(variantTypes.asRef());
                            } else {
                                newSource.setType(variantTypes.clone());
                            }
                            let mut fieldTy = variantTypes.getTupleTypes()[*index as usize].clone();
                            if isRef {
                                fieldTy = fieldTy.asRef();
                            }
                            let newDest = bodyBuilder.createTempValueWithType(instruction.location.clone(), fieldTy);
                            let newKind = InstructionKind::FieldRef(newDest.clone(), newSource, fields.clone());
                            builder.replaceInstruction(newKind, instruction.location.clone());
                            let releaseCall = if isRef {
                                InstructionKind::FunctionCall(
                                    dest.clone(),
                                    CallInfo::new(getBoxGetFnName(), vec![newDest.clone()]),
                                )
                            } else {
                                InstructionKind::FunctionCall(
                                    dest.clone(),
                                    CallInfo::new(getBoxReleaseFnName(), vec![newDest.clone()]),
                                )
                            };
                            builder.step();
                            builder.addInstruction(releaseCall, instruction.location.clone());
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

    let mut result = function.clone();
    let body = bodyBuilder.build();
    result.body = Some(body);

    // println!("Processed function: {}", result.name);
    // println!("Function: {}", result);

    result
}
