use core::panic;
use std::collections::{BTreeMap, BTreeSet};

use crate::siko::ir;
use crate::siko::ir::Data::Enum;
use crate::siko::ir::Function::{
    Block as IrBlock, BlockId, InstructionId, InstructionKind, ValueKind,
};
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::syntax::Expr::{BinaryOp, Expr, SimpleExpr, UnaryOp};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use crate::siko::syntax::Statement::StatementKind;
use crate::siko::{ir::Function::Body, syntax::Statement::Block};

use super::Environment::Environment;
use super::Error::ResolverError;
use super::MatchResolver::MatchResolver;
use super::ModuleResolver::ModuleResolver;

fn createOpName(traitName: &str, method: &str) -> QualifiedName {
    let stdOps = Box::new(QualifiedName::Module("Std.Ops".to_string()));
    QualifiedName::Item(
        Box::new(QualifiedName::Item(stdOps.clone(), traitName.to_string())),
        method.to_string(),
    )
}

pub struct ExprResolver<'a> {
    body: Body,
    blockId: u32,
    valueId: u32,
    moduleResolver: &'a ModuleResolver,
    emptyVariants: &'a BTreeSet<QualifiedName>,
    variants: &'a BTreeMap<QualifiedName, QualifiedName>,
    enums: &'a BTreeMap<QualifiedName, Enum>,
    loopIds: Vec<InstructionId>,
}

impl<'a> ExprResolver<'a> {
    pub fn new(
        moduleResolver: &'a ModuleResolver,
        emptyVariants: &'a BTreeSet<QualifiedName>,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> ExprResolver<'a> {
        ExprResolver {
            body: Body::new(),
            blockId: 0,
            valueId: 0,
            moduleResolver: moduleResolver,
            emptyVariants: emptyVariants,
            variants: variants,
            enums: enums,
            loopIds: Vec::new(),
        }
    }

    fn createBlock(&mut self) -> IrBlock {
        let blockId = BlockId { id: self.blockId };
        self.blockId += 1;
        IrBlock::new(blockId)
    }

    pub fn resolveBlock<'e>(&mut self, block: &Block, env: &'e Environment<'e>) -> BlockId {
        let mut irBlock = self.createBlock();
        let id = irBlock.id;
        let mut env = Environment::child(env);
        let mut lastHasSemicolon = false;
        for (index, statement) in block.statements.iter().enumerate() {
            if index == block.statements.len() - 1 && statement.hasSemicolon {
                lastHasSemicolon = true;
            }
            match &statement.kind {
                StatementKind::Let(pat, rhs) => {
                    let rhsId = self.resolveExpr(rhs, &mut env, &mut irBlock);
                    self.resolvePattern(pat, &mut env, &mut irBlock, rhsId);
                }
                StatementKind::Assign(_lhs, _rhs) => {}
                StatementKind::Expr(expr) => {
                    self.resolveExpr(expr, &mut env, &mut irBlock);
                }
            }
        }
        if block.statements.is_empty() || lastHasSemicolon {
            irBlock.add(InstructionKind::Tuple(Vec::new()), block.location.clone());
        }
        self.body.addBlock(irBlock);
        id
    }

    fn resolveExpr(
        &mut self,
        expr: &Expr,
        env: &mut Environment,
        irBlock: &mut IrBlock,
    ) -> InstructionId {
        match &expr.expr {
            SimpleExpr::Value(name) => match env.resolve(&name.name) {
                Some(name) => {
                    return irBlock.add(
                        InstructionKind::ValueRef(name, Vec::new(), Vec::new()),
                        expr.location.clone(),
                    );
                }
                None => {
                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report();
                }
            },
            SimpleExpr::SelfValue => {
                return irBlock.add(
                    InstructionKind::ValueRef(
                        ValueKind::Arg("self".to_string()),
                        Vec::new(),
                        Vec::new(),
                    ),
                    expr.location.clone(),
                )
            }
            SimpleExpr::Name(name) => {
                let irName = self.moduleResolver.resolverName(name);
                if self.emptyVariants.contains(&irName) {
                    return irBlock.add(
                        InstructionKind::FunctionCall(irName, Vec::new()),
                        expr.location.clone(),
                    );
                }
                ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report();
            }
            SimpleExpr::FieldAccess(receiver, name) => {
                let id;
                let mut names = Vec::new();
                let mut current = receiver;
                names.push(name.toString());
                loop {
                    if let SimpleExpr::FieldAccess(receiver, name) = &current.expr {
                        current = receiver;
                        names.push(name.toString());
                    } else {
                        id = self.resolveExpr(&current, env, irBlock);
                        break;
                    }
                }
                names.reverse();
                let var_name = format!("tmp_{}", id.simple());
                let bind_id = irBlock.add(
                    InstructionKind::Bind(var_name.clone(), id),
                    expr.location.clone(),
                );
                return irBlock.add(
                    InstructionKind::ValueRef(
                        ValueKind::Value(var_name, bind_id),
                        names,
                        Vec::new(),
                    ),
                    expr.location.clone(),
                );
            }
            SimpleExpr::Call(callable, args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env, irBlock);
                    irArgs.push(argId)
                }
                match &callable.expr {
                    SimpleExpr::Name(name) => {
                        let irName = self.moduleResolver.resolverName(name);
                        return irBlock.add(
                            InstructionKind::FunctionCall(irName, irArgs),
                            expr.location.clone(),
                        );
                    }
                    SimpleExpr::Value(name) => {
                        if let Some(name) = env.resolve(&name.name) {
                            let refId = irBlock.add(
                                InstructionKind::ValueRef(name, Vec::new(), Vec::new()),
                                expr.location.clone(),
                            );
                            return irBlock.add(
                                InstructionKind::DynamicFunctionCall(refId, irArgs),
                                expr.location.clone(),
                            );
                        } else {
                            let irName = self.moduleResolver.resolverName(name);
                            return irBlock.add(
                                InstructionKind::FunctionCall(irName, irArgs),
                                expr.location.clone(),
                            );
                        }
                    }
                    _ => {
                        let callableId = self.resolveExpr(&callable, env, irBlock);
                        return irBlock.add(
                            InstructionKind::DynamicFunctionCall(callableId, irArgs),
                            expr.location.clone(),
                        );
                    }
                }
            }
            SimpleExpr::If(cond, trueBranch, falseBranch) => {
                let condId = self.resolveExpr(cond, env, irBlock);
                let trueBranchId = match &trueBranch.expr {
                    SimpleExpr::Block(block) => self.resolveBlock(block, env),
                    _ => panic!("If true branch is not a block!"),
                };
                let falseBranchId = match falseBranch {
                    Some(falseBranch) => {
                        let falseBranchId = match &falseBranch.expr {
                            SimpleExpr::Block(block) => self.resolveBlock(block, env),
                            _ => panic!("If false branch is not a block!"),
                        };
                        Some(falseBranchId)
                    }
                    None => None,
                };
                return irBlock.add(
                    InstructionKind::If(condId, trueBranchId, falseBranchId),
                    expr.location.clone(),
                );
            }
            SimpleExpr::For(_, _, _) => todo!(),
            SimpleExpr::Loop(pattern, init, body) => {
                let initId = self.resolveExpr(&init, env, irBlock);
                let loopBlockId = BlockId { id: self.blockId };
                self.blockId += 1;
                let mut loopBlock = IrBlock::new(loopBlockId);
                let mut loopEnv = Environment::child(env);
                let valueId = self.valueId;
                self.valueId += 1;
                let name = format!("loopVar_{}", valueId);
                loopEnv.addLoopValue(name.clone());
                let varRefId = loopBlock.add(
                    InstructionKind::ValueRef(
                        ValueKind::LoopVar(name.clone()),
                        Vec::new(),
                        Vec::new(),
                    ),
                    expr.location.clone(),
                );
                self.resolvePattern(pattern, &mut loopEnv, &mut loopBlock, varRefId);
                let loopId = irBlock.peekNextInstructionId();
                self.loopIds.push(loopId);
                let bodyBlockId = match &body.expr {
                    SimpleExpr::Block(block) => self.resolveBlock(block, &loopEnv),
                    _ => panic!("If true branch is not a block!"),
                };
                self.loopIds.pop();
                loopBlock.add(
                    InstructionKind::BlockRef(bodyBlockId),
                    expr.location.clone(),
                );
                self.body.addBlock(loopBlock);
                return irBlock.add(
                    InstructionKind::Loop(name, initId, loopBlockId),
                    expr.location.clone(),
                );
            }
            SimpleExpr::BinaryOp(op, lhs, rhs) => {
                let lhsId = self.resolveExpr(lhs, env, irBlock);
                let rhsId = self.resolveExpr(rhs, env, irBlock);
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
                irBlock.add(
                    InstructionKind::FunctionCall(name, vec![lhsId, rhsId]),
                    expr.location.clone(),
                )
            }
            SimpleExpr::UnaryOp(op, rhs) => {
                let rhsId = self.resolveExpr(rhs, env, irBlock);
                let name = match op {
                    UnaryOp::Not => createOpName("Not", "not"),
                };
                let id = Identifier {
                    name: format!("{}", name),
                    location: expr.location.clone(),
                };
                let name = self.moduleResolver.resolverName(&id);
                irBlock.add(
                    InstructionKind::FunctionCall(name, vec![rhsId]),
                    expr.location.clone(),
                )
            }
            SimpleExpr::Match(_, branches) => {
                let mut patterns = Vec::new();
                for b in branches {
                    patterns.push(b.pattern.clone());
                }
                let mut matchResolver = MatchResolver::new(patterns);
                matchResolver.check(self.moduleResolver, self.variants, self.enums);
                irBlock.add(InstructionKind::Tuple(vec![]), expr.location.clone())
            }
            SimpleExpr::Block(block) => {
                let blockId = self.resolveBlock(block, env);
                return irBlock.add(InstructionKind::BlockRef(blockId), expr.location.clone());
            }
            SimpleExpr::Tuple(args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env, irBlock);
                    irArgs.push(argId)
                }
                return irBlock.add(InstructionKind::Tuple(irArgs), expr.location.clone());
            }
            SimpleExpr::StringLiteral(v) => irBlock.add(
                InstructionKind::StringLiteral(v.clone()),
                expr.location.clone(),
            ),
            SimpleExpr::IntegerLiteral(v) => irBlock.add(
                InstructionKind::IntegerLiteral(v.clone()),
                expr.location.clone(),
            ),
            SimpleExpr::CharLiteral(v) => irBlock.add(
                InstructionKind::CharLiteral(v.clone()),
                expr.location.clone(),
            ),
            SimpleExpr::Return(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env, irBlock),
                    None => irBlock.add(InstructionKind::Tuple(Vec::new()), expr.location.clone()),
                };
                return irBlock.add(InstructionKind::Return(argId), expr.location.clone());
            }
            SimpleExpr::Break(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env, irBlock),
                    None => irBlock.add(InstructionKind::Tuple(Vec::new()), expr.location.clone()),
                };
                let loopId = match self.loopIds.last() {
                    Some(loopId) => loopId,
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(),
                };
                return irBlock.add(
                    InstructionKind::Break(argId, *loopId),
                    expr.location.clone(),
                );
            }
            SimpleExpr::Continue(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env, irBlock),
                    None => irBlock.add(InstructionKind::Tuple(Vec::new()), expr.location.clone()),
                };
                let loopId = match self.loopIds.last() {
                    Some(loopId) => loopId,
                    None => ResolverError::ContinueOutsideLoop(expr.location.clone()).report(),
                };
                return irBlock.add(
                    InstructionKind::Continue(argId, *loopId),
                    expr.location.clone(),
                );
            }
            SimpleExpr::Ref(arg) => {
                let arg = self.resolveExpr(arg, env, irBlock);
                return irBlock.add(InstructionKind::Ref(arg), expr.location.clone());
            }
        }
    }

    fn resolvePattern(
        &mut self,
        pat: &Pattern,
        env: &mut Environment,
        irBlock: &mut IrBlock,
        value: InstructionId,
    ) -> InstructionId {
        match &pat.pattern {
            SimplePattern::Named(_name, _args) => todo!(),
            SimplePattern::Bind(name, _) => {
                let valueId = self.valueId;
                self.valueId += 1;
                let new = format!("{}_{}", name.name, valueId);
                let bindId = irBlock.add(
                    InstructionKind::Bind(new.clone(), value),
                    pat.location.clone(),
                );
                env.addValue(name.toString(), new, bindId);
                bindId
            }
            SimplePattern::Tuple(args) => {
                for (index, arg) in args.iter().enumerate() {
                    let indexId = irBlock.add(
                        InstructionKind::TupleIndex(value, index as u32),
                        pat.location.clone(),
                    );
                    self.resolvePattern(arg, env, irBlock, indexId);
                }
                InstructionId::empty()
            }
            SimplePattern::StringLiteral(_) => todo!(),
            SimplePattern::IntegerLiteral(_) => todo!(),
            SimplePattern::Wildcard => {
                let valueId = self.valueId;
                self.valueId += 1;
                let new = format!("wildcard_{}", valueId);
                irBlock.add(
                    InstructionKind::Bind(new.clone(), value),
                    pat.location.clone(),
                )
            }
        }
    }

    pub fn resolve<'e>(&mut self, body: &Block, env: &'e Environment<'e>) {
        self.resolveBlock(body, env);
        self.body.blocks.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn body(self) -> Body {
        self.body
    }
}
