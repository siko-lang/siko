use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

use super::{ConstraintContext::ConstraintContext, Type::Type};

#[derive(Debug, Clone)]
pub enum Parameter {
    Named(String, Type, bool), // mutable
    SelfParam(bool, Type),     // mutable
}

impl Parameter {
    pub fn getName(&self) -> String {
        match &self {
            Parameter::Named(n, _, _) => n.clone(),
            Parameter::SelfParam(_, _) => "self".to_string(),
        }
    }

    pub fn getType(&self) -> Type {
        match &self {
            Parameter::Named(_, ty, _) => ty.clone(),
            Parameter::SelfParam(_, ty) => ty.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BlockId {
    pub id: u32,
}

impl BlockId {
    pub fn first() -> BlockId {
        BlockId { id: 0 }
    }
}

impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.id)
    }
}

#[derive(Clone, PartialEq)]
pub struct EnumCase {
    pub index: u32,
    pub branch: BlockId,
}

impl std::fmt::Debug for EnumCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.index, self.branch)
    }
}

#[derive(Clone, PartialEq)]
pub struct IntegerCase {
    pub value: Option<String>,
    pub branch: BlockId,
}

impl std::fmt::Debug for IntegerCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(v) => {
                write!(f, "({}, {})", v, self.branch)
            }
            None => {
                write!(f, "(<default>, {})", self.branch)
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct StringCase {
    pub value: Option<String>,
    pub branch: BlockId,
}

impl std::fmt::Debug for StringCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(v) => {
                write!(f, "({}, {})", v, self.branch)
            }
            None => {
                write!(f, "(<default>, {})", self.branch)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub value: String,
    pub location: Location,
    pub ty: Option<Type>,
    pub index: u32,
}

impl Variable {
    pub fn getType(&self) -> &Type {
        match &self.ty {
            Some(ty) => ty,
            None => panic!("No type found for var {}", self.value),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "${}/{}: {}", self.value, self.index, ty)
        } else {
            write!(f, "${}/{}", self.value, self.index)
        }
    }
}

impl std::fmt::Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq)]
pub struct FieldInfo {
    pub name: String,
    pub location: Location,
    pub ty: Option<Type>,
}

impl Display for FieldInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "f/{}: {}", self.name, ty)
        } else {
            write!(f, "f/{}", self.name)
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct BlockInfo {
    pub id: String,
}

impl Display for BlockInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Clone, PartialEq)]
pub enum InstructionKind {
    FunctionCall(Variable, QualifiedName, Vec<Variable>),
    MethodCall(Variable, Variable, String, Vec<Variable>),
    DynamicFunctionCall(Variable, Variable, Vec<Variable>),
    ValueRef(Variable, Variable),
    FieldRef(Variable, Variable, String),
    TupleIndex(Variable, Variable, i32),
    Bind(Variable, Variable, bool), //mutable
    Tuple(Variable, Vec<Variable>),
    StringLiteral(Variable, String),
    IntegerLiteral(Variable, String),
    CharLiteral(Variable, char),
    Return(Variable, Variable),
    Ref(Variable, Variable),
    Drop(Vec<String>),
    Jump(Variable, BlockId),
    Assign(Variable, Variable),
    FieldAssign(Variable, Variable, Vec<FieldInfo>),
    DeclareVar(Variable),
    Transform(Variable, Variable, u32),
    EnumSwitch(Variable, Vec<EnumCase>),
    IntegerSwitch(Variable, Vec<IntegerCase>),
    StringSwitch(Variable, Vec<StringCase>),
    BlockStart(BlockInfo),
    BlockEnd(BlockInfo),
}

impl Display for InstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dump())
    }
}

impl Debug for InstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dump())
    }
}

impl InstructionKind {
    pub fn getResultVar(&self) -> Option<Variable> {
        match self {
            InstructionKind::FunctionCall(v, _, _) => Some(v.clone()),
            InstructionKind::MethodCall(v, _, _, _) => Some(v.clone()),
            InstructionKind::DynamicFunctionCall(v, _, _) => Some(v.clone()),
            InstructionKind::ValueRef(v, _) => Some(v.clone()),
            InstructionKind::FieldRef(v, _, _) => Some(v.clone()),
            InstructionKind::TupleIndex(v, _, _) => Some(v.clone()),
            InstructionKind::Bind(v, _, _) => Some(v.clone()),
            InstructionKind::Tuple(v, _) => Some(v.clone()),
            InstructionKind::StringLiteral(v, _) => Some(v.clone()),
            InstructionKind::IntegerLiteral(v, _) => Some(v.clone()),
            InstructionKind::CharLiteral(v, _) => Some(v.clone()),
            InstructionKind::Return(v, _) => Some(v.clone()),
            InstructionKind::Ref(v, _) => Some(v.clone()),
            InstructionKind::Drop(_) => None,
            InstructionKind::Jump(v, _) => Some(v.clone()),
            InstructionKind::Assign(_, _) => None,
            InstructionKind::FieldAssign(_, _, _) => None,
            InstructionKind::DeclareVar(v) => Some(v.clone()),
            InstructionKind::Transform(v, _, _) => Some(v.clone()),
            InstructionKind::EnumSwitch(_, _) => None,
            InstructionKind::IntegerSwitch(_, _) => None,
            InstructionKind::StringSwitch(_, _) => None,
            InstructionKind::BlockStart(_) => None,
            InstructionKind::BlockEnd(_) => None,
        }
    }

    pub fn collectVariables(&self) -> Vec<Variable> {
        match self {
            InstructionKind::FunctionCall(var, _, args) => {
                let mut vars = vec![var.clone()];
                vars.extend(args.clone());
                vars
            }
            InstructionKind::MethodCall(var, obj, _, args) => {
                let mut vars = vec![var.clone(), obj.clone()];
                vars.extend(args.clone());
                vars
            }
            InstructionKind::DynamicFunctionCall(var, func, args) => {
                let mut vars = vec![var.clone(), func.clone()];
                vars.extend(args.clone());
                vars
            }
            InstructionKind::ValueRef(var, target) => vec![var.clone(), target.clone()],
            InstructionKind::FieldRef(var, target, _) => vec![var.clone(), target.clone()],
            InstructionKind::TupleIndex(var, target, _) => vec![var.clone(), target.clone()],
            InstructionKind::Bind(var, value, _) => vec![var.clone(), value.clone()],
            InstructionKind::Tuple(var, elements) => {
                let mut vars = vec![var.clone()];
                vars.extend(elements.clone());
                vars
            }
            InstructionKind::StringLiteral(var, _) => vec![var.clone()],
            InstructionKind::IntegerLiteral(var, _) => vec![var.clone()],
            InstructionKind::CharLiteral(var, _) => vec![var.clone()],
            InstructionKind::Return(var, value) => vec![var.clone(), value.clone()],
            InstructionKind::Ref(var, target) => vec![var.clone(), target.clone()],
            InstructionKind::Drop(_) => vec![],
            InstructionKind::Jump(var, _) => vec![var.clone()],
            InstructionKind::Assign(var, value) => vec![var.clone(), value.clone()],
            InstructionKind::FieldAssign(var, value, _) => vec![var.clone(), value.clone()],
            InstructionKind::DeclareVar(var) => vec![var.clone()],
            InstructionKind::Transform(var, target, _) => vec![var.clone(), target.clone()],
            InstructionKind::EnumSwitch(var, _) => {
                vec![var.clone()]
            }
            InstructionKind::IntegerSwitch(var, _) => {
                vec![var.clone()]
            }
            InstructionKind::StringSwitch(var, _) => {
                vec![var.clone()]
            }
            InstructionKind::BlockStart(_) => Vec::new(),
            InstructionKind::BlockEnd(_) => Vec::new(),
        }
    }

    pub fn dump(&self) -> String {
        match self {
            InstructionKind::FunctionCall(dest, name, args) => format!("{} = call({}({:?}))", dest, name, args),
            InstructionKind::MethodCall(dest, receiver, name, args) => format!("{} = methodcall({}.{}({:?}))", dest, receiver, name, args),
            InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                format!("{} = DYN_CALL({}, {:?})", dest, callable, args)
            }
            InstructionKind::ValueRef(dest, v) => format!("{} = {}", dest, v),
            InstructionKind::FieldRef(dest, v, name) => format!("{} = {}.{}", dest, v, name),
            InstructionKind::TupleIndex(dest, v, idx) => format!("{} = {}.t{}", dest, v, idx),
            InstructionKind::Bind(v, rhs, mutable) => {
                if *mutable {
                    format!("mut ${} = {}", v, rhs)
                } else {
                    format!("${} = {}", v, rhs)
                }
            }
            InstructionKind::Tuple(dest, args) => format!("{} = tuple({:?})", dest, args),
            InstructionKind::StringLiteral(dest, v) => format!("{} = s:[{}]", dest, v),
            InstructionKind::IntegerLiteral(dest, v) => format!("{} = i:[{}]", dest, v),
            InstructionKind::CharLiteral(dest, v) => format!("{} = c:[{}]", dest, v),
            InstructionKind::Return(dest, id) => format!("{} = return({})", dest, id),
            InstructionKind::Ref(dest, id) => format!("{} = &({})", dest, id),
            InstructionKind::Drop(values) => {
                format!("drop({})", values.join(", "))
            }
            InstructionKind::Jump(dest, id) => {
                format!("{} = jump({})", dest, id)
            }
            InstructionKind::Assign(v, arg) => format!("assign({}, {})", v, arg),
            InstructionKind::FieldAssign(v, arg, fields) => {
                let fields = fields.iter().map(|info| info.to_string()).collect::<Vec<_>>().join(", ");
                format!("fieldassign({}, {}, {})", v, arg, fields)
            }
            InstructionKind::DeclareVar(v) => format!("declare({})", v),
            InstructionKind::Transform(dest, arg, index) => format!("{} = transform({}, {})", dest, arg, index),
            InstructionKind::EnumSwitch(root, cases) => format!("enumswitch({}, {:?})", root, cases),
            InstructionKind::IntegerSwitch(root, cases) => format!("integerswitch({}, {:?})", root, cases),
            InstructionKind::StringSwitch(root, cases) => format!("stringswitch({}, {:?})", root, cases),
            InstructionKind::BlockStart(info) => format!("blockstart({})", info),
            InstructionKind::BlockEnd(info) => format!("blockend({})", info),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub implicit: bool,
    pub kind: InstructionKind,
    pub location: Location,
}

impl Instruction {
    pub fn dump(&self) {
        println!("    {}", self);
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind.dump())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn new(id: BlockId) -> Block {
        Block {
            id: id,
            instructions: Vec::new(),
        }
    }

    pub fn add(&mut self, kind: InstructionKind, location: Location) {
        self.addWithImplicit(kind, location, false)
    }

    pub fn addWithImplicit(&mut self, kind: InstructionKind, location: Location, implicit: bool) {
        self.instructions.push(Instruction {
            implicit: implicit,
            kind: kind,
            location: location,
        });
    }

    pub fn dump(&self) {
        println!("  Block {}:", self.id);
        for instruction in &self.instructions {
            instruction.dump();
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Block {}:", self.id)?;
        for instruction in &self.instructions {
            writeln!(f, "{}", instruction)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub blocks: Vec<Block>,
    pub varTypes: BTreeMap<String, Type>,
}

impl Body {
    pub fn new() -> Body {
        Body {
            blocks: Vec::new(),
            varTypes: BTreeMap::new(),
        }
    }

    pub fn addBlock(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn getBlockById(&self, id: BlockId) -> &Block {
        &self.blocks[id.id as usize]
    }

    pub fn setType(&mut self, var: Variable, ty: Type) {
        self.varTypes.insert(var.value, ty);
    }

    pub fn dump(&self) {
        for block in &self.blocks {
            block.dump();
        }
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for block in &self.blocks {
            write!(f, "{}", block)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionKind {
    UserDefined,
    VariantCtor(i64),
    ClassCtor,
    Extern,
    TraitMemberDecl(QualifiedName),
    TraitMemberDefinition(QualifiedName),
}

impl FunctionKind {
    pub fn isTraitCall(&self) -> Option<QualifiedName> {
        match self {
            FunctionKind::TraitMemberDecl(qn) | FunctionKind::TraitMemberDefinition(qn) => Some(qn.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: QualifiedName,
    pub params: Vec<Parameter>,
    pub result: Type,
    pub body: Option<Body>,
    pub constraintContext: ConstraintContext,
    pub kind: FunctionKind,
}

impl Function {
    pub fn new(
        name: QualifiedName,
        params: Vec<Parameter>,
        result: Type,
        body: Option<Body>,
        constraintContext: ConstraintContext,
        kind: FunctionKind,
    ) -> Function {
        Function {
            name: name,
            params: params,
            result: result,
            body: body,
            constraintContext: constraintContext,
            kind: kind,
        }
    }

    pub fn getBlockById(&self, id: BlockId) -> &Block {
        if let Some(body) = &self.body {
            body.getBlockById(id)
        } else {
            panic!("getBlockById: no body found");
        }
    }

    pub fn getFirstBlock(&self) -> &Block {
        if let Some(body) = &self.body {
            &body.blocks[0]
        } else {
            panic!("getFirstBlock: no body found");
        }
    }

    pub fn getType(&self) -> Type {
        let mut args = Vec::new();
        for param in &self.params {
            match &param {
                Parameter::Named(_, ty, _) => args.push(ty.clone()),
                Parameter::SelfParam(_, ty) => args.push(ty.clone()),
            }
        }
        Type::Function(args, Box::new(self.result.clone()))
    }

    pub fn dump(&self) {
        println!("{}", self.name);
        match &self.body {
            Some(body) => body.dump(),
            None => println!("  <no body>"),
        }
    }

    pub fn instructions<'a>(&'a self) -> InstructionIterator<'a> {
        InstructionIterator::new(self)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.name, self.getType())?;
        writeln!(f, "constraints {}", self.constraintContext)?;
        match &self.body {
            Some(body) => write!(f, "{}", body),
            None => write!(f, "  <no body>"),
        }
    }
}

pub struct InstructionIterator<'a> {
    f: &'a Function,
    block: usize,
    instruction: usize,
}

impl<'a> InstructionIterator<'a> {
    fn new(f: &'a Function) -> InstructionIterator<'a> {
        InstructionIterator { f, block: 0, instruction: 0 }
    }
}

impl<'a> Iterator for InstructionIterator<'a> {
    type Item = &'a Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(body) = &self.f.body {
            if self.block >= body.blocks.len() {
                return None;
            }
            let block = &body.blocks[self.block];
            let item = &block.instructions[self.instruction];
            self.instruction += 1;
            if self.instruction >= block.instructions.len() {
                self.instruction = 0;
                self.block += 1;
            }
            return Some(item);
        } else {
            return None;
        }
    }
}
