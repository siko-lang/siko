use std::{collections::BTreeMap, iter::zip};

use crate::siko::{
    ir::{
        Data::{Class, Enum},
        Function::{Function, InstructionId, InstructionKind, Parameter, ValueKind},
        Type::Type,
    },
    qualifiedname::QualifiedName,
};

use super::{Substitution::Substitution, TypeVarAllocator::TypeVarAllocator};

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
            types: BTreeMap::new(),
        }
    }

    pub fn run(&mut self, f: &Function) {
        self.initialize(f);
        self.check(f);
        self.dump(f);
    }

    pub fn initialize(&mut self, f: &Function) {
        //println!("Initializing {}", f.name);
        for param in &f.params {
            match &param {
                Parameter::Named(name, ty, mutable) => {
                    self.types.insert(TypedId::Value(name.clone()), ty.clone());
                }
                Parameter::SelfParam(mutable, ty) => {
                    self.types.insert(TypedId::SelfValue, ty.clone());
                }
            }
        }
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    self.types
                        .insert(TypedId::Instruction(instruction.id), self.allocator.next());
                    match &instruction.kind {
                        InstructionKind::Bind(name, _) => {
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

    fn unify(&mut self, ty1: Type, ty2: Type) {
        self.substitution.unify(&ty1, &ty2);
    }

    fn instantiateType(&mut self, ty: Type) -> Type {
        let mut vars = ty.collectVars(Vec::new());
        vars.sort();
        vars.dedup();
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
                            assert!(fields.is_empty());
                            match &value {
                                ValueKind::Arg(name) => {
                                    self.unify(
                                        self.getValueType(name),
                                        self.getInstructionType(instruction.id),
                                    );
                                }
                                ValueKind::Implicit(id) => self.unify(
                                    self.getInstructionType(*id),
                                    self.getInstructionType(instruction.id),
                                ),
                                ValueKind::Value(name, bindId) => self.unify(
                                    self.getValueType(name),
                                    self.getInstructionType(instruction.id),
                                ),
                            }
                        }
                        InstructionKind::Bind(_, _) => {}
                        InstructionKind::Tuple(_) => todo!(),
                        InstructionKind::TupleIndex(_, _) => todo!(),
                        InstructionKind::StringLiteral(_) => todo!(),
                        InstructionKind::IntegerLiteral(_) => todo!(),
                        InstructionKind::CharLiteral(_) => todo!(),
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
}
