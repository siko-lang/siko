use crate::siko::{
    hir::{
        Block::BlockId,
        Body::Body,
        ConstraintContext::ConstraintContext,
        Function::{Function, FunctionKind, Parameter},
        Instruction::InstructionKind,
        Type::{Type, TypeVar},
        Unifier::Unifier,
        Variable::VariableName,
    },
    typechecker::Typechecker::ClosureTypeInfo,
};

fn getArgName(index: u32) -> String {
    format!("arg{}", index)
}

pub struct ClosureSeparator<'a> {
    function: &'a mut Function,
    closureEntry: BlockId,
    closureTypeInfo: &'a ClosureTypeInfo,
    closureBody: Body,
    unifier: &'a mut Unifier,
    typeVars: Vec<TypeVar>,
}

impl<'a> ClosureSeparator<'a> {
    pub fn new(
        function: &'a mut Function,
        closureEntry: BlockId,
        closureTypeInfo: &'a ClosureTypeInfo,
        unifier: &'a mut Unifier,
    ) -> Self {
        ClosureSeparator {
            function,
            closureEntry,
            closureTypeInfo,
            closureBody: Body::new(),
            unifier,
            typeVars: Vec::new(),
        }
    }

    pub fn process(&mut self) -> Function {
        self.processBlock(self.closureEntry);
        let mut params = Vec::new();
        let mut argIndex = 0;
        for arg in &self.closureTypeInfo.envTypes {
            let arg_name = getArgName(argIndex);
            let arg = self.unifier.apply(arg.clone());
            let param = Parameter::Named(arg_name, arg, false);
            params.push(param);
            argIndex += 1;
        }
        for arg in &self.closureTypeInfo.argTypes {
            let arg_name = getArgName(argIndex);
            let arg = self.unifier.apply(arg.clone());
            let param = Parameter::Named(arg_name, arg, false);
            params.push(param);
            argIndex += 1;
        }
        let mut constraintContext = ConstraintContext::new();
        for tyVar in self.typeVars.drain(..) {
            let tyVar = Type::Var(tyVar);
            if !constraintContext.typeParameters.contains(&tyVar) {
                constraintContext.addTypeParam(tyVar);
            }
        }
        let resultTy = self
            .closureTypeInfo
            .resultType
            .clone()
            .expect("Closure must have result type");
        let resultTy = self.unifier.apply(resultTy);
        // println!("Closure result type: {}", resultTy);
        // println!("Constraint context: {}", constraintContext);
        let closureFn = Function::new(
            self.closureTypeInfo
                .name
                .as_ref()
                .expect("Closure must have name")
                .clone(),
            params,
            resultTy,
            Some(self.closureBody.clone()),
            constraintContext,
            FunctionKind::UserDefined,
        );
        // println!("Closure function created: {}", closureFn);
        closureFn
    }

    fn processBlock(&mut self, blockId: BlockId) {
        let mut block = self.function.getBlockById(blockId).clone();
        for instructionIndex in 0..block.size() {
            let instr = block.getInstruction(instructionIndex);
            let allVars = instr.kind.collectVariables();
            let mut typeVars = Vec::new();
            for var in allVars {
                let var = self.unifier.apply(var);
                let ty = var.getType();
                typeVars = ty.collectVarsStable(typeVars);
            }
            self.typeVars.append(&mut typeVars);
            match instr.kind {
                InstructionKind::Assign(lhs, rhs) => match rhs.name() {
                    VariableName::ClosureArg(_, index) => {
                        let arg_name = getArgName(index);
                        let rhs = rhs.cloneInto(VariableName::Arg(arg_name));
                        block.replace(
                            instructionIndex,
                            InstructionKind::Assign(lhs, rhs),
                            instr.location,
                            instr.implicit,
                        );
                    }
                    VariableName::LambdaArg(_, index) => {
                        let arg_name = getArgName(self.closureTypeInfo.envTypes.len() as u32 + index);
                        let rhs = rhs.cloneInto(VariableName::Arg(arg_name));
                        block.replace(
                            instructionIndex,
                            InstructionKind::Assign(lhs, rhs),
                            instr.location,
                            instr.implicit,
                        );
                    }
                    _ => {}
                },
                InstructionKind::Jump(_, target) => {
                    self.processBlock(target);
                }
                InstructionKind::EnumSwitch(_, cases) => {
                    for c in cases {
                        self.processBlock(c.branch);
                    }
                }
                InstructionKind::ClosureReturn(_, v, arg) => {
                    let kind = InstructionKind::Return(v, arg);
                    block.replace(instructionIndex, kind, instr.location, instr.implicit);
                }
                _ => {}
            }
        }
        let origBody = self.function.body.as_mut().unwrap();
        origBody.removeBlock(blockId);
        if block.getId() == self.closureEntry {
            block.setId(BlockId::first());
        }
        self.closureBody.addBlock(block.clone());
    }
}
