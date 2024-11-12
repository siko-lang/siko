use std::collections::{btree_map::Entry, BTreeMap};

use super::{
    Data::Struct,
    Function::{Block, Function, FunctionKind, Instruction, Param, Value, Variable},
    Program::Program,
    Type::Type,
};

use crate::siko::llvm::Data::Struct as LStruct;
use crate::siko::llvm::Function::Block as LBlock;
use crate::siko::llvm::Function::Branch as LBranch;
use crate::siko::llvm::Function::Function as LFunction;
use crate::siko::llvm::Function::Instruction as LInstruction;
use crate::siko::llvm::Function::Param as LParam;
use crate::siko::llvm::Function::Value as LValue;
use crate::siko::llvm::Function::Variable as LVariable;
use crate::siko::llvm::Program::Program as LProgram;
use crate::siko::llvm::Type::Type as LType;
use crate::siko::llvm::{Constant::StringConstant, Data::Field as LField};

pub struct Builder<'a> {
    program: &'a Program,
    constants: BTreeMap<String, String>,
    refMap: BTreeMap<String, String>,
    nextTmp: u32,
}

impl<'a> Builder<'a> {
    pub fn new(program: &'a Program) -> Builder<'a> {
        Builder {
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
            name: format!("%{}", name),
            ty: self.lowerType(&v.ty),
        }
    }

    fn tmpVar(&mut self, v: &Variable) -> LVariable {
        self.nextTmp += 1;
        LVariable {
            name: format!("%tmp_{}_{}", v.name, self.nextTmp),
            ty: self.lowerType(&v.ty),
        }
    }

    pub fn lower(&mut self) -> LProgram {
        //println!("Before lowering {}", self.program);

        let mut program = LProgram::new();

        for (_, s) in &self.program.structs {
            program.structs.insert(s.name.clone(), self.lowerStruct(s));
        }

        for f in &self.program.functions {
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
        let mut llvmBlock = LBlock {
            id: block.id.clone(),
            instructions: Vec::new(),
        };
        for instruction in &block.instructions {
            match instruction {
                Instruction::Declare(_) => {
                    // declares are processed at the beginning
                }
                Instruction::Reference(dest, src) => {
                    let mut src = src.clone();
                    src.ty = dest.ty.clone();
                    let llvmInstruction = LInstruction::Store(self.lowerVar(dest), LValue::Variable(self.lowerVar(&src)));
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Call(dest, name, args) => {
                    let mut llvmArgs = Vec::new();
                    for arg in args {
                        let var = self.lowerVar(arg);
                        let var = if var.ty.isPtr() {
                            let tmp = self.tmpVar(arg);
                            let llvmInstruction = LInstruction::LoadVar(tmp.clone(), var);
                            llvmBlock.instructions.push(llvmInstruction);
                            tmp
                        } else {
                            var
                        };
                        llvmArgs.push(var);
                    }
                    llvmArgs.push(self.lowerVar(dest));
                    let llvmInstruction = LInstruction::FunctionCall(name.clone(), llvmArgs);
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Assign(dest, src) => {
                    let llvmInstruction = LInstruction::Store(
                        self.lowerVar(dest),
                        match src {
                            Value::Void => unreachable!(),
                            Value::Numeric(v) => LValue::Numeric(v.clone(), LType::Int64),
                            Value::Var(v) => LValue::Variable(self.lowerVar(v)),
                        },
                    );
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Return(v) => match v {
                    Value::Void => {
                        let llvmInstruction = LInstruction::Return(LValue::Void);
                        llvmBlock.instructions.push(llvmInstruction);
                    }
                    Value::Var(v) => {
                        if v.ty.isPtr() {
                            let llvmInstruction = LInstruction::Return(LValue::Void);
                            llvmBlock.instructions.push(llvmInstruction);
                        } else {
                            let llvmInstruction = LInstruction::Memcpy(self.lowerVar(v), self.lowerVar(&getResultVar(v.ty.clone())));
                            llvmBlock.instructions.push(llvmInstruction);
                            let llvmInstruction = LInstruction::Return(LValue::Void);
                            llvmBlock.instructions.push(llvmInstruction);
                        }
                    }
                    _ => {
                        unreachable!()
                    }
                },
                Instruction::Memcpy(src, dest) => {
                    if src.ty.isPtr() {
                        let llvmInstruction = LInstruction::Store(self.lowerVar(dest), LValue::Variable(self.lowerVar(src)));
                        llvmBlock.instructions.push(llvmInstruction);
                    } else {
                        let llvmInstruction = LInstruction::Memcpy(self.lowerVar(src), self.lowerVar(dest));
                        llvmBlock.instructions.push(llvmInstruction);
                    }
                }
                Instruction::GetFieldRef(dest, root, index) => {
                    let llvmInstruction = LInstruction::GetFieldRef(self.lowerVar(dest), self.lowerVar(root), *index);
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::IntegerLiteral(var, value) => {
                    let tmpVar = self.tmpVar(var);
                    let llvmInstruction = LInstruction::GetFieldRef(tmpVar.clone(), self.lowerVar(var), 0);
                    llvmBlock.instructions.push(llvmInstruction);
                    let llvmInstruction = LInstruction::Store(tmpVar, LValue::Numeric(value.clone(), LType::Int64));
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::StringLiteral(var, value) => {
                    let tmpVar = self.tmpVar(var);
                    let llvmInstruction = LInstruction::GetFieldRef(tmpVar.clone(), self.lowerVar(var), 0);
                    llvmBlock.instructions.push(llvmInstruction);
                    let i8Ptr = LType::Ptr(Box::new(LType::Int8));
                    let new = self.constants.len();
                    let strLen = value.len();
                    let value = match self.constants.entry(value.clone()) {
                        Entry::Occupied(v) => v.get().clone(),
                        Entry::Vacant(v) => {
                            let newStr = format!("str_{}", new);
                            v.insert(newStr.clone());
                            newStr
                        }
                    };
                    let llvmInstruction = LInstruction::Store(tmpVar, LValue::String(value.clone(), i8Ptr));
                    llvmBlock.instructions.push(llvmInstruction);
                    let tmpVar2 = self.tmpVar(var);
                    let llvmInstruction = LInstruction::GetFieldRef(tmpVar2.clone(), self.lowerVar(var), 1);
                    llvmBlock.instructions.push(llvmInstruction);
                    let llvmInstruction = LInstruction::Store(tmpVar2, LValue::Numeric(format!("{}", strLen), LType::Int64));
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Jump(name) => {
                    let llvmInstruction = LInstruction::Jump(name.clone());
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::EnumSwitch(var, cases) => {
                    let switchVar = Variable {
                        name: format!("switch_var_{}", block.id),
                        ty: Type::Int32,
                    };
                    let tmpVar = self.tmpVar(&switchVar);
                    let tmpVar2 = self.tmpVar(&switchVar);
                    let llvmInstruction = LInstruction::GetFieldRef(tmpVar.clone(), self.lowerVar(var), 0);
                    llvmBlock.instructions.push(llvmInstruction);
                    let llvmInstruction = LInstruction::LoadVar(tmpVar2.clone(), tmpVar);
                    llvmBlock.instructions.push(llvmInstruction);
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
                    let llvmInstruction = LInstruction::Switch(tmpVar2.clone(), cases[0].branch.clone(), branches);
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::IntegerSwitch(var, cases) => {
                    let switchVar = Variable {
                        name: format!("switch_var_{}", block.id),
                        ty: Type::Int64,
                    };
                    let tmpVar = self.tmpVar(&switchVar);
                    let tmpVar2 = self.tmpVar(&switchVar);
                    let llvmInstruction = LInstruction::GetFieldRef(tmpVar.clone(), self.lowerVar(var), 0);
                    llvmBlock.instructions.push(llvmInstruction);
                    let llvmInstruction = LInstruction::LoadVar(tmpVar2.clone(), tmpVar);
                    llvmBlock.instructions.push(llvmInstruction);
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
                    let llvmInstruction = LInstruction::Switch(tmpVar2.clone(), cases[defaultIndex].branch.clone(), branches);
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Transform(dest, src, index) => {
                    let u = self.program.getUnion(&src.ty.getUnion());
                    let v = &u.variants[*index as usize];
                    //println!("{} {} {} {}", dest.ty, src.ty, index, v.ty);
                    let mut recastVar = dest.clone();
                    recastVar.ty = Type::Struct(v.name.clone());
                    let recastVar = self.tmpVar(&recastVar);
                    let llvmInstruction = LInstruction::Bitcast(recastVar.clone(), self.lowerVar(src));
                    llvmBlock.instructions.push(llvmInstruction);
                    let llvmInstruction = LInstruction::GetFieldRef(self.lowerVar(dest), recastVar, 1);
                    llvmBlock.instructions.push(llvmInstruction);
                }
            };
        }
        llvmBlock
    }

    fn lowerFunction(&mut self, f: &Function) -> LFunction {
        let mut args: Vec<_> = f.args.iter().map(|p| self.lowerParam(p)).collect();
        args.push(LParam {
            name: getResultVarName(),
            ty: self.lowerType(&f.result),
        });
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
                let mut llvmBlocks: Vec<LBlock> = blocks.iter().map(|b| self.lowerBlock(b)).collect();
                for var in localVars {
                    let llvmInstruction = LInstruction::Allocate(self.lowerVar(&var));
                    llvmBlocks[0].instructions.insert(0, llvmInstruction)
                }
                LFunction {
                    name: f.name.clone(),
                    args: args,
                    result: LType::Void,
                    blocks: llvmBlocks,
                }
            }
            FunctionKind::ClassCtor => {
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
                    let fieldVar = Variable {
                        name: format!("field{}", index),
                        ty: field.ty.clone(),
                    };
                    block
                        .instructions
                        .push(LInstruction::GetFieldRef(self.lowerVar(&fieldVar), self.lowerVar(&this), index as i32));
                    let argVar = Variable {
                        name: field.name.clone(),
                        ty: field.ty.clone(),
                    };
                    block
                        .instructions
                        .push(LInstruction::Memcpy(self.lowerVar(&argVar), self.lowerVar(&fieldVar)));
                }
                block
                    .instructions
                    .push(LInstruction::Memcpy(self.lowerVar(&this), self.lowerVar(&getResultVar(this.ty.clone()))));
                block.instructions.push(LInstruction::Return(LValue::Void));
                LFunction {
                    name: f.name.clone(),
                    args: args,
                    result: LType::Void,
                    blocks: vec![block],
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
                    unreachable!()
                };
                let variant = &u.variants[*index as usize];
                let s = self.program.getStruct(&variant.ty.getStruct());
                let this = Variable {
                    name: "this".to_string(),
                    ty: Type::Struct(variant.name.clone()),
                };
                block.instructions.push(LInstruction::Allocate(self.lowerVar(&this)));
                let tagVar = Variable {
                    name: format!("tag"),
                    ty: Type::Int32,
                };
                let typedPayloadVar = Variable {
                    name: format!("payload1"),
                    ty: variant.ty.clone(),
                };
                block
                    .instructions
                    .push(LInstruction::GetFieldRef(self.lowerVar(&tagVar), self.lowerVar(&this), 0));
                block.instructions.push(LInstruction::Store(
                    self.lowerVar(&tagVar),
                    LValue::Numeric(format!("{}", index), LType::Int32),
                ));
                block
                    .instructions
                    .push(LInstruction::GetFieldRef(self.lowerVar(&typedPayloadVar), self.lowerVar(&this), 1));
                for (index, field) in s.fields.iter().enumerate() {
                    let fieldVar = Variable {
                        name: format!("field{}", index),
                        ty: field.ty.clone(),
                    };
                    block.instructions.push(LInstruction::GetFieldRef(
                        self.lowerVar(&fieldVar),
                        self.lowerVar(&typedPayloadVar),
                        index as i32,
                    ));
                    let argVar = Variable {
                        name: field.name.clone(),
                        ty: field.ty.clone(),
                    };
                    block
                        .instructions
                        .push(LInstruction::Memcpy(self.lowerVar(&argVar), self.lowerVar(&fieldVar)));
                }
                block
                    .instructions
                    .push(LInstruction::Memcpy(self.lowerVar(&this), self.lowerVar(&getResultVar(this.ty.clone()))));
                block.instructions.push(LInstruction::Return(LValue::Void));
                LFunction {
                    name: f.name.clone(),
                    args: args,
                    result: LType::Void,
                    blocks: vec![block],
                }
            }
            FunctionKind::Extern => {
                let mut args: Vec<_> = f.args.iter().map(|p| self.lowerParam(p)).collect();
                args.push(LParam {
                    name: getResultVarName(),
                    ty: self.lowerType(&f.result),
                });
                LFunction {
                    name: f.name.clone(),
                    args: args,
                    result: LType::Void,
                    blocks: Vec::new(),
                }
            }
        }
    }

    fn lowerStruct(&self, s: &Struct) -> LStruct {
        let mut fields = Vec::new();
        for f in &s.fields {
            let llvmField = LField {
                name: f.name.clone(),
                ty: self.lowerType(&f.ty),
            };
            fields.push(llvmField);
        }
        let llvmStruct = LStruct {
            name: s.name.clone(),
            fields: fields,
            size: s.size,
            alignment: s.alignment,
        };
        llvmStruct
    }

    fn lowerType(&self, ty: &Type) -> LType {
        match ty {
            Type::Void => LType::Void,
            Type::Int8 => LType::Int8,
            Type::Int16 => LType::Int16,
            Type::Int32 => LType::Int32,
            Type::Int64 => LType::Int64,
            Type::Char => LType::Int8,
            Type::Struct(n) => LType::Struct(n.clone()),
            Type::Union(n) => LType::Struct(n.clone()),
            Type::Ptr(t) => LType::Ptr(Box::new(self.lowerType(t))),
            Type::ByteArray(s) => LType::ByteArray(*s),
        }
    }
}

fn getResultVarName() -> String {
    "fn_result".to_string()
}

fn getResultVar(ty: Type) -> Variable {
    Variable {
        name: getResultVarName(),
        ty: Type::Ptr(Box::new(ty)),
    }
}
