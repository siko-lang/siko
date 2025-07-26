use core::panic;

use crate::siko::{hir::Instruction::Mutability, location::Location::Location, qualifiedname::QualifiedName};

use super::{
    BodyBuilder::BodyBuilder,
    Function::BlockId,
    Instruction::{FieldInfo, Instruction, InstructionKind, JumpDirection},
    Type::Type,
    Variable::{Variable, VariableName},
};

#[derive(Clone, Copy)]
pub enum Mode {
    Append,
    Iterator(usize),
}

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

    pub fn getInstruction(&self) -> Option<Instruction> {
        match self.mode {
            Mode::Append => panic!("Cannot get instruction in append mode"),
            Mode::Iterator(index) => self.bodyBuilder.getInstruction(self.blockId, index),
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

    pub fn addAssign(&mut self, target: Variable, source: Variable, location: Location) {
        self.addInstruction(InstructionKind::Assign(target, source), location);
    }

    pub fn addReturn(&mut self, value: Variable, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(VariableName::Ret, location.clone());
        self.addInstruction(InstructionKind::Return(result.clone(), value), location);
        result
    }

    pub fn addRef(&mut self, arg: Variable, location: Location) -> Variable {
        let value = self.bodyBuilder.createTempValue(VariableName::Ref, location.clone());
        self.addInstruction(InstructionKind::Ref(value.clone(), arg), location.clone());
        value
    }

    pub fn addFunctionCall(
        &mut self,
        functionName: QualifiedName,
        args: Vec<Variable>,
        location: Location,
    ) -> Variable {
        // for each arg create a temp value and a converter instruction
        let mut tempArgs = Vec::new();
        for arg in &args {
            let tempValue = self.bodyBuilder.createTempValue(VariableName::Tmp, location.clone());
            self.addInstruction(
                InstructionKind::Converter(tempValue.clone(), arg.clone()),
                location.clone(),
            );
            tempArgs.push(tempValue);
        }
        let result = self.bodyBuilder.createTempValue(VariableName::Call, location.clone());
        self.addDeclare(result.clone(), location.clone());
        self.addInstruction(
            InstructionKind::FunctionCall(result.clone(), functionName, tempArgs),
            location,
        );
        result
    }

    pub fn addTypedFunctionCall(
        &mut self,
        functionName: QualifiedName,
        args: Vec<Variable>,
        location: Location,
        ty: Type,
    ) -> Variable {
        let mut result = self.bodyBuilder.createTempValue(VariableName::Call, location.clone());
        result.ty = Some(ty);
        self.addInstruction(
            InstructionKind::FunctionCall(result.clone(), functionName, args),
            location,
        );
        result
    }

    pub fn addMethodCall(
        &mut self,
        name: String,
        receiver: Variable,
        args: Vec<Variable>,
        location: Location,
    ) -> Variable {
        let result = self.bodyBuilder.createTempValue(VariableName::Call, location.clone());
        // for each arg and the receiver create a temp value and a converter instruction
        let receiverTemp = self.bodyBuilder.createTempValue(VariableName::Tmp, location.clone());
        self.addInstruction(
            InstructionKind::Converter(receiverTemp.clone(), receiver.clone()),
            location.clone(),
        );
        let mut tempArgs = Vec::new();
        for arg in &args {
            let tempValue = self.bodyBuilder.createTempValue(VariableName::Tmp, location.clone());
            self.addInstruction(
                InstructionKind::Converter(tempValue.clone(), arg.clone()),
                location.clone(),
            );
            tempArgs.push(tempValue);
        }
        self.addDeclare(result.clone(), location.clone());
        self.addInstruction(
            InstructionKind::MethodCall(result.clone(), receiverTemp, name, tempArgs),
            location,
        );
        result
    }

    pub fn addDynamicFunctionCall(&mut self, value: Variable, args: Vec<Variable>, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(VariableName::Call, location.clone());
        self.addInstruction(
            InstructionKind::DynamicFunctionCall(result.clone(), value, args),
            location,
        );
        result
    }

    pub fn addFieldRef(&mut self, receiver: Variable, field: String, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValue(VariableName::FieldRef, location.clone());
        self.addInstruction(
            InstructionKind::FieldRef(result.clone(), receiver, field),
            location.clone(),
        );
        result
    }

    pub fn addTypedFieldRef(&mut self, receiver: Variable, field: String, location: Location, ty: Type) -> Variable {
        let mut result = self
            .bodyBuilder
            .createTempValue(VariableName::FieldRef, location.clone());
        result.ty = Some(ty);
        self.addInstruction(
            InstructionKind::FieldRef(result.clone(), receiver, field),
            location.clone(),
        );
        result
    }

    pub fn addStringLiteral(&mut self, literal: String, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValue(VariableName::Literal, location.clone());
        self.addInstruction(
            InstructionKind::StringLiteral(result.clone(), literal),
            location.clone(),
        );
        result
    }

    pub fn addIntegerLiteral(&mut self, literal: String, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValue(VariableName::Literal, location.clone());
        self.addInstruction(
            InstructionKind::IntegerLiteral(result.clone(), literal),
            location.clone(),
        );
        result
    }

    pub fn addCharLiteral(&mut self, literal: char, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValue(VariableName::Literal, location.clone());
        self.addInstruction(InstructionKind::CharLiteral(result.clone(), literal), location.clone());
        result
    }

    pub fn addUnit(&mut self, location: Location) -> Variable {
        let mut result = self.bodyBuilder.createTempValue(VariableName::Unit, location.clone());
        result.ty = Some(Type::getUnitType());
        self.addDeclare(result.clone(), location.clone());
        self.addInstruction(InstructionKind::Tuple(result.clone(), Vec::new()), location.clone());
        result
    }

    pub fn addTuple(&mut self, args: Vec<Variable>, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(VariableName::Tuple, location.clone());
        self.addInstruction(InstructionKind::Tuple(result.clone(), args), location.clone());
        result
    }

    pub fn addTupleIndex(&mut self, tuple: Variable, index: i32, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValue(VariableName::TupleIndex, location.clone());
        self.addInstruction(
            InstructionKind::TupleIndex(result.clone(), tuple, index),
            location.clone(),
        );
        result
    }

    pub fn addTypedTupleIndex(&mut self, tuple: Variable, index: i32, location: Location, ty: Type) -> Variable {
        let mut result = self
            .bodyBuilder
            .createTempValue(VariableName::TupleIndex, location.clone());
        result.ty = Some(ty);
        self.addInstruction(
            InstructionKind::TupleIndex(result.clone(), tuple, index),
            location.clone(),
        );
        result
    }

    pub fn addJump(&mut self, target: BlockId, direction: JumpDirection, location: Location) -> Variable {
        let result = self.bodyBuilder.createTempValue(VariableName::Jump, location.clone());
        self.addInstruction(
            InstructionKind::Jump(result.clone(), target, direction),
            location.clone(),
        );
        result
    }

    pub fn addDeclare(&mut self, name: Variable, location: Location) {
        self.addDeclareWithMutability(name, location, Mutability::Mutable);
    }

    pub fn addDeclareWithMutability(&mut self, name: Variable, location: Location, mutability: Mutability) {
        self.addInstruction(InstructionKind::DeclareVar(name, mutability), location.clone());
    }

    pub fn addBind(&mut self, name: Variable, rhs: Variable, mutable: bool, location: Location) {
        self.addInstruction(InstructionKind::Bind(name, rhs, mutable), location.clone());
    }

    pub fn addConverter(&mut self, lhs: Variable, rhs: Variable, location: Location) {
        self.addInstruction(InstructionKind::Converter(lhs, rhs), location.clone());
    }

    pub fn addFieldAssign(&mut self, receiver: Variable, rhs: Variable, fields: Vec<FieldInfo>, location: Location) {
        self.addInstruction(InstructionKind::FieldAssign(receiver, rhs, fields), location.clone());
    }

    pub fn addTransform(&mut self, value: Variable, index: u32, location: Location) -> Variable {
        let result = self
            .bodyBuilder
            .createTempValue(VariableName::Transform, location.clone());
        self.addInstruction(
            InstructionKind::Transform(result.clone(), value, index),
            location.clone(),
        );
        result
    }

    pub fn addTypedTransform(&mut self, value: Variable, index: u32, location: Location, ty: Type) -> Variable {
        let mut result = self
            .bodyBuilder
            .createTempValue(VariableName::Transform, location.clone());
        result.ty = Some(ty);
        self.addInstruction(
            InstructionKind::Transform(result.clone(), value, index),
            location.clone(),
        );
        result
    }

    pub fn getBlockId(&self) -> BlockId {
        self.blockId
    }
}
