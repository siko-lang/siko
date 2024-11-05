use std::{collections::BTreeMap, fmt};

use crate::siko::util::DependencyProcessor;

use super::{
    Data::{Field, Struct, Union},
    Function::{Block, Function, FunctionKind, Instruction, Param, Value, Variable},
    Type::Type,
};

use crate::siko::llvm::Data::Field as LField;
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

pub struct Program {
    pub functions: Vec<Function>,
    pub structs: BTreeMap<String, Struct>,
    pub unions: BTreeMap<String, Union>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Program:\n")?;
        write!(f, "\nFunctions:\n")?;
        for function in &self.functions {
            write!(f, "{}\n", function)?;
        }
        write!(f, "\nStructs:\n")?;
        for (_, s) in &self.structs {
            write!(f, "{}\n", s)?;
        }
        write!(f, "\nUnions:\n")?;
        for (_, u) in &self.unions {
            write!(f, "{}\n", u)?;
        }
        Ok(())
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

impl Program {
    pub fn new() -> Program {
        Program {
            functions: Vec::new(),
            structs: BTreeMap::new(),
            unions: BTreeMap::new(),
        }
    }

    pub fn process(&mut self) -> LProgram {
        self.calculateSizeAndAlignment();

        self.convertUnions();

        self.lower()
    }

    fn getStruct(&self, n: &String) -> Struct {
        match self.structs.get(n) {
            Some(s) => s.clone(),
            None => panic!("struct {} not found", n),
        }
    }

    fn getUnion(&self, n: &String) -> Union {
        self.unions.get(n).cloned().expect("union not found")
    }

    fn convertUnions(&mut self) {
        for (n, u) in &self.unions {
            let tag = Field {
                name: format!("tag"),
                ty: Type::Int32,
            };
            let payload = Field {
                name: format!("payload"),
                ty: Type::ByteArray(u.payloadSize),
            };
            let s = Struct {
                name: n.clone(),
                fields: vec![tag, payload],
                size: u.size,
                alignment: u.alignment,
            };
            self.structs.insert(n.clone(), s);
            for v in &u.variants {
                let mut variantStruct = if let Type::Struct(vName) = &v.ty {
                    self.getStruct(vName)
                } else {
                    unreachable!()
                };
                variantStruct.name = v.name.clone();
                self.structs.insert(v.name.clone(), variantStruct);
            }
        }
    }

    fn calculateSizeAndAlignment(&mut self) {
        let mut allDeps = BTreeMap::new();

        for (_, s) in &self.structs {
            allDeps.insert(s.name.clone(), Vec::new());
        }

        for (_, u) in &self.unions {
            allDeps.insert(u.name.clone(), Vec::new());
        }

        for (_, s) in &self.structs {
            for f in &s.fields {
                match &f.ty {
                    Type::Struct(n) => {
                        let deps = allDeps.get_mut(&s.name).expect("deps not found");
                        deps.push(n.clone());
                    }
                    Type::Union(n) => {
                        let deps = allDeps.get_mut(&s.name).expect("deps not found");
                        deps.push(n.clone());
                    }
                    _ => {}
                }
            }
        }

        for (_, u) in &self.unions {
            for v in &u.variants {
                match &v.ty {
                    Type::Struct(n) => {
                        let deps = allDeps.get_mut(&u.name).expect("deps not found");
                        deps.push(n.clone());
                    }
                    Type::Union(n) => {
                        let deps = allDeps.get_mut(&u.name).expect("deps not found");
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
            let itemName = &group.items[0];
            if self.structs.contains_key(itemName) {
                let mut item = self.getStruct(itemName);
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
                        Type::Union(n) => self.getUnion(n).size,
                        Type::Ptr(_) => 8,
                        Type::ByteArray(s) => *s,
                    };
                    let alignment = match &f.ty {
                        Type::Void => 0,
                        Type::Int8 => 1,
                        Type::Int16 => 2,
                        Type::Int32 => 4,
                        Type::Int64 => 8,
                        Type::Char => 1,
                        Type::Struct(n) => self.getStruct(n).alignment,
                        Type::Union(n) => self.getUnion(n).alignment,
                        Type::Ptr(_) => 8,
                        Type::ByteArray(_) => 1,
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

            if self.unions.contains_key(itemName) {
                let mut item = self.getUnion(itemName);
                let mut offset = 4;
                let mut totalAlignment = 4;
                let mut maxSize = 0;
                for v in &item.variants {
                    let size = match &v.ty {
                        Type::Void => 0,
                        Type::Int8 => 1,
                        Type::Int16 => 2,
                        Type::Int32 => 4,
                        Type::Int64 => 8,
                        Type::Char => 1,
                        Type::Struct(n) => self.getStruct(n).size,
                        Type::Union(n) => self.getUnion(n).size,
                        Type::Ptr(_) => 8,
                        Type::ByteArray(s) => *s,
                    };
                    let alignment = match &v.ty {
                        Type::Void => 0,
                        Type::Int8 => 1,
                        Type::Int16 => 2,
                        Type::Int32 => 4,
                        Type::Int64 => 8,
                        Type::Char => 1,
                        Type::Struct(n) => self.getStruct(n).alignment,
                        Type::Union(n) => self.getUnion(n).alignment,
                        Type::Ptr(_) => 8,
                        Type::ByteArray(_) => 1,
                    };
                    totalAlignment = std::cmp::max(totalAlignment, alignment);
                    maxSize = std::cmp::max(maxSize, size);
                }
                offset += maxSize;
                let padding = (totalAlignment - (offset % totalAlignment)) % totalAlignment;
                offset += padding;
                item.alignment = totalAlignment;
                item.size = offset;
                item.payloadSize = maxSize;
                self.unions.insert(item.name.clone(), item);
            }
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
        //println!("Before lowering {}", self);

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
                Instruction::Declare(_) => {
                    // declares are processed at the beginning
                }
                Instruction::Reference(dest, src) => {
                    let llvmInstruction = LInstruction::LoadVar(self.lowerVar(dest), self.lowerVar(src));
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Call(dest, name, args) => {
                    let mut llvmArgs = Vec::new();
                    for arg in args {
                        llvmArgs.push(self.lowerVar(arg));
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
                        let llvmInstruction = LInstruction::Memcpy(self.lowerVar(v), self.lowerVar(&getResultVar(v.ty.clone())));
                        llvmBlock.instructions.push(llvmInstruction);
                        let llvmInstruction = LInstruction::Return(LValue::Void);
                        llvmBlock.instructions.push(llvmInstruction);
                    }
                    _ => {
                        unreachable!()
                    }
                },
                Instruction::Memcpy(src, dest) => {
                    let llvmInstruction = LInstruction::Memcpy(self.lowerVar(src), self.lowerVar(dest));
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::GetFieldRef(dest, root, index) => {
                    let llvmInstruction = LInstruction::GetFieldRef(self.lowerVar(dest), self.lowerVar(root), *index);
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::IntegerLiteral(var, value) => {
                    let tmpVar = self.tmpVar(var, 1);
                    let llvmInstruction = LInstruction::GetFieldRef(tmpVar.clone(), self.lowerVar(var), 0);
                    llvmBlock.instructions.push(llvmInstruction);
                    let llvmInstruction = LInstruction::Store(tmpVar, LValue::Numeric(value.clone(), LType::Int64));
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
                    let tmpVar = self.tmpVar(&switchVar, 1);
                    let tmpVar2 = self.tmpVar(&switchVar, 2);
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
                            value: LValue::Numeric(format!("{}", index), LType::Int32),
                            block: case.branch.clone(),
                        };
                        branches.push(branch);
                    }
                    let llvmInstruction = LInstruction::Switch(tmpVar2.clone(), cases[0].branch.clone(), branches);
                    llvmBlock.instructions.push(llvmInstruction);
                }
                Instruction::Transform(dest, src, _) => {
                    let llvmInstruction = LInstruction::Bitcast(self.lowerVar(dest), self.lowerVar(src));
                    llvmBlock.instructions.push(llvmInstruction);
                }
            };
        }
        llvmBlock
    }

    fn lowerFunction(&self, f: &Function) -> LFunction {
        match &f.kind {
            FunctionKind::UserDefined(blocks) => {
                let mut llvmBlocks: Vec<LBlock> = blocks.iter().map(|b| self.lowerBlock(b)).collect();
                let mut localVars = Vec::new();
                for block in blocks {
                    for instruction in &block.instructions {
                        if let Instruction::Declare(var) = instruction {
                            localVars.push(var.clone());
                        }
                    }
                }
                for var in localVars {
                    let llvmInstruction = LInstruction::Allocate(self.lowerVar(&var));
                    llvmBlocks[0].instructions.insert(0, llvmInstruction)
                }
                let mut args: Vec<_> = f.args.iter().map(|p| self.lowerParam(p)).collect();
                args.push(LParam {
                    name: getResultVarName(),
                    ty: LType::Ptr(Box::new(self.lowerType(&f.result))),
                });
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
                let s = self.getStruct(&f.name);
                block.instructions.push(LInstruction::Allocate(self.lowerVar(&this)));
                for (index, field) in s.fields.iter().enumerate() {
                    let fieldVar = Variable {
                        name: format!("field{}", index),
                        ty: Type::Int64,
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
                        .push(LInstruction::Memcpy(self.lowerVar(&fieldVar), self.lowerVar(&argVar)));
                }
                block
                    .instructions
                    .push(LInstruction::Memcpy(self.lowerVar(&this), self.lowerVar(&getResultVar(this.ty.clone()))));
                block.instructions.push(LInstruction::Return(LValue::Void));
                let mut args: Vec<_> = f.args.iter().map(|p| self.lowerParam(p)).collect();
                args.push(LParam {
                    name: getResultVarName(),
                    ty: LType::Ptr(Box::new(self.lowerType(&f.result))),
                });
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
                let this = Variable {
                    name: "this".to_string(),
                    ty: f.result.clone(),
                };
                let u = if let Type::Union(u) = &f.result {
                    self.getUnion(u)
                } else {
                    unreachable!()
                };
                let variant = &u.variants[*index as usize];
                let s = self.getStruct(&f.name);
                block.instructions.push(LInstruction::Allocate(self.lowerVar(&this)));
                let tagVar = Variable {
                    name: format!("tag"),
                    ty: Type::Int32,
                };
                let untypedPayloadVar = Variable {
                    name: format!("payload1"),
                    ty: Type::Int8,
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
                    .push(LInstruction::GetFieldRef(self.lowerVar(&untypedPayloadVar), self.lowerVar(&this), 1));
                let payloadVar = Variable {
                    name: format!("payload2"),
                    ty: Type::Struct(variant.name.clone()),
                };
                block
                    .instructions
                    .push(LInstruction::Bitcast(self.lowerVar(&payloadVar), self.lowerVar(&untypedPayloadVar)));
                for (index, field) in s.fields.iter().enumerate() {
                    let fieldVar = Variable {
                        name: format!("field{}", index),
                        ty: field.ty.clone(),
                    };
                    block.instructions.push(LInstruction::GetFieldRef(
                        self.lowerVar(&fieldVar),
                        self.lowerVar(&payloadVar),
                        index as i32,
                    ));
                    let argVar = Variable {
                        name: field.name.clone(),
                        ty: field.ty.clone(),
                    };
                    block
                        .instructions
                        .push(LInstruction::Memcpy(self.lowerVar(&fieldVar), self.lowerVar(&argVar)));
                }
                block
                    .instructions
                    .push(LInstruction::Memcpy(self.lowerVar(&this), self.lowerVar(&getResultVar(this.ty.clone()))));
                block.instructions.push(LInstruction::Return(LValue::Void));
                let mut args: Vec<_> = f.args.iter().map(|p| self.lowerParam(p)).collect();
                args.push(LParam {
                    name: getResultVarName(),
                    ty: LType::Ptr(Box::new(self.lowerType(&f.result))),
                });
                LFunction {
                    name: f.name.clone(),
                    args: args,
                    result: LType::Void,
                    blocks: vec![block],
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
