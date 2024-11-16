use std::{
    collections::{BTreeMap, BTreeSet},
    iter::zip,
};

use crate::siko::{
    hir::{
        Apply::{instantiateClass, instantiateEnum, Apply, ApplyVariable},
        Data::{Class, Enum},
        Function::{Function, Instruction, InstructionKind, Parameter, Variable},
        Program::Program,
        Substitution::{TypeSubstitution, VariableSubstitution},
        TraitMethodSelector::TraitMethodSelector,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::QualifiedName,
};

use super::Error::TypecheckerError;

fn reportError(ctx: &ReportContext, ty1: Type, ty2: Type, location: Location) {
    TypecheckerError::TypeMismatch(format!("{}", ty1), format!("{}", ty2), location).report(ctx)
}

pub struct Typechecker<'a> {
    ctx: &'a ReportContext,
    program: &'a Program,
    traitMethodSelector: &'a TraitMethodSelector,
    allocator: TypeVarAllocator,
    substitution: TypeSubstitution,
    methodCalls: BTreeMap<Variable, QualifiedName>,
    varSwap: VariableSubstitution,
    types: BTreeMap<String, Type>,
    selfType: Option<Type>,
    mutables: BTreeSet<String>,
    implicitRefs: BTreeSet<Variable>,
}

impl<'a> Typechecker<'a> {
    pub fn new(ctx: &'a ReportContext, program: &'a Program, traitMethodSelector: &'a TraitMethodSelector) -> Typechecker<'a> {
        Typechecker {
            ctx: ctx,
            program: program,
            traitMethodSelector: traitMethodSelector,
            allocator: TypeVarAllocator::new(),
            substitution: TypeSubstitution::new(),
            methodCalls: BTreeMap::new(),
            varSwap: VariableSubstitution::new(),
            types: BTreeMap::new(),
            selfType: None,
            mutables: BTreeSet::new(),
            implicitRefs: BTreeSet::new(),
        }
    }

    pub fn run(&mut self, f: &Function) -> Function {
        self.initialize(f);
        //self.dump(f);
        self.check(f);
        //self.dump(f);
        self.generate(f)
    }

    fn initializeVar(&mut self, var: &Variable) {
        match &var.ty {
            Some(ty) => {
                self.types.insert(var.value.clone(), ty.clone());
            }
            None => {
                let ty = self.allocator.next();
                self.types.insert(var.value.clone(), ty.clone());
            }
        }
    }

    pub fn initialize(&mut self, f: &Function) {
        //println!("Initializing {}", f.name);
        for param in &f.params {
            match &param {
                Parameter::Named(name, ty, mutable) => {
                    self.types.insert(name.clone(), ty.clone());
                    if *mutable {
                        self.mutables.insert(name.clone());
                    }
                }
                Parameter::SelfParam(mutable, ty) => {
                    let name = format!("self");
                    self.types.insert(name.clone(), ty.clone());
                    self.selfType = Some(ty.clone());
                    if *mutable {
                        self.mutables.insert(name);
                    }
                }
            }
        }
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    match &instruction.kind {
                        InstructionKind::FunctionCall(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::MethodCall(var, _, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::DynamicFunctionCall(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::ValueRef(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::FieldRef(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::TupleIndex(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::Bind(var, _, mutable) => {
                            self.initializeVar(var);
                            if *mutable {
                                self.mutables.insert(var.value.clone());
                            }
                        }
                        InstructionKind::Tuple(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::StringLiteral(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::IntegerLiteral(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::CharLiteral(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::Return(var, _) => {
                            self.types.insert(var.value.clone(), Type::Never);
                        }
                        InstructionKind::Ref(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::Drop(_) => {}
                        InstructionKind::Jump(var, _) => {
                            self.types.insert(var.value.clone(), Type::Never);
                        }
                        InstructionKind::Assign(_, _) => {}
                        InstructionKind::DeclareVar(var) => {
                            self.initializeVar(var);
                            self.mutables.insert(var.value.clone());
                        }
                        InstructionKind::Transform(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::EnumSwitch(_, _) => {}
                        InstructionKind::IntegerSwitch(_, _) => {}
                        InstructionKind::StringSwitch(_, _) => {}
                    }
                }
            }
        }
    }

    fn getType(&self, var: &Variable) -> Type {
        match self.types.get(&var.value) {
            Some(ty) => ty.clone(),
            None => panic!("No type found for {}!", var),
        }
    }

    fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = unify(&mut self.substitution, &ty1, &ty2) {
            reportError(self.ctx, ty1.apply(&self.substitution), ty2.apply(&self.substitution), location);
        }
    }

    fn instantiateType(&mut self, ty: Type) -> Type {
        let vars = ty.collectVars(BTreeSet::new());
        let mut sub = TypeSubstitution::new();
        for var in &vars {
            sub.add(Type::Var(var.clone()), self.allocator.next());
        }
        ty.apply(&sub)
    }

    fn instantiateEnum(&mut self, e: &Enum, ty: &Type) -> Enum {
        instantiateEnum(&mut self.allocator, e, ty)
    }

    fn instantiateClass(&mut self, c: &Class, ty: &Type) -> Class {
        instantiateClass(&mut self.allocator, c, ty)
    }

    fn checkFunctionCall(&mut self, args: &Vec<Variable>, resultVar: &Variable, fnType: Type) {
        //println!("checkFunctionCall: {}", fnType);
        let fnType = self.instantiateType(fnType);
        let (fnArgs, mut fnResult) = match fnType.splitFnType() {
            Some((fnArgs, fnResult)) => (fnArgs, fnResult),
            None => return,
        };
        if args.len() != fnArgs.len() {
            TypecheckerError::ArgCountMismatch(fnArgs.len() as u32, args.len() as u32, resultVar.location.clone()).report(self.ctx);
        }
        if fnArgs.len() > 0 {
            fnResult = fnResult.changeSelfType(fnArgs[0].clone());
        }
        for (arg, fnArg) in zip(args, fnArgs) {
            let mut argTy = self.getType(arg);
            argTy = argTy.apply(&self.substitution);
            let fnArg = fnArg.apply(&self.substitution);
            if !argTy.isReference() && fnArg.isReference() {
                argTy = Type::Reference(Box::new(argTy), None);
                //println!("IMPLICIT REF FOR {}", arg);
                self.implicitRefs.insert(arg.clone());
            }
            self.unify(argTy, fnArg, arg.location.clone());
        }
        self.unify(self.getType(resultVar), fnResult, resultVar.location.clone());
    }

    fn lookupMethod(&mut self, receiverType: Type, methodName: &String, location: Location) -> QualifiedName {
        match receiverType.clone().unpackRef() {
            Type::Named(name, _, _) => {
                if let Some(classDef) = self.program.classes.get(&name) {
                    let classDef = self.instantiateClass(classDef, &receiverType.unpackRef());
                    for m in &classDef.methods {
                        if m.name == *methodName {
                            //println!("Added {} {}", dest, m.fullName);
                            return m.fullName.clone();
                        }
                    }
                    if let Some(methodName) = self.traitMethodSelector.get(methodName) {
                        return methodName;
                    }
                    TypecheckerError::MethoddNotFound(methodName.clone(), location.clone()).report(self.ctx);
                } else if let Some(enumDef) = self.program.enums.get(&name) {
                    let enumDef = self.instantiateEnum(enumDef, &receiverType.unpackRef());
                    for m in &enumDef.methods {
                        if m.name == *methodName {
                            return m.fullName.clone();
                        }
                    }
                    if let Some(methodName) = self.traitMethodSelector.get(&methodName) {
                        return methodName;
                    }
                    TypecheckerError::MethoddNotFound(methodName.clone(), location.clone()).report(self.ctx);
                } else {
                    TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
                }
            }
            _ => {
                TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
            }
        };
    }

    fn check(&mut self, f: &Function) {
        if f.body.is_none() {
            return;
        };
        for instruction in f.instructions() {
            //println!("Type checking {}", instruction);
            match &instruction.kind {
                InstructionKind::FunctionCall(dest, name, args) => {
                    let f = self.program.functions.get(name).expect("Function not found");
                    let fnType = f.getType();

                    self.checkFunctionCall(args, dest, fnType);
                }
                InstructionKind::MethodCall(dest, receiver, methodName, args) => {
                    let receiverType = self.getType(receiver);
                    let receiverType = receiverType.apply(&self.substitution);
                    let name = self.lookupMethod(receiverType, methodName, instruction.location.clone());
                    self.methodCalls.insert(dest.clone(), name.clone());
                    let f = self.program.functions.get(&name).expect("Function not found");
                    let fnType = f.getType();
                    let mut args = args.clone();
                    args.insert(0, receiver.clone());
                    self.checkFunctionCall(&args, dest, fnType);
                }
                InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                    let fnType = self.getType(callable);
                    self.checkFunctionCall(&args, dest, fnType);
                }
                InstructionKind::ValueRef(dest, value) => {
                    let receiverType = self.getType(value);
                    self.unify(receiverType, self.getType(dest), instruction.location.clone());
                }
                InstructionKind::Bind(name, rhs, _) => {
                    self.unify(self.getType(name), self.getType(rhs), instruction.location.clone());
                }
                InstructionKind::Tuple(dest, args) => {
                    let mut argTypes = Vec::new();
                    for arg in args {
                        argTypes.push(self.getType(arg));
                    }
                    self.unify(self.getType(dest), Type::Tuple(argTypes), instruction.location.clone());
                }
                InstructionKind::StringLiteral(dest, _) => {
                    self.unify(self.getType(dest), Type::getStringType(), instruction.location.clone());
                }
                InstructionKind::IntegerLiteral(dest, _) => {
                    self.unify(self.getType(dest), Type::getIntType(), instruction.location.clone());
                }
                InstructionKind::CharLiteral(dest, _) => {
                    self.unify(self.getType(dest), Type::getCharType(), instruction.location.clone());
                }
                InstructionKind::Return(_, arg) => {
                    let mut result = f.result.clone();
                    if let Some(selfType) = self.selfType.clone() {
                        result = result.changeSelfType(selfType);
                    }
                    self.unify(result, self.getType(arg), instruction.location.clone());
                }
                InstructionKind::Ref(dest, arg) => {
                    let arg_type = self.getType(arg);
                    self.unify(
                        self.getType(dest),
                        Type::Reference(Box::new(arg_type), None),
                        instruction.location.clone(),
                    );
                }
                InstructionKind::Drop(_) => {}
                InstructionKind::Jump(_, _) => {}
                InstructionKind::Assign(name, rhs) => {
                    if !self.mutables.contains(&name.value) {
                        TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                    }
                    self.unify(self.getType(name), self.getType(rhs), instruction.location.clone());
                }
                InstructionKind::DeclareVar(_) => {}
                InstructionKind::Transform(dest, root, index) => {
                    let rootTy = self.getType(root);
                    let rootTy = rootTy.apply(&self.substitution);
                    match rootTy.getName() {
                        Some(name) => {
                            let e = self.program.enums.get(&name).expect("not an enum in transform!");
                            let e = self.instantiateEnum(e, &rootTy);
                            let v = &e.variants[*index as usize];
                            self.unify(self.getType(dest), Type::Tuple(v.items.clone()), instruction.location.clone());
                        }
                        None => {
                            TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                        }
                    };
                }
                InstructionKind::EnumSwitch(_root, _cases) => {}
                InstructionKind::IntegerSwitch(_root, _cases) => {}
                InstructionKind::StringSwitch(_root, _cases) => {}
                InstructionKind::FieldRef(dest, receiver, fieldName) => {
                    let receiverType = self.getType(receiver);
                    let receiverType = receiverType.apply(&self.substitution);
                    match receiverType.clone().unpackRef() {
                        Type::Named(name, _, _) => {
                            if let Some(classDef) = self.program.classes.get(&name) {
                                let classDef = self.instantiateClass(classDef, &receiverType.unpackRef());
                                let mut found = false;
                                for f in &classDef.fields {
                                    if f.name == *fieldName {
                                        self.unify(self.getType(dest), f.ty.clone(), instruction.location.clone());
                                        found = true;
                                    }
                                }
                                if !found {
                                    TypecheckerError::FieldNotFound(fieldName.clone(), instruction.location.clone()).report(self.ctx);
                                }
                            } else {
                                TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                            }
                        }
                        _ => {
                            TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                        }
                    }
                }
                InstructionKind::TupleIndex(dest, receiver, index) => {
                    let receiverType = self.getType(receiver);
                    let receiverType = receiverType.apply(&self.substitution);
                    match receiverType {
                        Type::Tuple(t) => {
                            if *index as usize >= t.len() {
                                TypecheckerError::FieldNotFound(format!(".{}", index), instruction.location.clone()).report(&self.ctx);
                            }
                            let fieldType = t[*index as usize].clone();
                            self.unify(self.getType(dest), fieldType, instruction.location.clone());
                        }
                        _ => TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx),
                    }
                }
            }
        }
    }

    pub fn verify(&self, f: &Function) {
        if let Some(body) = &f.body {
            let fnType = f.getType();
            let publicVars = fnType.collectVars(BTreeSet::new());
            for block in &body.blocks {
                for instruction in &block.instructions {
                    if let Some(v) = instruction.kind.getResultVar() {
                        if let Some(ty) = v.ty {
                            let vars = ty.collectVars(BTreeSet::new());
                            if !vars.is_empty() && vars != publicVars {
                                self.dump(f);
                                println!("MISSING: {} {}", instruction, ty);
                                TypecheckerError::TypeAnnotationNeeded(v.location.clone()).report(self.ctx);
                            }
                        } else {
                            TypecheckerError::TypeAnnotationNeeded(v.location.clone()).report(self.ctx);
                        }
                    }
                }
            }
        }
    }

    pub fn dump(&self, f: &Function) {
        println!("Dumping {}", f.name);
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    match instruction.kind.getResultVar() {
                        Some(v) => match v.ty {
                            Some(ty) => {
                                println!("{} : {}", instruction, ty);
                            }
                            None => {
                                let ty = self.getType(&v);
                                let ty = ty.apply(&self.substitution);
                                println!("{} : {} inferred", instruction, ty);
                            }
                        },
                        None => {
                            println!("{}", instruction);
                        }
                    }
                }
            }
        }
    }

    pub fn generate(&mut self, f: &Function) -> Function {
        //println!("Generating {}", f.name);
        if f.body.is_none() {
            return f.clone();
        }
        let mut result = f.clone();
        if let Some(selfType) = self.selfType.clone() {
            result.result = result.result.changeSelfType(selfType);
        }

        let mut nextImplicitRef = 0;
        let body = &mut result.body.as_mut().unwrap();

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                if let InstructionKind::MethodCall(dest, root, _, args) = &mut instruction.kind {
                    if let Some(fnName) = self.methodCalls.get(&dest) {
                        let mut newArgs = Vec::new();
                        newArgs.push(root.clone());
                        newArgs.extend(args.clone());
                        instruction.kind = InstructionKind::FunctionCall(dest.asFixed(), fnName.clone(), newArgs);
                    }
                }
            }
        }

        for block in &mut body.blocks {
            let mut index = 0;
            loop {
                if index >= block.instructions.len() {
                    break;
                }
                let instruction = block.instructions[index].clone();
                let vars = instruction.kind.collectVariables();
                for var in vars {
                    if self.implicitRefs.contains(&var) {
                        let mut dest = var.clone();
                        let fixedVar = var.asFixed();
                        dest.value = format!("implicitRef{}", nextImplicitRef);
                        nextImplicitRef += 1;
                        let ty = Type::Reference(Box::new(self.getType(&var)), None);
                        self.types.insert(dest.value.clone(), ty);
                        self.varSwap.add(var.asNotFixed(), dest.clone());
                        let kind = InstructionKind::Ref(dest.asFixed(), fixedVar);
                        let implicitRef = Instruction {
                            implicit: true,
                            kind: kind,
                            location: instruction.location.clone(),
                        };
                        block.instructions.insert(index, implicitRef);
                        self.implicitRefs.remove(&var);
                    }
                }
                index += 1;
            }
        }

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                instruction.kind = instruction.kind.applyVar(&self.varSwap);
            }
        }

        let mut varSwap = VariableSubstitution::new();
        varSwap.forced = true;

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                for var in instruction.kind.collectVariables() {
                    let ty = self.getType(&var);
                    let ty = ty.apply(&self.substitution);
                    let mut newVar = var.clone();
                    newVar.ty = Some(ty.clone());
                    let useVar = var.asNotFixed();
                    let mut newUseVar = var.asNotFixed();
                    newUseVar.ty = Some(ty);
                    if newVar != var {
                        varSwap.add(var, newVar);
                    }
                    if useVar != newUseVar {
                        varSwap.add(useVar, newUseVar);
                    }
                }
            }
        }

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                instruction.kind = instruction.kind.applyVar(&varSwap);
            }
        }
        //self.dump(&result);
        self.verify(&result);
        result
    }
}
