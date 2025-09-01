use core::panic;

use crate::siko::{
    hir::{
        Apply::Apply,
        Function::FunctionKind,
        Instruction::{
            CallContextInfo, CallInfo, ImplicitContextIndex, ImplicitContextOperation, ImplicitIndex,
            InstanceReference, Instruction, InstructionKind, SyntaxBlockId, WithContext,
        },
        Substitution::Substitution,
        Type::Type,
        Utils::createResolvers,
        Variable::Variable,
    },
    location::Report::Report,
    monomorphizer::{
        Context::HandlerResolutionStore,
        Handler::HandlerResolution,
        Monomorphizer::Monomorphizer,
        Queue::Key,
        Utils::{createTypeSubstitution, Monomorphize},
    },
    qualifiedname::{builtins::getAutoDropFnName, QualifiedName},
};

impl Monomorphize for CallInfo {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        let args = self.args.process(sub, mono);
        CallInfo {
            name: self.name.clone(),
            args,
            context: self.context.clone(),
            instanceRefs: self.instanceRefs.clone(),
        }
    }
}

pub fn processInstruction(
    input: &Instruction,
    sub: &Substitution,
    mono: &mut Monomorphizer,
    syntaxBlockId: &SyntaxBlockId,
    handlerResolutionStore: &mut HandlerResolutionStore,
    impls: &Vec<QualifiedName>,
) -> Instruction {
    //println!("MONO INSTR {}", input);
    let mut instruction = input.clone();
    let kind: InstructionKind = match &input.kind {
        InstructionKind::FunctionCall(dest, info) => {
            let mut info = info.clone();
            // println!(
            //     "Calling {} in block {} with impls {:?}",
            //     info.name, syntaxBlockId, impls
            // );
            let target_fn = match mono.program.getFunction(&info.name) {
                Some(f) => f,
                None => {
                    let slogan = format!(
                        "Function {} not found during monomorphization, maybe std is missing?",
                        format!("{}", mono.ctx.yellow(&info.name.toString()))
                    );
                    let r = Report::new(mono.ctx, slogan, None);
                    r.print();
                    std::process::exit(1);
                }
            };
            //println!("Target function: {}", target_fn.kind);
            let (target_fn, resolution, contextSyntaxBlockId) = match &target_fn.kind {
                FunctionKind::EffectMemberDecl(_) => {
                    //println!("Effect member call in mono!");
                    let (handlerResolution, contextSyntaxBlockId) = handlerResolutionStore.get(syntaxBlockId);
                    let handler = handlerResolution.getEffectHandler(&info.name);
                    if handler.is_none() {
                        let slogan = format!(
                            "Effect method not present in current effect context: {}",
                            format!("{}", mono.ctx.yellow(&info.name.toString()))
                        );
                        let r = Report::new(mono.ctx, slogan, Some(input.location.clone()));
                        r.print();
                        std::process::exit(1);
                    }
                    let handler = handler.unwrap();
                    handler.markUsed();
                    let f = mono
                        .program
                        .getFunction(&handler.name)
                        .expect("effect resolved function not found in mono");
                    (f, handlerResolution.clone(), contextSyntaxBlockId)
                }
                FunctionKind::EffectMemberDefinition(_) => {
                    let (handlerResolution, contextSyntaxBlockId) = handlerResolutionStore.get(syntaxBlockId);
                    let resolvedName = handlerResolution.getEffectHandler(&info.name);
                    let f = if let Some(handler) = resolvedName {
                        handler.markUsed();
                        mono.program
                            .getFunction(&handler.name)
                            .expect("effect resolved function not found in mono")
                    } else {
                        target_fn
                    };
                    (f, handlerResolution.clone(), contextSyntaxBlockId)
                }
                FunctionKind::TraitMemberDecl(_) | FunctionKind::TraitMemberDefinition(_) => {
                    let (handlerResolution, contextSyntaxBlockId) = handlerResolutionStore.get(syntaxBlockId);
                    let (_, mut fnCallResolver) = createResolvers(&target_fn, mono.ctx, &mono.program);
                    // let fnType = target_fn.getType();
                    // // println!(
                    // //     "Trait member call in mono: {} {} {}",
                    // //     info.name, fnType, _traitName
                    // // );
                    // let (_fn_ty, context_ty) = prepareTypes(sub, dest, info, fnType);
                    // // println!("trait call fn type {}", _fn_ty);
                    // // println!("trait call context type {}", context_ty);
                    // // println!("all available implementations: {:?}", impls);
                    //println!("Substitution: {} ", sub);
                    let mut args = Vec::new();
                    for arg in &info.args {
                        //println!("arg {}", arg);
                        args.push(arg.clone().apply(&sub));
                    }
                    let dest = dest.clone().apply(&sub);
                    // {
                    //     println!("Trait member call in mono! {}", target_fn.name);
                    //     let mut argTypes = Vec::new();
                    //     for arg in &args {
                    //         let argType = arg.getType().clone().apply(&sub);
                    //         argTypes.push(argType);
                    //     }
                    //     println!("Args: {}", crate::siko::hir::Type::formatTypes(&argTypes));
                    //     println!("Dest: {}", dest);
                    // }
                    let checkresult = fnCallResolver.resolve(&target_fn, &args, &dest, instruction.location.clone());
                    //println!("Resolved to {}", checkresult.fnName);
                    let targetfn = mono
                        .program
                        .getFunction(&checkresult.fnName)
                        .expect("function not found in mono");
                    let checkresult = fnCallResolver.resolve(&targetfn, &args, &dest, instruction.location.clone());
                    //println!("Resolved to {}", checkresult.fnName);
                    let targetFn = mono
                        .program
                        .getFunction(&checkresult.fnName)
                        .expect("function not found in mono");
                    info.instanceRefs = checkresult.instanceRefs;
                    (targetFn, handlerResolution.clone(), contextSyntaxBlockId)
                }
                _ => {
                    let (handlerResolution, contextSyntaxBlockId) = handlerResolutionStore.get(syntaxBlockId);
                    (target_fn, handlerResolution.clone(), contextSyntaxBlockId)
                }
            };
            //println!("Target function: {} {}", target_fn, contextSyntaxBlockId);
            let (_, context_ty) = prepareTypes(sub, dest, &info, target_fn.getType());
            // println!("fn type {}", fn_ty);
            // println!("context type {}", context_ty);
            let name = target_fn.name.clone();
            let target_fn = mono.program.functions.get(&name).expect("function not found in mono");
            //println!("real {} {}", target_fn.getType(), target_fn.constraintContext);
            let sub = createTypeSubstitution(context_ty.clone(), target_fn.getType().clone());
            //println!("target ctx {}", target_fn.constraintContext);
            let ty_args: Vec<_> = target_fn
                .constraintContext
                .typeParameters
                .iter()
                .map(|ty| ty.clone().apply(&sub))
                .collect();
            //println!("{} type args {}", name, formatTypes(&ty_args));
            let (resolution, callCtx) =
                if target_fn.kind.isCtor() || target_fn.kind.isExternC() || target_fn.kind.isBuiltin() {
                    (HandlerResolution::new(), None)
                } else {
                    let info = if resolution.isEmptyImplicits() {
                        None
                    } else {
                        Some(CallContextInfo { contextSyntaxBlockId })
                    };
                    (resolution, info)
                };
            let mut resolvedImpls = Vec::new();
            for instanceRef in &info.instanceRefs {
                match instanceRef {
                    InstanceReference::Direct(name) => {
                        resolvedImpls.push(name.clone());
                    }
                    InstanceReference::Indirect(index) => {
                        if let Some(name) = impls.get(*index as usize) {
                            resolvedImpls.push(name.clone());
                        } else {
                            panic!("indirect instance reference out of bounds {} impls {:?}", index, impls);
                        }
                    }
                }
            }
            let fn_name = mono.getMonoName(&name, &ty_args, resolution.clone(), resolvedImpls.clone());
            //println!("MONO CALL: {}", fn_name);
            mono.addKey(Key::Function(name.clone(), ty_args, resolution, resolvedImpls));
            let mut callInfo = CallInfo::new(fn_name, info.args.clone());
            callInfo.context = callCtx;
            InstructionKind::FunctionCall(dest.clone(), callInfo)
        }
        InstructionKind::Ref(dest, src) => {
            if dest.getType().isReference() && src.getType().isReference() {
                InstructionKind::Assign(dest.clone(), src.clone())
            } else {
                InstructionKind::Ref(dest.clone(), src.clone())
            }
        }
        InstructionKind::Drop(dropRes, dropVar) => {
            let ty = dropVar.getType().clone().apply(sub);
            let (handlerResolution, contextSyntaxBlockId) = handlerResolutionStore.get(syntaxBlockId);
            let monoName = mono.getMonoName(
                &getAutoDropFnName(),
                &vec![ty.clone()],
                handlerResolution.clone(),
                Vec::new(),
            );
            mono.addKey(Key::AutoDropFn(
                getAutoDropFnName(),
                ty.clone(),
                handlerResolution.clone(),
            ));
            let callCtx = if handlerResolution.isEmptyImplicits() {
                None
            } else {
                Some(CallContextInfo { contextSyntaxBlockId })
            };
            let mut callInfo = CallInfo::new(monoName.clone(), vec![dropVar.clone()]);
            callInfo.context = callCtx;
            InstructionKind::FunctionCall(dropRes.clone(), callInfo)
        }
        InstructionKind::ReadImplicit(dest, index) => {
            let implicitName = match index {
                ImplicitIndex::Resolved(_, _) => panic!("implicit index already resolved in mono"),
                ImplicitIndex::Unresolved(name) => name,
            };
            let (handlerResolution, id) = handlerResolutionStore.get(syntaxBlockId);
            if let Some(handler) = handlerResolution.getImplicitHandler(implicitName) {
                handler.markUsed();
                InstructionKind::ReadImplicit(
                    dest.process(sub, mono),
                    ImplicitIndex::Resolved(handler.index.clone(), id),
                )
            } else {
                // report error
                let slogan = format!(
                    "Implicit variable not present in current implicit context: {}",
                    format!("{}", mono.ctx.yellow(&implicitName.toString()))
                );
                let r = Report::new(mono.ctx, slogan, Some(input.location.clone()));
                r.print();
                std::process::exit(1);
            }
        }
        InstructionKind::WriteImplicit(index, src) => {
            let implicitName = match index {
                ImplicitIndex::Resolved(_, _) => panic!("implicit index already resolved in mono"),
                ImplicitIndex::Unresolved(name) => name,
            };
            let (handlerResolution, id) = handlerResolutionStore.get(syntaxBlockId);
            if let Some(handler) = handlerResolution.getImplicitHandler(implicitName) {
                handler.markUsed();
                InstructionKind::WriteImplicit(
                    ImplicitIndex::Resolved(handler.index.clone(), id),
                    src.process(sub, mono),
                )
            } else {
                // report error
                let slogan = format!(
                    "Implicit variable not present in current implicit context: {}",
                    format!("{}", mono.ctx.yellow(&implicitName.toString()))
                );
                let r = Report::new(mono.ctx, slogan, Some(input.location.clone()));
                r.print();
                std::process::exit(1);
            }
        }
        k => k.clone(),
    };
    instruction.kind = processInstructionKind(kind, sub, mono, syntaxBlockId, handlerResolutionStore);
    instruction
}

fn prepareTypes(sub: &Substitution, dest: &Variable, info: &CallInfo, targetFnType: Type) -> (Type, Type) {
    let fn_ty = targetFnType;
    let fnResult = fn_ty.getResult();
    let fn_ty = if fnResult.hasSelfType() {
        //println!("fn type before {}", fn_ty);
        let (args, result) = fn_ty.splitFnType().unwrap();
        let result = result.changeSelfType(args[0].clone());
        let fn_ty = Type::Function(args, Box::new(result));
        //println!("fn type after {}", fn_ty);
        fn_ty
    } else {
        fn_ty
    };
    let arg_types: Vec<_> = info
        .args
        .iter()
        .map(|arg| {
            //println!("arg {}", arg);
            let ty = arg.getType();
            ty.clone().apply(&sub)
        })
        .collect();
    //println!("sub {}", sub);
    let result = dest.getType().clone().apply(sub);
    let context_ty = Type::Function(arg_types, Box::new(result));
    (fn_ty, context_ty)
}

pub fn processInstructionKind(
    input: InstructionKind,
    sub: &Substitution,
    mono: &mut Monomorphizer,
    syntaxBlockId: &SyntaxBlockId,
    handlerResolutionStore: &mut HandlerResolutionStore,
) -> InstructionKind {
    match input {
        InstructionKind::FunctionCall(dest, info) => {
            InstructionKind::FunctionCall(dest.process(sub, mono), info.process(sub, mono))
        }
        InstructionKind::Converter(dest, source) => {
            InstructionKind::Converter(dest.process(sub, mono), source.process(sub, mono))
        }
        InstructionKind::MethodCall(_, _, _, _) => {
            unreachable!("method in mono??")
        }
        InstructionKind::DynamicFunctionCall(dest, root, args) => InstructionKind::DynamicFunctionCall(
            dest.process(sub, mono),
            root.process(sub, mono),
            args.process(sub, mono),
        ),
        InstructionKind::FieldRef(dest, root, fields) => InstructionKind::FieldRef(
            dest.process(sub, mono),
            root.process(sub, mono),
            fields.process(sub, mono),
        ),
        InstructionKind::Bind(_, _, _) => {
            panic!("Bind instruction found in Monomorphizer, this should not happen");
        }
        InstructionKind::Tuple(dest, args) => InstructionKind::Tuple(dest.process(sub, mono), args.process(sub, mono)),
        InstructionKind::StringLiteral(dest, lit) => {
            InstructionKind::StringLiteral(dest.process(sub, mono), lit.clone())
        }
        InstructionKind::IntegerLiteral(dest, lit) => {
            InstructionKind::IntegerLiteral(dest.process(sub, mono), lit.clone())
        }
        InstructionKind::CharLiteral(dest, lit) => InstructionKind::CharLiteral(dest.process(sub, mono), lit),
        InstructionKind::Return(dest, arg) => InstructionKind::Return(dest.process(sub, mono), arg.process(sub, mono)),
        InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.process(sub, mono), arg.process(sub, mono)),
        InstructionKind::PtrOf(dest, arg) => InstructionKind::PtrOf(dest.process(sub, mono), arg.process(sub, mono)),
        InstructionKind::DropPath(id) => {
            panic!(
                "DropListPlaceholder found in Monomorphizer, this should not happen: {}",
                id
            );
        }
        InstructionKind::DropMetadata(id) => {
            panic!("DropMetadata found in Monomorphizer, this should not happen: {}", id);
        }
        InstructionKind::Drop(dest, dropVar) => {
            InstructionKind::Drop(dest.process(sub, mono), dropVar.process(sub, mono))
        }
        InstructionKind::Jump(dest, block_id) => InstructionKind::Jump(dest.process(sub, mono), block_id),
        InstructionKind::Assign(dest, rhs) => InstructionKind::Assign(dest.process(sub, mono), rhs.process(sub, mono)),
        InstructionKind::FieldAssign(dest, rhs, fields) => InstructionKind::FieldAssign(
            dest.process(sub, mono),
            rhs.process(sub, mono),
            fields.process(sub, mono),
        ),
        InstructionKind::AddressOfField(dest, receiver, fields) => InstructionKind::AddressOfField(
            dest.process(sub, mono),
            receiver.process(sub, mono),
            fields.process(sub, mono),
        ),
        InstructionKind::DeclareVar(var, mutability) => {
            InstructionKind::DeclareVar(var.process(sub, mono), mutability.clone())
        }
        InstructionKind::Transform(dest, root, index) => {
            InstructionKind::Transform(dest.process(sub, mono), root.process(sub, mono), index.clone())
        }
        InstructionKind::EnumSwitch(root, cases) => InstructionKind::EnumSwitch(root.process(sub, mono), cases.clone()),
        InstructionKind::IntegerSwitch(root, cases) => {
            InstructionKind::IntegerSwitch(root.process(sub, mono), cases.clone())
        }
        InstructionKind::BlockStart(info) => InstructionKind::BlockStart(info.clone()),
        InstructionKind::BlockEnd(info) => InstructionKind::BlockEnd(info.clone()),
        InstructionKind::With(v, mut info) => {
            let (handlerResolution, parentSyntaxBlockId) = handlerResolutionStore.get(syntaxBlockId);
            info.parentSyntaxBlockId = parentSyntaxBlockId;
            let mut handlerResolution = handlerResolution.clone();
            let mut addedImplicit = false;
            let mut operations = Vec::new();
            let originalContextTypes = handlerResolution.getContextTypes(mono);
            for (index, _) in originalContextTypes.iter().enumerate() {
                operations.push(ImplicitContextOperation::Copy(ImplicitContextIndex(index)));
            }
            for c in &info.contexts {
                match c {
                    WithContext::EffectHandler(handler) => {
                        handlerResolution.addEffectHandler(
                            handler.method.clone(),
                            handler.handler.clone(),
                            handler.location.clone(),
                        );
                    }
                    WithContext::Implicit(h) => {
                        addedImplicit = true;
                        let op =
                            handlerResolution.addImplicitHandler(h.implicit.clone(), h.location.clone(), h.var.clone());
                        match op {
                            ImplicitContextOperation::Copy(_) => {
                                panic!("Copy operation when adding new handler")
                            }
                            ImplicitContextOperation::Add(index, var) => {
                                operations.push(ImplicitContextOperation::Add(index, var));
                            }
                            ImplicitContextOperation::Overwrite(index, var) => {
                                operations[index.0] = ImplicitContextOperation::Add(index, var);
                            }
                        }
                    }
                }
            }
            if addedImplicit {
                let contextTypes = handlerResolution.getContextTypes(mono);
                // println!("Context types: {}", formatTypes(&contextTypes));
                // println!("Context type: {}", ty);
                // println!("Context operations: {:?}", operations);
                info.contextTypes = contextTypes;
                info.operations = operations;
            }
            handlerResolutionStore.insert(info.syntaxBlockId.clone(), handlerResolution);
            InstructionKind::With(v, info)
        }
        InstructionKind::ReadImplicit(var, index) => {
            InstructionKind::ReadImplicit(var.process(sub, mono), index.clone())
        }
        InstructionKind::WriteImplicit(index, var) => {
            InstructionKind::WriteImplicit(index.clone(), var.process(sub, mono))
        }
        InstructionKind::LoadPtr(dest, src) => {
            InstructionKind::LoadPtr(dest.process(sub, mono), src.process(sub, mono))
        }
        InstructionKind::StorePtr(dest, src) => {
            InstructionKind::StorePtr(dest.process(sub, mono), src.process(sub, mono))
        }
        InstructionKind::CreateClosure(var, info) => {
            InstructionKind::CreateClosure(var.process(sub, mono), info.clone())
            // TODO
        }
    }
}
