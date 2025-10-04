use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::siko::hir::{
    Block::{Block, BlockInner},
    Instruction::{Arguments, CallInfo, ClosureCreateInfo, Instruction, InstructionKind, UnresolvedArgument},
    Variable::{Variable, VariableInfo, VariableName},
    VariableAllocator::VariableAllocator,
};

pub trait VariableCopier {
    fn copy(&mut self, var: &Variable) -> Variable;
}

pub struct IdentityCopier {
    map: BTreeMap<*const RefCell<VariableInfo>, Variable>,
}

impl IdentityCopier {
    pub fn new() -> IdentityCopier {
        IdentityCopier { map: BTreeMap::new() }
    }
    fn get(&self, var: &Variable) -> Option<Variable> {
        //println!("Getting var: {:?}", Rc::as_ptr(&var.info));
        self.map.get(&Rc::as_ptr(&var.info)).cloned()
    }

    fn insert(&mut self, from: &Variable, to: Variable) {
        //println!("Mapping var: {:?} => {}", Rc::as_ptr(&from.info), to);
        self.map.insert(Rc::as_ptr(&from.info), to);
    }

    pub fn copy(&mut self, var: &Variable) -> Variable {
        //println!("Copying var: {}", var);
        if let Some(mut v) = self.get(var) {
            v.kind = var.kind.clone();
            v.location = var.location.clone();
            v
        } else {
            let v = var.cloneNew();
            //println!("  copied to: {}", v);
            self.insert(var, v.clone());
            v
        }
    }
}

impl VariableCopier for IdentityCopier {
    fn copy(&mut self, var: &Variable) -> Variable {
        self.copy(var)
    }
}

pub struct VariableInlineCopier {
    allocator: VariableAllocator,
    map: BTreeMap<VariableName, Variable>,
}

impl VariableInlineCopier {
    pub fn new(allocator: VariableAllocator, args: BTreeMap<String, Variable>) -> VariableInlineCopier {
        let mut varMap = BTreeMap::new();
        for (name, arg) in args {
            varMap.insert(VariableName::Arg(name), arg);
        }
        VariableInlineCopier { allocator, map: varMap }
    }
}

impl VariableCopier for VariableInlineCopier {
    fn copy(&mut self, var: &Variable) -> Variable {
        if let Some(v) = self.map.get(&var.name()) {
            return v.clone();
        }
        let newName = self.allocator.allocateNewInlineName(var.name());
        let v = var.cloneInto(newName);
        self.map.insert(var.name(), v.clone());
        v
    }
}

pub struct CopyHandler<'a> {
    copier: &'a mut dyn VariableCopier,
}

impl<'a> CopyHandler<'a> {
    pub fn new(copier: &'a mut dyn VariableCopier) -> CopyHandler<'a> {
        CopyHandler { copier }
    }

    pub fn copy(&mut self, var: &Variable) -> Variable {
        self.copier.copy(var)
    }
}

pub trait VariableCopy {
    fn copy(&self, map: &mut CopyHandler) -> Self;
}

impl VariableCopy for Variable {
    fn copy(&self, map: &mut CopyHandler) -> Variable {
        map.copy(self)
    }
}

impl<T: VariableCopy> VariableCopy for Vec<T> {
    fn copy(&self, map: &mut CopyHandler) -> Vec<T> {
        self.iter().map(|item| item.copy(map)).collect()
    }
}

impl VariableCopy for UnresolvedArgument {
    fn copy(&self, map: &mut CopyHandler) -> UnresolvedArgument {
        match self {
            UnresolvedArgument::Positional(variable) => UnresolvedArgument::Positional(variable.copy(map)),
            UnresolvedArgument::Named(name, location, variable) => {
                UnresolvedArgument::Named(name.clone(), location.clone(), variable.copy(map))
            }
        }
    }
}

impl VariableCopy for Arguments {
    fn copy(&self, map: &mut CopyHandler) -> Arguments {
        match self {
            Arguments::Resolved(vars) => Arguments::Resolved(vars.copy(map)),
            Arguments::Unresolved(args) => Arguments::Unresolved(args.copy(map)),
        }
    }
}

impl VariableCopy for CallInfo {
    fn copy(&self, map: &mut CopyHandler) -> CallInfo {
        CallInfo {
            name: self.name.clone(),
            args: self.args.copy(map),
            context: self.context.clone(),
            instanceRefs: self.instanceRefs.clone(),
            coroutineSpawn: self.coroutineSpawn,
        }
    }
}

impl VariableCopy for ClosureCreateInfo {
    fn copy(&self, map: &mut CopyHandler) -> ClosureCreateInfo {
        ClosureCreateInfo {
            closureParams: self.closureParams.copy(map),
            context: self.context.clone(),
            body: self.body,
            name: self.name.clone(),
            fnArgCount: self.fnArgCount,
        }
    }
}

impl VariableCopy for InstructionKind {
    fn copy(&self, map: &mut CopyHandler) -> InstructionKind {
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
            InstructionKind::Tuple(v, args) => InstructionKind::Tuple(v.copy(map), args.copy(map)),
            InstructionKind::StringLiteral(v, l) => InstructionKind::StringLiteral(v.copy(map), l.clone()),
            InstructionKind::IntegerLiteral(v, l) => InstructionKind::IntegerLiteral(v.copy(map), l.clone()),
            InstructionKind::CharLiteral(v, l) => InstructionKind::CharLiteral(v.copy(map), l.clone()),
            InstructionKind::Return(v, a) => InstructionKind::Return(v.copy(map), a.copy(map)),
            InstructionKind::Ref(v, a) => InstructionKind::Ref(v.copy(map), a.copy(map)),
            InstructionKind::PtrOf(v, a) => InstructionKind::PtrOf(v.copy(map), a.copy(map)),
            InstructionKind::DropPath(p) => InstructionKind::DropPath(p.clone()),
            InstructionKind::DropMetadata(k) => InstructionKind::DropMetadata(k.clone()),
            InstructionKind::Drop(v, a) => InstructionKind::Drop(v.copy(map), a.copy(map)),
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
            InstructionKind::IntegerOp(dest, v1, v2, op) => {
                InstructionKind::IntegerOp(dest.copy(map), v1.copy(map), v2.copy(map), op.clone())
            }
            InstructionKind::Yield(v, a) => InstructionKind::Yield(v.copy(map), a.copy(map)),
            InstructionKind::FunctionPtr(v, name) => InstructionKind::FunctionPtr(v.copy(map), name.clone()),
            InstructionKind::FunctionPtrCall(v, f, args) => {
                InstructionKind::FunctionPtrCall(v.copy(map), f.copy(map), args.copy(map))
            }
            InstructionKind::Sizeof(v, t) => InstructionKind::Sizeof(v.copy(map), t.copy(map)),
            InstructionKind::Transmute(v, t) => InstructionKind::Transmute(v.copy(map), t.copy(map)),
            InstructionKind::CreateUninitializedArray(v) => InstructionKind::CreateUninitializedArray(v.copy(map)),
            InstructionKind::ArrayLen(v, arr) => InstructionKind::ArrayLen(v.copy(map), arr.copy(map)),
        }
    }
}

impl VariableCopy for Instruction {
    fn copy(&self, map: &mut CopyHandler) -> Instruction {
        Instruction {
            implicit: self.implicit,
            kind: self.kind.copy(map),
            location: self.location.clone(),
        }
    }
}

impl VariableCopy for BlockInner {
    fn copy(&self, map: &mut CopyHandler) -> BlockInner {
        let instructions = self.instructions.copy(map);
        BlockInner {
            id: self.id,
            instructions,
        }
    }
}

impl VariableCopy for Block {
    fn copy(&self, map: &mut CopyHandler) -> Block {
        Block {
            inner: Rc::new(RefCell::new(self.inner.borrow().copy(map))),
        }
    }
}
