use std::collections::BTreeMap;

use crate::siko::util::DependencyProcessor;

use super::{
    Data::Struct,
    Function::{Block, Function, Instruction, Param, Value},
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
        let mut nextVar = 1;
        let mut llvmBlock = LBlock {
            id: block.id.clone(),
            instructions: Vec::new(),
        };
        for instruction in &block.instructions {
            match instruction {
                Instruction::StackAllocate(variable) => {
                    let var = LVariable {
                        name: variable.name.clone(),
                        ty: self.lowerType(&variable.ty),
                    };
                    let llvmInstruction = LInstruction::Allocate(var);
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Assignment(lvalue, rvalue) => {
                    let dest = match lvalue {
                        Value::Void => panic!("Invalid lvalue: void"),
                        Value::LiteralNumeric(_) => {
                            panic!("Invalid lvalue: literal")
                        }
                        Value::Var(variable) => LVariable {
                            name: variable.name.clone(),
                            ty: self.lowerType(&variable.ty),
                        },
                        Value::Field(value, _) => {
                            todo!()
                        }
                    };
                    let src = match rvalue {
                        Value::Void => panic!("Invalid rvalue: void"),
                        Value::LiteralNumeric(v) => LValue::Numeric(*v),
                        Value::Var(variable) => {
                            let var = LVariable {
                                name: variable.name.clone(),
                                ty: self.lowerType(&variable.ty),
                            };
                            LValue::Variable(var)
                        }
                        Value::Field(value, _) => {
                            todo!()
                        }
                    };
                    let llvmInstruction = LInstruction::Store(dest, src);
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Return(v) => {
                    let value = match v {
                        Value::Void => LValue::Void,
                        Value::LiteralNumeric(v) => LValue::Numeric(*v),
                        Value::Var(variable) => {
                            let var = LVariable {
                                name: variable.name.clone(),
                                ty: self.lowerType(&variable.ty),
                            };
                            LValue::Variable(var)
                        }
                        Value::Field(value, _) => unreachable!(),
                    };
                    let llvmInstruction = LInstruction::Return(value);
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
