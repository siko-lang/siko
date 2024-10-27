use core::panic;
use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::Data::Enum;
use crate::siko::hir::Function::{Block as IrBlock, BlockId, InstructionId, InstructionKind, ValueKind};
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

fn createOpName(traitName: &str, method: &str) -> QualifiedName {
    let stdOps = Box::new(QualifiedName::Module("Std.Ops".to_string()));
    QualifiedName::Item(Box::new(QualifiedName::Item(stdOps.clone(), traitName.to_string())), method.to_string())
}

#[derive(Debug, Clone)]
struct LoopInfo {
    body: BlockId,
    exit: BlockId,
    var: String,
}

pub struct ExprResolver<'a> {
    ctx: &'a ReportContext,
    body: Body,
    blockId: u32,
    valueId: u32,
    moduleResolver: &'a ModuleResolver<'a>,
    emptyVariants: &'a BTreeSet<QualifiedName>,
    variants: &'a BTreeMap<QualifiedName, QualifiedName>,
    enums: &'a BTreeMap<QualifiedName, Enum>,
    loopInfos: Vec<LoopInfo>,
    targetBlockId: BlockId,
}

impl<'a> ExprResolver<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        moduleResolver: &'a ModuleResolver,
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
            emptyVariants: emptyVariants,
            variants: variants,
            enums: enums,
            loopInfos: Vec::new(),
            targetBlockId: BlockId::first(),
        }
    }

    fn createBlock(&mut self) -> BlockId {
        let blockId = BlockId { id: self.blockId };
        self.blockId += 1;
        let irBlock = IrBlock::new(blockId);
        self.body.addBlock(irBlock);
        blockId
    }

    fn setTargetBlockId(&mut self, id: BlockId) {
        self.targetBlockId = id;
    }

    fn resolveBlock<'e>(&mut self, block: &Block, env: &'e Environment<'e>) {
        let mut env = Environment::child(env);
        let mut lastHasSemicolon = false;
        for (index, statement) in block.statements.iter().enumerate() {
            if index == block.statements.len() - 1 && statement.hasSemicolon {
                lastHasSemicolon = true;
            }
            match &statement.kind {
                StatementKind::Let(pat, rhs) => {
                    let rhsId = self.resolveExpr(rhs, &mut env);
                    self.resolvePattern(pat, &mut env, rhsId);
                }
                StatementKind::Assign(_lhs, _rhs) => {}
                StatementKind::Expr(expr) => {
                    self.resolveExpr(expr, &mut env);
                }
            }
        }
        if block.statements.is_empty() || lastHasSemicolon {
            self.addImplicitInstruction(InstructionKind::Tuple(Vec::new()), block.location.clone());
        }
    }

    fn addInstruction(&mut self, instruction: InstructionKind, location: Location) -> InstructionId {
        self.addInstructionToBlock(self.targetBlockId, instruction, location, false)
    }

    fn addImplicitInstruction(&mut self, instruction: InstructionKind, location: Location) -> InstructionId {
        self.addInstructionToBlock(self.targetBlockId, instruction, location, true)
    }

    fn addInstructionToBlock(&mut self, id: BlockId, instruction: InstructionKind, location: Location, implicit: bool) -> InstructionId {
        let irBlock = &mut self.body.blocks[id.id as usize];
        return irBlock.addWithImplicit(instruction, location, implicit);
    }

    fn resolveExpr(&mut self, expr: &Expr, env: &mut Environment) -> InstructionId {
        match &expr.expr {
            SimpleExpr::Value(name) => match env.resolve(&name.name) {
                Some(name) => {
                    return self.addInstruction(InstructionKind::ValueRef(name, Vec::new(), Vec::new()), expr.location.clone());
                }
                None => {
                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                }
            },
            SimpleExpr::SelfValue => {
                return self.addInstruction(
                    InstructionKind::ValueRef(ValueKind::Arg("self".to_string(), 0), Vec::new(), Vec::new()),
                    expr.location.clone(),
                )
            }
            SimpleExpr::Name(name) => {
                let irName = self.moduleResolver.resolverName(name);
                if self.emptyVariants.contains(&irName) {
                    return self.addInstruction(InstructionKind::FunctionCall(irName, Vec::new()), expr.location.clone());
                }
                ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
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
                        id = self.resolveExpr(&current, env);
                        break;
                    }
                }
                names.reverse();
                let var_name = format!("tmp_{}", id.simple());
                let bind_id = self.addInstruction(InstructionKind::Bind(var_name.clone(), id), expr.location.clone());
                return self.addInstruction(
                    InstructionKind::ValueRef(ValueKind::Value(var_name, bind_id), names, Vec::new()),
                    expr.location.clone(),
                );
            }
            SimpleExpr::Call(callable, args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    irArgs.push(argId)
                }
                match &callable.expr {
                    SimpleExpr::Name(name) => {
                        let irName = self.moduleResolver.resolverName(name);
                        return self.addInstruction(InstructionKind::FunctionCall(irName, irArgs), expr.location.clone());
                    }
                    SimpleExpr::Value(name) => {
                        if let Some(name) = env.resolve(&name.name) {
                            let refId = self.addInstruction(InstructionKind::ValueRef(name, Vec::new(), Vec::new()), expr.location.clone());
                            return self.addInstruction(InstructionKind::DynamicFunctionCall(refId, irArgs), expr.location.clone());
                        } else {
                            let irName = self.moduleResolver.resolverName(name);
                            return self.addInstruction(InstructionKind::FunctionCall(irName, irArgs), expr.location.clone());
                        }
                    }
                    _ => {
                        let callableId = self.resolveExpr(&callable, env);
                        return self.addInstruction(InstructionKind::DynamicFunctionCall(callableId, irArgs), expr.location.clone());
                    }
                }
            }
            SimpleExpr::If(cond, trueBranch, falseBranch) => {
                let condId = self.resolveExpr(cond, env);
                let name = self.createValue("if_var");
                let declareId = self.addInstruction(InstructionKind::DeclareVar(name.clone()), expr.location.clone());
                let currentBlockId = self.targetBlockId;
                let contBlockId = self.createBlock();
                let trueBlockId = self.createBlock();
                match &trueBranch.expr {
                    SimpleExpr::Block(block) => {
                        self.setTargetBlockId(trueBlockId);
                        self.resolveBlock(block, env);
                        self.addInstruction(
                            InstructionKind::Assign(name.clone(), self.body.getBlockById(trueBlockId).getLastId()),
                            expr.location.clone(),
                        );
                        self.addInstruction(InstructionKind::Jump(contBlockId), expr.location.clone());
                    }
                    _ => panic!("If true branch is not a block!"),
                };
                let falseBranchId = match falseBranch {
                    Some(falseBranch) => {
                        let falseBranchId = match &falseBranch.expr {
                            SimpleExpr::Block(block) => {
                                let falseBlockId = self.createBlock();
                                self.setTargetBlockId(falseBlockId);
                                self.resolveBlock(block, env);
                                self.addInstruction(
                                    InstructionKind::Assign(name.clone(), self.body.getBlockById(falseBlockId).getLastId()),
                                    expr.location.clone(),
                                );
                                self.addInstruction(InstructionKind::Jump(contBlockId), expr.location.clone());
                                falseBlockId
                            }
                            _ => panic!("If false branch is not a block!"),
                        };
                        Some(falseBranchId)
                    }
                    None => None,
                };
                self.addInstructionToBlock(
                    currentBlockId,
                    InstructionKind::If(condId, trueBlockId, falseBranchId),
                    expr.location.clone(),
                    false,
                );

                self.setTargetBlockId(contBlockId);
                let ifValueId = self.addInstruction(
                    InstructionKind::ValueRef(ValueKind::Value(name, declareId), Vec::new(), Vec::new()),
                    expr.location.clone(),
                );
                ifValueId
            }
            SimpleExpr::For(_, _, _) => todo!(),
            SimpleExpr::Loop(pattern, init, body) => {
                let initId = self.resolveExpr(&init, env);
                let name = self.createValue("loop_var");
                let bindId = self.addInstruction(InstructionKind::Bind(name.clone(), initId), init.location.clone());
                let loopBodyId = self.createBlock();
                let loopExitId = self.createBlock();
                let finalValueId = self.addInstructionToBlock(
                    loopExitId,
                    InstructionKind::ValueRef(ValueKind::Value(name.clone(), bindId), Vec::new(), Vec::new()),
                    expr.location.clone(),
                    true,
                );
                self.addInstruction(InstructionKind::Jump(loopBodyId), expr.location.clone());
                let mut loopEnv = Environment::child(env);
                loopEnv.addTmpValue(name.clone(), bindId);
                self.setTargetBlockId(loopBodyId);
                let varRefId = self.addImplicitInstruction(
                    InstructionKind::ValueRef(ValueKind::Value(name.clone(), bindId), Vec::new(), Vec::new()),
                    expr.location.clone(),
                );
                self.resolvePattern(pattern, &mut loopEnv, varRefId);
                self.loopInfos.push(LoopInfo {
                    body: loopBodyId,
                    exit: loopExitId,
                    var: name.clone(),
                });
                match &body.expr {
                    SimpleExpr::Block(block) => self.resolveBlock(block, &loopEnv),
                    _ => panic!("If true branch is not a block!"),
                }
                self.addImplicitInstruction(
                    InstructionKind::Assign(name.clone(), self.body.getBlockById(loopBodyId).getLastId()),
                    init.location.clone(),
                );
                self.addImplicitInstruction(InstructionKind::Jump(loopBodyId), expr.location.clone());
                self.loopInfos.pop();
                self.setTargetBlockId(loopExitId);
                finalValueId
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
                self.addInstruction(InstructionKind::FunctionCall(name, vec![lhsId, rhsId]), expr.location.clone())
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
                self.addInstruction(InstructionKind::FunctionCall(name, vec![rhsId]), expr.location.clone())
            }
            SimpleExpr::Match(body, branches) => {
                let mut patterns = Vec::new();
                for b in branches {
                    patterns.push(b.pattern.clone());
                }
                let mut matchResolver = MatchCompiler::new(self.ctx, body.location.clone(), patterns, self.moduleResolver, self.variants, self.enums);
                matchResolver.check();
                self.addInstruction(InstructionKind::Tuple(vec![]), expr.location.clone())
            }
            SimpleExpr::Block(block) => {
                self.resolveBlock(block, env);
                self.body.getBlockById(self.targetBlockId).getLastId()
            }
            SimpleExpr::Tuple(args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    irArgs.push(argId)
                }
                return self.addInstruction(InstructionKind::Tuple(irArgs), expr.location.clone());
            }
            SimpleExpr::StringLiteral(v) => self.addInstruction(InstructionKind::StringLiteral(v.clone()), expr.location.clone()),
            SimpleExpr::IntegerLiteral(v) => self.addInstruction(InstructionKind::IntegerLiteral(v.clone()), expr.location.clone()),
            SimpleExpr::CharLiteral(v) => self.addInstruction(InstructionKind::CharLiteral(v.clone()), expr.location.clone()),
            SimpleExpr::Return(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => self.addInstruction(InstructionKind::Tuple(Vec::new()), expr.location.clone()),
                };
                return self.addInstruction(InstructionKind::Return(argId), expr.location.clone());
            }
            SimpleExpr::Break(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => self.addInstruction(InstructionKind::Tuple(Vec::new()), expr.location.clone()),
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.addInstruction(InstructionKind::Assign(info.var, argId), expr.location.clone());
                return self.addInstruction(InstructionKind::Jump(info.exit), expr.location.clone());
            }
            SimpleExpr::Continue(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => self.addInstruction(InstructionKind::Tuple(Vec::new()), expr.location.clone()),
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.addInstruction(InstructionKind::Assign(info.var, argId), expr.location.clone());
                return self.addInstruction(InstructionKind::Jump(info.body), expr.location.clone());
            }
            SimpleExpr::Ref(arg) => {
                let arg = self.resolveExpr(arg, env);
                let argInstr = self.body.getInstruction(arg);
                let arg = match &argInstr.kind {
                    InstructionKind::ValueRef(_, _, _) => arg,
                    _ => {
                        let var_name = self.createValue("tmp");
                        let bind_id = self.addInstruction(InstructionKind::Bind(var_name.clone(), arg), expr.location.clone());
                        self.addInstruction(
                            InstructionKind::ValueRef(ValueKind::Value(var_name, bind_id), Vec::new(), Vec::new()),
                            expr.location.clone(),
                        )
                    }
                };
                return self.addInstruction(InstructionKind::Ref(arg), expr.location.clone());
            }
        }
    }

    fn createValue(&mut self, name: &str) -> String {
        let valueId = self.valueId;
        self.valueId += 1;
        format!("{}_{}", name, valueId)
    }

    fn resolvePattern(&mut self, pat: &Pattern, env: &mut Environment, value: InstructionId) {
        match &pat.pattern {
            SimplePattern::Named(_name, _args) => todo!(),
            SimplePattern::Bind(name, _) => {
                let new = self.createValue(&name.name);
                let bindId = self.addInstruction(InstructionKind::Bind(new.clone(), value), pat.location.clone());
                env.addValue(name.toString(), new, bindId);
            }
            SimplePattern::Tuple(args) => {
                for (index, arg) in args.iter().enumerate() {
                    let indexId = self.addInstruction(InstructionKind::TupleIndex(value, index as u32), pat.location.clone());
                    self.resolvePattern(arg, env, indexId);
                }
            }
            SimplePattern::StringLiteral(_) => todo!(),
            SimplePattern::IntegerLiteral(_) => todo!(),
            SimplePattern::Wildcard => {
                let new = self.createValue("wildcard");
                self.addInstruction(InstructionKind::Bind(new.clone(), value), pat.location.clone());
            }
        }
    }

    pub fn resolve<'e>(&mut self, body: &Block, env: &'e Environment<'e>) {
        let id = self.createBlock();
        self.setTargetBlockId(id);
        self.resolveBlock(body, env);
        let lastId = self.body.getBlockById(self.targetBlockId).getLastId();
        let lastInstruction = self.body.getInstruction(lastId);
        if let InstructionKind::Return(_) = lastInstruction.kind {
        } else {
            self.addImplicitInstruction(InstructionKind::Return(lastId), body.location.clone());
        }
        self.body.blocks.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn body(self) -> Body {
        self.body
    }
}
