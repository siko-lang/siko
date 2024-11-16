use core::panic;
use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::Data::Enum;
use crate::siko::hir::Function::{Block as IrBlock, BlockId, InstructionKind, Variable};
use crate::siko::location::Location::Location;
use crate::siko::location::Report::ReportContext;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::matchcompiler::Compiler::MatchCompiler;
use crate::siko::syntax::Expr::{BinaryOp, Expr, SimpleExpr, UnaryOp};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use crate::siko::syntax::Statement::StatementKind;
use crate::siko::{hir::Function::Body, syntax::Statement::Block};

use super::Environment::Environment;
use super::Error::ResolverError;
use super::ModuleResolver::ModuleResolver;
use super::TypeResolver::TypeResolver;

fn createOpName(traitName: &str, method: &str) -> QualifiedName {
    let stdOps = Box::new(QualifiedName::Module("Std.Ops".to_string()));
    QualifiedName::Item(Box::new(QualifiedName::Item(stdOps.clone(), traitName.to_string())), method.to_string())
}

#[derive(Debug, Clone)]
struct LoopInfo {
    body: BlockId,
    exit: BlockId,
    var: Variable,
}

pub struct ExprResolver<'a> {
    pub ctx: &'a ReportContext,
    pub body: Body,
    blockId: u32,
    valueId: u32,
    pub moduleResolver: &'a ModuleResolver<'a>,
    typeResolver: &'a TypeResolver<'a>,
    emptyVariants: &'a BTreeSet<QualifiedName>,
    pub variants: &'a BTreeMap<QualifiedName, QualifiedName>,
    pub enums: &'a BTreeMap<QualifiedName, Enum>,
    loopInfos: Vec<LoopInfo>,
    targetBlockId: BlockId,
    varIndices: BTreeMap<String, u32>,
}

impl<'a> ExprResolver<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        moduleResolver: &'a ModuleResolver,
        typeResolver: &'a TypeResolver<'a>,
        emptyVariants: &'a BTreeSet<QualifiedName>,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> ExprResolver<'a> {
        ExprResolver {
            ctx: ctx,
            body: Body::new(),
            blockId: 0,
            valueId: 0,
            moduleResolver: moduleResolver,
            typeResolver: typeResolver,
            emptyVariants: emptyVariants,
            variants: variants,
            enums: enums,
            loopInfos: Vec::new(),
            targetBlockId: BlockId::first(),
            varIndices: BTreeMap::new(),
        }
    }

    pub fn createBlock(&mut self) -> BlockId {
        let blockId = BlockId { id: self.blockId };
        self.blockId += 1;
        let irBlock = IrBlock::new(blockId);
        self.body.addBlock(irBlock);
        blockId
    }

    pub fn setTargetBlockId(&mut self, id: BlockId) {
        self.targetBlockId = id;
    }

    pub fn getTargetBlockId(&mut self) -> BlockId {
        self.targetBlockId
    }

    fn indexVar(&mut self, mut var: Variable) -> Variable {
        let index = self.varIndices.entry(var.value.clone()).or_insert(1);
        var.index = *index;
        *index += 1;
        var
    }

    fn resolveBlock<'e>(&mut self, block: &Block, env: &'e Environment<'e>) -> Variable {
        let mut env = Environment::child(env);
        let mut lastHasSemicolon = false;
        let mut lastDoesNotReturn = false;
        let mut blockValue = self.createValue("block", block.location.clone());
        for (index, statement) in block.statements.iter().enumerate() {
            if index == block.statements.len() - 1 && statement.hasSemicolon {
                lastHasSemicolon = true;
            }
            match &statement.kind {
                StatementKind::Let(pat, rhs, ty) => {
                    let rhs = self.resolveExpr(rhs, &mut env);
                    if let Some(ty) = ty {
                        let ty = self.typeResolver.resolveType(ty);
                        self.body.setType(rhs.clone(), ty);
                    }
                    self.resolvePattern(pat, &mut env, rhs);
                }
                StatementKind::Assign(lhs, rhs) => {
                    let rhsId = self.resolveExpr(rhs, &mut env);
                    match &lhs.expr {
                        SimpleExpr::Value(name) => {
                            let value = env.resolve(&name.toString());
                            match value {
                                Some(value) => {
                                    self.addInstruction(InstructionKind::Assign(value.asFixed(), rhsId), lhs.location.clone());
                                }
                                None => {
                                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                                }
                            }
                        }
                        _ => {
                            ResolverError::InvalidAssignment(lhs.location.clone()).report(self.ctx);
                        }
                    }
                }
                StatementKind::Expr(expr) => {
                    lastDoesNotReturn = expr.expr.doesNotReturn();
                    let var = self.resolveExpr(expr, &mut env);
                    blockValue = var;
                }
            }
        }
        if block.statements.is_empty() || lastHasSemicolon {
            if !lastDoesNotReturn {
                let unitValue = self.createValue("unit", block.location.clone());
                self.addImplicitInstruction(InstructionKind::Tuple(unitValue.clone(), Vec::new()), block.location.clone());
                blockValue = unitValue;
            }
        }
        blockValue
    }

    pub fn addInstruction(&mut self, instruction: InstructionKind, location: Location) {
        self.addInstructionToBlock(self.targetBlockId, instruction, location, false)
    }

    pub fn addImplicitInstruction(&mut self, instruction: InstructionKind, location: Location) {
        self.addInstructionToBlock(self.targetBlockId, instruction, location, true)
    }

    pub fn addInstructionToBlock(&mut self, id: BlockId, instruction: InstructionKind, location: Location, implicit: bool) {
        let irBlock = &mut self.body.blocks[id.id as usize];
        return irBlock.addWithImplicit(instruction, location, implicit);
    }

    pub fn resolveExpr(&mut self, expr: &Expr, env: &mut Environment) -> Variable {
        match &expr.expr {
            SimpleExpr::Value(name) => match env.resolve(&name.name) {
                Some(var) => self.indexVar(var),
                None => {
                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                }
            },
            SimpleExpr::SelfValue => Variable {
                value: "self".to_string(),
                location: expr.location.clone(),
                ty: None,
                fixed: false,
                index: 0,
            },
            SimpleExpr::Name(name) => {
                let irName = self.moduleResolver.resolverName(name);
                if self.emptyVariants.contains(&irName) {
                    let value = self.createValue("call", expr.location.clone());
                    self.addInstruction(InstructionKind::FunctionCall(value.asFixed(), irName, Vec::new()), expr.location.clone());
                    return value;
                }
                ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
            }
            SimpleExpr::FieldAccess(receiver, name) => {
                let id = self.resolveExpr(receiver, env);
                let value = self.createValue("fieldRef", expr.location.clone());
                self.addInstruction(InstructionKind::FieldRef(value.asFixed(), id, name.toString()), expr.location.clone());
                value
            }
            SimpleExpr::Call(callable, args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    let argId = self.indexVar(argId);
                    irArgs.push(argId)
                }
                match &callable.expr {
                    SimpleExpr::Name(name) => {
                        let irName = self.moduleResolver.resolverName(name);
                        if self.enums.get(&irName).is_some() {
                            ResolverError::NotAConstructor(name.name.clone(), name.location.clone()).report(self.ctx);
                        }
                        let value = self.createValue("call", expr.location.clone());
                        self.addInstruction(InstructionKind::FunctionCall(value.asFixed(), irName, irArgs), expr.location.clone());
                        value
                    }
                    SimpleExpr::Value(name) => {
                        if let Some(name) = env.resolve(&name.name) {
                            let valueRef = self.createValue("valueRef", expr.location.clone());
                            self.addInstruction(InstructionKind::ValueRef(valueRef.asFixed(), name), expr.location.clone());
                            let value = self.createValue("call", expr.location.clone());
                            self.addInstruction(
                                InstructionKind::DynamicFunctionCall(value.clone(), valueRef, irArgs),
                                expr.location.clone(),
                            );
                            value
                        } else {
                            let irName = self.moduleResolver.resolverName(name);
                            let value = self.createValue("call", expr.location.clone());
                            self.addInstruction(InstructionKind::FunctionCall(value.asFixed(), irName, irArgs), expr.location.clone());
                            value
                        }
                    }
                    _ => {
                        let callableId = self.resolveExpr(&callable, env);
                        let value = self.createValue("call", expr.location.clone());
                        self.addInstruction(
                            InstructionKind::DynamicFunctionCall(value.asFixed(), callableId, irArgs),
                            expr.location.clone(),
                        );
                        value
                    }
                }
            }
            SimpleExpr::MethodCall(receiver, name, args) => {
                let receiver = self.resolveExpr(&receiver, env);
                let value = self.createValue("call", expr.location.clone());
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    let argId = self.indexVar(argId);
                    irArgs.push(argId)
                }
                self.addInstruction(
                    InstructionKind::MethodCall(value.asFixed(), receiver, name.toString(), irArgs),
                    expr.location.clone(),
                );
                value
            }
            SimpleExpr::For(_, _, _) => todo!(),
            SimpleExpr::Loop(pattern, init, body) => {
                let initId = self.resolveExpr(&init, env);
                let name = self.createValue("loopVar", expr.location.clone());
                self.addInstruction(InstructionKind::Bind(name.clone(), initId, true), init.location.clone());
                let loopBodyId = self.createBlock();
                let loopExitId = self.createBlock();
                let finalValue = self.createValue("finalValueRef", expr.location.clone());
                self.addInstructionToBlock(
                    loopExitId,
                    InstructionKind::ValueRef(finalValue.asFixed(), name.clone()),
                    expr.location.clone(),
                    true,
                );
                let jumpValue = self.createValue("jump", expr.location.clone());
                self.addInstruction(InstructionKind::Jump(jumpValue, loopBodyId), expr.location.clone());
                let mut loopEnv = Environment::child(env);
                self.setTargetBlockId(loopBodyId);
                self.resolvePattern(pattern, &mut loopEnv, name.clone());
                self.loopInfos.push(LoopInfo {
                    body: loopBodyId,
                    exit: loopExitId,
                    var: name.clone(),
                });
                let blockValue = match &body.expr {
                    SimpleExpr::Block(block) => self.resolveBlock(block, &loopEnv),
                    _ => panic!("If true branch is not a block!"),
                };
                self.addImplicitInstruction(InstructionKind::Assign(name.clone(), blockValue), init.location.clone());
                let jumpValue = self.createValue("jump", expr.location.clone());
                self.addImplicitInstruction(InstructionKind::Jump(jumpValue.asFixed(), loopBodyId), expr.location.clone());
                self.loopInfos.pop();
                self.setTargetBlockId(loopExitId);
                finalValue
            }
            SimpleExpr::BinaryOp(op, lhs, rhs) => {
                let lhsId = self.resolveExpr(lhs, env);
                let rhsId = self.resolveExpr(rhs, env);
                let name = match op {
                    BinaryOp::And => createOpName("And", "and"),
                    BinaryOp::Or => createOpName("Or", "or"),
                    BinaryOp::Add => createOpName("Add", "add"),
                    BinaryOp::Sub => createOpName("Sub", "sub"),
                    BinaryOp::Mul => createOpName("Mul", "mul"),
                    BinaryOp::Div => createOpName("Div", "div"),
                    BinaryOp::Equal => createOpName("PartialEq", "eq"),
                    BinaryOp::NotEqual => createOpName("PartialEq", "ne"),
                    BinaryOp::LessThan => createOpName("PartialOrd", "lessThan"),
                    BinaryOp::GreaterThan => createOpName("PartialOrd", "greaterThan"),
                    BinaryOp::LessThanOrEqual => createOpName("PartialOrd", "lessOrEqual"),
                    BinaryOp::GreaterThanOrEqual => createOpName("PartialOrd", "greaterOrEqual"),
                };
                let id = Identifier {
                    name: format!("{}", name),
                    location: expr.location.clone(),
                };
                let name = self.moduleResolver.resolverName(&id);
                let value = self.createValue("call", expr.location.clone());
                self.addInstruction(
                    InstructionKind::FunctionCall(value.asFixed(), name, vec![lhsId, rhsId]),
                    expr.location.clone(),
                );
                value
            }
            SimpleExpr::UnaryOp(op, rhs) => {
                let rhsId = self.resolveExpr(rhs, env);
                let name = match op {
                    UnaryOp::Not => createOpName("Not", "not"),
                };
                let id = Identifier {
                    name: format!("{}", name),
                    location: expr.location.clone(),
                };
                let name = self.moduleResolver.resolverName(&id);
                let value = self.createValue("call", expr.location.clone());
                self.addInstruction(InstructionKind::FunctionCall(value.asFixed(), name, vec![rhsId]), expr.location.clone());
                value
            }
            SimpleExpr::Match(body, branches) => {
                let bodyId = self.resolveExpr(body, env);
                let mut matchResolver = MatchCompiler::new(self, bodyId, expr.location.clone(), body.location.clone(), branches.clone(), env);
                matchResolver.compile()
            }
            SimpleExpr::Block(block) => self.resolveBlock(block, env),
            SimpleExpr::Tuple(args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    irArgs.push(argId)
                }
                let value = self.createValue("tuple", expr.location.clone());
                self.addInstruction(InstructionKind::Tuple(value.asFixed(), irArgs), expr.location.clone());
                value
            }
            SimpleExpr::StringLiteral(v) => {
                let value = self.createValue("literal", expr.location.clone());
                self.addInstruction(InstructionKind::StringLiteral(value.asFixed(), v.clone()), expr.location.clone());
                value
            }
            SimpleExpr::IntegerLiteral(v) => {
                let value = self.createValue("lit", expr.location.clone());
                self.addInstruction(InstructionKind::IntegerLiteral(value.asFixed(), v.clone()), expr.location.clone());
                value
            }
            SimpleExpr::CharLiteral(v) => {
                let value = self.createValue("lit", expr.location.clone());
                self.addInstruction(InstructionKind::CharLiteral(value.asFixed(), v.clone()), expr.location.clone());
                value
            }
            SimpleExpr::Return(arg) => {
                let value = self.createValue("ret", expr.location.clone());
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => {
                        let value = self.createValue("unit", expr.location.clone());
                        self.addInstruction(InstructionKind::Tuple(value.asFixed(), Vec::new()), expr.location.clone());
                        value
                    }
                };
                self.addInstruction(InstructionKind::Return(value.asFixed(), argId), expr.location.clone());
                value
            }
            SimpleExpr::Break(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => {
                        let value = self.createValue("unit", expr.location.clone());
                        self.addInstruction(InstructionKind::Tuple(value.asFixed(), Vec::new()), expr.location.clone());
                        value
                    }
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.addInstruction(InstructionKind::Assign(info.var, argId), expr.location.clone());
                let jumpValue = self.createValue("jump", expr.location.clone());
                self.addInstruction(InstructionKind::Jump(jumpValue.asFixed(), info.exit), expr.location.clone());
                jumpValue
            }
            SimpleExpr::Continue(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => {
                        let value = self.createValue("unit", expr.location.clone());
                        self.addInstruction(InstructionKind::Tuple(value.asFixed(), Vec::new()), expr.location.clone());
                        value
                    }
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.addInstruction(InstructionKind::Assign(info.var, argId), expr.location.clone());
                let jumpValue = self.createValue("jump", expr.location.clone());
                self.addInstruction(InstructionKind::Jump(jumpValue.asFixed(), info.body), expr.location.clone());
                jumpValue
            }
            SimpleExpr::Ref(arg) => {
                let arg = self.resolveExpr(arg, env);
                let value = self.createValue("ref", expr.location.clone());
                self.addInstruction(InstructionKind::Ref(value.asFixed(), arg), expr.location.clone());
                value
            }
        }
    }

    pub fn createValue(&mut self, name: &str, location: Location) -> Variable {
        let valueId = self.valueId;
        self.valueId += 1;
        Variable {
            value: format!("{}_{}", name, valueId),
            location: location,
            ty: None,
            fixed: false,
            index: 0,
        }
    }

    fn resolvePattern(&mut self, pat: &Pattern, env: &mut Environment, root: Variable) {
        match &pat.pattern {
            SimplePattern::Named(_name, _args) => todo!(),
            SimplePattern::Bind(name, mutable) => {
                let new = self.createValue(&name.name, pat.location.clone());
                self.addInstruction(InstructionKind::Bind(new.asFixed(), root, *mutable), pat.location.clone());
                env.addValue(name.toString(), new);
            }
            SimplePattern::Tuple(args) => {
                for (index, arg) in args.iter().enumerate() {
                    let tupleValue = self.createValue("tupleIndex", pat.location.clone());
                    self.addInstruction(
                        InstructionKind::TupleIndex(tupleValue.clone(), root.clone(), index as i32),
                        pat.location.clone(),
                    );
                    self.resolvePattern(arg, env, tupleValue);
                }
            }
            SimplePattern::StringLiteral(_) => todo!(),
            SimplePattern::IntegerLiteral(_) => todo!(),
            SimplePattern::Wildcard => {}
        }
    }

    pub fn resolve<'e>(&mut self, body: &Block, env: &'e Environment<'e>) {
        let id = self.createBlock();
        self.setTargetBlockId(id);
        let value = self.resolveBlock(body, env);
        let lastBlock = self.body.getBlockById(self.targetBlockId);
        let mut addReturn = true;
        if let Some(lastInstruction) = lastBlock.instructions.last() {
            if let InstructionKind::Return(_, _) = lastInstruction.kind {
                addReturn = false;
            }
        }
        if addReturn {
            let retValue = self.createValue("ret", body.location.clone());
            self.addImplicitInstruction(InstructionKind::Return(retValue, value), body.location.clone());
        }
        self.body.blocks.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn body(self) -> Body {
        self.body
    }
}
