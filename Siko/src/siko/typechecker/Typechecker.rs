use std::{
    collections::{BTreeMap, BTreeSet},
    iter::zip,
};

use crate::siko::{
    ir::{
        Data::{Class, Enum},
        Function::{
            Body, Function, Instruction, InstructionId, InstructionKind, Parameter, ValueKind,
        },
        Type::Type,
    },
    location::Location::Location,
    qualifiedname::QualifiedName,
};

use super::{
    Error::TypecheckerError, Substitution::Substitution, TypeVarAllocator::TypeVarAllocator,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TypedId {
    Instruction(InstructionId),
    Value(String),
    SelfValue,
}

pub struct Typechecker<'a> {
    functions: &'a BTreeMap<QualifiedName, Function>,
    classes: &'a BTreeMap<QualifiedName, Class>,
    enums: &'a BTreeMap<QualifiedName, Enum>,
    allocator: TypeVarAllocator,
    substitution: Substitution,
    methodSources: BTreeMap<InstructionId, QualifiedName>,
    methodCalls: BTreeMap<InstructionId, InstructionId>,
    types: BTreeMap<TypedId, Type>,
}

impl<'a> Typechecker<'a> {
    pub fn new(
        functions: &'a BTreeMap<QualifiedName, Function>,
        classes: &'a BTreeMap<QualifiedName, Class>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> Typechecker<'a> {
        Typechecker {
            functions: functions,
            classes: classes,
            enums: enums,
            allocator: TypeVarAllocator::new(),
            substitution: Substitution::new(),
            methodSources: BTreeMap::new(),
            methodCalls: BTreeMap::new(),
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
                    self.types
                        .insert(TypedId::Instruction(instruction.id), ty.clone());
                    match &instruction.kind {
                        InstructionKind::Bind(name, _) => {
                            self.types
                                .insert(TypedId::Value(name.to_string()), self.allocator.next());
                        }
                        InstructionKind::Loop(name, _, _) => {
                            self.types
                                .insert(TypedId::Value(name.to_string()), self.allocator.next());
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
        self.types
            .get(&TypedId::Instruction(id))
            .expect("not type for instruction")
            .clone()
    }

    fn getValueType(&self, v: &String) -> Type {
        self.types
            .get(&TypedId::Value(v.clone()))
            .expect("not type for value")
            .clone()
    }

    fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        self.substitution.unify(&ty1, &ty2, location);
    }

    fn instantiateType(&mut self, ty: Type) -> Type {
        let vars = ty.collectVars(BTreeSet::new());
        let mut sub = Substitution::new();
        for var in &vars {
            sub.add(var.clone(), self.allocator.next());
        }
        sub.apply(&ty)
    }

    fn checkFunctionCall(
        &mut self,
        args: &Vec<InstructionId>,
        body: &Body,
        instruction: &Instruction,
        fnType: Type,
    ) {
        let fnType = self.instantiateType(fnType);
        let (fnArgs, fnResult) = fnType.splitFnType();
        if args.len() != fnArgs.len() {
            TypecheckerError::ArgCountMismatch(
                fnArgs.len() as u32,
                args.len() as u32,
                instruction.location.clone(),
            )
            .report();
        }
        for (arg, fnArg) in zip(args, fnArgs) {
            self.substitution.unify(
                &self.getInstructionType(*arg),
                &fnArg,
                body.getInstruction(*arg).location.clone(),
            );
        }
        self.substitution.unify(
            &self.getInstructionType(instruction.id),
            &fnResult,
            instruction.location.clone(),
        );
    }

    pub fn check(&mut self, f: &Function) {
        //println!("Typechecking {}", f.name);
        let body = if let Some(body) = &f.body {
            body
        } else {
            return;
        };
        for block in &body.blocks {
            for instruction in &block.instructions {
                //println!("Type checking {}", instruction);
                match &instruction.kind {
                    InstructionKind::FunctionCall(name, args) => {
                        let f = self.functions.get(name).expect("Function not found");
                        let fnType = f.getType();
                        self.checkFunctionCall(args, body, instruction, fnType);
                    }
                    InstructionKind::DynamicFunctionCall(callable, args) => {
                        match self.methodSources.get(callable) {
                            Some(name) => {
                                let f = self.functions.get(&name).expect("Function not found");
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
                        }
                    }
                    InstructionKind::If(cond, t, f) => {
                        let trueLast = &body.getBlockById(*t).getLastId();
                        self.substitution.unify(
                            &self.getInstructionType(*cond),
                            &Type::getBoolType(),
                            body.getInstruction(*cond).location.clone(),
                        );
                        match f {
                            Some(f) => {
                                let falseLast = &body.getBlockById(*f).getLastId();
                                self.substitution.unify(
                                    &self.getInstructionType(*trueLast),
                                    &self.getInstructionType(*falseLast),
                                    instruction.location.clone(),
                                );
                            }
                            None => {
                                self.substitution.unify(
                                    &self.getInstructionType(*trueLast),
                                    &Type::getUnitType(),
                                    instruction.location.clone(),
                                );
                            }
                        }
                        self.substitution.unify(
                            &self.getInstructionType(*trueLast),
                            &self.getInstructionType(instruction.id),
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::BlockRef(id) => {
                        let last = &body.getBlockById(*id).getLastId();
                        self.substitution.unify(
                            &self.getInstructionType(*last),
                            &self.getInstructionType(instruction.id),
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::ValueRef(value, fields) => {
                        let mut receiverType = match &value {
                            ValueKind::Arg(name) => self.getValueType(name),
                            ValueKind::Implicit(id) => self.getInstructionType(*id),
                            ValueKind::LoopVar(name) => self.getValueType(name),
                            ValueKind::Value(name, _) => self.getValueType(name),
                        };
                        if fields.is_empty() {
                            self.unify(
                                receiverType,
                                self.getInstructionType(instruction.id),
                                instruction.location.clone(),
                            );
                        } else {
                            for (index, field) in fields.iter().enumerate() {
                                let receiver = self.substitution.apply(&receiverType);
                                match &receiver {
                                    Type::Named(name, _) => {
                                        // TODO
                                        if let Some(c) = self.classes.get(name) {
                                            let mut found = false;
                                            for f in &c.fields {
                                                if f.name == *field {
                                                    found = true;
                                                    receiverType = f.ty.clone();
                                                    break;
                                                }
                                            }
                                            if !found && index == fields.len() - 1 {
                                                for m in &c.methods {
                                                    if m.name == *field {
                                                        found = true;
                                                        self.methodSources.insert(
                                                            instruction.id,
                                                            m.fullName.clone(),
                                                        );
                                                        break;
                                                    }
                                                }
                                            }
                                            if !found {
                                                TypecheckerError::FieldNotFound(
                                                    field.clone(),
                                                    instruction.location.clone(),
                                                )
                                                .report();
                                            }
                                        } else {
                                            TypecheckerError::TypeAnnotationNeeded(
                                                instruction.location.clone(),
                                            )
                                            .report()
                                        }
                                    }
                                    _ => TypecheckerError::TypeAnnotationNeeded(
                                        instruction.location.clone(),
                                    )
                                    .report(),
                                }
                            }
                            self.unify(
                                receiverType,
                                self.getInstructionType(instruction.id),
                                instruction.location.clone(),
                            );
                        }
                    }
                    InstructionKind::Bind(name, rhs) => {
                        self.unify(
                            self.getValueType(name),
                            self.getInstructionType(*rhs),
                            instruction.location.clone(),
                        );
                        self.substitution.unify(
                            &self.getInstructionType(instruction.id),
                            &Type::getUnitType(),
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::Tuple(args) => {
                        let mut argTypes = Vec::new();
                        for arg in args {
                            argTypes.push(self.getInstructionType(*arg));
                        }
                        self.substitution.unify(
                            &self.getInstructionType(instruction.id),
                            &Type::Tuple(argTypes),
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::TupleIndex(_, _) => todo!(),
                    InstructionKind::StringLiteral(_) => {
                        self.substitution.unify(
                            &self.getInstructionType(instruction.id),
                            &Type::getStringType(),
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::IntegerLiteral(_) => {
                        self.substitution.unify(
                            &self.getInstructionType(instruction.id),
                            &Type::getIntType(),
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::CharLiteral(_) => {
                        self.substitution.unify(
                            &self.getInstructionType(instruction.id),
                            &Type::getCharType(),
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::Loop(name, init, loopBody) => {
                        self.substitution.unify(
                            &self.getInstructionType(*init),
                            &self.getValueType(name),
                            instruction.location.clone(),
                        );
                        self.substitution.unify(
                            &self.getInstructionType(*init),
                            &self.getInstructionType(body.getBlockById(*loopBody).getLastId()),
                            instruction.location.clone(),
                        );
                        self.substitution.unify(
                            &self.getInstructionType(instruction.id),
                            &Type::Never,
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::Continue(arg, loopId) => {
                        let loopInstruction = body.getInstruction(*loopId);
                        match &loopInstruction.kind {
                            InstructionKind::Loop(_, init, _) => {
                                self.substitution.unify(
                                    &self.getInstructionType(*init),
                                    &self.getInstructionType(*arg),
                                    instruction.location.clone(),
                                );
                            }
                            _ => panic!("Loop instruction is not a loop!"),
                        }
                        self.substitution.unify(
                            &self.getInstructionType(instruction.id),
                            &Type::Never,
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::Break(arg, loopId) => {
                        self.substitution.unify(
                            &self.getInstructionType(*loopId),
                            &self.getInstructionType(*arg),
                            instruction.location.clone(),
                        );
                        self.substitution.unify(
                            &self.getInstructionType(instruction.id),
                            &Type::Never,
                            instruction.location.clone(),
                        );
                    }
                    InstructionKind::Return(arg) => {
                        self.substitution.unify(
                            &f.result,
                            &self.getInstructionType(*arg),
                            instruction.location.clone(),
                        );
                        self.substitution.unify(
                            &self.getInstructionType(instruction.id),
                            &Type::Never,
                            instruction.location.clone(),
                        );
                    }
                }
            }
        }
        let block = &body.blocks[0];
        let last = block
            .instructions
            .last()
            .expect("Empty block in type check!");
        self.substitution.unify(
            &self.getInstructionType(last.id),
            &f.result,
            last.location.clone(),
        );
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
                    if vars != publicVars {
                        TypecheckerError::TypeAnnotationNeeded(instruction.location.clone())
                            .report();
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
                    if self.methodSources.contains_key(&instruction.id) {
                        match &instruction.kind {
                            InstructionKind::ValueRef(v, fields) => {
                                let mut fields = fields.clone();
                                fields.pop();
                                instruction.kind = InstructionKind::ValueRef(v.clone(), fields);
                            }
                            kind => panic!(
                                "Unexpected instruction kind for method source while rewriting! {}",
                                kind.dump()
                            ),
                        }
                    }
                    if let Some(source) = self.methodCalls.get(&instruction.id) {
                        match &instruction.kind {
                            InstructionKind::DynamicFunctionCall(_, args) => {
                                let name = self
                                    .methodSources
                                    .get(source)
                                    .expect("Method not found for call!");
                                let mut newArgs = Vec::new();
                                newArgs.push(*source);
                                newArgs.extend(args);
                                instruction.kind =
                                    InstructionKind::FunctionCall(name.clone(), newArgs);
                            }
                            kind => panic!(
                                "Unexpected instruction kind for method call while rewriting! {}",
                                kind.dump()
                            ),
                        }
                    }
                    // let ty = self.getType(&TypedId::Instruction(instruction.id));
                    // let ty = self.substitution.apply(&ty);
                    // println!("{} : {}", instruction, ty);
                }
            }
        }
        result
    }
}
