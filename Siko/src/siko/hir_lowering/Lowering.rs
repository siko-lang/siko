use crate::siko::{
    hir::{
        Data::Class as HirClass,
        Function::{Function as HirFunction, InstructionKind as HirInstructionKind},
        Program::Program as HirProgram,
        Type::Type as HirType,
    },
    mir::{
        Data::{Field as MirField, Struct},
        Function::{Block, Function as MirFunction, Instruction, Value, Variable},
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
    let mut mir_function = MirFunction {
        name: convertName(&function.name),
        args: Vec::new(),
        result: lowerType(&function.result),
        blocks: Vec::new(),
    };
    (function.name.clone(), result.clone());
    let mut block = Block {
        id: format!("block0"),
        instructions: Vec::new(),
    };
    if function.name.toString() == "Int.Int" {
        let var1 = Variable {
            name: "%1".to_string(),
            ty: MirType::Int64,
        };
        block
            .instructions
            .push(Instruction::StackAllocate(var1.clone()));
        block
            .instructions
            .push(Instruction::Return(Value::Var(var1)));
    } else {
        for instruction in function.instructions() {
            let idVar = format!("%{}", instruction.id.getId());
            match &instruction.kind {
                HirInstructionKind::FunctionCall(name, _) => {
                    let ty = lowerType(instruction.ty.as_ref().expect("no ty"));
                }
                HirInstructionKind::Tuple(_) => {}
                HirInstructionKind::Drop(_) => {}
                k => panic!("NYI {}", k),
            }
        }
        block.instructions.push(Instruction::Return(Value::Void));
    }
    mir_function.blocks.push(block);
    mir_function
}

pub fn lowerType(ty: &HirType) -> MirType {
    match ty {
        HirType::Named(name, vec, lifetime_info) => MirType::Struct(name.toString()),
        HirType::Tuple(vec) => MirType::Void,
        HirType::Function(vec, _) => todo!(),
        HirType::Var(type_var) => todo!(),
        HirType::Reference(_, lifetime) => todo!(),
        HirType::SelfType => todo!(),
        HirType::Never => todo!(),
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
