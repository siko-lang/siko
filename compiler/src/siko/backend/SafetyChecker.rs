use crate::siko::{
    hir::{
        Function::FunctionKind,
        Instruction::{Instruction, InstructionKind},
        Program::Program,
    },
    location::Report::{Report, ReportContext},
};

pub struct SafetyChecker<'a> {
    program: &'a Program,
}

impl<'a> SafetyChecker<'a> {
    pub fn new(program: &'a Program) -> SafetyChecker<'a> {
        SafetyChecker { program }
    }

    pub fn check(&self, reportCtx: &ReportContext) {
        let mut failed = false;
        for (name, f) in &self.program.functions {
            let location = if let FunctionKind::UserDefined(loc) = &f.kind {
                loc.clone()
            } else {
                continue;
            };
            let body = match &f.body {
                Some(b) => b,
                None => continue,
            };
            let mut containsUnsafe = false;
            for blockId in body.blocks.keys() {
                let block = f.getBlockById(*blockId);
                for instr in &block.getInner().borrow().instructions {
                    if !self.isSafeInstruction(instr) {
                        containsUnsafe = true;
                        break;
                    }
                }
            }
            if containsUnsafe && f.isRegular() {
                let report = Report::new(
                    reportCtx,
                    format!(
                        "Function {} contains unsafe instructions but is not marked as unsafe or safe",
                        reportCtx.yellow(&name.toString())
                    ),
                    Some(location.clone()),
                );
                report.print();
                failed = true;
            }
            if !containsUnsafe && !f.isRegular() {
                let report = Report::new(
                    reportCtx,
                    format!(
                        "Function {} does not contain unsafe instructions and should not be marked as {}",
                        reportCtx.yellow(&name.toString()),
                        if f.isSafe() { "safe" } else { "unsafe" }
                    ),
                    Some(location.clone()),
                );
                report.print();
                failed = true;
            }
        }
        if failed {
            std::process::exit(1);
        }
    }

    pub fn isSafeInstruction(&self, instr: &Instruction) -> bool {
        match &instr.kind {
            InstructionKind::FunctionCall(_, info) => {
                let f = self.program.functions.get(&info.name).expect("Function not found");
                if f.kind.isExtern() {
                    return false;
                }
                f.isSafe() || f.isRegular()
            }
            InstructionKind::Converter(_, _) => {
                unreachable!("Converter should not appear in safety checker")
            }
            InstructionKind::MethodCall(_, _, _, _) => {
                unreachable!("Method call should not appear in safety checker")
            }
            InstructionKind::DynamicFunctionCall(_, _, _) => true,
            InstructionKind::FieldRef(_, _, _) => true,
            InstructionKind::Bind(_, _, _) => true,
            InstructionKind::Tuple(_, _) => true,
            InstructionKind::StringLiteral(_, _) => true,
            InstructionKind::IntegerLiteral(_, _) => true,
            InstructionKind::CharLiteral(_, _) => true,
            InstructionKind::Return(_, _) => true,
            InstructionKind::Ref(_, _) => true,
            InstructionKind::PtrOf(_, _) => false,
            InstructionKind::DropPath(_) => {
                unreachable!("DropPath should not appear in safety checker")
            }
            InstructionKind::DropMetadata(_) => {
                unreachable!("DropMetadata should not appear in safety checker")
            }
            InstructionKind::Drop(_, _) => {
                unreachable!("Drop should not appear in safety checker")
            }
            InstructionKind::Jump(_, _) => true,
            InstructionKind::Assign(_, _) => true,
            InstructionKind::FieldAssign(_, _, _) => true,
            InstructionKind::AddressOfField(_, _, _) => false,
            InstructionKind::DeclareVar(_, _) => true,
            InstructionKind::Transform(_, _, _) => true,
            InstructionKind::EnumSwitch(_, _) => true,
            InstructionKind::IntegerSwitch(_, _) => true,
            InstructionKind::BlockStart(_) => true,
            InstructionKind::BlockEnd(_) => true,
            InstructionKind::With(_, _) => true,
            InstructionKind::ReadImplicit(_, _) => true,
            InstructionKind::WriteImplicit(_, _) => true,
            InstructionKind::LoadPtr(_, _) => false,
            InstructionKind::StorePtr(_, _) => false,
            InstructionKind::CreateClosure(_, _) => true,
            InstructionKind::ClosureReturn(_, _, _) => true,
            InstructionKind::IntegerOp(_, _, _, _) => true,
            InstructionKind::Yield(_, _) => true,
        }
    }
}
