use crate::siko::{
    hir::{
        Data::Class as HirClass,
        Function::{Function as HirFunction, InstructionKind as HirInstructionKind},
        Program::Program as HirProgram,
        Type::Type as HirType,
    },
    mir::{
        Data::{Class as MirClass, Field as MirField},
        Function::{
            Aligment, AllocInfo, BasicBlock, Function as MirFunction, Instruction, InstructionKind,
            Variable,
        },
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
};

pub fn lowerFunction(function: &HirFunction) -> MirFunction {
    let result = lowerType(&function.result);
    let mut mir_function = MirFunction::new(function.name.clone(), result.clone());
    let mut block = BasicBlock::new();
    if function.name.toString() == "Int.Int" {
        let var1 = Variable {
            name: "%1".to_string(),
            ty: MirType::I32,
            alignment: Aligment { alignment: 4 },
        };
        let var2 = Variable {
            name: "%2".to_string(),
            ty: result,
            alignment: Aligment { alignment: 4 },
        };
        let info = AllocInfo { var: var1.clone() };
        block
            .instructions
            .push(Instruction::new(InstructionKind::Allocate(info)));
        block
            .instructions
            .push(Instruction::new(InstructionKind::LoadVar(
                var2.clone(),
                var1,
            )));
        block
            .instructions
            .push(Instruction::new(InstructionKind::Return(var2)));
    } else {
        for instruction in function.instructions() {
            let idVar = format!("%{}", instruction.id.getId());
            match &instruction.kind {
                HirInstructionKind::FunctionCall(name, _) => {
                    let ty = lowerType(instruction.ty.as_ref().expect("no ty"));
                    let var = Variable {
                        name: idVar,
                        ty: ty,
                        alignment: Aligment { alignment: 4 },
                    };
                    block
                        .instructions
                        .push(Instruction::new(InstructionKind::FunctionCall(
                            var,
                            name.clone(),
                        )));
                }
                HirInstructionKind::Tuple(_) => {}
                HirInstructionKind::Drop(_) => {}
                k => panic!("NYI {}", k),
            }
        }
        block
            .instructions
            .push(Instruction::new(InstructionKind::ReturnVoid));
    }
    mir_function.blocks.push(block);
    mir_function
}

pub fn lowerType(ty: &HirType) -> MirType {
    match ty {
        HirType::Named(name, vec, lifetime_info) => MirType::Named(name.clone()),
        HirType::Tuple(vec) => MirType::Void,
        HirType::Function(vec, _) => todo!(),
        HirType::Var(type_var) => todo!(),
        HirType::Reference(_, lifetime) => todo!(),
        HirType::SelfType => todo!(),
        HirType::Never => todo!(),
    }
}

pub fn lowerClass(c: &HirClass) -> MirClass {
    let mut mirClass = MirClass::new(c.name.clone());
    if c.name.toString() == "Int.Int" {
        mirClass
            .fields
            .push(MirField::new("value".to_string(), MirType::I32));
    }
    for f in &c.fields {
        let mirField = MirField::new(f.name.clone(), lowerType(&f.ty));
        mirClass.fields.push(mirField);
    }
    mirClass
}

pub fn lowerProgram(program: &HirProgram) -> MirProgram {
    let mut mir_program = MirProgram::new();

    for (_, c) in &program.classes {
        let c = lowerClass(c);
        mir_program.classes.push(c);
    }

    for (_, function) in &program.functions {
        let f = lowerFunction(function);
        mir_program.functions.push(f);
    }

    mir_program
}
