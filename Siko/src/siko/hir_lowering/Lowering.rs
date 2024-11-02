use crate::siko::{
    hir::{
        Data::{Class as HirClass, Enum as HirEnum},
        Function::{Block, BlockId, Function as HirFunction, FunctionKind, InstructionId, InstructionKind as HirInstructionKind},
        Program::Program as HirProgram,
        Type::Type as HirType,
    },
    mir::{
        Data::{Field as MirField, Struct, Union, Variant as MirVariant},
        Function::{
            Block as MirBlock, EnumCase as MirEnumCase, Function as MirFunction, FunctionKind as MirFunctionKind, Instruction, Param as MirParam,
            Value, Variable,
        },
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
    qualifiedname::{getTrueName, QualifiedName},
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
        let ty = lowerType(i.ty.as_ref().expect("no ty"), &self.program);
        let name = format!("i_{}_{}", id.getBlockById().id, id.getId() + 1);
        Variable { name: name, ty: ty }
    }

    fn getBlockName(&self, blockId: BlockId) -> String {
        format!("block{}", blockId.id)
    }

    fn lowerBlock(&mut self, hirBlock: &Block) -> MirBlock {
        let mut block = MirBlock {
            id: self.getBlockName(hirBlock.id),
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
                    block.instructions.push(Instruction::Declare(idVar.clone()));
                    block.instructions.push(Instruction::Call(idVar, convertName(name), args));
                }
                HirInstructionKind::Tuple(_) => {
                    unreachable!()
                }
                HirInstructionKind::Drop(_) => {}
                HirInstructionKind::DeclareVar(var) => {
                    let i = self.function.getInstruction(instruction.id);
                    let ty = lowerType(i.ty.as_ref().expect("no ty"), &self.program);
                    let var = Variable { name: var.clone(), ty: ty };
                    block.instructions.push(Instruction::Declare(var.clone()));
                }
                HirInstructionKind::If(_, _, _) => {}
                HirInstructionKind::ValueRef(name, _, _) => {
                    let i = self.function.getInstruction(instruction.id);
                    let ty = lowerType(i.ty.as_ref().expect("no ty"), &self.program);
                    let var = Variable {
                        name: name.getValue(),
                        ty: ty,
                    };
                    block.instructions.push(Instruction::Declare(idVar.clone()));
                    block.instructions.push(Instruction::Memcpy(var, idVar));
                }
                HirInstructionKind::Assign(_, _) => {}
                HirInstructionKind::Bind(var, rhs) => {
                    let i = self.function.getInstruction(*rhs);
                    let ty = lowerType(i.ty.as_ref().expect("no ty"), &self.program);
                    let var = Variable {
                        name: var.to_string(),
                        ty: ty,
                    };
                    let rhs = self.buildInstructionVar(rhs);
                    block.instructions.push(Instruction::Declare(var.clone()));
                    block.instructions.push(Instruction::Memcpy(rhs, var));
                }
                HirInstructionKind::Jump(blockId) => {
                    block.instructions.push(Instruction::Jump(self.getBlockName(*blockId)));
                }
                HirInstructionKind::Return(v) => {
                    block.instructions.push(Instruction::Return(Value::Var(self.buildInstructionVar(v))));
                }
                HirInstructionKind::IntegerLiteral(v) => {
                    block.instructions.push(Instruction::Declare(idVar.clone()));
                    block.instructions.push(Instruction::IntegerLiteral(idVar, v.to_string()));
                }
                HirInstructionKind::EnumSwitch(root, cases) => {
                    let root = self.buildInstructionVar(root);
                    let mut mirCases = Vec::new();
                    for case in cases {
                        let mirCase = MirEnumCase {
                            name: convertName(&case.name),
                            branch: self.getBlockName(case.branch),
                        };
                        mirCases.push(mirCase);
                    }
                    block.instructions.push(Instruction::EnumSwitch(root, mirCases));
                }
                HirInstructionKind::Transform(root, ty) => {
                    let root = self.buildInstructionVar(root);
                    block.instructions.push(Instruction::Transform(idVar, root, format!("{}", ty)));
                }
                k => panic!("NYI {}", k),
            }
        }
        block
    }

    fn lowerFunction(&mut self) -> MirFunction {
        let mut args = Vec::new();
        for arg in &self.function.params {
            let arg = MirParam {
                name: format!("{}", arg.getName()),
                ty: lowerType(&arg.getType(), &self.program),
            };
            args.push(arg);
        }

        let mut blocks = Vec::new();
        match self.function.kind {
            FunctionKind::ClassCtor => {
                blocks.push(self.createClassCtor());
            }
            FunctionKind::UserDefined => {
                if let Some(body) = self.function.body.clone() {
                    for block in &body.blocks {
                        let mirBlock = self.lowerBlock(block);
                        blocks.push(mirBlock);
                    }
                }
            }
            FunctionKind::VariantCtor(_) => {
                if self.function.name == getTrueName().monomorphized("".to_string()) {
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
                    block.instructions.push(Instruction::Reference(var2.clone(), var1.clone()));
                    block.instructions.push(Instruction::Return(Value::Void));
                    blocks.push(block);
                }
            }
            FunctionKind::Extern => {}
        }
        let mut mirFunction = MirFunction {
            name: convertName(&self.function.name),
            args: args,
            result: lowerType(&self.function.result, &self.program),
            kind: MirFunctionKind::UserDefined(blocks),
        };
        mirFunction
    }

    fn createClassCtor(&mut self) -> MirBlock {
        let mut block = MirBlock {
            id: format!("block0"),
            instructions: Vec::new(),
        };
        let this = Variable {
            name: "this".to_string(),
            ty: lowerType(&self.function.result, &self.program),
        };
        let s = self.program.getClass(&self.function.result.getName().unwrap());
        block.instructions.push(Instruction::Declare(this.clone()));
        for (index, field) in s.fields.iter().enumerate() {
            let fieldVar = Variable {
                name: format!("field{}", index),
                ty: MirType::Int64,
            };
            block
                .instructions
                .push(Instruction::GetFieldRef(fieldVar.clone(), this.clone(), index as i32));
            let argVar = Variable {
                name: field.name.clone(),
                ty: lowerType(&field.ty, &self.program),
            };
            block.instructions.push(Instruction::Memcpy(fieldVar, argVar));
        }
        block.instructions.push(Instruction::Return(Value::Var(this)));
        block
    }
}

pub fn convertName(name: &QualifiedName) -> String {
    format!(
        "{}",
        name.toString()
            .replace(".", "_")
            .replace("(", "")
            .replace(")", "")
            .replace(",", "_")
            .replace(" ", "_")
    )
}

pub fn lowerType(ty: &HirType, program: &HirProgram) -> MirType {
    match ty {
        HirType::Named(name, _, _) => {
            if program.classes.get(name).is_some() {
                MirType::Struct(convertName(name))
            } else {
                MirType::Union(convertName(name))
            }
        }
        HirType::Tuple(_) => unreachable!("Tuple in MIR"),
        HirType::Function(_, _) => todo!(),
        HirType::Var(_) => todo!(),
        HirType::Reference(_, _) => todo!(),
        HirType::SelfType => todo!(),
        HirType::Never => MirType::Void,
    }
}

pub fn lowerClass(c: &HirClass, program: &HirProgram) -> Struct {
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
            ty: lowerType(&f.ty, program),
        };
        fields.push(mirField);
    }
    Struct {
        name: convertName(&c.name),
        fields: fields,
        size: 0,
        alignment: 0,
    }
}

pub fn lowerEnum(e: &HirEnum, program: &HirProgram) -> Union {
    let mut variants = Vec::new();

    for v in &e.variants {
        assert_eq!(v.items.len(), 1);
        let mirVariant = MirVariant {
            name: convertName(&v.name),
            ty: lowerType(&v.items[0], program),
        };
        variants.push(mirVariant);
    }
    Union {
        name: convertName(&e.name),
        variants: variants,
        size: 0,
        alignment: 0,
    }
}

pub fn lowerProgram(program: &HirProgram) -> MirProgram {
    let mut mirProgram = MirProgram::new();

    for (n, c) in &program.classes {
        let c = lowerClass(c, program);
        mirProgram.structs.insert(convertName(n), c);
    }

    for (n, e) in &program.enums {
        let u = lowerEnum(e, program);
        mirProgram.unions.insert(convertName(n), u);
    }

    for (_, function) in &program.functions {
        let mut builder = Builder::new(program, function);
        let f = builder.lowerFunction();
        mirProgram.functions.push(f);
    }

    mirProgram
}
