use crate::siko::{
    hir::{
        Data::Class as HirClass,
        Function::{
            Block, Function as HirFunction, FunctionKind, InstructionId,
            InstructionKind as HirInstructionKind,
        },
        Program::Program as HirProgram,
        Type::Type as HirType,
    },
    mir::{
        Data::{Field as MirField, Struct},
        Function::{
            Block as MirBlock, Function as MirFunction, Instruction, Param as MirParam, Value,
            Variable,
        },
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
    qualifiedname::{getIntTypeName, getTrueName, QualifiedName},
};

pub struct Builder<'a> {
    program: &'a HirProgram,
    function: &'a HirFunction,
}

impl<'a> Builder<'a> {
    pub fn new(program: &'a HirProgram, function: &'a HirFunction) -> Builder<'a> {
        Builder {
            program: program,
            function: function,
        }
    }

    fn buildInstructionVar(&self, id: &InstructionId) -> Variable {
        let i = self.function.getInstruction(*id);
        let ty = lowerType(i.ty.as_ref().expect("no ty"));
        let name = format!("i{}", id.getId() + 1);
        Variable { name: name, ty: ty }
    }

    fn lowerBlock(&mut self, hirBlock: &Block) -> MirBlock {
        let mut block = MirBlock {
            id: format!("block{}", hirBlock.id.id),
            instructions: Vec::new(),
        };
        for instruction in &hirBlock.instructions {
            if let HirInstructionKind::Drop(_) = instruction.kind {
                continue;
            }
            let idVar = self.buildInstructionVar(&instruction.id);
            match &instruction.kind {
                HirInstructionKind::FunctionCall(name, args) => {
                    let args = args.iter().map(|id| self.buildInstructionVar(id)).collect();
                    block
                        .instructions
                        .push(Instruction::Call(idVar, convertName(name), args));
                }
                HirInstructionKind::Tuple(_) => {
                    block.instructions.push(Instruction::Declare(idVar.clone()));
                    block
                        .instructions
                        .push(Instruction::Assign(idVar, Value::Numeric(0)));
                }
                HirInstructionKind::Drop(_) => {}
                HirInstructionKind::DeclareVar(_) => {}
                HirInstructionKind::If(_, _, _) => {}
                HirInstructionKind::ValueRef(_, _, _) => {}
                HirInstructionKind::Assign(_, _) => {}
                HirInstructionKind::Bind(_, _) => {}
                HirInstructionKind::Jump(_) => {}
                HirInstructionKind::Return(v) => {
                    block
                        .instructions
                        .push(Instruction::Return(self.buildInstructionVar(v)));
                }
                HirInstructionKind::IntegerLiteral(v) => {
                    block
                        .instructions
                        .push(Instruction::IntegerLiteral(idVar, v.to_string()));
                }
                k => panic!("NYI {}", k),
            }
        }
        block
    }

    fn lowerFunction(&mut self) -> MirFunction {
        let result = lowerType(&self.function.result);
        let mut args = Vec::new();
        for arg in &self.function.params {
            let arg = MirParam {
                name: format!("{}", arg.getName()),
                ty: lowerType(&arg.getType()),
            };
            args.push(arg);
        }
        let mut mirFunction = MirFunction {
            name: convertName(&self.function.name),
            args: args,
            result: lowerType(&self.function.result),
            blocks: Vec::new(),
        };
        (self.function.name.clone(), result.clone());

        match &self.function.name {
            name if *name == getTrueName().monomorphized("".to_string()) => {
                let var1 = Variable {
                    name: "var1".to_string(),
                    ty: MirType::Int64,
                };
                let var2 = Variable {
                    name: "var2".to_string(),
                    ty: MirType::Int64,
                };
                let mut block = MirBlock {
                    id: format!("block0"),
                    instructions: Vec::new(),
                };
                block.instructions.push(Instruction::Declare(var1.clone()));
                block
                    .instructions
                    .push(Instruction::Reference(var2.clone(), var1.clone()));
                block.instructions.push(Instruction::Return(var2));
                mirFunction.blocks.push(block);
            }
            _ => {
                if self.function.kind == FunctionKind::ClassCtor {
                    if self.function.name == getIntTypeName().monomorphized("".to_string()) {
                        let var1 = Variable {
                            name: "var1".to_string(),
                            ty: MirType::Int64,
                        };
                        let mut block = MirBlock {
                            id: format!("block0"),
                            instructions: Vec::new(),
                        };
                        block.instructions.push(Instruction::Declare(var1.clone()));
                        block.instructions.push(Instruction::Return(var1));
                        mirFunction.blocks.push(block);
                    } else {
                        let mut block = MirBlock {
                            id: format!("block0"),
                            instructions: Vec::new(),
                        };
                        let this = Variable {
                            name: "this".to_string(),
                            ty: lowerType(&self.function.result),
                        };
                        let s = self
                            .program
                            .getClass(&self.function.result.getName().unwrap());
                        block.instructions.push(Instruction::Declare(this.clone()));
                        for (index, field) in s.fields.iter().enumerate() {
                            let fieldVar = Variable {
                                name: format!("field{}", index),
                                ty: MirType::Int64,
                            };
                            block.instructions.push(Instruction::GetFieldRef(
                                fieldVar,
                                this.clone(),
                                index as i32,
                            ));
                        }
                        block.instructions.push(Instruction::Return(this));
                        mirFunction.blocks.push(block);
                    }
                } else {
                    if let Some(body) = self.function.body.clone() {
                        for block in &body.blocks {
                            let mirBlock = self.lowerBlock(block);
                            mirFunction.blocks.push(mirBlock);
                        }
                    } else {
                        panic!("No body for {:?} {:?}", self.function.name, getTrueName());
                    }
                }
            }
        }

        mirFunction
    }
}

pub fn convertName(name: &QualifiedName) -> String {
    format!("@{}", name.toString().replace(".", "_"))
}

pub fn lowerType(ty: &HirType) -> MirType {
    match ty {
        HirType::Named(name, _, _) => {
            if name.toString() == "Int.Int" {
                MirType::Int64
            } else {
                MirType::Struct(name.toString())
            }
        }
        HirType::Tuple(_) => MirType::Int32,
        HirType::Function(_, _) => todo!(),
        HirType::Var(_) => todo!(),
        HirType::Reference(_, _) => todo!(),
        HirType::SelfType => todo!(),
        HirType::Never => MirType::Void,
    }
}

pub fn lowerClass(c: &HirClass) -> Struct {
    let mut fields = Vec::new();
    if c.name.toString() == "Int.Int" {
        fields.push(MirField {
            name: "value".to_string(),
            ty: MirType::Int64,
        });
    }

    for f in &c.fields {
        let mirField = MirField {
            name: f.name.clone(),
            ty: lowerType(&f.ty),
        };
        fields.push(mirField);
    }
    Struct {
        name: c.name.toString(),
        fields: fields,
        size: 0,
        alignment: 0,
    }
}

pub fn lowerProgram(program: &HirProgram) -> MirProgram {
    let mut mir_program = MirProgram::new();

    for (n, c) in &program.classes {
        let c = lowerClass(c);
        mir_program.structs.insert(n.toString(), c);
    }

    for (_, function) in &program.functions {
        let mut builder = Builder::new(program, function);
        let f = builder.lowerFunction();
        mir_program.functions.push(f);
    }

    mir_program
}
