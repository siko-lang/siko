use std::collections::BTreeMap;

use crate::siko::{
    hir::{
        Block::BlockId,
        BodyBuilder::BodyBuilder,
        Function::{Function, ParamInfo, Parameter},
        Instruction::{
            CallInfo, FieldId, FieldInfo, ImplicitContextOperation, ImplicitIndex, InstructionKind, SyntaxBlockId,
            SyntaxBlockIdSegment,
        },
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    monomorphizer::Monomorphizer::Monomorphizer,
};

pub struct ImplicitContextBuilder<'a, 'b> {
    mono: &'a mut Monomorphizer<'b>,
}

impl<'a, 'b> ImplicitContextBuilder<'a, 'b> {
    pub fn new(mono: &'a mut Monomorphizer<'b>) -> Self {
        ImplicitContextBuilder { mono }
    }

    pub fn process(&mut self) -> Program {
        let mut result = self.mono.monomorphizedProgram.clone();
        let functions = result.functions.clone();
        for (name, function) in &functions {
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

        let mut function = function.clone();

        // println!("Processing implicit context for function: {}", function.name);
        // println!("Processing implicit context for function: {}", function);
        let mut builder = bodyBuilder.iterator(BlockId::first());
        let mut addInitialContext = true;
        if let (_, Some(context)) = function.name.getUnmonomorphized() {
            if !context.handlerResolution.isEmptyImplicits() {
                addInitialContext = false;
                //println!("Non main user function, patching context {}", function.name);
                let contextTypes = context.handlerResolution.getContextTypes(self.mono);
                let implicitContextArgName = format!("siko_implicit_context");
                let argVar = Variable::newWithType(
                    VariableName::Arg(implicitContextArgName.clone()),
                    Location::empty(),
                    Type::Tuple(contextTypes.clone()),
                );
                let contextVar = bodyBuilder.createTempValueWithType(Location::empty(), argVar.getType().clone());
                builder.addDeclare(contextVar.clone(), Location::empty());
                builder.step();
                builder.addAssign(contextVar.clone(), argVar, Location::empty());
                contextVarMap.insert(SyntaxBlockId::new(), contextVar.clone());
                contextVarMap.insert(SyntaxBlockId::new().add(SyntaxBlockIdSegment { value: 0 }), contextVar);
                let param = Parameter::Named(
                    implicitContextArgName.clone(),
                    Type::Tuple(contextTypes.clone()),
                    ParamInfo::new(),
                );
                function.params.insert(0, param);
            }
        }
        if addInitialContext {
            let contextVar = bodyBuilder.createTempValueWithType(Location::empty(), Type::getUnitType());
            builder.addDeclare(contextVar.clone(), Location::empty());
            builder.step();
            builder.addInstruction(
                InstructionKind::Tuple(contextVar.clone(), Vec::new()),
                Location::empty(),
            );
            contextVarMap.insert(SyntaxBlockId::new(), contextVar.clone());
            contextVarMap.insert(SyntaxBlockId::new().add(SyntaxBlockIdSegment { value: 0 }), contextVar);
        }

        for blockId in &allBlockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    match instruction.kind {
                        InstructionKind::With(_, info) => {
                            let contextVar = bodyBuilder.createTempValueWithType(
                                instruction.location.clone(),
                                Type::Tuple(info.contextTypes.clone()),
                            );
                            //println!(
                            //    "Pushing context variable {} to context map with {}",
                            //    contextVar, info.syntaxBlockId
                            //);
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
                                        //println!("Copying context variable at index {}", index);
                                        let prevContext = contextVarMap
                                            .get(&info.parentSyntaxBlockId)
                                            .expect("Parent context not found");
                                        let fieldTy = info.contextTypes[index.0].clone();
                                        let fieldRefVar = bodyBuilder
                                            .createTempValueWithType(instruction.location.clone(), fieldTy.clone());
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
                                    ImplicitContextOperation::Add(_, var) => {
                                        // println!("Adding context variable {} at index {}", var, index);
                                        let ptrVar = bodyBuilder.createTempValueWithType(
                                            instruction.location.clone(),
                                            var.getType().asPtr(),
                                        );
                                        contextTypes.push(ptrVar.getType().clone());
                                        builder.addInstruction(
                                            InstructionKind::PtrOf(ptrVar.clone(), var),
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
                        InstructionKind::ReadImplicit(dest, index) => match index {
                            ImplicitIndex::Resolved(index, id) => {
                                let contextVar = if let Some(contextVar) = contextVarMap.get(&id) {
                                    contextVar.clone()
                                } else {
                                    panic!("Context variable not found for id in implicit context builder {}", id);
                                };
                                let fieldRefVar = bodyBuilder
                                    .createTempValueWithType(instruction.location.clone(), dest.getType().asPtr());
                                let fieldTy = dest.getType().clone();
                                let fieldInfo = FieldInfo {
                                    name: FieldId::Indexed(index.0 as u32),
                                    location: instruction.location.clone(),
                                    ty: Some(fieldTy),
                                };
                                let kind = InstructionKind::FieldRef(fieldRefVar.clone(), contextVar, vec![fieldInfo]);
                                builder.replaceInstruction(kind, instruction.location.clone());
                                builder.step();
                                let kind = InstructionKind::LoadPtr(dest, fieldRefVar);
                                builder.addInstruction(kind, instruction.location.clone());
                            }
                            _ => {
                                panic!("Implicit context index not resolved in implicit context builder");
                            }
                        },
                        InstructionKind::WriteImplicit(index, src) => match index {
                            ImplicitIndex::Resolved(index, id) => {
                                let contextVar = if let Some(contextVar) = contextVarMap.get(&id) {
                                    contextVar.clone()
                                } else {
                                    panic!("Context variable not found for id in implicit context builder {}", id);
                                };
                                let fieldRefVar = bodyBuilder
                                    .createTempValueWithType(instruction.location.clone(), src.getType().asPtr());
                                let fieldTy = src.getType().clone();
                                let fieldInfo = FieldInfo {
                                    name: FieldId::Indexed(index.0 as u32),
                                    location: instruction.location.clone(),
                                    ty: Some(fieldTy),
                                };
                                let kind = InstructionKind::FieldRef(fieldRefVar.clone(), contextVar, vec![fieldInfo]);
                                builder.replaceInstruction(kind, instruction.location.clone());
                                builder.step();
                                let kind = InstructionKind::StorePtr(fieldRefVar, src);
                                builder.addInstruction(kind, instruction.location.clone());
                            }
                            _ => {
                                panic!("Implicit context index not resolved in implicit context builder");
                            }
                        },
                        InstructionKind::FunctionCall(dest, info) => {
                            if let Some(ctx) = &info.context {
                                let contextVar = if let Some(contextVar) = contextVarMap.get(&ctx.contextSyntaxBlockId)
                                {
                                    contextVar.clone()
                                } else {
                                    panic!(
                                    "Context variable not found for id '{}' in implicit context builder for function call '{}'",
                                    ctx.contextSyntaxBlockId, info.name
                                );
                                };
                                let mut args = info.args.getVariables().clone();
                                args.insert(0, contextVar);
                                //println!(
                                //    "Patching function call '{}' to include implicit context, new args: {:?} in syntax block {}",
                                //    info.name, args, ctx.contextSyntaxBlockId
                                //);
                                let kind =
                                    InstructionKind::FunctionCall(dest.clone(), CallInfo::new(info.name.clone(), args));
                                builder.replaceInstruction(kind, instruction.location.clone());
                            }
                        }
                        InstructionKind::BlockEnd(_) => {
                            builder.removeInstruction();
                            continue;
                        }
                        InstructionKind::BlockStart(_) => {
                            builder.removeInstruction();
                            continue;
                        }
                        InstructionKind::DeclareVar(_, _) => {
                            builder.removeInstruction();
                            continue;
                        }
                        _ => {}
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }

        function.body = Some(bodyBuilder.build());

        // println!("Implicit context builder processed function: {}", function.name);
        // println!("Function: {}", function);

        function
    }
}
