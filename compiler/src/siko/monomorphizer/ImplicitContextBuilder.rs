use std::collections::BTreeMap;

use crate::siko::{
    hir::{
        BodyBuilder::BodyBuilder,
        Function::Function,
        Instruction::{FieldId, FieldInfo, ImplicitContextOperation, InstructionKind, SyntaxBlockId},
        Program::Program,
        Type::Type,
        Variable::Variable,
    },
    qualifiedname::builtins::getNativePtrToPtrName,
};

pub struct ImplicitContextBuilder<'a> {
    program: &'a Program,
}

impl<'a> ImplicitContextBuilder<'a> {
    pub fn new(p: &'a Program) -> Self {
        ImplicitContextBuilder { program: p }
    }

    pub fn process(&mut self) -> Program {
        let mut result = self.program.clone();
        for (name, function) in &self.program.functions {
            let f = self.processFunction(&function);
            result.functions.insert(name.clone(), f);
        }
        result
    }

    fn processFunction(&mut self, function: &Function) -> Function {
        let body = match &function.body {
            Some(body) => body.clone(),
            None => return function.clone(),
        };
        let mut bodyBuilder = BodyBuilder::withBody(body);
        let mut contextVarMap: BTreeMap<SyntaxBlockId, Variable> = BTreeMap::new();
        let allBlockIds = bodyBuilder.getAllBlockIds();

        for blockId in &allBlockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    match instruction.kind {
                        InstructionKind::With(_, info) => {
                            let mut contextVar = bodyBuilder.createTempValue(instruction.location.clone());
                            contextVar.ty = Some(Type::Tuple(info.contextTypes.clone()));
                            // println!(
                            //     "Pushing context variable {} to context map with {}",
                            //     contextVar, info.syntaxBlockId
                            // );
                            contextVarMap.insert(info.syntaxBlockId.clone(), contextVar.clone());
                        }
                        _ => {}
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }

        for blockId in allBlockIds {
            let mut builder = bodyBuilder.iterator(blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    match instruction.kind {
                        InstructionKind::With(var, info) => {
                            let jump = InstructionKind::Jump(var, info.blockId);
                            //println!("Building implicit context for with, info {}", info);
                            let contextVar = contextVarMap
                                .get(&info.syntaxBlockId)
                                .expect("Context variable not found for syntax block")
                                .clone();
                            let mut contextTypes = Vec::new();
                            let mut args = Vec::new();
                            for op in info.operations {
                                match op {
                                    ImplicitContextOperation::Copy(index) => {
                                        println!("Copying context variable at index {}", index);
                                        let prevContext = contextVarMap
                                            .get(&info.parentSyntaxBlockId)
                                            .expect("Parent context not found");
                                        let mut fieldRefVar = bodyBuilder.createTempValue(instruction.location.clone());
                                        let fieldTy = info.contextTypes[index.0].clone();
                                        fieldRefVar.ty = Some(fieldTy.clone());
                                        let fieldInfo = FieldInfo {
                                            name: FieldId::Indexed(index.0 as u32),
                                            location: instruction.location.clone(),
                                            ty: Some(fieldTy),
                                        };
                                        let kind = InstructionKind::FieldRef(
                                            fieldRefVar.clone(),
                                            prevContext.clone(),
                                            vec![fieldInfo],
                                        );
                                        builder.addInstruction(kind, instruction.location.clone());
                                        builder.step();
                                        args.push(fieldRefVar);
                                    }
                                    ImplicitContextOperation::Add(index, var) => {
                                        println!("Adding context variable {} at index {}", var, index);
                                        let mut refVar = bodyBuilder.createTempValue(instruction.location.clone());
                                        refVar.ty = Some(Type::Reference(Box::new(var.getType().clone()), None));
                                        let mut ptrVar = bodyBuilder.createTempValue(instruction.location.clone());
                                        let ptrTy = Type::Ptr(Box::new(var.getType().clone()));
                                        ptrVar.ty = Some(ptrTy.clone());
                                        contextTypes.push(ptrTy);
                                        builder.addInstruction(
                                            InstructionKind::Ref(refVar.clone(), var),
                                            instruction.location.clone(),
                                        );
                                        builder.step();
                                        builder.addInstruction(
                                            InstructionKind::FunctionCall(
                                                ptrVar.clone(),
                                                getNativePtrToPtrName(),
                                                vec![refVar],
                                                None,
                                            ),
                                            instruction.location.clone(),
                                        );
                                        builder.step();
                                        args.push(ptrVar);
                                    }
                                    ImplicitContextOperation::Overwrite(_, _) => {
                                        unreachable!(
                                            "Overwrite operation should not be used in implicit context building"
                                        );
                                    }
                                }
                            }
                            let contextCreate = InstructionKind::Tuple(contextVar, args);
                            builder.addInstruction(contextCreate, instruction.location.clone());
                            builder.step();
                            builder.replaceInstruction(jump, instruction.location.clone());
                        }
                        _ => {}
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }

        let mut function = function.clone();
        function.body = Some(bodyBuilder.build());

        // println!("Implicit context builder processed function: {}", function.name);
        // println!("Function: {}", function);

        function
    }
}
