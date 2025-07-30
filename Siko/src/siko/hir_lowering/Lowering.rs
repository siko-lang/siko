use core::panic;

use crate::siko::{
    hir::{
        Data::{Enum as HirEnum, Struct as HirStruct},
        Function::{Block, BlockId, Function as HirFunction, FunctionKind},
        Instruction::InstructionKind as HirInstructionKind,
        Program::Program as HirProgram,
        Type::Type as HirType,
        Variable::Variable,
    },
    mir::{
        Data::{Field as MirField, Struct, Union, Variant as MirVariant},
        Function::{
            Block as MirBlock, EnumCase as MirEnumCase, Function as MirFunction, FunctionKind as MirFunctionKind,
            Instruction, IntegerCase as MirIntegerCase, Param as MirParam, Value, Variable as MirVariable,
        },
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
    qualifiedname::{
        getBoolTypeName, getFalseName, getI32TypeName, getIntTypeName, getNativePtrToRefName, getTrueName,
        getU8TypeName,
    },
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

    fn buildVariable(&self, id: &Variable) -> MirVariable {
        let ty = lowerType(&id.getType(), &self.program);
        let name = convertName(&id.value);
        MirVariable { name: name, ty: ty }
    }

    fn getBlockName(&self, blockId: BlockId) -> String {
        format!("block{}", blockId.id)
    }

    fn lowerBlock(&mut self, hirBlock: &Block) -> Option<MirBlock> {
        if hirBlock.instructions.is_empty() {
            return None;
        }
        let mut block = MirBlock {
            id: self.getBlockName(hirBlock.id),
            instructions: Vec::new(),
        };
        for instruction in &hirBlock.instructions {
            match &instruction.kind {
                HirInstructionKind::FunctionCall(dest, name, args) => {
                    if name.base() == getTrueName() {
                        let dest = self.buildVariable(dest);
                        block.instructions.push(Instruction::Declare(dest.clone()));
                        block
                            .instructions
                            .push(Instruction::IntegerLiteral(dest, "1".to_string()));
                        continue;
                    }
                    if name.base() == getFalseName() {
                        let dest = self.buildVariable(dest);
                        block.instructions.push(Instruction::Declare(dest.clone()));
                        block
                            .instructions
                            .push(Instruction::IntegerLiteral(dest, "0".to_string()));
                        continue;
                    }
                    if name.base() == getNativePtrToRefName() {
                        let dest = self.buildVariable(dest);
                        let arg = &args[0];
                        block.instructions.push(Instruction::Declare(dest.clone()));
                        block
                            .instructions
                            .push(Instruction::Memcpy(self.buildVariable(arg), dest.clone()));
                    } else {
                        let args = args.iter().map(|var| self.buildVariable(var)).collect();
                        let dest = self.buildVariable(dest);
                        block.instructions.push(Instruction::Declare(dest.clone()));
                        block
                            .instructions
                            .push(Instruction::Call(dest, convertName(name), args));
                    }
                }
                HirInstructionKind::Tuple(_, _) => {
                    unreachable!("tuples in MIR??")
                }
                HirInstructionKind::Drop(_, _) => unreachable!("drop in MIR??"),
                HirInstructionKind::DeclareVar(var, _) => {
                    let var = self.buildVariable(var);
                    block.instructions.push(Instruction::Declare(var.clone()));
                }
                HirInstructionKind::Assign(lhs, rhs) => {
                    let rhs = self.buildVariable(rhs);
                    let dest = self.buildVariable(lhs);
                    block.instructions.push(Instruction::Memcpy(rhs, dest));
                }
                HirInstructionKind::Bind(_, _, _) => {
                    panic!("Bind instruction found in Lowering, this should not happen");
                }
                HirInstructionKind::Jump(_, blockId, _) => {
                    block.instructions.push(Instruction::Jump(self.getBlockName(*blockId)));
                }
                HirInstructionKind::Return(_, v) => {
                    block
                        .instructions
                        .push(Instruction::Return(Value::Var(self.buildVariable(v))));
                }
                HirInstructionKind::IntegerLiteral(dest, v) => {
                    let dest = self.buildVariable(dest);
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block
                        .instructions
                        .push(Instruction::IntegerLiteral(dest, v.to_string()));
                }
                HirInstructionKind::StringLiteral(dest, v) => {
                    let dest = self.buildVariable(dest);
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block.instructions.push(Instruction::StringLiteral(dest, v.to_string()));
                }
                HirInstructionKind::EnumSwitch(root, cases) => {
                    if root.getType().getName().unwrap().base() == getBoolTypeName() {
                        let dest = self.buildVariable(root);
                        let mut mirCases = Vec::new();
                        for case in cases {
                            let mirCase = MirIntegerCase {
                                value: Some(format!("{}", case.index)),
                                branch: self.getBlockName(case.branch),
                            };
                            mirCases.push(mirCase);
                        }
                        block.instructions.push(Instruction::IntegerSwitch(dest, mirCases));
                    } else {
                        let root = self.buildVariable(root);
                        let mut mirCases = Vec::new();
                        for case in cases {
                            let mirCase = MirEnumCase {
                                index: case.index,
                                branch: self.getBlockName(case.branch),
                            };
                            mirCases.push(mirCase);
                        }
                        block.instructions.push(Instruction::EnumSwitch(root, mirCases));
                    }
                }
                HirInstructionKind::IntegerSwitch(root, cases) => {
                    let root = self.buildVariable(root);
                    let mut mirCases = Vec::new();
                    for case in cases {
                        let mirCase = MirIntegerCase {
                            value: case.value.clone(),
                            branch: self.getBlockName(case.branch),
                        };
                        mirCases.push(mirCase);
                    }
                    block.instructions.push(Instruction::IntegerSwitch(root, mirCases));
                }
                HirInstructionKind::Transform(dest, root, index) => {
                    if root.getType().getName().unwrap().base() == getBoolTypeName() {
                        block.instructions.push(Instruction::Declare(self.buildVariable(dest)));
                    } else {
                        let dest = self.buildVariable(dest);
                        let root = self.buildVariable(root);
                        block.instructions.push(Instruction::Transform(dest, root, *index));
                    }
                }
                HirInstructionKind::FieldRef(dest, root, fields) => {
                    let dest = self.buildVariable(dest);
                    let mut currentReceiver = self.buildVariable(root);
                    let mut receiverTy = root.getType();
                    for (index, field) in fields.iter().enumerate() {
                        let tmpVariable = MirVariable {
                            name: format!("{}_{}.{}", root.value, index, field.name),
                            ty: lowerType(field.ty.as_ref().expect("no type"), &self.program),
                        };
                        let destVar = if index == fields.len() - 1 {
                            dest.clone()
                        } else {
                            tmpVariable.clone()
                        };
                        let structName = receiverTy.getName().expect("no name for field ty");
                        let s = self.program.structs.get(&structName).expect("structDef not found");
                        let (_, index) = s.getField(&field.name.name());

                        block
                            .instructions
                            .push(Instruction::GetFieldRef(destVar, currentReceiver, index));
                        currentReceiver = tmpVariable;
                        receiverTy = field.ty.as_ref().expect("no type for field ref");
                    }
                }
                HirInstructionKind::FieldAssign(dest, root, fields) => {
                    let mut indices = Vec::new();
                    let mut ty = dest.ty.as_ref().expect("no type");
                    for field in fields {
                        let structName = ty.getName().expect("no name for field ref root");
                        let c = self.program.structs.get(&structName).expect("structDef not found");
                        let (_, index) = c.getField(&field.name.name());
                        indices.push(index);
                        ty = field.ty.as_ref().expect("field without ty!");
                    }
                    let root = self.buildVariable(root);
                    let dest = self.buildVariable(dest);
                    block.instructions.push(Instruction::SetField(dest, root, indices));
                }
                HirInstructionKind::Ref(dest, arg) => {
                    let dest = self.buildVariable(dest);
                    let arg = self.buildVariable(arg);
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block.instructions.push(Instruction::Reference(dest, arg));
                }
                HirInstructionKind::BlockStart(_) => {}
                HirInstructionKind::BlockEnd(_) => {}
                k => panic!("NYI {}", k),
            }
        }
        Some(block)
    }

    fn lowerFunction(&mut self) -> Option<MirFunction> {
        //println!("Lowering {}", self.function.name);
        let mut args = Vec::new();
        for arg in &self.function.params {
            let arg = MirParam {
                name: format!("{}", arg.getName()),
                ty: lowerType(&arg.getType(), &self.program),
            };
            args.push(arg);
        }

        let kind = match self.function.kind {
            FunctionKind::StructCtor => MirFunctionKind::StructCtor,
            FunctionKind::UserDefined | FunctionKind::TraitMemberDefinition(_) => {
                let mut blocks = Vec::new();
                if let Some(body) = self.function.body.clone() {
                    for block in &body.blocks {
                        if let Some(mirBlock) = self.lowerBlock(block) {
                            blocks.push(mirBlock);
                        }
                    }
                }
                MirFunctionKind::UserDefined(blocks)
            }
            FunctionKind::VariantCtor(i) => {
                if self.function.name.base() == getTrueName() {
                    return None;
                }
                if self.function.name.base() == getFalseName() {
                    return None;
                }
                MirFunctionKind::VariantCtor(i)
            }
            FunctionKind::Extern => {
                if self.function.name.base() == getNativePtrToRefName() {
                    return None;
                }
                MirFunctionKind::Extern
            }
            FunctionKind::TraitMemberDecl(_) => return None,
        };
        let mirFunction = MirFunction {
            name: convertName(&self.function.name),
            args: args,
            result: lowerType(&self.function.result, &self.program),
            kind: kind,
        };
        Some(mirFunction)
    }
}

pub fn convertName<T: ToString>(name: &T) -> String {
    format!(
        "{}",
        name.to_string()
            .replace(".", "_")
            .replace("(", "_t_")
            .replace(")", "_t_")
            .replace(",", "_")
            .replace(" ", "_")
            .replace("*", "s")
            .replace("#", "_")
            .replace("/", "_")
            .replace("[", "_")
            .replace("]", "_")
            .replace("&", "_r_")
    )
}

pub fn lowerType(ty: &HirType, program: &HirProgram) -> MirType {
    match ty {
        HirType::Named(name, _) => {
            if program.structs.get(name).is_some() {
                if name.base() == getIntTypeName() {
                    MirType::Int64
                } else if name.base() == getU8TypeName() {
                    MirType::UInt8
                } else if name.base() == getI32TypeName() {
                    MirType::Int32
                } else {
                    MirType::Struct(convertName(name))
                }
            } else {
                if name.base() == getBoolTypeName() {
                    MirType::Int64
                } else {
                    MirType::Union(convertName(name))
                }
            }
        }
        HirType::Tuple(_) => unreachable!("Tuple in MIR"),
        HirType::Function(_, _) => todo!(),
        HirType::Var(_) => unreachable!("Type variable in MIR"),
        HirType::Reference(ty, _) => MirType::Ptr(Box::new(lowerType(ty, program))),
        HirType::Ptr(ty) => MirType::Ptr(Box::new(lowerType(ty, program))),
        HirType::SelfType => todo!(),
        HirType::Never(_) => MirType::Void,
        HirType::OwnershipVar(_, _, _) => {
            panic!("OwnershipVar found in lowerType {}", ty);
        }
    }
}

pub fn lowerStruct(c: &HirStruct, program: &HirProgram) -> Struct {
    //println!("Lowering structDef {}", c.name);
    let mut fields = Vec::new();
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
        payloadSize: 0,
    }
}

pub fn lowerProgram(program: &HirProgram) -> MirProgram {
    let mut mirProgram = MirProgram::new();

    //println!("Lowering structs");

    for (n, c) in &program.structs {
        if n.base() == getIntTypeName() {
            continue;
        }
        if n.base() == getU8TypeName() {
            continue;
        }
        let c = lowerStruct(c, program);
        mirProgram.structs.insert(convertName(n), c);
    }

    //println!("Lowering enums");

    for (n, e) in &program.enums {
        if n.base() == getBoolTypeName() {
            continue;
        }
        let u = lowerEnum(e, program);
        mirProgram.unions.insert(convertName(n), u);
    }

    //println!("Lowering functions");

    for (_, function) in &program.functions {
        let mut builder = Builder::new(program, function);
        if let Some(f) = builder.lowerFunction() {
            mirProgram.functions.push(f);
        }
    }

    mirProgram
}
