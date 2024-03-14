use std::collections::BTreeSet;

use crate::siko::ir::Function::{
    Block as IrBlock, BlockId, InstructionId, InstructionKind, ValueKind,
};
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::syntax::Expr::{Expr, SimpleExpr};
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use crate::siko::syntax::Statement::StatementKind;
use crate::siko::{ir::Function::Body, syntax::Statement::Block};

use super::Environment::Environment;
use super::Error::ResolverError;
use super::ModuleResolver::ModuleResolver;

pub struct ExprResolver<'a> {
    body: Body,
    blockId: u32,
    valueId: u32,
    moduleResolver: &'a ModuleResolver,
    emptyVariants: &'a BTreeSet<QualifiedName>,
}

impl<'a> ExprResolver<'a> {
    pub fn new(
        moduleResolver: &'a ModuleResolver,
        emptyVariants: &'a BTreeSet<QualifiedName>,
    ) -> ExprResolver<'a> {
        ExprResolver {
            body: Body::new(),
            blockId: 0,
            valueId: 0,
            moduleResolver: moduleResolver,
            emptyVariants: emptyVariants,
        }
    }

    pub fn resolveBlock<'e>(&mut self, block: &Block, env: &'e Environment<'e>) -> BlockId {
        let blockId = BlockId { id: self.blockId };
        self.blockId += 1;
        let mut irBlock = IrBlock::new(blockId);
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
        blockId
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
                        InstructionKind::ValueRef(name, Vec::new()),
                        expr.location.clone(),
                    );
                }
                None => {
                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report();
                }
            },
            SimpleExpr::SelfValue => {
                return irBlock.add(
                    InstructionKind::ValueRef(ValueKind::Arg("self".to_string()), Vec::new()),
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
                return irBlock.add(
                    InstructionKind::ValueRef(ValueKind::Implicit(id), names),
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
                                InstructionKind::ValueRef(name, Vec::new()),
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
                let trueBranchId = self.resolveExpr(trueBranch, env, irBlock);
                let falseBranchId = match falseBranch {
                    Some(falseBranch) => {
                        let falseBranchId = self.resolveExpr(falseBranch, env, irBlock);
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
            SimpleExpr::BinaryOp(_, _, _) => todo!(),
            SimpleExpr::Match(_, _) => todo!(),
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
            SimpleExpr::Return(_) => todo!(),
            SimpleExpr::Break(_) => todo!(),
            SimpleExpr::Continue(_) => todo!(),
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
            SimplePattern::StringLiteral(_, _) => todo!(),
            SimplePattern::IntegerLiteral(_, _) => todo!(),
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
