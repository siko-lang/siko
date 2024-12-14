use crate::siko::{
    hir::{
        Data::{Class as HirClass, Enum as HirEnum},
        Function::{Block, BlockId, Function as HirFunction, FunctionKind, InstructionKind as HirInstructionKind, Variable},
        Program::Program as HirProgram,
        Type::Type as HirType,
    },
    mir::{
        Data::{Field as MirField, Struct, Union, Variant as MirVariant},
        Function::{
            Block as MirBlock, EnumCase as MirEnumCase, Function as MirFunction, FunctionKind as MirFunctionKind, Instruction,
            IntegerCase as MirIntegerCase, Param as MirParam, Value, Variable as MirVariable,
        },
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
    qualifiedname::{getIntTypeName, getPtrToRefName, QualifiedName},
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
        let name = format!("{}", id.value);
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
            if let HirInstructionKind::Drop(_) = instruction.kind {
                continue;
            }
            match &instruction.kind {
                HirInstructionKind::FunctionCall(dest, name, args) => {
                    if name.base() == getPtrToRefName() {
                        let dest = self.buildVariable(dest);
                        let arg = &args[0];
                        block.instructions.push(Instruction::Declare(dest.clone()));
                        block.instructions.push(Instruction::Memcpy(self.buildVariable(arg), dest.clone()));
                    } else {
                        let args = args.iter().map(|var| self.buildVariable(var)).collect();
                        let dest = self.buildVariable(dest);
                        block.instructions.push(Instruction::Declare(dest.clone()));
                        block.instructions.push(Instruction::Call(dest, convertName(name), args));
                    }
                }
                HirInstructionKind::Tuple(_, _) => {
                    unreachable!("tuples in MIR??")
                }
                HirInstructionKind::Drop(_) => {}
                HirInstructionKind::DeclareVar(var) => {
                    let var = self.buildVariable(var);
                    block.instructions.push(Instruction::Declare(var.clone()));
                }
                HirInstructionKind::ValueRef(dest, name) => {
                    let dest = self.buildVariable(dest);
                    let var = MirVariable {
                        name: name.value.clone(),
                        ty: dest.ty.clone(),
                    };
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block.instructions.push(Instruction::Store(var, dest));
                }
                HirInstructionKind::Assign(name, rhs) => {
                    let rhs = self.buildVariable(rhs);
                    let var = MirVariable {
                        name: name.value.clone(),
                        ty: rhs.ty.clone(),
                    };
                    block.instructions.push(Instruction::Memcpy(rhs, var));
                }
                HirInstructionKind::Bind(var, rhs, _) => {
                    let rhs = self.buildVariable(rhs);
                    let var = self.buildVariable(var);
                    block.instructions.push(Instruction::Declare(var.clone()));
                    block.instructions.push(Instruction::Memcpy(rhs, var));
                }
                HirInstructionKind::Jump(_, blockId) => {
                    block.instructions.push(Instruction::Jump(self.getBlockName(*blockId)));
                }
                HirInstructionKind::Return(_, v) => {
                    block.instructions.push(Instruction::Return(Value::Var(self.buildVariable(v))));
                }
                HirInstructionKind::IntegerLiteral(dest, v) => {
                    let dest = self.buildVariable(dest);
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block.instructions.push(Instruction::IntegerLiteral(dest, v.to_string()));
                }
                HirInstructionKind::StringLiteral(dest, v) => {
                    let dest = self.buildVariable(dest);
                    block.instructions.push(Instruction::Declare(dest.clone()));
                    block.instructions.push(Instruction::StringLiteral(dest, v.to_string()));
                }
                HirInstructionKind::EnumSwitch(root, cases) => {
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
                    let dest = self.buildVariable(dest);
                    let root = self.buildVariable(root);
                    block.instructions.push(Instruction::Transform(dest, root, *index));
                }
                HirInstructionKind::TupleIndex(dest, root, index) => {
                    let dest = self.buildVariable(dest);
                    let root = self.buildVariable(root);
                    block.instructions.push(Instruction::GetFieldRef(dest, root, *index));
                }
                HirInstructionKind::FieldRef(dest, root, name) => {
                    let dest = self.buildVariable(dest);
                    let className = root.ty.as_ref().expect("no type").getName().expect("no name for field ref root");
                    let c = self.program.classes.get(&className).expect("class not found");
                    let (_, index) = c.getField(name);
                    let root = self.buildVariable(root);
                    block.instructions.push(Instruction::GetFieldRef(dest, root, index));
                }
                HirInstructionKind::FieldAssign(dest, root, fields) => {
                    let mut indices = Vec::new();
                    let mut ty = dest.ty.as_ref().expect("no type");
                    for field in fields {
                        let className = ty.getName().expect("no name for field ref root");
                        let c = self.program.classes.get(&className).expect("class not found");
                        let (_, index) = c.getField(&field.name);
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
            FunctionKind::ClassCtor => MirFunctionKind::ClassCtor,
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
            FunctionKind::VariantCtor(i) => MirFunctionKind::VariantCtor(i),
            FunctionKind::Extern => {
                if self.function.name.base() == getPtrToRefName() {
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

pub fn convertName(name: &QualifiedName) -> String {
    format!(
        "{}",
        name.toString()
            .replace(".", "_")
            .replace("(", "")
            .replace(")", "")
            .replace(",", "_")
            .replace(" ", "_")
            .replace("#", "_")
            .replace("/", "_")
            .replace("[", "_")
            .replace("]", "_")
            .replace("&", "_r_")
    )
}

pub fn lowerType(ty: &HirType, program: &HirProgram) -> MirType {
    match ty {
        HirType::Named(name, _, _) => {
            if program.classes.get(name).is_some() {
                if name.base() == getIntTypeName() {
                    MirType::Int64
                } else {
                    MirType::Struct(convertName(name))
                }
            } else {
                MirType::Union(convertName(name))
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

pub fn lowerClass(c: &HirClass, program: &HirProgram) -> Struct {
    //println!("Lowering class {}", c.name);
    let mut fields = Vec::new();
    if c.name.toString() == "Int.Int" {
        fields.push(MirField {
            name: "value".to_string(),
            ty: MirType::Int64,
        });
    }
    if c.name.toString() == "String.String" {
        fields.push(MirField {
            name: "value".to_string(),
            ty: MirType::Ptr(Box::new(MirType::Int8)),
        });
        fields.push(MirField {
            name: "length".to_string(),
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
        payloadSize: 0,
    }
}

pub fn lowerProgram(program: &HirProgram) -> MirProgram {
    let mut mirProgram = MirProgram::new();

    //println!("Lowering classes");

    for (n, c) in &program.classes {
        let c = lowerClass(c, program);
        mirProgram.structs.insert(convertName(n), c);
    }

    //println!("Lowering enums");

    for (n, e) in &program.enums {
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
