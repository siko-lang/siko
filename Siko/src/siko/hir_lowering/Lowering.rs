use crate::siko::{
    hir::{
        Data::Class as HirClass,
        Function::{Function as HirFunction, FunctionKind, InstructionKind as HirInstructionKind},
        Program::Program as HirProgram,
        Type::Type as HirType,
    },
    mir::{
        Data::{Field as MirField, Struct},
        Function::{
            Block, Function as MirFunction, Instruction, Param as MirParam, Value, Variable,
        },
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
    qualifiedname::QualifiedName,
};

pub fn convertName(name: &QualifiedName) -> String {
    format!("@{}", name.toString().replace(".", "_"))
}

pub fn lowerFunction(function: &HirFunction) -> MirFunction {
    let result = lowerType(&function.result);
    let mut args = Vec::new();
    for arg in &function.params {
        let arg = MirParam {
            name: format!("%{}", arg.getName()),
            ty: lowerType(&arg.getType()),
        };
        args.push(arg);
    }
    let mut mir_function = MirFunction {
        name: convertName(&function.name),
        args: args,
        result: lowerType(&function.result),
        blocks: Vec::new(),
    };
    (function.name.clone(), result.clone());
    let mut block = Block {
        id: format!("block0"),
        instructions: Vec::new(),
    };
    if function.kind == FunctionKind::ClassCtor {
        let var1 = Variable {
            name: "%1".to_string(),
            ty: MirType::Int64,
        };
        let var2 = Variable {
            name: "%2".to_string(),
            ty: MirType::Int64,
        };
        block
            .instructions
            .push(Instruction::StackAllocate(var1.clone()));
        block
            .instructions
            .push(Instruction::Reference(var2.clone(), var1.clone()));
        block
            .instructions
            .push(Instruction::Return(Value::Var(var2)));
    } else {
        if function.name.toString() == "Int.Int" {
            let var1 = Variable {
                name: "%1".to_string(),
                ty: MirType::Int64,
            };
            let var2 = Variable {
                name: "%2".to_string(),
                ty: MirType::Int64,
            };
            block
                .instructions
                .push(Instruction::StackAllocate(var1.clone()));
            block
                .instructions
                .push(Instruction::Reference(var2.clone(), var1.clone()));
            block
                .instructions
                .push(Instruction::Return(Value::Var(var2)));
        } else {
            let mut lastId = String::new();
            let mut lastTy = MirType::Void;
            for instruction in function.instructions() {
                if let HirInstructionKind::Drop(_) = instruction.kind {
                    continue;
                }
                let idVar = format!("%{}", instruction.id.getId() + 1);
                let ty = lowerType(instruction.ty.as_ref().expect("no ty"));
                lastId = idVar.clone();
                lastTy = ty.clone();
                match &instruction.kind {
                    HirInstructionKind::FunctionCall(name, args) => {
                        let args = args
                            .iter()
                            .map(|id| {
                                let i = function.getInstruction(*id);
                                let ty = lowerType(i.ty.as_ref().expect("no ty"));
                                let name = format!("%{}", id.getId() + 1);
                                Variable { name: name, ty: ty }
                            })
                            .collect();

                        block.instructions.push(Instruction::Call(
                            Variable {
                                name: idVar,
                                ty: ty,
                            },
                            convertName(name),
                            args,
                        ));
                    }
                    HirInstructionKind::Tuple(_) => {}
                    HirInstructionKind::Drop(_) => {}
                    HirInstructionKind::DeclareVar(_) => {}
                    HirInstructionKind::If(_, _, _) => {}
                    HirInstructionKind::ValueRef(_, _, _) => {}
                    HirInstructionKind::Assign(_, _) => {}
                    HirInstructionKind::Bind(_, _) => {}
                    HirInstructionKind::Jump(_) => {}
                    HirInstructionKind::Return(_) => {}
                    k => panic!("NYI {}", k),
                }
            }
            if lastTy == MirType::Void {
                block.instructions.push(Instruction::Return(Value::Void));
            } else {
                let var = Variable {
                    name: lastId,
                    ty: lastTy,
                };
                block
                    .instructions
                    .push(Instruction::Return(Value::Var(var)));
            }
        }
    }
    mir_function.blocks.push(block);
    mir_function
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
        HirType::Tuple(_) => MirType::Void,
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
        let f = lowerFunction(function);
        mir_program.functions.push(f);
    }

    mir_program
}
