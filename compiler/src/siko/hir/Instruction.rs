use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::backend::drop::Path::Path;
use crate::siko::hir::Block::BlockId;
use crate::siko::hir::Type::formatTypes;
use crate::siko::hir::Variable::CopyMap;
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
pub struct CallInfo {
    pub name: QualifiedName,
    pub args: Vec<Variable>,
    pub context: Option<CallContextInfo>,
    pub instanceRefs: Vec<InstanceReference>,
}

impl CallInfo {
    pub fn new(name: QualifiedName, args: Vec<Variable>) -> Self {
        CallInfo {
            name,
            args,
            context: None,
            instanceRefs: Vec::new(),
        }
    }

    pub fn copy(&self, map: &mut CopyMap) -> CallInfo {
        CallInfo {
            name: self.name.clone(),
            args: self.args.iter().map(|a| a.copy(map)).collect(),
            context: self.context.clone(),
            instanceRefs: self.instanceRefs.clone(),
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
            "function_call({}, [{}]{}{})",
            self.name,
            self.args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "),
            contextStr,
            instanceStr
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

    pub fn copy(&self, map: &mut CopyMap) -> ClosureCreateInfo {
        ClosureCreateInfo {
            closureParams: self.closureParams.iter().map(|p| p.copy(map)).collect(),
            body: self.body,
            name: self.name.clone(),
            fnArgCount: self.fnArgCount,
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
pub enum InstructionKind {
    FunctionCall(Variable, CallInfo),
    Converter(Variable, Variable),
    MethodCall(Variable, Variable, String, Vec<Variable>),
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
    pub fn copy(&self, map: &mut CopyMap) -> InstructionKind {
        match self {
            InstructionKind::FunctionCall(v, info) => InstructionKind::FunctionCall(v.copy(map), info.copy(map)),
            InstructionKind::Converter(v1, v2) => InstructionKind::Converter(v1.copy(map), v2.copy(map)),
            InstructionKind::MethodCall(v, r, n, a) => {
                InstructionKind::MethodCall(v.copy(map), r.copy(map), n.clone(), a.clone())
            }
            InstructionKind::DynamicFunctionCall(v, f, a) => {
                InstructionKind::DynamicFunctionCall(v.copy(map), f.copy(map), a.clone())
            }
            InstructionKind::FieldRef(v, r, i) => InstructionKind::FieldRef(v.copy(map), r.copy(map), i.clone()),
            InstructionKind::Bind(v, s, m) => InstructionKind::Bind(v.copy(map), s.copy(map), *m),
            InstructionKind::Tuple(v, a) => {
                InstructionKind::Tuple(v.copy(map), a.iter().map(|x| x.copy(map)).collect())
            }
            InstructionKind::StringLiteral(v, l) => InstructionKind::StringLiteral(v.copy(map), l.clone()),
            InstructionKind::IntegerLiteral(v, l) => InstructionKind::IntegerLiteral(v.copy(map), l.clone()),
            InstructionKind::CharLiteral(v, l) => InstructionKind::CharLiteral(v.copy(map), l.clone()),
            InstructionKind::Return(v, a) => InstructionKind::Return(v.copy(map), a.copy(map)),
            InstructionKind::Ref(v, a) => InstructionKind::Ref(v.copy(map), a.copy(map)),
            InstructionKind::PtrOf(v, a) => InstructionKind::PtrOf(v.copy(map), a.copy(map)),
            InstructionKind::DropPath(p) => InstructionKind::DropPath(p.clone()),
            InstructionKind::DropMetadata(k) => InstructionKind::DropMetadata(k.clone()),
            InstructionKind::Drop(v, a) => InstructionKind::Drop(v.clone(), a.clone()),
            InstructionKind::Jump(v, b) => InstructionKind::Jump(v.copy(map), b.clone()),
            InstructionKind::Assign(d, s) => InstructionKind::Assign(d.copy(map), s.copy(map)),
            InstructionKind::FieldAssign(d, r, i) => InstructionKind::FieldAssign(d.copy(map), r.copy(map), i.clone()),
            InstructionKind::AddressOfField(d, r, i) => {
                InstructionKind::AddressOfField(d.copy(map), r.copy(map), i.clone())
            }
            InstructionKind::DeclareVar(v, m) => InstructionKind::DeclareVar(v.copy(map), m.clone()),
            InstructionKind::Transform(d, a, i) => InstructionKind::Transform(d.copy(map), a.copy(map), i.clone()),
            InstructionKind::IntegerSwitch(v, c) => InstructionKind::IntegerSwitch(v.copy(map), c.clone()),
            InstructionKind::With(v, info) => InstructionKind::With(v.copy(map), info.clone()),
            InstructionKind::EnumSwitch(variable, enum_cases) => {
                InstructionKind::EnumSwitch(variable.copy(map), enum_cases.clone())
            }
            InstructionKind::BlockStart(syntax_block_id) => InstructionKind::BlockStart(syntax_block_id.clone()),
            InstructionKind::BlockEnd(syntax_block_id) => InstructionKind::BlockEnd(syntax_block_id.clone()),
            InstructionKind::ReadImplicit(variable, implicit_index) => {
                InstructionKind::ReadImplicit(variable.copy(map), implicit_index.clone())
            }
            InstructionKind::WriteImplicit(implicit_index, variable) => {
                InstructionKind::WriteImplicit(implicit_index.clone(), variable.copy(map))
            }
            InstructionKind::LoadPtr(variable, variable1) => {
                InstructionKind::LoadPtr(variable.copy(map), variable1.copy(map))
            }
            InstructionKind::StorePtr(variable, variable1) => {
                InstructionKind::StorePtr(variable.copy(map), variable1.copy(map))
            }
            InstructionKind::CreateClosure(v, info) => InstructionKind::CreateClosure(v.copy(map), info.copy(map)),
            InstructionKind::ClosureReturn(block_id, variable, return_value) => {
                InstructionKind::ClosureReturn(block_id.clone(), variable.copy(map), return_value.copy(map))
            }
        }
    }

    pub fn setVariableKinds(&self) -> InstructionKind {
        match self {
            InstructionKind::FunctionCall(dest, info) => {
                let mut info = info.clone();
                info.args = useVars(info.args);
                InstructionKind::FunctionCall(dest.clone(), info)
            }
            InstructionKind::Converter(v1, v2) => InstructionKind::Converter(v1.clone(), v2.useVar()),
            InstructionKind::MethodCall(dest, receiver, name, args) => {
                InstructionKind::MethodCall(dest.clone(), receiver.useVar(), name.clone(), useVars(args.clone()))
            }
            InstructionKind::DynamicFunctionCall(dest, closure, args) => {
                InstructionKind::DynamicFunctionCall(dest.clone(), closure.useVar(), useVars(args.clone()))
            }
            InstructionKind::FieldRef(dest, receiver, infos) => {
                InstructionKind::FieldRef(dest.clone(), receiver.useVar(), infos.clone())
            }
            InstructionKind::Bind(dest, src, mutability) => {
                InstructionKind::Bind(dest.clone(), src.useVar(), mutability.clone())
            }
            InstructionKind::Tuple(dest, args) => InstructionKind::Tuple(dest.clone(), useVars(args.clone())),
            InstructionKind::StringLiteral(v, lit) => InstructionKind::StringLiteral(v.clone(), lit.clone()),
            InstructionKind::IntegerLiteral(v, lit) => InstructionKind::IntegerLiteral(v.clone(), lit.clone()),
            InstructionKind::CharLiteral(v, lit) => InstructionKind::CharLiteral(v.clone(), lit.clone()),
            InstructionKind::Return(v, arg) => InstructionKind::Return(v.clone(), arg.useVar()),
            InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.clone(), arg.useVar()),
            InstructionKind::PtrOf(dest, arg) => InstructionKind::PtrOf(dest.clone(), arg.useVar()),
            InstructionKind::DropPath(p) => InstructionKind::DropPath(p.clone()),
            InstructionKind::DropMetadata(kind) => InstructionKind::DropMetadata(kind.clone()),
            InstructionKind::Drop(dest, arg) => InstructionKind::Drop(dest.clone(), arg.useVar()),
            InstructionKind::Jump(v, blockId) => InstructionKind::Jump(v.clone(), blockId.clone()),
            InstructionKind::Assign(dest, src) => InstructionKind::Assign(dest.clone(), src.useVar()),
            InstructionKind::FieldAssign(dest, rhs, infos) => {
                InstructionKind::FieldAssign(dest.clone(), rhs.useVar(), infos.clone())
            }
            InstructionKind::AddressOfField(dest, rhs, infos) => {
                InstructionKind::AddressOfField(dest.clone(), rhs.useVar(), infos.clone())
            }
            InstructionKind::DeclareVar(v, mutability) => InstructionKind::DeclareVar(v.clone(), mutability.clone()),
            InstructionKind::Transform(dest, arg, index) => {
                InstructionKind::Transform(dest.clone(), arg.useVar(), index.clone())
            }
            InstructionKind::EnumSwitch(arg, cases) => InstructionKind::EnumSwitch(arg.useVar(), cases.clone()),
            InstructionKind::IntegerSwitch(arg, cases) => InstructionKind::IntegerSwitch(arg.useVar(), cases.clone()),
            InstructionKind::BlockStart(id) => InstructionKind::BlockStart(id.clone()),
            InstructionKind::BlockEnd(id) => InstructionKind::BlockEnd(id.clone()),
            InstructionKind::With(v, info) => InstructionKind::With(v.clone(), info.clone()),
            InstructionKind::ReadImplicit(v, index) => InstructionKind::ReadImplicit(v.clone(), index.clone()),
            InstructionKind::WriteImplicit(index, v) => InstructionKind::WriteImplicit(index.clone(), v.useVar()),
            InstructionKind::LoadPtr(dest, src) => InstructionKind::LoadPtr(dest.clone(), src.useVar()),
            InstructionKind::StorePtr(dest, src) => InstructionKind::StorePtr(dest.clone(), src.useVar()),
            InstructionKind::CreateClosure(v, info) => {
                let mut info = info.clone();
                info.closureParams = info.closureParams.iter().map(|p| p.useVar()).collect();
                InstructionKind::CreateClosure(v.clone(), info)
            }
            InstructionKind::ClosureReturn(block_id, variable, return_value) => {
                InstructionKind::ClosureReturn(block_id.clone(), variable.clone(), return_value.useVar())
            }
        }
    }

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
        }
    }

    pub fn replaceVar(&self, from: Variable, to: Variable) -> InstructionKind {
        match self {
            InstructionKind::FunctionCall(var, info) => {
                let new_var = var.replace(&from, to.clone());
                let mut info = info.clone();
                info.args = info.args.iter().map(|arg| arg.replace(&from, to.clone())).collect();
                InstructionKind::FunctionCall(new_var, info)
            }
            InstructionKind::Converter(var, source) => {
                let new_var = var.replace(&from, to.clone());
                let new_source = source.replace(&from, to);
                InstructionKind::Converter(new_var, new_source)
            }
            InstructionKind::MethodCall(var, obj, name, args) => {
                let new_var = var.replace(&from, to.clone());
                let new_obj = obj.replace(&from, to.clone());
                let new_args = args.iter().map(|arg| arg.replace(&from, to.clone())).collect();
                InstructionKind::MethodCall(new_var, new_obj, name.clone(), new_args)
            }
            InstructionKind::DynamicFunctionCall(var, func, args) => {
                let new_var = var.replace(&from, to.clone());
                let new_func = func.replace(&from, to.clone());
                let new_args = args.iter().map(|arg| arg.replace(&from, to.clone())).collect();
                InstructionKind::DynamicFunctionCall(new_var, new_func, new_args)
            }
            InstructionKind::FieldRef(var, target, name) => {
                let new_var = var.replace(&from, to.clone());
                let new_target = target.replace(&from, to);
                InstructionKind::FieldRef(new_var, new_target, name.clone())
            }
            InstructionKind::Bind(var, value, mutable) => {
                let new_var = var.replace(&from, to.clone());
                let new_value = value.replace(&from, to);
                InstructionKind::Bind(new_var, new_value, *mutable)
            }
            InstructionKind::Tuple(var, elements) => {
                let new_var = var.replace(&from, to.clone());
                let new_elements = elements.iter().map(|elem| elem.replace(&from, to.clone())).collect();
                InstructionKind::Tuple(new_var, new_elements)
            }
            InstructionKind::StringLiteral(var, value) => {
                let new_var = var.replace(&from, to.clone());
                InstructionKind::StringLiteral(new_var, value.clone())
            }
            InstructionKind::IntegerLiteral(var, value) => {
                let new_var = var.replace(&from, to.clone());
                InstructionKind::IntegerLiteral(new_var, value.clone())
            }
            InstructionKind::CharLiteral(var, value) => {
                let new_var = var.replace(&from, to.clone());
                InstructionKind::CharLiteral(new_var, value.clone())
            }
            InstructionKind::Return(var, value) => {
                let new_var = var.replace(&from, to.clone());
                let new_value = value.replace(&from, to);
                InstructionKind::Return(new_var, new_value)
            }
            InstructionKind::Ref(var, target) => {
                let new_var = var.replace(&from, to.clone());
                let new_target = target.replace(&from, to);
                InstructionKind::Ref(new_var, new_target)
            }
            InstructionKind::PtrOf(var, target) => {
                let new_var = var.replace(&from, to.clone());
                let new_target = target.replace(&from, to);
                InstructionKind::PtrOf(new_var, new_target)
            }
            InstructionKind::DropPath(_) => self.clone(),
            InstructionKind::DropMetadata(_) => self.clone(),
            InstructionKind::Drop(_, _) => self.clone(),
            InstructionKind::Jump(var, id) => {
                let new_var = var.replace(&from, to.clone());
                InstructionKind::Jump(new_var, id.clone())
            }
            InstructionKind::Assign(var, arg) => {
                let new_var = var.replace(&from, to.clone());
                let new_arg = arg.replace(&from, to);
                InstructionKind::Assign(new_var, new_arg)
            }
            InstructionKind::FieldAssign(var, arg, fields) => {
                let new_var = var.replace(&from, to.clone());
                let new_arg = arg.replace(&from, to);
                InstructionKind::FieldAssign(new_var, new_arg, fields.clone())
            }
            InstructionKind::AddressOfField(var, target, fields) => {
                let new_var = var.replace(&from, to.clone());
                let new_target = target.replace(&from, to);
                InstructionKind::AddressOfField(new_var, new_target, fields.clone())
            }
            InstructionKind::DeclareVar(var, mutability) => {
                let new_var = var.replace(&from, to);
                InstructionKind::DeclareVar(new_var, mutability.clone())
            }
            InstructionKind::Transform(var, arg, info) => {
                let new_var = var.replace(&from, to.clone());
                let new_arg = arg.replace(&from, to);
                InstructionKind::Transform(new_var, new_arg, info.clone())
            }
            InstructionKind::EnumSwitch(root, cases) => {
                let new_root = root.replace(&from, to);
                InstructionKind::EnumSwitch(new_root, cases.clone())
            }
            InstructionKind::IntegerSwitch(root, cases) => {
                let new_root = root.replace(&from, to);
                InstructionKind::IntegerSwitch(new_root, cases.clone())
            }
            InstructionKind::BlockStart(info) => InstructionKind::BlockStart(info.clone()),
            InstructionKind::BlockEnd(info) => InstructionKind::BlockEnd(info.clone()),
            InstructionKind::With(v, info) => {
                let mut info = info.clone();
                for c in &mut info.contexts {
                    match c {
                        WithContext::EffectHandler(_) => {}
                        WithContext::Implicit(handler) => {
                            handler.var = handler.var.replace(&from, to.clone());
                        }
                    }
                }
                InstructionKind::With(v.replace(&from, to), info)
            }
            InstructionKind::ReadImplicit(var, name) => {
                InstructionKind::ReadImplicit(var.replace(&from, to.clone()), name.clone())
            }
            InstructionKind::WriteImplicit(name, var) => {
                InstructionKind::WriteImplicit(name.clone(), var.replace(&from, to.clone()))
            }
            InstructionKind::LoadPtr(var, target) => {
                InstructionKind::LoadPtr(var.replace(&from, to.clone()), target.replace(&from, to))
            }
            InstructionKind::StorePtr(var, target) => {
                InstructionKind::StorePtr(var.replace(&from, to.clone()), target.replace(&from, to))
            }
            InstructionKind::CreateClosure(var, info) => {
                let mut info = info.clone();
                info.closureParams = info
                    .closureParams
                    .iter()
                    .map(|p| p.replace(&from, to.clone()))
                    .collect();
                InstructionKind::CreateClosure(var.replace(&from, to.clone()), info)
            }
            InstructionKind::ClosureReturn(block_id, variable, return_value) => {
                InstructionKind::ClosureReturn(block_id.clone(), variable.clone(), return_value.clone())
            }
        }
    }

    pub fn collectVariables(&self) -> Vec<Variable> {
        match self {
            InstructionKind::FunctionCall(var, info) => {
                let mut vars = vec![var.clone()];
                vars.extend(info.args.clone());
                vars
            }
            InstructionKind::Converter(var, target) => {
                vec![var.clone(), target.clone()]
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
            InstructionKind::FieldRef(var, target, _) => {
                vec![var.clone(), target.clone()]
            }
            InstructionKind::Bind(var, value, _) => {
                vec![var.clone(), value.clone()]
            }
            InstructionKind::Tuple(var, elements) => {
                let mut vars = vec![var.clone()];
                vars.extend(elements.clone());
                vars
            }
            InstructionKind::StringLiteral(var, _) => vec![var.clone()],
            InstructionKind::IntegerLiteral(var, _) => vec![var.clone()],
            InstructionKind::CharLiteral(var, _) => vec![var.clone()],
            InstructionKind::Return(var, value) => {
                vec![var.clone(), value.clone()]
            }
            InstructionKind::Ref(var, target) => {
                vec![var.clone(), target.clone()]
            }
            InstructionKind::PtrOf(var, target) => {
                vec![var.clone(), target.clone()]
            }
            InstructionKind::DropPath(_) => vec![],
            InstructionKind::DropMetadata(_) => vec![],
            InstructionKind::Drop(_, _) => vec![],
            InstructionKind::Jump(var, _) => vec![var.clone()],
            InstructionKind::Assign(var, value) => {
                vec![var.clone(), value.clone()]
            }
            InstructionKind::FieldAssign(var, value, _) => {
                vec![var.clone(), value.clone()]
            }
            InstructionKind::AddressOfField(var, target, _) => {
                vec![var.clone(), target.clone()]
            }
            InstructionKind::DeclareVar(var, _) => vec![var.clone()],
            InstructionKind::Transform(var, target, _) => {
                vec![var.clone(), target.clone()]
            }
            InstructionKind::EnumSwitch(var, _) => {
                vec![var.clone()]
            }
            InstructionKind::IntegerSwitch(var, _) => {
                vec![var.clone()]
            }
            InstructionKind::BlockStart(_) => Vec::new(),
            InstructionKind::BlockEnd(_) => Vec::new(),
            InstructionKind::With(v, info) => {
                let mut result = Vec::new();
                for c in &info.contexts {
                    match c {
                        WithContext::EffectHandler(_) => {}
                        WithContext::Implicit(handler) => {
                            result.push(handler.var.clone());
                        }
                    }
                }
                result.push(v.clone());
                result
            }
            InstructionKind::ReadImplicit(var, _) => vec![var.clone()],
            InstructionKind::WriteImplicit(_, var) => vec![var.clone()],
            InstructionKind::LoadPtr(dest, src) => vec![dest.clone(), src.clone()],
            InstructionKind::StorePtr(dest, src) => vec![dest.clone(), src.clone()],
            InstructionKind::CreateClosure(var, info) => {
                let mut vars = vec![var.clone()];
                vars.extend(info.closureParams.clone());
                vars
            }
            InstructionKind::ClosureReturn(_, variable, return_value) => {
                vec![variable.clone(), return_value.clone()]
            }
        }
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
                format!("{} = methodcall({}.{}({:?}))", dest, receiver, name, args)
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

    pub fn copy(&self, map: &mut CopyMap) -> Instruction {
        Instruction {
            implicit: self.implicit,
            kind: self.kind.copy(map),
            location: self.location.clone(),
        }
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
