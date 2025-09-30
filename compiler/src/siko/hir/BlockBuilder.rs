use core::panic;
use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::hir::Block::BlockId;
use crate::siko::hir::Instruction::Arguments;
use crate::siko::hir::Instruction::CallInfo;
use crate::siko::hir::Instruction::TransformInfo;
use crate::siko::{
    hir::Instruction::{Mutability, SyntaxBlockId},
    location::Location::Location,
    qualifiedname::QualifiedName,
};

use super::{
    BodyBuilder::BodyBuilder,
    Instruction::{FieldInfo, Instruction, InstructionKind},
    Type::Type,
    Variable::Variable,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstructionRef {
    pub blockId: BlockId,
    pub instructionId: u32,
}

impl Display for InstructionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.blockId, self.instructionId)
    }
}

impl Debug for InstructionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Copy)]
pub enum Mode {
    Append,
    Iterator(usize),
}

#[derive(Clone)]
pub struct BlockBuilder {
    bodyBuilder: BodyBuilder,
    blockId: BlockId,
    isImplicit: bool,
    mode: Mode,
}

impl BlockBuilder {
    pub fn new(blockId: BlockId, bodyBuilder: BodyBuilder, mode: Mode) -> BlockBuilder {
        BlockBuilder {
            bodyBuilder: bodyBuilder,
            blockId,
            isImplicit: false,
            mode: mode,
        }
    }

    pub fn current(&mut self) {
        self.bodyBuilder.setTargetBlockId(self.blockId);
    }

    pub fn implicit(&self) -> BlockBuilder {
        BlockBuilder {
            bodyBuilder: self.bodyBuilder.clone(),
            blockId: self.blockId,
            isImplicit: true,
            mode: self.mode,
        }
    }

    pub fn getBodyBuilder(&self) -> BodyBuilder {
        self.bodyBuilder.clone()
    }

    pub fn getInstruction(&self) -> Option<Instruction> {
        match self.mode {
            Mode::Append => panic!("Cannot get instruction in append mode"),
            Mode::Iterator(index) => self.bodyBuilder.getInstruction(self.blockId, index),
        }
    }

    pub fn getInstructionAt(&self, index: usize) -> Option<Instruction> {
        self.bodyBuilder.getInstruction(self.blockId, index)
    }

    pub fn getInstructionRef(&self) -> InstructionRef {
        InstructionRef {
            blockId: self.blockId,
            instructionId: match self.mode {
                Mode::Append => {
                    panic!("Cannot get instruction ref in append mode")
                }
                Mode::Iterator(index) => index as u32,
            },
        }
    }

    pub fn step(&mut self) {
        match self.mode {
            Mode::Append => panic!("Cannot step in append mode"),
            Mode::Iterator(index) => {
                self.mode = Mode::Iterator(index + 1);
            }
        }
    }

    pub fn stepTo(&mut self, index: usize) {
        match self.mode {
            Mode::Append => panic!("Cannot step in append mode"),
            Mode::Iterator(_) => {
                self.mode = Mode::Iterator(index);
            }
        }
    }

    pub fn iterator(&self) -> BlockBuilder {
        match self.mode {
            Mode::Append => BlockBuilder {
                bodyBuilder: self.bodyBuilder.clone(),
                blockId: self.blockId,
                isImplicit: self.isImplicit,
                mode: Mode::Iterator(self.bodyBuilder.getBlockSize(self.blockId)),
            },
            Mode::Iterator(_) => self.clone(),
        }
    }

    pub fn addInstruction(&mut self, instruction: InstructionKind, location: Location) {
        match self.mode {
            Mode::Append => {
                self.bodyBuilder
                    .addInstruction(self.blockId, instruction, location, self.isImplicit);
            }
            Mode::Iterator(index) => {
                self.bodyBuilder
                    .insertInstruction(self.blockId, index, instruction, location, self.isImplicit);
            }
        }
    }

    pub fn replaceInstruction(&mut self, instruction: InstructionKind, location: Location) {
        match self.mode {
            Mode::Append => {
                panic!("Cannot replace instruction in append mode");
            }
            Mode::Iterator(index) => {
                self.bodyBuilder
                    .replaceInstruction(self.blockId, index, instruction, location, self.isImplicit);
            }
        }
    }

    pub fn removeInstruction(&mut self) {
        match self.mode {
            Mode::Append => panic!("Cannot remove instruction in append mode"),
            Mode::Iterator(index) => {
                self.bodyBuilder.removeInstruction(self.blockId, index);
            }
        }
    }

    pub fn removeInstructionAt(&mut self, index: usize) {
        self.bodyBuilder.removeInstruction(self.blockId, index);
    }

    pub fn addAssign(&mut self, target: Variable, source: Variable, location: Location) {
        self.addInstruction(InstructionKind::Assign(target, source), location);
    }

    pub fn addReturn(&mut self, value: Variable, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValueWithType(location.clone(), Type::getNeverType());
        self.addInstruction(InstructionKind::Return(result.clone(), value), location);
        result
    }

    pub fn addClosureReturn(&mut self, blockId: BlockId, value: Variable, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValueWithType(location.clone(), Type::getNeverType());
        self.addInstruction(InstructionKind::ClosureReturn(blockId, result.clone(), value), location);
        result
    }

    pub fn addRef(&mut self, arg: Variable, location: Location) -> Variable {
        let value = self.bodyBuilder.createTempValue(location.clone());
        self.addInstruction(InstructionKind::Ref(value.clone(), arg), location.clone());
        value
    }

    pub fn addFunctionCall<T: Into<Arguments>>(
        &mut self,
        functionName: QualifiedName,
        args: T,
        location: Location,
    ) -> Variable {
        self.addFunctionCallInner(functionName, args.into(), location, false)
    }

    pub fn addCoroutineFunctionCall<T: Into<Arguments>>(
        &mut self,
        functionName: QualifiedName,
        args: T,
        location: Location,
    ) -> Variable {
        self.addFunctionCallInner(functionName, args.into(), location, true)
    }

    pub fn addFunctionCallInner(
        &mut self,
        functionName: QualifiedName,
        args: Arguments,
        location: Location,
        coroutineSpawn: bool,
    ) -> Variable {
        let mut info = CallInfo::new(functionName, self.processFunctionInfo(args, &location));
        let result = self.bodyBuilder.createTempValue(location.clone());
        info.coroutineSpawn = coroutineSpawn;
        self.addInstruction(InstructionKind::FunctionCall(result.clone(), info), location);
        result
    }

    fn processFunctionInfo(&mut self, args: Arguments, location: &Location) -> Arguments {
        match args {
            Arguments::Resolved(args) => {
                let mut tempArgs = Vec::new();
                for arg in args {
                    let tempValue = self.bodyBuilder.createTempValue(location.clone());
                    self.addInstruction(
                        InstructionKind::Converter(tempValue.clone(), arg.clone()),
                        location.clone(),
                    );
                    tempArgs.push(tempValue);
                }
                tempArgs.into()
            }
            Arguments::Unresolved(unresolvedArgs) => {
                let mut tempArgs = Vec::new();
                for arg in unresolvedArgs {
                    let input = arg.getVariable();
                    let tempValue = self.bodyBuilder.createTempValue(location.clone());
                    self.addInstruction(
                        InstructionKind::Converter(tempValue.clone(), input.clone()),
                        location.clone(),
                    );
                    tempArgs.push(arg.withVariable(tempValue));
                }
                tempArgs.into()
            }
        }
    }

    pub fn addTypedFunctionCall(
        &mut self,
        functionName: QualifiedName,
        args: Vec<Variable>,
        location: Location,
        ty: Type,
    ) -> Variable {
        let result = self.bodyBuilder.createTempValueWithType(location.clone(), ty);
        self.addInstruction(
            InstructionKind::FunctionCall(result.clone(), CallInfo::new(functionName, args)),
            location,
        );
        result
    }

    pub fn addMethodCall<T: Into<Arguments>>(
        &mut self,
        name: String,
        receiver: Variable,
        args: T,
        location: Location,
    ) -> Variable {
        let result = self.bodyBuilder.createTempValue(location.clone());
        // for each arg and the receiver create a temp value and a converter instruction
        let receiverTemp = self.bodyBuilder.createTempValue(location.clone());
        self.addInstruction(
            InstructionKind::Converter(receiverTemp.clone(), receiver.clone()),
            location.clone(),
        );
        let args = self.processFunctionInfo(args.into(), &location);
        self.addInstruction(
            InstructionKind::MethodCall(result.clone(), receiverTemp, name, args),
            location,
        );
        result
    }

    pub fn addDynamicFunctionCall(&mut self, value: Variable, args: Vec<Variable>, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(location.clone());
        self.addInstruction(
            InstructionKind::DynamicFunctionCall(result.clone(), value, args),
            location,
        );
        result
    }

    pub fn addFieldRef(&mut self, receiver: Variable, fields: Vec<FieldInfo>, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(location.clone());
        self.addInstruction(
            InstructionKind::FieldRef(result.clone(), receiver, fields),
            location.clone(),
        );
        result
    }

    pub fn addTypedFieldRef(
        &mut self,
        receiver: Variable,
        fields: Vec<FieldInfo>,
        location: Location,
        ty: Type,
    ) -> Variable {
        let result = self.bodyBuilder.createTempValueWithType(location.clone(), ty);
        self.addInstruction(
            InstructionKind::FieldRef(result.clone(), receiver, fields),
            location.clone(),
        );
        result
    }

    pub fn addStringLiteral(&mut self, literal: String, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(location.clone());
        self.addInstruction(
            InstructionKind::StringLiteral(result.clone(), literal),
            location.clone(),
        );
        result
    }

    pub fn addIntegerLiteral(&mut self, literal: String, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(location.clone());
        self.addInstruction(
            InstructionKind::IntegerLiteral(result.clone(), literal),
            location.clone(),
        );
        result
    }

    pub fn addCharLiteral(&mut self, literal: String, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(location.clone());
        self.addInstruction(InstructionKind::CharLiteral(result.clone(), literal), location.clone());
        result
    }

    pub fn addUnit(&mut self, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValueWithType(location.clone(), Type::getUnitType());
        self.addInstruction(InstructionKind::Tuple(result.clone(), Vec::new()), location.clone());
        result
    }

    pub fn addTuple(&mut self, args: Vec<Variable>, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(location.clone());
        self.addInstruction(InstructionKind::Tuple(result.clone(), args), location.clone());
        result
    }

    pub fn addJump(&mut self, target: BlockId, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValueWithType(location.clone(), Type::getNeverType());
        self.addInstruction(InstructionKind::Jump(result.clone(), target), location.clone());
        result
    }

    pub fn addDeclare(&mut self, name: Variable, location: Location) {
        self.implicit()
            .addDeclareWithMutability(name, location, Mutability::Mutable);
    }

    pub fn addDeclareWithMutability(&mut self, name: Variable, location: Location, mutability: Mutability) {
        self.addInstruction(InstructionKind::DeclareVar(name, mutability), location.clone());
    }

    pub fn addBind(&mut self, name: Variable, rhs: Variable, mutable: bool, location: Location) {
        self.addInstruction(InstructionKind::Bind(name, rhs, mutable), location.clone());
    }

    pub fn addFieldAssign(&mut self, receiver: Variable, rhs: Variable, fields: Vec<FieldInfo>, location: Location) {
        self.addInstruction(InstructionKind::FieldAssign(receiver, rhs, fields), location.clone());
    }

    pub fn addTransform(&mut self, value: Variable, index: u32, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(location.clone());
        self.addInstruction(
            InstructionKind::Transform(result.clone(), value, TransformInfo { variantIndex: index }),
            location.clone(),
        );
        result
    }

    pub fn addTypedTransform(&mut self, value: Variable, index: u32, location: Location, ty: Type) -> Variable {
        let result = self.bodyBuilder.createTempValueWithType(location.clone(), ty);
        self.addInstruction(
            InstructionKind::Transform(result.clone(), value, TransformInfo { variantIndex: index }),
            location.clone(),
        );
        result
    }

    pub fn addBlockStart(&mut self, syntaxBlockId: SyntaxBlockId, location: Location) {
        self.addInstruction(InstructionKind::BlockStart(syntaxBlockId), location);
    }

    pub fn addBlockEnd(&mut self, syntaxBlockId: SyntaxBlockId, location: Location) {
        self.addInstruction(InstructionKind::BlockEnd(syntaxBlockId), location);
    }

    pub fn getBlockId(&self) -> BlockId {
        self.blockId
    }

    pub fn splitBlock(&self, offset: usize) -> BlockId {
        match self.mode {
            Mode::Append => panic!("Cannot cut block in append mode"),
            Mode::Iterator(index) => self.bodyBuilder.splitBlock(self.blockId, index + offset),
        }
    }

    pub fn getBlockSize(&self) -> usize {
        self.bodyBuilder.getBlockSize(self.blockId)
    }

    pub fn getLastInstruction(&self) -> Option<Instruction> {
        self.bodyBuilder.getLastInstruction(self.blockId)
    }

    pub fn isValid(&self) -> bool {
        self.bodyBuilder.isValid(self.blockId)
    }
}
