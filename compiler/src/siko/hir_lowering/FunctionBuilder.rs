use crate::siko::{
    backend::RemoveTuples::getUnitTypeName,
    hir::{
        Block::{Block, BlockId},
        Function::{ExternKind, Function as HirFunction, FunctionKind},
        Instruction::{InstructionKind as HirInstructionKind, IntegerOp as HirIntegerOp},
        Type::Type as HirType,
        Variable::Variable,
    },
    hir_lowering::{Lowering::Lowering, NameManager::cleanupName},
    mir::Function::{
        Block as MirBlock, EnumCase as MirEnumCase, ExternInfo, ExternKind as MirExternKind, Function as MirFunction,
        FunctionKind as MirFunctionKind, Instruction, IntegerCase as MirIntegerCase, IntegerOp, Param as MirParam,
        Variable as MirVariable,
    },
    qualifiedname::builtins::{getBoolTypeName, getFalseName, getTrueName},
};

pub struct Builder<'a> {
    lowering: &'a Lowering,
    function: &'a HirFunction,
}

impl<'a> Builder<'a> {
    pub fn new(lowering: &'a Lowering, function: &'a HirFunction) -> Builder<'a> {
        Builder {
            lowering: lowering,
            function: function,
        }
    }

    fn buildVariable(&self, id: &Variable) -> MirVariable {
        let ty = self.lowering.lowerType(&id.getType());
        let name = cleanupName(&id.name());
        MirVariable { name: name, ty: ty }
    }

    fn getBlockName(&self, blockId: BlockId) -> String {
        format!("block{}", blockId.id)
    }

    fn lowerBlock(&mut self, hirBlock: &Block) -> Option<MirBlock> {
        if hirBlock.isEmpty() {
            return None;
        }
        let mut block = MirBlock {
            id: self.getBlockName(hirBlock.getId()),
            instructions: Vec::new(),
        };
        let inner = hirBlock.getInner();
        for instruction in &inner.borrow().instructions {
            match &instruction.kind {
                HirInstructionKind::FunctionCall(dest, info) => {
                    let f = match self.lowering.program.getFunction(&info.name) {
                        Some(f) => f,
                        None => panic!("Function not found {}", info.name),
                    };
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
                    let fnName = if f.kind.isExternC() {
                        cleanupName(&f.name)
                    } else {
                        self.lowering.nameManager.processName(&info.name)
                    };
                    let args = info.args.iter().map(|var| self.buildVariable(var)).collect();
                    if dest.getType().isNever()
                        || dest.getType().isVoid()
                        || (f.kind.isExternC() && dest.getType() == getUnitTypeName())
                    {
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
                            let value = match case.index {
                                Some(v) => Some(format!("{}", v)),
                                None => None,
                            };
                            let mirCase = MirIntegerCase {
                                value,
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
                HirInstructionKind::Transform(dest, root, info) => {
                    if root.getType().getName().unwrap() == getBoolTypeName() {
                        block.instructions.push(Instruction::Declare(self.buildVariable(dest)));
                    } else {
                        let dest = self.buildVariable(dest);
                        let root = self.buildVariable(root);
                        block
                            .instructions
                            .push(Instruction::Transform(dest, root, info.variantIndex));
                    }
                }
                HirInstructionKind::FieldRef(dest, root, fields) => {
                    let dest = self.buildVariable(dest);
                    let mut currentReceiver = self.buildVariable(root);
                    let mut receiverTy = root.getType();
                    for (index, field) in fields.iter().enumerate() {
                        let tmpVariable = MirVariable {
                            name: format!("{}_{}_{}", root.name(), index, field.name),
                            ty: self.lowering.lowerType(field.ty.as_ref().expect("no type")),
                        };
                        let destVar = if index == fields.len() - 1 {
                            dest.clone()
                        } else {
                            tmpVariable.clone()
                        };
                        let structName = receiverTy.getName().expect("no name for field ty");
                        let s = self
                            .lowering
                            .program
                            .structs
                            .get(&structName)
                            .expect("structDef not found");
                        let (_, index) = s.getField(&field.name.name());

                        block
                            .instructions
                            .push(Instruction::GetFieldRef(destVar, currentReceiver, index));
                        currentReceiver = tmpVariable;
                        receiverTy = field.ty.clone().expect("no type for field ref");
                    }
                }
                HirInstructionKind::FieldAssign(dest, root, fields) => {
                    let mut indices = Vec::new();
                    let mut ty = dest.getType();
                    for field in fields {
                        let structName = ty.getName().expect("no name for field ref root");
                        let c = self
                            .lowering
                            .program
                            .structs
                            .get(&structName)
                            .expect("structDef not found");
                        let (_, index) = c.getField(&field.name.name());
                        indices.push(index);
                        ty = field.ty.clone().expect("field without ty!");
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
                            receiverTy = *inner;
                        }
                        let structName = receiverTy.getName().expect("no name for field ref root");
                        let c = self
                            .lowering
                            .program
                            .structs
                            .get(&structName)
                            .expect("structDef not found");
                        let (_, findex) = c.getField(&field.name.name());
                        receiverTy = field.ty.clone().expect("no type for field ref");

                        let tmpVariable = MirVariable {
                            name: format!("{}_{}_{}", root.name, index, field.name),
                            ty: self.lowering.lowerType(field.ty.as_ref().expect("no type")),
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
                HirInstructionKind::CreateClosure(_, _) => {
                    panic!("CreateClosure instruction found in Lowering, this should not happen");
                }
                HirInstructionKind::ClosureReturn(_, _, _) => {
                    panic!("ClosureReturn instruction found in Lowering, this should not happen");
                }
                HirInstructionKind::IntegerOp(dest, left, right, op) => {
                    let dest = self.buildVariable(dest);
                    let left = self.buildVariable(left);
                    let right = self.buildVariable(right);
                    let mirOp = match op {
                        HirIntegerOp::Add => IntegerOp::Add,
                        HirIntegerOp::Sub => IntegerOp::Sub,
                        HirIntegerOp::Mul => IntegerOp::Mul,
                        HirIntegerOp::Div => IntegerOp::Div,
                        HirIntegerOp::Mod => IntegerOp::Mod,
                        HirIntegerOp::Eq => IntegerOp::Eq,
                        HirIntegerOp::LessThan => IntegerOp::LessThan,
                    };
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block
                        .instructions
                        .push(Instruction::IntegerOp(dest, left, right, mirOp));
                }
                HirInstructionKind::Yield(_, _) => {
                    unreachable!("Yield in MIR Lowering");
                }
            }
        }
        Some(block)
    }

    pub fn lowerFunction(&mut self) -> Option<MirFunction> {
        //println!("Lowering {}", self.function.name);
        let mut args = Vec::new();
        for arg in &self.function.params {
            let arg = MirParam {
                name: format!("{}", arg.getName()),
                ty: self.lowering.lowerType(&arg.getType()),
            };
            args.push(arg);
        }

        let kind = match self.function.kind {
            FunctionKind::StructCtor => MirFunctionKind::StructCtor,
            FunctionKind::UserDefined(_)
            | FunctionKind::TraitMemberDefinition(_)
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
            FunctionKind::Extern(ref kind) => {
                let mirKind = match &kind {
                    ExternKind::C(header) => {
                        let info = ExternInfo {
                            name: self.function.name.getShortName(),
                            headerName: header.clone(),
                        };
                        MirExternKind::C(info)
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
        };
        let fnName = if self.function.kind.isExternC() {
            cleanupName(&self.function.name)
        } else {
            self.lowering.nameManager.processName(&self.function.name)
        };
        let mirFunction = MirFunction {
            name: fnName,
            fullName: self.function.name.to_string(),
            args: args,
            result: self.lowering.lowerType(&self.function.result.getReturnType()),
            kind: kind,
        };
        Some(mirFunction)
    }
}
