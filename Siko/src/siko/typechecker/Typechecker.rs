use std::{collections::BTreeMap, f32::consts::E, iter::zip};

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

    fn unify(&mut self, ty1: Type, ty2: Type) {
        self.substitution.unify(&ty1, &ty2);
    }

    pub fn check(&mut self, f: &Function) {
        //println!("Typechecking {}", f.name);
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    //println!("Type checking {}", instruction);
                    match &instruction.kind {
                        InstructionKind::FunctionCall(name, args) => {
                            println!("Checking fn {}", name);
                            let f = self.functions.get(name).expect("Function not found");
                            let fnType = f.getType();
                            println!("fn type {}", fnType);
                            let (fnArgs, fnResult) = fnType.splitFnType();
                            for (arg, fnArg) in zip(args, fnArgs) {
                                self.substitution
                                    .unify(&self.getInstructionType(*arg), &fnArg);
                            }
                            self.substitution
                                .unify(&self.getInstructionType(instruction.id), &fnResult);
                        }
                        InstructionKind::DynamicFunctionCall(_, _) => {}
                        InstructionKind::If(_, _, _) => {}
                        InstructionKind::BlockRef(_) => {}
                        InstructionKind::ValueRef(value, fields) => match &value {
                            ValueKind::Arg(name) => {
                                let ty1 = self.getType(&TypedId::Value(name.clone()));
                                let ty2 = self.getType(&&TypedId::Instruction(instruction.id));
                                self.unify(ty1, ty2);
                            }
                            ValueKind::Implicit(id) => {}
                            ValueKind::Value(name, bindId) => {}
                        },
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
