use std::collections::BTreeMap;

use crate::siko::util::DependencyProcessor;

use super::{
    Data::Struct,
    Function::{Block, Function, Instruction, Param, Value, Variable},
    Type::Type,
};

use crate::siko::llvm::Data::Field as LField;
use crate::siko::llvm::Data::Struct as LStruct;
use crate::siko::llvm::Function::Block as LBlock;
use crate::siko::llvm::Function::Function as LFunction;
use crate::siko::llvm::Function::Instruction as LInstruction;
use crate::siko::llvm::Function::Param as LParam;
use crate::siko::llvm::Function::Value as LValue;
use crate::siko::llvm::Function::Variable as LVariable;
use crate::siko::llvm::Program::Program as LProgram;
use crate::siko::llvm::Type::Type as LType;

pub struct Program {
    pub functions: Vec<Function>,
    pub structs: BTreeMap<String, Struct>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: Vec::new(),
            structs: BTreeMap::new(),
        }
    }

    pub fn process(&mut self) -> LProgram {
        self.calculateSizeAndAlignment();

        self.lower()
    }

    fn getStruct(&self, n: &String) -> Struct {
        self.structs.get(n).cloned().expect("struct not found")
    }

    fn calculateSizeAndAlignment(&mut self) {
        let mut allDeps = BTreeMap::new();
        for (_, s) in &self.structs {
            allDeps.insert(s.name.clone(), Vec::new());
        }
        for (_, s) in &self.structs {
            for f in &s.fields {
                match &f.ty {
                    Type::Struct(n) => {
                        let deps = allDeps.get_mut(&s.name).expect("deps not found");
                        deps.push(n.clone());
                    }
                    _ => {}
                }
            }
        }

        let groups = DependencyProcessor::processDependencies(&allDeps);

        for group in &groups {
            if group.items.len() != 1 {
                panic!("Minic: cyclic data dependency {:?}", groups);
            }
            let mut item = self.getStruct(&group.items[0]);
            let mut offset = 0;
            let mut totalAlignment = 4;
            for f in &item.fields {
                let size = match &f.ty {
                    Type::Void => 0,
                    Type::Int8 => 1,
                    Type::Int16 => 2,
                    Type::Int32 => 4,
                    Type::Int64 => 8,
                    Type::Char => 1,
                    Type::Struct(n) => self.getStruct(n).size,
                    Type::Ptr(_) => 8,
                };
                let alignment = match &f.ty {
                    Type::Void => 0,
                    Type::Int8 => 1,
                    Type::Int16 => 2,
                    Type::Int32 => 4,
                    Type::Int64 => 8,
                    Type::Char => 1,
                    Type::Struct(n) => self.getStruct(n).alignment,
                    Type::Ptr(_) => 8,
                };
                totalAlignment = std::cmp::max(totalAlignment, alignment);
                offset += size;
                let padding = (alignment - (offset % alignment)) % alignment;
                offset += padding;
            }
            let padding = (totalAlignment - (offset % totalAlignment)) % totalAlignment;
            offset += padding;
            item.alignment = totalAlignment;
            item.size = offset;
            self.structs.insert(item.name.clone(), item);
        }
    }

    fn lowerVar(&self, v: &Variable) -> LVariable {
        LVariable {
            name: format!("%{}", v.name),
            ty: self.lowerType(&v.ty),
        }
    }

    fn tmpVar(&self, v: &Variable, index: u32) -> LVariable {
        LVariable {
            name: format!("%tmp_{}_{}", v.name, index),
            ty: self.lowerType(&v.ty),
        }
    }
    fn lower(&self) -> LProgram {
        let mut program = LProgram::new();

        for (_, s) in &self.structs {
            program.structs.insert(s.name.clone(), self.lowerStruct(s));
        }

        for f in &self.functions {
            let f = self.lowerFunction(f);
            program.functions.push(f);
        }

        program
    }

    fn lowerParam(&self, p: &Param) -> LParam {
        LParam {
            name: p.name.clone(),
            ty: self.lowerType(&p.ty),
        }
    }

    fn lowerBlock(&self, block: &Block) -> LBlock {
        let mut llvmBlock = LBlock {
            id: block.id.clone(),
            instructions: Vec::new(),
        };
        for instruction in &block.instructions {
            match instruction {
                Instruction::Declare(var) => {
                    if var.ty.isSimple() {
                        let tmp = self.tmpVar(var, 1);
                        let llvmInstruction = LInstruction::Allocate(tmp.clone());
                        llvmBlock.instructions.push(llvmInstruction);
                        let llvmInstruction = LInstruction::LoadVar(self.lowerVar(var), tmp);
                        llvmBlock.instructions.push(llvmInstruction);
                    } else {
                        let llvmInstruction = LInstruction::Allocate(self.lowerVar(var));
                        llvmBlock.instructions.push(llvmInstruction);
                    }
                }
                Instruction::Reference(dest, src) => {
                    let llvmInstruction =
                        LInstruction::LoadVar(self.lowerVar(dest), self.lowerVar(src));
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Call(dest, name, args) => {
                    let mut llvmArgs = Vec::new();
                    for arg in args {
                        llvmArgs.push(self.lowerVar(arg));
                    }
                    let llvmInstruction =
                        LInstruction::FunctionCall(self.lowerVar(dest), name.clone(), llvmArgs);
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Assign(dest, src) => {
                    let llvmInstruction = LInstruction::Store(
                        self.lowerVar(dest),
                        match src {
                            Value::Numeric(v) => LValue::Numeric(*v),
                            Value::Var(v) => LValue::Variable(self.lowerVar(v)),
                        },
                    );
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Return(v) => {
                    if v.ty.isSimple() {
                        let llvmInstruction =
                            LInstruction::Return(LValue::Variable(self.lowerVar(v)));
                        llvmBlock.instructions.push(llvmInstruction);
                    } else {
                        let tmp = self.tmpVar(v, 1);
                        let llvmInstruction = LInstruction::LoadVar(tmp.clone(), self.lowerVar(v));
                        llvmBlock.instructions.push(llvmInstruction);
                        let llvmInstruction = LInstruction::Return(LValue::Variable(tmp));
                        llvmBlock.instructions.push(llvmInstruction);
                    }
                }
                Instruction::GetFieldRef(dest, root, index) => {
                    let llvmInstruction =
                        LInstruction::GetFieldRef(self.lowerVar(dest), self.lowerVar(root), *index);
                    llvmBlock.instructions.push(llvmInstruction);
                }
            };
        }
        llvmBlock
    }

    fn lowerFunction(&self, f: &Function) -> LFunction {
        LFunction {
            name: f.name.clone(),
            args: f.args.iter().map(|p| self.lowerParam(p)).collect(),
            result: self.lowerType(&f.result),
            blocks: f.blocks.iter().map(|b| self.lowerBlock(b)).collect(),
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
            Type::Ptr(_) => todo!(),
        }
    }
}
