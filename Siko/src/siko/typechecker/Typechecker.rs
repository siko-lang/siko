use std::{
    collections::{BTreeMap, BTreeSet},
    iter::zip,
};

use crate::siko::{
    hir::{
        Function::{Body, Function, Instruction, InstructionId, InstructionKind, Parameter, ValueKind},
        Program::Program,
        Substitution::Substitution,
        TraitMethodSelector::TraitMethodSelector,
        Type::Type,
    },
    location::Location::Location,
    qualifiedname::QualifiedName,
};

use super::{Error::TypecheckerError, TypeVarAllocator::TypeVarAllocator};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TypedId {
    Instruction(InstructionId),
    Value(String),
    SelfValue,
}

fn reportError(ty1: Type, ty2: Type, location: Location) {
    TypecheckerError::TypeMismatch(format!("{}", ty1), format!("{}", ty2), location).report()
}

pub struct Typechecker<'a> {
    program: &'a Program,
    traitMethodSelector: &'a TraitMethodSelector,
    allocator: TypeVarAllocator,
    substitution: Substitution,
    methodSources: BTreeMap<InstructionId, QualifiedName>,
    methodCalls: BTreeMap<InstructionId, InstructionId>,
    indices: BTreeMap<InstructionId, Vec<u32>>,
    types: BTreeMap<TypedId, Type>,
}

impl<'a> Typechecker<'a> {
    pub fn new(program: &'a Program, traitMethodSelector: &'a TraitMethodSelector) -> Typechecker<'a> {
        Typechecker {
            program: program,
            traitMethodSelector: traitMethodSelector,
            allocator: TypeVarAllocator::new(),
            substitution: Substitution::new(),
            methodSources: BTreeMap::new(),
            methodCalls: BTreeMap::new(),
            indices: BTreeMap::new(),
            types: BTreeMap::new(),
        }
    }

    pub fn run(&mut self, f: &Function) -> Function {
        self.initialize(f);
        //self.dump(f);
        self.check(f);
        self.verify(f);
        //self.dump(f);
        self.generate(f)
    }

    pub fn initialize(&mut self, f: &Function) {
        //println!("Initializing {}", f.name);
        for param in &f.params {
            match &param {
                Parameter::Named(name, ty, _) => {
                    self.types.insert(TypedId::Value(name.clone()), ty.clone());
                }
                Parameter::SelfParam(_, ty) => {
                    self.types.insert(TypedId::SelfValue, ty.clone());
                }
            }
        }
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    let ty = self.allocator.next();
                    self.types.insert(TypedId::Instruction(instruction.id), ty.clone());
                    match &instruction.kind {
                        InstructionKind::DeclareVar(name) => {
                            self.types.insert(TypedId::Value(name.to_string()), self.allocator.next());
                        }
                        InstructionKind::Bind(name, _) => {
                            self.types.insert(TypedId::Value(name.to_string()), self.allocator.next());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn getType(&self, id: &TypedId) -> Type {
        self.types.get(id).expect("No type found!").clone()
    }

    fn getInstructionType(&self, id: InstructionId) -> Type {
        self.types.get(&TypedId::Instruction(id)).expect("not type for instruction").clone()
    }

    fn getValueType(&self, v: &String) -> Type {
        self.types.get(&TypedId::Value(v.clone())).expect("not type for value").clone()
    }

    fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = self.substitution.unify(&ty1, &ty2) {
            reportError(ty1, ty2, location);
        }
    }

    fn instantiateType(&mut self, ty: Type) -> Type {
        let vars = ty.collectVars(BTreeSet::new());
        let mut sub = Substitution::new();
        for var in &vars {
            sub.add(var.clone(), self.allocator.next());
        }
        sub.apply(&ty)
    }

    fn checkFunctionCall(&mut self, args: &Vec<InstructionId>, body: &Body, instruction: &Instruction, fnType: Type) {
        //println!("checkFunctionCall: {}", fnType);
        let fnType = self.instantiateType(fnType);
        let (fnArgs, fnResult) = match fnType.splitFnType() {
            Some((fnArgs, fnResult)) => (fnArgs, fnResult),
            None => return,
        };
        if args.len() != fnArgs.len() {
            TypecheckerError::ArgCountMismatch(fnArgs.len() as u32, args.len() as u32, instruction.location.clone()).report();
        }
        for (arg, fnArg) in zip(args, fnArgs) {
            self.unify(self.getInstructionType(*arg), fnArg, body.getInstruction(*arg).location.clone());
        }
        self.unify(self.getInstructionType(instruction.id), fnResult, instruction.location.clone());
    }

    fn check(&mut self, f: &Function) {
        let body = if let Some(body) = &f.body {
            body
        } else {
            return;
        };
        for instruction in f.instructions() {
            //println!("Type checking {}", instruction);
            match &instruction.kind {
                InstructionKind::FunctionCall(name, args) => {
                    let f = self.program.functions.get(name).expect("Function not found");
                    let fnType = f.getType();
                    self.checkFunctionCall(args, body, instruction, fnType);
                }
                InstructionKind::DynamicFunctionCall(callable, args) => match self.methodSources.get(callable) {
                    Some(name) => {
                        let f = self.program.functions.get(&name).expect("Function not found");
                        let fnType = f.getType();
                        let mut newArgs = Vec::new();
                        newArgs.push(*callable);
                        newArgs.extend(args);
                        self.checkFunctionCall(&newArgs, body, instruction, fnType);
                        self.methodCalls.insert(instruction.id, *callable);
                    }
                    None => {
                        let fnType = self.getInstructionType(*callable);
                        self.checkFunctionCall(&args, body, instruction, fnType);
                    }
                },
                InstructionKind::If(cond, _, _) => {
                    self.unify(
                        self.getInstructionType(*cond),
                        Type::getBoolType(),
                        body.getInstruction(*cond).location.clone(),
                    );
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::ValueRef(value, fields, _) => {
                    let mut receiverType = match &value {
                        ValueKind::Arg(name, _) => self.getValueType(name),
                        ValueKind::Value(name, _) => self.getValueType(name),
                    };
                    if fields.is_empty() {
                        self.unify(receiverType, self.getInstructionType(instruction.id), instruction.location.clone());
                    } else {
                        let mut indices = Vec::new();
                        for (index, field) in fields.iter().enumerate() {
                            let receiver = self.substitution.apply(&receiverType);
                            match &receiver {
                                Type::Named(name, _, _) => {
                                    // TODO
                                    if let Some(c) = self.program.classes.get(name) {
                                        let mut found = false;
                                        for (index, f) in c.fields.iter().enumerate() {
                                            if f.name == *field {
                                                indices.push(index as u32);
                                                found = true;
                                                receiverType = f.ty.clone();
                                                break;
                                            }
                                        }
                                        if !found && index == fields.len() - 1 {
                                            for m in &c.methods {
                                                if m.name == *field {
                                                    found = true;
                                                    self.methodSources.insert(instruction.id, m.fullName.clone());
                                                    break;
                                                }
                                            }
                                        }
                                        if !found {
                                            if let Some(methodName) = self.traitMethodSelector.get(&field) {
                                                found = true;
                                                self.methodSources.insert(instruction.id, methodName);
                                            }
                                        }
                                        if !found {
                                            TypecheckerError::FieldNotFound(field.clone(), instruction.location.clone()).report();
                                        }
                                    } else {
                                        TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report()
                                    }
                                }
                                _ => TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(),
                            }
                        }
                        self.indices.insert(instruction.id, indices);
                        self.unify(receiverType, self.getInstructionType(instruction.id), instruction.location.clone());
                    }
                }
                InstructionKind::Bind(name, rhs) => {
                    self.unify(self.getValueType(name), self.getInstructionType(*rhs), instruction.location.clone());
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::Tuple(args) => {
                    let mut argTypes = Vec::new();
                    for arg in args {
                        argTypes.push(self.getInstructionType(*arg));
                    }
                    self.unify(
                        self.getInstructionType(instruction.id),
                        Type::Tuple(argTypes),
                        instruction.location.clone(),
                    );
                }
                InstructionKind::TupleIndex(_, _) => todo!(),
                InstructionKind::StringLiteral(_) => {
                    self.unify(
                        self.getInstructionType(instruction.id),
                        Type::getStringType(),
                        instruction.location.clone(),
                    );
                }
                InstructionKind::IntegerLiteral(_) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getIntType(), instruction.location.clone());
                }
                InstructionKind::CharLiteral(_) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getCharType(), instruction.location.clone());
                }
                InstructionKind::Return(arg) => {
                    self.unify(f.result.clone(), self.getInstructionType(*arg), instruction.location.clone());
                    self.unify(self.getInstructionType(instruction.id), Type::Never, instruction.location.clone());
                }
                InstructionKind::Ref(arg) => {
                    let arg_type = self.getInstructionType(*arg);
                    self.unify(
                        self.getInstructionType(instruction.id),
                        Type::Reference(Box::new(arg_type), None),
                        instruction.location.clone(),
                    );
                }
                InstructionKind::Drop(_) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::Jump(_) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::Assign(name, rhs) => {
                    self.unify(self.getValueType(name), self.getInstructionType(*rhs), instruction.location.clone());
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::DeclareVar(_) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
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
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = self.substitution.apply(&ty);
                    let vars = ty.collectVars(BTreeSet::new());
                    if !vars.is_empty() && vars != publicVars {
                        self.dump(f);
                        println!("{} {}", instruction, ty);
                        TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report();
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
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = self.substitution.apply(&ty);
                    println!("{} : {}", instruction, ty);
                }
            }
        }
    }

    pub fn generate(&self, f: &Function) -> Function {
        //println!("Generating {}", f.name);
        let mut result = f.clone();
        if let Some(body) = &mut result.body {
            for block in &mut body.blocks {
                for instruction in &mut block.instructions {
                    if self.indices.contains_key(&instruction.id) {
                        match &instruction.kind {
                            InstructionKind::ValueRef(v, fields, _) => {
                                instruction.kind =
                                    InstructionKind::ValueRef(v.clone(), fields.clone(), self.indices.get(&instruction.id).unwrap().clone());
                            }
                            kind => panic!("Unexpected instruction kind for indices while rewriting! {}", kind.dump()),
                        }
                    }
                    if self.methodSources.contains_key(&instruction.id) {
                        match &instruction.kind {
                            InstructionKind::ValueRef(v, fields, indices) => {
                                let mut fields = fields.clone();
                                fields.pop();
                                instruction.kind = InstructionKind::ValueRef(v.clone(), fields, indices.clone());
                            }
                            kind => panic!("Unexpected instruction kind for method source while rewriting! {}", kind.dump()),
                        }
                    }
                    if let Some(source) = self.methodCalls.get(&instruction.id) {
                        match &instruction.kind {
                            InstructionKind::DynamicFunctionCall(_, args) => {
                                let name = self.methodSources.get(source).expect("Method not found for call!");
                                let mut newArgs = Vec::new();
                                newArgs.push(*source);
                                newArgs.extend(args);
                                instruction.kind = InstructionKind::FunctionCall(name.clone(), newArgs);
                            }
                            kind => panic!("Unexpected instruction kind for method call while rewriting! {}", kind.dump()),
                        }
                    }
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = self.substitution.apply(&ty);
                    //println!("{} : {}", instruction, ty);
                    instruction.ty = Some(ty);
                }
            }
        }
        result
    }
}
