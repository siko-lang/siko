use std::{
    collections::{BTreeMap, BTreeSet},
    iter::zip,
};

use crate::siko::{
    ir::{
        Data::{Class, Enum},
        Function::{Function, InstructionId, InstructionKind, Parameter, ValueKind},
        Type::Type,
    },
    qualifiedname::QualifiedName,
    util::error,
};

use super::{Substitution::Substitution, TypeVarAllocator::TypeVarAllocator};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TypedId {
    Instruction(InstructionId),
    Value(String),
    SelfValue,
}

#[derive(Clone)]
enum Constraint {
    FieldConstraint(Type, String, Type),
}

pub struct Typechecker<'a> {
    functions: &'a BTreeMap<QualifiedName, Function>,
    classes: &'a BTreeMap<QualifiedName, Class>,
    enums: &'a BTreeMap<QualifiedName, Enum>,
    allocator: TypeVarAllocator,
    substitution: Substitution,
    constraints: Vec<Constraint>,
    receivers: BTreeMap<InstructionId, Type>,
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
            constraints: Vec::new(),
            receivers: BTreeMap::new(),
            types: BTreeMap::new(),
        }
    }

    pub fn run(&mut self, f: &Function) {
        self.initialize(f);
        self.check(f);
        self.verify(f);
        //self.dump(f);
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
                        InstructionKind::ValueRef(_, fields) => {
                            let mut receiver = self.allocator.next();
                            self.receivers.insert(instruction.id, receiver.clone());
                            for (index, field) in fields.iter().enumerate() {
                                let fieldType = if index == fields.len() - 1 {
                                    ty.clone()
                                } else {
                                    self.allocator.next()
                                };
                                self.constraints.push(Constraint::FieldConstraint(
                                    receiver,
                                    field.clone(),
                                    fieldType.clone(),
                                ));
                                receiver = fieldType;
                            }
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

    fn unify(&mut self, ty1: Type, ty2: Type) {
        self.substitution.unify(&ty1, &ty2);
    }

    fn instantiateType(&mut self, ty: Type) -> Type {
        let vars = ty.collectVars(BTreeSet::new());
        let mut sub = Substitution::new();
        for var in &vars {
            sub.add(var.clone(), self.allocator.next());
        }
        sub.apply(&ty)
    }

    pub fn check(&mut self, f: &Function) {
        //println!("Typechecking {}", f.name);
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    //println!("Type checking {}", instruction);
                    match &instruction.kind {
                        InstructionKind::FunctionCall(name, args) => {
                            let f = self.functions.get(name).expect("Function not found");
                            let fnType = f.getType();
                            let fnType = self.instantiateType(fnType);
                            let (fnArgs, fnResult) = fnType.splitFnType();
                            if args.len() != fnArgs.len() {
                                error(format!("incorrect args"));
                            }
                            for (arg, fnArg) in zip(args, fnArgs) {
                                self.substitution
                                    .unify(&self.getInstructionType(*arg), &fnArg);
                            }
                            self.substitution
                                .unify(&self.getInstructionType(instruction.id), &fnResult);
                        }
                        InstructionKind::DynamicFunctionCall(_, _) => {}
                        InstructionKind::If(cond, t, f) => {
                            self.substitution
                                .unify(&self.getInstructionType(*cond), &Type::getBoolType());
                            match f {
                                Some(f) => {
                                    self.substitution.unify(
                                        &self.getInstructionType(*t),
                                        &self.getInstructionType(*f),
                                    );
                                }
                                None => {
                                    self.substitution
                                        .unify(&self.getInstructionType(*t), &Type::getUnitType());
                                }
                            }
                            self.substitution.unify(
                                &self.getInstructionType(*t),
                                &self.getInstructionType(instruction.id),
                            );
                        }
                        InstructionKind::BlockRef(id) => {
                            let block = &body.blocks[id.id as usize];
                            let last = block
                                .instructions
                                .last()
                                .expect("Empty block in type check!");
                            self.substitution.unify(
                                &self.getInstructionType(last.id),
                                &self.getInstructionType(instruction.id),
                            );
                        }
                        InstructionKind::ValueRef(value, fields) => {
                            let receiverType = match &value {
                                ValueKind::Arg(name) => self.getValueType(name),
                                ValueKind::Implicit(id) => self.getInstructionType(*id),
                                ValueKind::Value(name, _) => self.getValueType(name),
                            };
                            if fields.is_empty() {
                                self.unify(receiverType, self.getInstructionType(instruction.id));
                            } else {
                                self.unify(
                                    receiverType,
                                    self.receivers
                                        .get(&instruction.id)
                                        .cloned()
                                        .expect("no receiver found"),
                                );
                            }
                        }
                        InstructionKind::Bind(name, rhs) => {
                            self.unify(self.getValueType(name), self.getInstructionType(*rhs));
                            self.substitution.unify(
                                &self.getInstructionType(instruction.id),
                                &Type::getUnitType(),
                            );
                        }
                        InstructionKind::Tuple(_) => todo!(),
                        InstructionKind::TupleIndex(_, _) => todo!(),
                        InstructionKind::StringLiteral(_) => todo!(),
                        InstructionKind::IntegerLiteral(_) => todo!(),
                        InstructionKind::CharLiteral(_) => todo!(),
                    }
                }
            }
        }
        for constraint in self.constraints.clone() {
            match constraint {
                Constraint::FieldConstraint(receiver, field, fieldType) => {
                    let receiver = self.substitution.apply(&receiver);
                    match &receiver {
                        Type::Named(name, _) => {
                            // TODO
                            if let Some(c) = self.classes.get(name) {
                                let mut found = false;
                                for f in &c.fields {
                                    if f.name == *field {
                                        found = true;
                                        self.unify(fieldType.clone(), f.ty.clone());
                                        break;
                                    }
                                }
                                if !found {
                                    for m in &c.methods {
                                        if m.name == *field {
                                            found = true;
                                            println!("Calling method??");
                                            break;
                                        }
                                    }
                                }
                                if !found {
                                    error(format!("field '{}' not found", field))
                                }
                            } else {
                                error(format!("field receiver is not a class!"))
                            }
                        } //ty => error(format!("field receiver is not a class! {}", ty)),
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn verify(&self, f: &Function) {
        if let Some(body) = &f.body {
            let fnType = f.getType();
            let publicVars = fnType.collectVars(BTreeSet::new());
            let mut vars = BTreeSet::new();
            for block in &body.blocks {
                for instruction in &block.instructions {
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = self.substitution.apply(&ty);
                    vars = ty.collectVars(vars);
                }
            }
            if vars != publicVars {
                self.dump(f);
                error(format!("type check/inference failed for {}", f.name));
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
}
