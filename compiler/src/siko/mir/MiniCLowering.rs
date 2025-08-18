use core::panic;
use std::collections::{btree_map::Entry, BTreeMap};

use super::{
    Data::Struct,
    Function::{Block, Function, FunctionKind, Instruction, Param, Value, Variable},
    Program::Program,
    Type::Type,
};

use crate::siko::minic::Function::Branch as LBranch;
use crate::siko::minic::Function::Function as LFunction;
use crate::siko::minic::Function::Instruction as LInstruction;
use crate::siko::minic::Function::Param as LParam;
use crate::siko::minic::Function::Value as LValue;
use crate::siko::minic::Function::Variable as LVariable;
use crate::siko::minic::Function::{Block as LBlock, GetMode};
use crate::siko::minic::Program::Program as LProgram;
use crate::siko::minic::Type::Type as LType;
use crate::siko::minic::{Constant::StringConstant, Data::Field as LField};
use crate::siko::{minic::Data::Struct as LStruct, mir::Function::ExternKind};

pub struct MinicBuilder<'a> {
    program: &'a Program,
    constants: BTreeMap<String, String>,
    refMap: BTreeMap<String, String>,
    nextTmp: u32,
}

impl<'a> MinicBuilder<'a> {
    pub fn new(program: &'a Program) -> MinicBuilder<'a> {
        MinicBuilder {
            program: program,
            constants: BTreeMap::new(),
            refMap: BTreeMap::new(),
            nextTmp: 0,
        }
    }

    fn resolveVar(&self, v: &String) -> String {
        match self.refMap.get(v) {
            Some(name) => self.resolveVar(name),
            None => v.clone(),
        }
    }

    fn lowerVar(&self, v: &Variable) -> LVariable {
        let name = self.resolveVar(&v.name);
        LVariable {
            name: format!("{}", name),
            ty: self.lowerType(&v.ty),
        }
    }

    fn tmpVar(&mut self, v: &Variable) -> LVariable {
        self.nextTmp += 1;
        LVariable {
            name: format!("tmp_{}_{}", v.name, self.nextTmp),
            ty: self.lowerType(&v.ty),
        }
    }

    pub fn lower(&mut self) -> LProgram {
        //println!("Before lowering {}", self.program);

        let mut program = LProgram::new();

        for (_, s) in &self.program.structs {
            program.structs.insert(s.name.clone(), self.lowerStruct(s));
        }

        for (_, f) in &self.program.functions {
            let f = self.lowerFunction(f);
            program.functions.push(f);
        }

        for (key, value) in &self.constants {
            program.strings.push(StringConstant {
                name: value.clone(),
                value: key.clone(),
            });
        }

        program
    }

    fn lowerParam(&self, p: &Param) -> LParam {
        LParam {
            name: p.name.clone(),
            ty: self.lowerType(&p.ty),
        }
    }

    fn lowerBlock(&mut self, block: &Block) -> LBlock {
        let mut minicBlock = LBlock {
            id: block.id.clone(),
            instructions: Vec::new(),
        };
        for instruction in &block.instructions {
            match instruction {
                Instruction::Declare(_) => {
                    // declares are processed at the beginning
                }
                Instruction::Reference(dest, src) => {
                    let minicInstruction = LInstruction::Reference(self.lowerVar(dest), self.lowerVar(&src));
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::Call(dest, name, args) => {
                    let f = self.program.functions.get(name).expect("Function not found");
                    let name = if let FunctionKind::Extern(ExternKind::C(name)) = &f.kind {
                        name.clone()
                    } else {
                        name.clone()
                    };
                    let mut minicArgs = Vec::new();
                    for arg in args {
                        minicArgs.push(self.lowerVar(arg));
                    }
                    let minicDest = if let Some(dest) = dest {
                        Some(self.lowerVar(dest))
                    } else {
                        None
                    };
                    let minicInstruction = LInstruction::FunctionCall(minicDest, name.clone(), minicArgs);
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::Assign(dest, src) => {
                    let minicInstruction = LInstruction::Store(
                        self.lowerVar(dest),
                        match src {
                            Value::Void => unreachable!(),
                            Value::Numeric(v) => LValue::Numeric(v.clone(), LType::Int64),
                            Value::Var(v) => LValue::Variable(self.lowerVar(v)),
                        },
                    );
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::Return(v) => match v {
                    Value::Void => {
                        let minicInstruction = LInstruction::Return(LValue::Void);
                        minicBlock.instructions.push(minicInstruction);
                    }
                    Value::Var(v) => {
                        let minicInstruction = LInstruction::Return(LValue::Variable(self.lowerVar(v)));
                        minicBlock.instructions.push(minicInstruction);
                    }
                    _ => {
                        unreachable!()
                    }
                },
                Instruction::Store(src, dest) => {
                    let minicInstruction =
                        LInstruction::Store(self.lowerVar(dest), LValue::Variable(self.lowerVar(src)));
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::Memcpy(src, dest) => {
                    if src.ty.isPtr() {
                        let minicInstruction = LInstruction::MemcpyPtr(self.lowerVar(src), self.lowerVar(dest));
                        minicBlock.instructions.push(minicInstruction);
                    } else {
                        let minicInstruction = LInstruction::Memcpy(self.lowerVar(src), self.lowerVar(dest));
                        minicBlock.instructions.push(minicInstruction);
                    }
                }
                Instruction::GetFieldRef(dest, root, index) => {
                    let field = &self.program.getStruct(&root.ty.getStruct()).fields[*index as usize];
                    let mode = if dest.ty.isPtr() {
                        let inner = dest.ty.getPtrInner();
                        if inner == field.ty {
                            GetMode::Ref
                        } else {
                            GetMode::Noop
                        }
                    } else {
                        GetMode::Noop
                    };
                    let minicInstruction =
                        LInstruction::GetField(self.lowerVar(dest), self.lowerVar(root), *index, mode);
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::SetField(dest, root, indices) => {
                    let minicInstruction =
                        LInstruction::SetField(self.lowerVar(dest), self.lowerVar(root), indices.clone());
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::IntegerLiteral(var, value) => {
                    let minicInstruction =
                        LInstruction::Store(self.lowerVar(var), LValue::Numeric(value.clone(), LType::Int64));
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::StringLiteral(var, value) => {
                    let i8Ptr = LType::Ptr(Box::new(LType::UInt8));
                    let mut tmpVar = self.tmpVar(var);
                    tmpVar.ty = i8Ptr.clone();
                    let mut tmpVar2 = self.tmpVar(var);
                    tmpVar2.ty = LType::Int64;
                    let new = self.constants.len();
                    let strLen = value.len();
                    let value = match self.constants.entry(value.clone()) {
                        Entry::Occupied(v) => v.get().clone(),
                        Entry::Vacant(v) => {
                            let newStr = format!("_siko_literal_str_{}", new);
                            v.insert(newStr.clone());
                            newStr
                        }
                    };
                    let minicInstruction = LInstruction::Store(tmpVar.clone(), LValue::String(value.clone(), i8Ptr));
                    minicBlock.instructions.push(minicInstruction);
                    let minicInstruction =
                        LInstruction::Store(tmpVar2.clone(), LValue::Numeric(format!("{}", strLen), LType::Int64));
                    minicBlock.instructions.push(minicInstruction);
                    let minicInstruction = LInstruction::SetField(self.lowerVar(var), tmpVar, vec![0]);
                    minicBlock.instructions.push(minicInstruction);
                    let minicInstruction = LInstruction::SetField(self.lowerVar(var), tmpVar2, vec![1]);
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::Jump(name) => {
                    let minicInstruction = LInstruction::Jump(name.clone());
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::EnumSwitch(var, cases) => {
                    let switchVar = Variable {
                        name: format!("switch_var_{}", block.id),
                        ty: Type::Int32,
                    };
                    let tmpVar = self.tmpVar(&switchVar);
                    let minicInstruction = LInstruction::GetField(tmpVar.clone(), self.lowerVar(var), 0, GetMode::Noop);
                    minicBlock.instructions.push(minicInstruction);
                    let mut branches = Vec::new();
                    for (index, case) in cases.iter().enumerate() {
                        if index == 0 {
                            continue;
                        }
                        let branch = LBranch {
                            value: LValue::Numeric(format!("{}", case.index), LType::Int32),
                            block: case.branch.clone(),
                        };
                        branches.push(branch);
                    }
                    let minicInstruction = LInstruction::Switch(tmpVar.clone(), cases[0].branch.clone(), branches);
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::IntegerSwitch(var, cases) => {
                    let mut branches = Vec::new();
                    let mut defaultIndex = 0;
                    for (index, case) in cases.iter().enumerate() {
                        match &case.value {
                            Some(v) => {
                                let branch = LBranch {
                                    value: LValue::Numeric(v.clone(), LType::Int64),
                                    block: case.branch.clone(),
                                };
                                branches.push(branch);
                            }
                            None => {
                                defaultIndex = index;
                            }
                        }
                    }
                    let minicInstruction =
                        LInstruction::Switch(self.lowerVar(var), cases[defaultIndex].branch.clone(), branches);
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::Transform(dest, src, index) => {
                    let u = self.program.getUnion(&src.ty.getUnion());
                    let v = &u.variants[*index as usize];
                    //println!("{} {} {} {}", dest.ty, src.ty, index, v.ty);
                    let mut recastVar = dest.clone();
                    recastVar.ty = Type::Struct(v.name.clone());
                    if dest.ty.isPtr() {
                        recastVar.ty = Type::Ptr(Box::new(recastVar.ty));
                    }
                    let recastVar = self.tmpVar(&recastVar);
                    let minicInstruction = LInstruction::Bitcast(recastVar.clone(), self.lowerVar(src));
                    minicBlock.instructions.push(minicInstruction);
                    let mode = if dest.ty.isPtr() { GetMode::Ref } else { GetMode::Noop };
                    let minicInstruction = LInstruction::GetField(self.lowerVar(dest), recastVar, 1, mode);
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::AddressOfField(dest, src, index) => {
                    let minicInstruction =
                        LInstruction::AddressOfField(self.lowerVar(dest), self.lowerVar(src), *index);
                    minicBlock.instructions.push(minicInstruction);
                }
                Instruction::LoadPtr(dest, src) => {
                    let minicInstruction = LInstruction::LoadPtr(self.lowerVar(dest), self.lowerVar(src));
                    minicBlock.instructions.push(minicInstruction);
                }
            };
        }
        minicBlock
    }

    fn lowerFunction(&mut self, f: &Function) -> LFunction {
        let args: Vec<_> = f.args.iter().map(|p| self.lowerParam(p)).collect();
        let resultTy = self.lowerType(&f.result);
        match &f.kind {
            FunctionKind::UserDefined(blocks) => {
                let mut localVars = Vec::new();
                for block in blocks {
                    for instruction in &block.instructions {
                        if let Instruction::Declare(var) = instruction {
                            localVars.push(var.clone());
                        }
                    }
                }
                let mut minicBlocks: Vec<LBlock> = blocks.iter().map(|b| self.lowerBlock(b)).collect();
                for var in localVars {
                    let minicInstruction = LInstruction::Allocate(self.lowerVar(&var));
                    minicBlocks[0].instructions.insert(0, minicInstruction)
                }
                LFunction {
                    name: f.name.clone(),
                    fullName: f.fullName.clone(),
                    args: args,
                    result: resultTy,
                    blocks: minicBlocks,
                    isBuiltin: false,
                }
            }
            FunctionKind::StructCtor => {
                let mut block = LBlock {
                    id: format!("block0"),
                    instructions: Vec::new(),
                };
                let this = Variable {
                    name: "this".to_string(),
                    ty: f.result.clone(),
                };
                let s = self.program.getStruct(&f.name);
                block.instructions.push(LInstruction::Allocate(self.lowerVar(&this)));
                for (index, field) in s.fields.iter().enumerate() {
                    let argVar = Variable {
                        name: field.name.clone(),
                        ty: Type::Ptr(Box::new(field.ty.clone())),
                    };
                    if field.ty.isPtr() {
                        block.instructions.push(LInstruction::SetField(
                            self.lowerVar(&this),
                            self.lowerVar(&argVar),
                            vec![index as i32],
                        ));
                    } else {
                        block.instructions.push(LInstruction::SetField(
                            self.lowerVar(&this),
                            self.lowerVar(&argVar),
                            vec![index as i32],
                        ));
                    }
                }
                block
                    .instructions
                    .push(LInstruction::Return(LValue::Variable(self.lowerVar(&this))));
                LFunction {
                    name: f.name.clone(),
                    fullName: f.fullName.clone(),
                    args: args,
                    result: resultTy,
                    blocks: vec![block],
                    isBuiltin: false,
                }
            }
            FunctionKind::VariantCtor(index) => {
                //println!("MIR: building variant ctor {}", f.name);
                let mut block = LBlock {
                    id: format!("block0"),
                    instructions: Vec::new(),
                };
                let u = if let Type::Union(u) = &f.result {
                    self.program.getUnion(u)
                } else {
                    panic!("Expected union type, found {}", f.result)
                };
                let variant = &u.variants[*index as usize];
                let s = self.program.getStruct(&variant.ty.getStruct());
                let this = Variable {
                    name: "this".to_string(),
                    ty: Type::Struct(variant.name.clone()),
                };
                block.instructions.push(LInstruction::Allocate(self.lowerVar(&this)));
                let mut tmp = self.tmpVar(&this);
                tmp.ty = LType::Int32;
                block.instructions.push(LInstruction::Store(
                    tmp.clone(),
                    LValue::Numeric(format!("{}", index), LType::Int32),
                ));
                block
                    .instructions
                    .push(LInstruction::SetField(self.lowerVar(&this), tmp, vec![0]));
                for (index, field) in s.fields.iter().enumerate() {
                    let argVar = Variable {
                        name: field.name.clone(),
                        ty: Type::Ptr(Box::new(field.ty.clone())),
                    };
                    block.instructions.push(LInstruction::SetField(
                        self.lowerVar(&this),
                        self.lowerVar(&argVar),
                        vec![1, index as i32],
                    ));
                }
                let mut tmp2 = self.tmpVar(&this);
                tmp2.ty = resultTy.clone();
                block
                    .instructions
                    .push(LInstruction::Bitcast(tmp2.clone(), self.lowerVar(&this)));
                block.instructions.push(LInstruction::Return(LValue::Variable(tmp2)));
                LFunction {
                    name: f.name.clone(),
                    fullName: f.fullName.clone(),
                    args: args,
                    result: resultTy,
                    blocks: vec![block],
                    isBuiltin: false,
                }
            }
            FunctionKind::Extern(kind) => {
                let name = if let ExternKind::C(name) = kind {
                    name.clone()
                } else {
                    f.name.clone()
                };
                LFunction {
                    name: name,
                    fullName: f.fullName.clone(),
                    args: args,
                    result: resultTy,
                    blocks: Vec::new(),
                    isBuiltin: match kind {
                        ExternKind::C(_) => false,
                        ExternKind::Builtin => true,
                    },
                }
            }
        }
    }

    fn lowerStruct(&self, s: &Struct) -> LStruct {
        let mut fields = Vec::new();
        for f in &s.fields {
            let minicField = LField {
                name: f.name.clone(),
                ty: self.lowerType(&f.ty),
            };
            fields.push(minicField);
        }
        let minicStruct = LStruct {
            name: s.name.clone(),
            fields: fields,
            size: s.size,
            alignment: s.alignment,
        };
        minicStruct
    }

    fn lowerType(&self, ty: &Type) -> LType {
        match ty {
            Type::Void => LType::Void,
            Type::UInt8 => LType::UInt8,
            Type::UInt32 => LType::UInt32,
            Type::UInt64 => LType::UInt64,
            Type::Int16 => LType::Int16,
            Type::Int32 => LType::Int32,
            Type::Int64 => LType::Int64,
            Type::Char => LType::UInt8,
            Type::Struct(n) => LType::Struct(n.clone()),
            Type::Union(n) => LType::Struct(n.clone()),
            Type::Ptr(t) => LType::Ptr(Box::new(self.lowerType(t))),
            Type::Array(s, itemSize) => LType::Array(*s, *itemSize),
        }
    }
}
