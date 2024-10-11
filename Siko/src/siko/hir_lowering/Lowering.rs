use crate::siko::{
    hir::{
        Data::Class as HirClass,
        Function::{Function as HirFunction, InstructionKind as HirInstructionKind},
        Program::Program as HirProgram,
        Type::Type as HirType,
    },
    mir::{
        self,
        Data::{Class as MirClass, Field as MirField},
        Function::{
            Aligment, BasicBlock, Function as MirFunction, Instruction, InstructionKind, Variable,
        },
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
};

pub fn lowerFunction(function: &HirFunction) -> MirFunction {
    let mut mir_function = MirFunction::new(function.name.clone());
    let mut block = BasicBlock::new();
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
