use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

use super::{
    BodyBuilder::BodyBuilder,
    Function::{BlockId, FieldInfo, InstructionKind, Variable},
};

pub struct BlockBuilder {
    bodyBuilder: BodyBuilder,
    blockId: BlockId,
    isImplicit: bool,
}

impl BlockBuilder {
    pub fn new(blockId: BlockId, bodyBuilder: BodyBuilder) -> BlockBuilder {
        BlockBuilder {
            bodyBuilder: bodyBuilder,
            blockId,
            isImplicit: false,
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
        }
    }

    pub fn addInstruction(&mut self, instruction: InstructionKind, location: Location) {
        self.bodyBuilder
            .addInstructionToBlock(self.blockId, instruction, location, self.isImplicit)
    }

    pub fn addAssign(&mut self, target: Variable, source: Variable, location: Location) {
        self.addInstruction(InstructionKind::Assign(target, source), location);
    }

    pub fn addReturn(&mut self, value: Variable, location: Location) -> Variable {
        let retValue = self.bodyBuilder.createValue("ret", location.clone());
        self.addInstruction(InstructionKind::Return(retValue.clone(), value), location);
        retValue
    }

    pub fn addRef(&mut self, arg: Variable, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("ref", location.clone());
        self.addInstruction(InstructionKind::Ref(value.clone(), arg), location.clone());
        value
    }

    pub fn addFunctionCall(&mut self, functionName: QualifiedName, args: Vec<Variable>, location: Location) -> Variable {
        let result = self.bodyBuilder.createValue("call", location.clone());
        self.addInstruction(InstructionKind::FunctionCall(result.clone(), functionName, args), location);
        result
    }

    pub fn addMethodCall(&mut self, name: String, receiver: Variable, args: Vec<Variable>, location: Location) -> Variable {
        let result = self.bodyBuilder.createValue("call", location.clone());
        self.addInstruction(InstructionKind::MethodCall(result.clone(), receiver, name, args), location);
        result
    }

    pub fn addDynamicFunctionCall(&mut self, value: Variable, args: Vec<Variable>, location: Location) -> Variable {
        let result = self.bodyBuilder.createValue("call", location.clone());
        self.addInstruction(InstructionKind::DynamicFunctionCall(result.clone(), value, args), location);
        result
    }

    pub fn addFieldRef(&mut self, receiveer: Variable, field: String, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("fieldRef", location.clone());
        self.addInstruction(InstructionKind::FieldRef(value.clone(), receiveer, field), location.clone());
        value
    }

    pub fn addStringLiteral(&mut self, literal: String, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("lit", location.clone());
        self.addInstruction(InstructionKind::StringLiteral(value.clone(), literal), location.clone());
        value
    }

    pub fn addIntegerLiteral(&mut self, literal: String, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("lit", location.clone());
        self.addInstruction(InstructionKind::IntegerLiteral(value.clone(), literal), location.clone());
        value
    }

    pub fn addCharLiteral(&mut self, literal: char, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("lit", location.clone());
        self.addInstruction(InstructionKind::CharLiteral(value.clone(), literal), location.clone());
        value
    }

    pub fn addUnit(&mut self, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("unit", location.clone());
        self.addInstruction(InstructionKind::Tuple(value.clone(), Vec::new()), location.clone());
        value
    }

    pub fn addTuple(&mut self, args: Vec<Variable>, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("tuple", location.clone());
        self.addInstruction(InstructionKind::Tuple(value.clone(), args), location.clone());
        value
    }

    pub fn addTupleIndex(&mut self, tuple: Variable, index: i32, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("tupleIndex", location.clone());
        self.addInstruction(InstructionKind::TupleIndex(value.clone(), tuple, index), location.clone());
        value
    }

    pub fn addJump(&mut self, target: BlockId, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("jump", location.clone());
        self.addInstruction(InstructionKind::Jump(value.clone(), target), location.clone());
        value
    }

    pub fn addDeclare(&mut self, name: Variable, location: Location) {
        self.addInstruction(InstructionKind::DeclareVar(name), location.clone());
    }

    pub fn addBind(&mut self, name: Variable, rhs: Variable, mutable: bool, location: Location) {
        self.addInstruction(InstructionKind::Bind(name, rhs, mutable), location.clone());
    }

    pub fn addFieldAssign(&mut self, receiver: Variable, rhs: Variable, fields: Vec<FieldInfo>, location: Location) {
        self.addInstruction(InstructionKind::FieldAssign(receiver, rhs, fields), location.clone());
    }

    pub fn getBlockId(&self) -> BlockId {
        self.blockId
    }
}
