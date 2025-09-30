use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::backend::drop::Path::Path;
use crate::siko::hir::Block::BlockId;
use crate::siko::hir::CollectVariables::CollectVariables;
use crate::siko::hir::Type::formatTypes;
use crate::siko::hir::Variable::VariableName;
use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

use super::Type::Type;
use super::Variable::Variable;

#[derive(Clone, PartialEq, Debug)]
pub enum FieldId {
    Named(String),
    Indexed(u32),
}

impl FieldId {
    pub fn name(&self) -> String {
        match self {
            FieldId::Named(name) => name.clone(),
            FieldId::Indexed(index) => {
                panic!("indexed field found in FieldId::name() {}", index)
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct FieldInfo {
    pub name: FieldId,
    pub location: Location,
    pub ty: Option<Type>,
}

impl Display for FieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldId::Named(name) => write!(f, "{}", name),
            FieldId::Indexed(index) => write!(f, "{}", index),
        }
    }
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

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct SyntaxBlockIdSegment {
    pub value: u32,
}

impl Display for SyntaxBlockIdSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Debug for SyntaxBlockIdSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct SyntaxBlockId {
    pub items: Vec<SyntaxBlockIdSegment>,
}

impl SyntaxBlockId {
    pub fn new() -> Self {
        SyntaxBlockId { items: Vec::new() }
    }

    pub fn add(&self, item: SyntaxBlockIdSegment) -> Self {
        let mut new_items = self.items.clone();
        new_items.push(item);
        SyntaxBlockId { items: new_items }
    }

    pub fn getParent(&self) -> SyntaxBlockId {
        assert_ne!(self.items.len(), 0, "Cannot be empty");
        if self.items.len() == 1 {
            self.clone()
        } else {
            SyntaxBlockId {
                items: self.items[0..self.items.len() - 1].to_vec(),
            }
        }
    }

    pub fn isParentOf(&self, other: &SyntaxBlockId) -> bool {
        if self.items.len() >= other.items.len() {
            return false;
        }
        for (i, j) in self.items.iter().zip(other.items.iter()) {
            if i != j {
                return false;
            }
        }
        true
    }

    pub fn differenceToParent(&self, other: &SyntaxBlockId) -> Vec<SyntaxBlockId> {
        if self.isParentOf(other) {
            return vec![];
        }
        //println!("Difference from {} to {}", self, other);
        let mut result = Vec::new();
        if other == self {
            return result;
        }
        let mut current = self.clone();
        loop {
            result.push(current.clone());
            let parent = current.getParent();
            //println!("Parent: {} {}", parent, other);
            if parent == *other {
                break;
            }
            if parent == current {
                panic!("Cannot find parent for {}", current);
            }
            current = parent;
        }
        result
    }

    pub fn isEmpty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Display for SyntaxBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(".")
        )
    }
}

impl Debug for SyntaxBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq)]
pub struct EnumCase {
    pub index: Option<u32>,
    pub branch: BlockId,
}

impl std::fmt::Debug for EnumCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.index {
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

#[derive(Clone, PartialEq)]
pub enum Mutability {
    Mutable,
    Immutable,
    ExplicitMutable,
}

impl Display for Mutability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mutability::Mutable => write!(f, "mutable"),
            Mutability::Immutable => write!(f, "immutable"),
            Mutability::ExplicitMutable => write!(f, "explicit mutable"),
        }
    }
}

impl Debug for Mutability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImplicitContextIndex(pub usize);

impl Display for ImplicitContextIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl Debug for ImplicitContextIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq)]
pub enum ImplicitIndex {
    Unresolved(QualifiedName),
    Resolved(ImplicitContextIndex, SyntaxBlockId),
}

impl Display for ImplicitIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImplicitIndex::Unresolved(name) => write!(f, "{}", name),
            ImplicitIndex::Resolved(index, id) => write!(f, "{}, {}", index, id),
        }
    }
}

impl Debug for ImplicitIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq)]
pub struct WithInfo {
    pub contexts: Vec<WithContext>,
    pub blockId: BlockId,
    pub parentSyntaxBlockId: SyntaxBlockId,
    pub syntaxBlockId: SyntaxBlockId,
    pub operations: Vec<ImplicitContextOperation>,
    pub contextTypes: Vec<Type>,
}

impl Display for WithInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "with_info({:?}, {}, {}, {:?}{})",
            self.contexts,
            self.blockId,
            self.syntaxBlockId,
            self.operations,
            formatTypes(&self.contextTypes)
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct CallContextInfo {
    pub contextSyntaxBlockId: SyntaxBlockId,
}

#[derive(Clone, PartialEq)]
pub enum InstanceReference {
    Direct(QualifiedName),
    Indirect(u32),
}

impl Display for InstanceReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceReference::Direct(name) => write!(f, "{}", name),
            InstanceReference::Indirect(index) => write!(f, "impl(#{})", index),
        }
    }
}

impl Debug for InstanceReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq)]
pub enum UnresolvedArgument {
    Positional(Variable),
    Named(String, Location, Variable),
}

impl UnresolvedArgument {
    pub fn getVariable(&self) -> &Variable {
        match self {
            UnresolvedArgument::Positional(var) => var,
            UnresolvedArgument::Named(_, _, var) => var,
        }
    }

    pub fn withVariable(&self, var: Variable) -> Self {
        match self {
            UnresolvedArgument::Positional(_) => UnresolvedArgument::Positional(var),
            UnresolvedArgument::Named(name, loc, _) => UnresolvedArgument::Named(name.clone(), loc.clone(), var),
        }
    }
}

impl Display for UnresolvedArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedArgument::Positional(var) => write!(f, "{}", var),
            UnresolvedArgument::Named(name, _, var) => write!(f, "{}: {}", name, var),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Arguments {
    Resolved(Vec<Variable>),
    Unresolved(Vec<UnresolvedArgument>),
}

impl Arguments {
    pub fn getVariables(&self) -> &Vec<Variable> {
        match self {
            Arguments::Resolved(vars) => vars,
            Arguments::Unresolved(_) => panic!("Cannot get variables from unresolved arguments"),
        }
    }

    pub fn addMethodReceiver(&mut self, receiver: Variable) {
        match self {
            Arguments::Resolved(ref mut vars) => {
                vars.insert(0, receiver);
            }
            Arguments::Unresolved(ref mut args) => {
                args.insert(0, UnresolvedArgument::Positional(receiver));
            }
        }
    }
}

impl Into<Arguments> for Vec<Variable> {
    fn into(self) -> Arguments {
        Arguments::Resolved(self)
    }
}

impl Into<Arguments> for Vec<UnresolvedArgument> {
    fn into(self) -> Arguments {
        Arguments::Unresolved(self)
    }
}

impl Into<Arguments> for () {
    fn into(self) -> Arguments {
        Arguments::Resolved(vec![])
    }
}

impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arguments::Resolved(args) => write!(
                f,
                "[{}]",
                args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")
            ),
            Arguments::Unresolved(args) => write!(
                f,
                "[{}]",
                args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")
            ),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct CallInfo {
    pub name: QualifiedName,
    pub args: Arguments,
    pub context: Option<CallContextInfo>,
    pub instanceRefs: Vec<InstanceReference>,
    pub coroutineSpawn: bool,
}

impl CallInfo {
    pub fn new<T: Into<Arguments>>(name: QualifiedName, args: T) -> Self {
        CallInfo {
            name,
            args: args.into(),
            context: None,
            instanceRefs: Vec::new(),
            coroutineSpawn: false,
        }
    }
}

impl Display for CallInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let contextStr = if let Some(context) = &self.context {
            format!(", context: {}", context.contextSyntaxBlockId)
        } else {
            "".to_string()
        };
        let instanceStr = if !self.instanceRefs.is_empty() {
            format!(
                ", instances: [{}]",
                self.instanceRefs
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            "".to_string()
        };
        write!(
            f,
            "function_call({}, [{}]{}{}, co: {})",
            self.name, self.args, contextStr, instanceStr, self.coroutineSpawn
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct ClosureCreateInfo {
    pub closureParams: Vec<Variable>,
    pub body: BlockId,
    pub name: QualifiedName,
    pub fnArgCount: u32,
}

impl ClosureCreateInfo {
    pub fn new(params: Vec<Variable>, body: BlockId, name: QualifiedName, fnArgCount: u32) -> Self {
        ClosureCreateInfo {
            closureParams: params,
            body,
            name,
            fnArgCount,
        }
    }
}

impl Display for ClosureCreateInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "create_closure({}, {}, {}, [{}])",
            self.name,
            self.body,
            self.fnArgCount,
            self.closureParams
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct TransformInfo {
    pub variantIndex: u32,
}

impl Display for TransformInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "transform({})", self.variantIndex)
    }
}

impl Debug for TransformInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq)]
pub enum IntegerOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    LessThan,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Clone, PartialEq)]
pub enum InstructionKind {
    FunctionCall(Variable, CallInfo),
    Converter(Variable, Variable),
    MethodCall(Variable, Variable, String, Arguments),
    DynamicFunctionCall(Variable, Variable, Vec<Variable>),
    FieldRef(Variable, Variable, Vec<FieldInfo>),
    Bind(Variable, Variable, bool), //mutable
    Tuple(Variable, Vec<Variable>),
    StringLiteral(Variable, String),
    IntegerLiteral(Variable, String),
    CharLiteral(Variable, String),
    Return(Variable, Variable),
    Ref(Variable, Variable),
    PtrOf(Variable, Variable),
    DropPath(Path),
    DropMetadata(VariableName),
    Drop(Variable, Variable),
    Jump(Variable, BlockId),
    Assign(Variable, Variable),
    FieldAssign(Variable, Variable, Vec<FieldInfo>),
    AddressOfField(Variable, Variable, Vec<FieldInfo>),
    DeclareVar(Variable, Mutability),
    Transform(Variable, Variable, TransformInfo),
    EnumSwitch(Variable, Vec<EnumCase>),
    IntegerSwitch(Variable, Vec<IntegerCase>),
    BlockStart(SyntaxBlockId),
    BlockEnd(SyntaxBlockId),
    With(Variable, WithInfo),
    ReadImplicit(Variable, ImplicitIndex),
    WriteImplicit(ImplicitIndex, Variable),
    LoadPtr(Variable, Variable),
    StorePtr(Variable, Variable),
    CreateClosure(Variable, ClosureCreateInfo),
    ClosureReturn(BlockId, Variable, Variable),
    IntegerOp(Variable, Variable, Variable, IntegerOp),
    Yield(Variable, Variable),
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

fn useVars(vars: Vec<Variable>) -> Vec<Variable> {
    vars.iter().map(|v| v.useVar()).collect()
}

impl InstructionKind {
    pub fn getResultVar(&self) -> Option<Variable> {
        match self {
            InstructionKind::FunctionCall(v, _) => Some(v.clone()),
            InstructionKind::Converter(v, _) => Some(v.clone()),
            InstructionKind::MethodCall(v, _, _, _) => Some(v.clone()),
            InstructionKind::DynamicFunctionCall(v, _, _) => Some(v.clone()),
            InstructionKind::FieldRef(v, _, _) => Some(v.clone()),
            InstructionKind::Bind(v, _, _) => Some(v.clone()),
            InstructionKind::Tuple(v, _) => Some(v.clone()),
            InstructionKind::StringLiteral(v, _) => Some(v.clone()),
            InstructionKind::IntegerLiteral(v, _) => Some(v.clone()),
            InstructionKind::CharLiteral(v, _) => Some(v.clone()),
            InstructionKind::Return(v, _) => Some(v.clone()),
            InstructionKind::Ref(v, _) => Some(v.clone()),
            InstructionKind::PtrOf(v, _) => Some(v.clone()),
            InstructionKind::DropPath(_) => None,
            InstructionKind::DropMetadata(_) => None,
            InstructionKind::Drop(_, _) => None,
            InstructionKind::Jump(v, _) => Some(v.clone()),
            InstructionKind::Assign(v, _) => Some(v.clone()),
            InstructionKind::FieldAssign(_, _, _) => None,
            InstructionKind::AddressOfField(v, _, _) => Some(v.clone()),
            InstructionKind::DeclareVar(v, _) => Some(v.clone()),
            InstructionKind::Transform(v, _, _) => Some(v.clone()),
            InstructionKind::EnumSwitch(_, _) => None,
            InstructionKind::IntegerSwitch(_, _) => None,
            InstructionKind::BlockStart(_) => None,
            InstructionKind::BlockEnd(_) => None,
            InstructionKind::With(v, _) => Some(v.clone()),
            InstructionKind::ReadImplicit(v, _) => Some(v.clone()),
            InstructionKind::WriteImplicit(_, _) => None,
            InstructionKind::LoadPtr(v, _) => Some(v.clone()),
            InstructionKind::StorePtr(v, _) => Some(v.clone()),
            InstructionKind::CreateClosure(v, _) => Some(v.clone()),
            InstructionKind::ClosureReturn(_, v, _) => Some(v.clone()),
            InstructionKind::IntegerOp(v, _, _, _) => Some(v.clone()),
            InstructionKind::Yield(v, _) => Some(v.clone()),
        }
    }

    pub fn collectVariables(&self) -> Vec<Variable> {
        let mut vars = Vec::new();
        CollectVariables::collectVariables(self, &mut vars);
        vars
    }

    pub fn dump(&self) -> String {
        match self {
            InstructionKind::FunctionCall(dest, info) => {
                format!("{} = {}", dest, info)
            }
            InstructionKind::Converter(dest, source) => {
                format!("{} = convert({})", dest, source)
            }
            InstructionKind::MethodCall(dest, receiver, name, args) => {
                format!("{} = methodcall({}.{}({}))", dest, receiver, name, args)
            }
            InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                format!("{} = dynamic_call({}, {:?})", dest, callable, args)
            }
            InstructionKind::FieldRef(dest, v, fields) => format!(
                "{} = ({}){}",
                dest,
                v,
                fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(".")
            ),
            InstructionKind::Bind(v, rhs, mutable) => {
                if *mutable {
                    format!("mut {} = {}", v, rhs)
                } else {
                    format!("{} = {}", v, rhs)
                }
            }
            InstructionKind::Tuple(dest, args) => {
                format!("{} = tuple({:?})", dest, args)
            }
            InstructionKind::StringLiteral(dest, v) => {
                format!("{} = s:[{}]", dest, v)
            }
            InstructionKind::IntegerLiteral(dest, v) => {
                format!("{} = i:[{}]", dest, v)
            }
            InstructionKind::CharLiteral(dest, v) => {
                format!("{} = c:[{}]", dest, v)
            }
            InstructionKind::Return(dest, id) => {
                format!("{} = return({})", dest, id)
            }
            InstructionKind::Ref(dest, id) => format!("{} = &({})", dest, id),
            InstructionKind::PtrOf(var, target) => format!("{} = ptr({})", var, target),
            InstructionKind::DropPath(path) => format!("drop_path({})", path),
            InstructionKind::DropMetadata(id) => {
                format!("drop_metadata({})", id)
            }
            InstructionKind::Drop(dest, value) => {
                format!("drop({}/{})", dest, value)
            }
            InstructionKind::Jump(dest, id) => {
                format!("{} = jump({})", dest, id)
            }
            InstructionKind::Assign(v, arg) => {
                format!("assign({}, {})", v, arg)
            }
            InstructionKind::FieldAssign(v, arg, fields) => {
                let fields = fields.iter().map(|info| info.to_string()).collect::<Vec<_>>().join(".");
                format!("fieldassign({}, {}, {})", v, arg, fields)
            }
            InstructionKind::AddressOfField(v, receiver, fields) => {
                let fields = fields.iter().map(|info| info.to_string()).collect::<Vec<_>>().join(".");
                format!("address_of_field({}, {}, {})", v, receiver, fields)
            }
            InstructionKind::DeclareVar(v, mutability) => {
                format!("declare({}, {:?})", v, mutability)
            }
            InstructionKind::Transform(dest, arg, info) => {
                format!("{} = transform({}, {:?})", dest, arg, info)
            }
            InstructionKind::EnumSwitch(root, cases) => {
                format!("enumswitch({}, {:?})", root, cases)
            }
            InstructionKind::IntegerSwitch(root, cases) => {
                format!("integerswitch({}, {:?})", root, cases)
            }
            InstructionKind::BlockStart(info) => {
                format!("blockstart({})", info)
            }
            InstructionKind::BlockEnd(info) => format!("blockend({})", info),
            InstructionKind::With(v, info) => {
                format!("with({}, {})", v, info)
            }
            InstructionKind::ReadImplicit(var, index) => {
                format!("read_implicit({}, {})", var, index)
            }
            InstructionKind::WriteImplicit(index, var) => {
                format!("write_implicit({}, {})", var, index)
            }
            InstructionKind::LoadPtr(dest, src) => {
                format!("load_ptr({}, {})", dest, src)
            }
            InstructionKind::StorePtr(dest, src) => {
                format!("store_ptr({}, {})", dest, src)
            }
            InstructionKind::CreateClosure(var, info) => {
                format!("create_closure({}, {})", var, info)
            }
            InstructionKind::ClosureReturn(blockId, variable, return_value) => {
                format!("closure_return({}, {}, {})", blockId, variable, return_value)
            }
            InstructionKind::IntegerOp(dest, v1, v2, op) => {
                let op_str = match op {
                    IntegerOp::Add => "+",
                    IntegerOp::Sub => "-",
                    IntegerOp::Mul => "*",
                    IntegerOp::Div => "/",
                    IntegerOp::Mod => "%",
                    IntegerOp::Eq => "==",
                    IntegerOp::LessThan => "<",
                    IntegerOp::ShiftLeft => "<<",
                    IntegerOp::ShiftRight => ">>",
                    IntegerOp::BitAnd => "&",
                    IntegerOp::BitOr => "|",
                    IntegerOp::BitXor => "^",
                };
                format!("{} = ({} {} {})", dest, v1, op_str, v2)
            }
            InstructionKind::Yield(v, a) => {
                format!("{} = yield({})", v, a)
            }
        }
    }

    pub fn getShortName(&self) -> &str {
        match self {
            InstructionKind::FunctionCall(_, _) => "call",
            InstructionKind::Converter(_, _) => "convert",
            InstructionKind::MethodCall(_, _, _, _) => "methodcall",
            InstructionKind::DynamicFunctionCall(_, _, _) => "dynamic_call",
            InstructionKind::FieldRef(_, _, _) => "fieldref",
            InstructionKind::Bind(_, _, _) => "bind",
            InstructionKind::Tuple(_, _) => "tuple",
            InstructionKind::StringLiteral(_, _) => "string_literal",
            InstructionKind::IntegerLiteral(_, _) => "integer_literal",
            InstructionKind::CharLiteral(_, _) => "char_literal",
            InstructionKind::Return(_, _) => "return",
            InstructionKind::Ref(_, _) => "ref",
            InstructionKind::PtrOf(_, _) => "ptr_of",
            InstructionKind::DropPath(_) => "drop_path",
            InstructionKind::DropMetadata(_) => "drop_metadata",
            InstructionKind::Drop(_, _) => "drop",
            InstructionKind::Jump(_, _) => "jump",
            InstructionKind::Assign(_, _) => "assign",
            InstructionKind::FieldAssign(_, _, _) => "field_assign",
            InstructionKind::AddressOfField(_, _, _) => "address_of_field",
            InstructionKind::DeclareVar(_, _) => "declare_var",
            InstructionKind::Transform(_, _, _) => "transform",
            InstructionKind::EnumSwitch(_, _) => "enum_switch",
            InstructionKind::IntegerSwitch(_, _) => "integer_switch",
            InstructionKind::BlockStart(_) => "block_start",
            InstructionKind::BlockEnd(_) => "block_end",
            InstructionKind::With(_, _) => "with",
            InstructionKind::ReadImplicit(_, _) => "read_implicit",
            InstructionKind::WriteImplicit(_, _) => "write_implicit",
            InstructionKind::LoadPtr(_, _) => "load_ptr",
            InstructionKind::StorePtr(_, _) => "store_ptr",
            InstructionKind::CreateClosure(_, _) => "create_closure",
            InstructionKind::ClosureReturn(_, _, _) => "closure_return",
            InstructionKind::IntegerOp(_, _, _, _) => "integer_op",
            InstructionKind::Yield(_, _) => "yield",
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

#[derive(PartialEq, Clone)]
pub struct EffectHandler {
    pub method: QualifiedName,
    pub handler: QualifiedName,
    pub location: Location,
    pub optional: bool,
}

impl Display for EffectHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.method, self.handler)
    }
}

#[derive(Clone, PartialEq)]
pub struct ImplicitHandler {
    pub implicit: QualifiedName,
    pub var: Variable,
    pub location: Location,
}

impl Display for ImplicitHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.implicit, self.var)
    }
}

#[derive(Clone, PartialEq)]
pub enum WithContext {
    EffectHandler(EffectHandler),
    Implicit(ImplicitHandler),
}

impl Display for WithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WithContext::EffectHandler(handler) => write!(f, "effect_handler({})", handler),
            WithContext::Implicit(handler) => write!(f, "implicit_handler({})", handler),
        }
    }
}

impl Debug for WithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImplicitContextOperation {
    Copy(ImplicitContextIndex),
    Add(ImplicitContextIndex, Variable),
    Overwrite(ImplicitContextIndex, Variable),
}

impl Display for ImplicitContextOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImplicitContextOperation::Copy(index) => write!(f, "copy({})", index),
            ImplicitContextOperation::Add(index, var) => write!(f, "add({}, {})", index, var),
            ImplicitContextOperation::Overwrite(index, var) => write!(f, "overwrite({}, {})", index, var),
        }
    }
}

impl Debug for ImplicitContextOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
