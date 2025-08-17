use crate::siko::{
    hir::{
        Apply::Apply,
        Function::FunctionKind,
        InstanceResolver::ResolutionResult,
        Instruction::{Instruction, InstructionKind, SyntaxBlockId, WithContext},
        Substitution::Substitution,
        Type::{formatTypes, Type},
        TypeVarAllocator::TypeVarAllocator,
    },
    location::Report::Report,
    monomorphizer::{
        Context::EffectResolutionStore,
        Effect::EffectResolution,
        Monomorphizer::Monomorphizer,
        Queue::Key,
        Utils::{createTypeSubstitution, Monomorphize},
    },
    qualifiedname::{
        builtins::{getAutoDropFnName, getDropFnName, getDropName},
        QualifiedName,
    },
};

pub fn processInstruction(
    input: &Instruction,
    sub: &Substitution,
    mono: &mut Monomorphizer,
    syntaxBlockId: &SyntaxBlockId,
    effectResolutionStore: &mut EffectResolutionStore,
) -> Instruction {
    //println!("MONO INSTR {}", input);
    let mut instruction = input.clone();
    let kind: InstructionKind = match &input.kind {
        InstructionKind::FunctionCall(dest, name, args) => {
            //println!("Calling {} in block {}", name, syntaxBlockId);
            let target_fn = mono.program.getFunction(name).expect("function not found in mono");
            //println!("Target function: {}", target_fn.kind);
            let (target_fn, resolution) = match target_fn.kind {
                FunctionKind::EffectMemberDecl(_) => {
                    //println!("Effect member call in mono!");
                    let effectResolution = effectResolutionStore.get(syntaxBlockId);
                    let handler = effectResolution.get(name);
                    if handler.is_none() {
                        let slogan = format!(
                            "Effect method not present in current effect context: {}",
                            format!("{}", mono.ctx.yellow(&name.toString()))
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
                    (f, effectResolution.clone())
                }
                FunctionKind::EffectMemberDefinition(_) => {
                    let effectResolution = effectResolutionStore.get(syntaxBlockId);
                    let resolvedName = effectResolution.get(name);
                    let f = if let Some(handler) = resolvedName {
                        handler.markUsed();
                        mono.program
                            .getFunction(&handler.name)
                            .expect("effect resolved function not found in mono")
                    } else {
                        target_fn
                    };
                    (f, effectResolution.clone())
                }
                _ => (target_fn, effectResolutionStore.get(syntaxBlockId).clone()),
            };
            //println!("Target function: {}", target_fn);
            let fn_ty = target_fn.getType();
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
            let arg_types: Vec<_> = args
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
            //println!("fn type {}", fn_ty);
            //println!("context type {}", context_ty);
            let sub = createTypeSubstitution(context_ty.clone(), fn_ty.clone());
            //println!("target ctx {}", target_fn.constraintContext);
            let name = getFunctionName(target_fn.kind.clone(), target_fn.name.clone(), mono, &sub);
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
            let resolution = if target_fn.kind.isCtor() {
                EffectResolution::new()
            } else {
                resolution
            };
            let fn_name = mono.getMonoName(&name, &ty_args, resolution.clone());
            //println!("MONO CALL: {}", fn_name);
            mono.addKey(Key::Function(name.clone(), ty_args, resolution));
            InstructionKind::FunctionCall(dest.clone(), fn_name, args.clone())
        }
        InstructionKind::Ref(dest, src) => {
            if dest.ty.as_ref().unwrap().isReference() && src.ty.as_ref().unwrap().isReference() {
                InstructionKind::Assign(dest.clone(), src.clone())
            } else {
                InstructionKind::Ref(dest.clone(), src.clone())
            }
        }
        InstructionKind::Drop(dropRes, dropVar) => {
            let ty = dropVar.ty.clone().apply(sub).unwrap();
            let effectResolution = effectResolutionStore.get(syntaxBlockId).clone();
            let monoName = mono.getMonoName(&getAutoDropFnName(), &vec![ty.clone()], effectResolution.clone());
            mono.addKey(Key::AutoDropFn(getAutoDropFnName(), ty.clone(), effectResolution));
            InstructionKind::FunctionCall(dropRes.clone(), monoName, vec![dropVar.clone()])
        }
        k => k.clone(),
    };
    instruction.kind = processInstructionKind(kind, sub, mono, syntaxBlockId, effectResolutionStore);
    instruction
}

pub fn processInstructionKind(
    input: InstructionKind,
    sub: &Substitution,
    mono: &mut Monomorphizer,
    syntaxBlockId: &SyntaxBlockId,
    effectResolutionStore: &mut EffectResolutionStore,
) -> InstructionKind {
    match input {
        InstructionKind::FunctionCall(dest, name, args) => {
            InstructionKind::FunctionCall(dest.process(sub, mono), name.clone(), args.process(sub, mono))
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
        InstructionKind::With(v, contexts, blockId, withSyntaxBlockId) => {
            let mut effectResolution = effectResolutionStore.get(syntaxBlockId).clone();
            for c in contexts {
                match c {
                    WithContext::EffectHandler(handler) => {
                        effectResolution.add(handler.method, handler.handler, handler.location);
                    }
                    WithContext::Implicit(_) => {
                        // Handle other contexts if needed
                    }
                }
            }
            effectResolutionStore.insert(withSyntaxBlockId, effectResolution);
            InstructionKind::Jump(v, blockId)
        }
        InstructionKind::GetImplicit(_, _) => todo!(),
    }
}

fn getFunctionName(
    kind: FunctionKind,
    name: QualifiedName,
    mono: &mut Monomorphizer,
    sub: &Substitution,
) -> QualifiedName {
    if let Some(traitName) = kind.isTraitCall() {
        //println!("Trait call in mono!");
        let traitDef = mono.program.getTrait(&traitName).unwrap();
        //println!("trait {}", traitDef);
        let mut allocator = TypeVarAllocator::new();
        let traitDef = traitDef.apply(&sub);
        //println!("trait ii {}", traitDef);
        if let Some(instances) = mono.program.instanceResolver.lookupInstances(&traitName) {
            let resolutionResult = instances.find(&mut allocator, &traitDef.params);
            match resolutionResult {
                ResolutionResult::Winner(instance) => {
                    //println!("instance  {}", instance);
                    for m in &instance.members {
                        let base = m.fullName.getTraitMemberName();
                        if base == name {
                            return m.fullName.clone();
                        }
                    }
                    let traitDef = mono.program.getTrait(&traitName).expect("trait not found in mono");
                    for m in &traitDef.members {
                        if m.fullName == name {
                            return m.fullName.clone();
                        }
                    }
                    panic!("instance member not found!")
                }
                ResolutionResult::Ambiguous(_) => {
                    panic!("Ambiguous instances in mono!");
                }
                ResolutionResult::NoInstanceFound => {
                    if traitName == getDropName() {
                        return getDropFnName();
                    } else {
                        panic!(
                            "instance not found in mono for {} {}!",
                            traitName,
                            formatTypes(&traitDef.params)
                        );
                    }
                }
            }
        } else {
            if traitName == getDropName() {
                return getDropFnName();
            } else {
                panic!("instances not found in mono for {}!", traitName);
            }
        }
    } else {
        name.clone()
    }
}
