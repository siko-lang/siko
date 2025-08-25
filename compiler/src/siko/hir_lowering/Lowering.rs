use base64::engine::general_purpose;
use base64::Engine;
use core::panic;

fn base64(s: &str) -> String {
    let encoded = general_purpose::STANDARD.encode(s);
    encoded
}

use crate::siko::{
    backend::RemoveTuples::getUnitTypeName,
    hir::{
        Data::{Enum as HirEnum, Struct as HirStruct},
        Function::{Block, BlockId, ExternKind, Function as HirFunction, FunctionKind},
        Instruction::InstructionKind as HirInstructionKind,
        Program::Program as HirProgram,
        Type::Type as HirType,
        Variable::Variable,
    },
    mir::{
        Data::{Field as MirField, Struct, Union, Variant as MirVariant},
        Function::{
            Block as MirBlock, EnumCase as MirEnumCase, ExternKind as MirExternKind, Function as MirFunction,
            FunctionKind as MirFunctionKind, Instruction, IntegerCase as MirIntegerCase, Param as MirParam,
            Variable as MirVariable,
        },
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
    qualifiedname::{
        builtins::{
            getBoolTypeName, getFalseName, getI32TypeName, getI8TypeName, getIntTypeName, getTrueName, getU64TypeName,
            getU8TypeName,
        },
        QualifiedName,
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
        let name = convertName(&id.name());
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
                HirInstructionKind::FunctionCall(dest, info) => {
                    let f = self.program.getFunction(&info.name).expect("Function not found");
                    if info.name == getTrueName() {
                        let dest = self.buildVariable(dest);
                        block.instructions.push(Instruction::Declare(dest.clone()));
                        block
                            .instructions
                            .push(Instruction::IntegerLiteral(dest, "1".to_string()));
                        continue;
                    }
                    if info.name == getFalseName() {
                        let dest = self.buildVariable(dest);
                        block.instructions.push(Instruction::Declare(dest.clone()));
                        block
                            .instructions
                            .push(Instruction::IntegerLiteral(dest, "0".to_string()));
                        continue;
                    }
                    let fnName = if f.kind.isCtor() || f.kind.isExternC() {
                        convertName(&f.name)
                    } else {
                        convertFunctionName(&info.name)
                    };
                    let args = info.args.iter().map(|var| self.buildVariable(var)).collect();
                    if dest.getType().isNever() || (f.kind.isExternC() && *dest.getType() == getUnitTypeName()) {
                        block.instructions.push(Instruction::Call(None, fnName, args));
                    } else {
                        let dest = self.buildVariable(dest);
                        block.instructions.push(Instruction::Declare(dest.clone()));
                        block.instructions.push(Instruction::Call(Some(dest), fnName, args));
                    }
                }
                HirInstructionKind::Tuple(_, _) => {
                    unreachable!("tuples in MIR??")
                }
                HirInstructionKind::Drop(_, _) => unreachable!("drop in MIR??"),
                HirInstructionKind::DropMetadata(_) => {
                    unreachable!("drop metadata in MIR??")
                }
                HirInstructionKind::DropPath(_) => {
                    unreachable!("drop path in MIR??")
                }
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
                HirInstructionKind::Jump(_, blockId) => {
                    block.instructions.push(Instruction::Jump(self.getBlockName(*blockId)));
                }
                HirInstructionKind::Return(_, v) => {
                    block.instructions.push(Instruction::Return(self.buildVariable(v)));
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
                HirInstructionKind::CharLiteral(dest, v) => {
                    let dest = self.buildVariable(dest);
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block
                        .instructions
                        .push(Instruction::IntegerLiteral(dest, v.to_string()));
                }
                HirInstructionKind::EnumSwitch(root, cases) => {
                    if root.getType().getName().unwrap() == getBoolTypeName() {
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
                    if root.getType().getName().unwrap() == getBoolTypeName() {
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
                            name: format!("{}_{}_{}", root.name(), index, field.name),
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
                    let mut ty = dest.getType();
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
                HirInstructionKind::PtrOf(dest, arg) => {
                    let dest = self.buildVariable(dest);
                    let arg = self.buildVariable(arg);
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block.instructions.push(Instruction::Reference(dest, arg));
                }
                HirInstructionKind::BlockStart(_) => {}
                HirInstructionKind::BlockEnd(_) => {}
                HirInstructionKind::AddressOfField(dest, root, fields) => {
                    let dest = self.buildVariable(dest);
                    let mut receiverTy = root.getType();
                    let root = self.buildVariable(root);
                    let mut currentReceiver = root.clone();
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    for (index, field) in fields.iter().enumerate() {
                        if let HirType::Ptr(inner) = receiverTy {
                            receiverTy = inner;
                        }
                        let structName = receiverTy.getName().expect("no name for field ref root");
                        let c = self.program.structs.get(&structName).expect("structDef not found");
                        let (_, findex) = c.getField(&field.name.name());
                        receiverTy = field.ty.as_ref().expect("no type for field ref");

                        let tmpVariable = MirVariable {
                            name: format!("{}_{}_{}", root.name, index, field.name),
                            ty: lowerType(field.ty.as_ref().expect("no type"), &self.program),
                        };
                        let destVar = if index == fields.len() - 1 {
                            dest.clone()
                        } else {
                            tmpVariable.clone()
                        };

                        if index == fields.len() - 1 {
                            block.instructions.push(Instruction::AddressOfField(
                                destVar.clone(),
                                currentReceiver,
                                findex,
                            ));
                        } else {
                            block
                                .instructions
                                .push(Instruction::GetFieldRef(destVar.clone(), currentReceiver, findex));
                        }
                        currentReceiver = destVar;
                    }
                }
                HirInstructionKind::Converter(_, _) => {
                    panic!("Converter instruction found in Lowering, this should not happen");
                }
                HirInstructionKind::MethodCall(_, _, _, _) => {
                    panic!("MethodCall instruction found in Lowering, this should not happen");
                }
                HirInstructionKind::DynamicFunctionCall(_, _, _) => {
                    panic!("DynamicFunctionCall instruction found in Lowering, this should not happen");
                }
                HirInstructionKind::With(_, _) => {
                    panic!("With instruction found in Lowering, this should not happen");
                }
                HirInstructionKind::ReadImplicit(_, _) => {
                    panic!("GetImplicit instruction found in Lowering, this should not happen");
                }
                HirInstructionKind::WriteImplicit(_, _) => {
                    panic!("WriteImplicit instruction found in Lowering, this should not happen");
                }
                HirInstructionKind::LoadPtr(dest, src) => {
                    let dest = self.buildVariable(dest);
                    let src = self.buildVariable(src);
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block.instructions.push(Instruction::LoadPtr(dest, src));
                }
                HirInstructionKind::StorePtr(dest, src) => {
                    let dest = self.buildVariable(dest);
                    let src = self.buildVariable(src);
                    block.instructions.push(Instruction::StorePtr(dest, src));
                }
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
            FunctionKind::UserDefined
            | FunctionKind::TraitMemberDefinition(_)
            | FunctionKind::ProtocolMemberDefinition(_)
            | FunctionKind::EffectMemberDefinition(_) => {
                let mut blocks = Vec::new();
                if let Some(body) = self.function.body.clone() {
                    for (_, block) in &body.blocks {
                        if let Some(mirBlock) = self.lowerBlock(block) {
                            blocks.push(mirBlock);
                        }
                    }
                }
                MirFunctionKind::UserDefined(blocks)
            }
            FunctionKind::VariantCtor(i) => {
                if self.function.name == getTrueName() {
                    return None;
                }
                if self.function.name == getFalseName() {
                    return None;
                }
                MirFunctionKind::VariantCtor(i)
            }
            FunctionKind::Extern(kind) => {
                let mirKind = match kind {
                    ExternKind::C => {
                        let name = self.function.name.getShortName();
                        MirExternKind::C(name)
                    }
                    ExternKind::Builtin => MirExternKind::Builtin,
                };
                MirFunctionKind::Extern(mirKind)
            }
            FunctionKind::TraitMemberDecl(_) => {
                unreachable!("TraitMemberDecl in MIR Lowering")
            }
            FunctionKind::EffectMemberDecl(_) => {
                unreachable!("EffectMemberDecl in MIR Lowering")
            }
            FunctionKind::ProtocolMemberDecl(_) => {
                unreachable!("ProtocolMemberDecl in MIR Lowering")
            }
        };
        let fnName = if self.function.kind.isCtor() || self.function.kind.isExternC() {
            convertName(&self.function.name)
        } else {
            convertFunctionName(&self.function.name)
        };
        let mirFunction = MirFunction {
            name: fnName,
            fullName: self.function.name.to_string(),
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
            .replace("{", "")
            .replace("}", "")
            .replace(",", "_")
            .replace(" ", "_")
            .replace("*", "s")
            .replace("#", "_")
            .replace("/", "_")
            .replace("[", "_")
            .replace("]", "_")
            .replace("&", "_r_")
            .replace(">", "_l_")
            .replace("-", "_minus_")
            .replace(":", "_colon_")
    )
}

pub fn convertFunctionName(name: &QualifiedName) -> String {
    let (base, context) = name.split();
    let c = base64(&context.to_string()).replace('=', "").replace("+", "");
    if c.is_empty() {
        convertName(&base)
    } else {
        convertName(&base) + "_" + &c
    }
}

pub fn lowerType(ty: &HirType, program: &HirProgram) -> MirType {
    match ty {
        HirType::Named(name, _) => {
            if program.structs.get(name).is_some() {
                if *name == getIntTypeName() {
                    MirType::Int64
                } else if *name == getU8TypeName() {
                    MirType::UInt8
                } else if *name == getI8TypeName() {
                    MirType::Int8
                } else if *name == getI32TypeName() {
                    MirType::Int32
                } else if *name == getU64TypeName() {
                    MirType::UInt64
                } else {
                    MirType::Struct(convertName(name))
                }
            } else {
                if *name == getBoolTypeName() {
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
        if *n == getIntTypeName() {
            continue;
        }
        if *n == getU8TypeName() {
            continue;
        }
        let c = lowerStruct(c, program);
        mirProgram.structs.insert(convertName(n), c);
    }

    //println!("Lowering enums");

    for (n, e) in &program.enums {
        if *n == getBoolTypeName() {
            continue;
        }
        let u = lowerEnum(e, program);
        mirProgram.unions.insert(convertName(n), u);
    }

    //println!("Lowering functions");

    for (_, function) in &program.functions {
        let mut builder = Builder::new(program, function);
        if let Some(f) = builder.lowerFunction() {
            mirProgram.functions.insert(f.name.clone(), f);
        }
    }

    mirProgram
}
