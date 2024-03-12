use std::collections::BTreeSet;

use crate::siko::ir::Function::{Block as IrBlock, BlockId, InstructionId, InstructionKind};
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::syntax::Expr::{Expr, SimpleExpr};
use crate::siko::syntax::Pattern::Pattern;
use crate::siko::syntax::Statement::StatementKind;
use crate::siko::util::error;
use crate::siko::{ir::Function::Body, syntax::Statement::Block};

use super::Environment::Environment;
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
        for statement in &block.statements {
            match &statement.kind {
                StatementKind::Let(pat, rhs) => {
                    let rhsId = self.resolveExpr(rhs, &mut env, &mut irBlock);
                    self.resolvePattern(pat, &mut env, &mut irBlock, rhsId);
                }
                StatementKind::Assign(lhs, rhs) => {}
                StatementKind::Expr(expr) => {
                    self.resolveExpr(expr, &mut env, &mut irBlock);
                }
            }
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
                    return irBlock.add(InstructionKind::ValueRef(name));
                }
                None => error(format!("Unknown value {}", name.name)),
            },
            SimpleExpr::SelfValue => todo!(),
            SimpleExpr::Name(name) => {
                let irName = self.moduleResolver.resolverName(name);
                if self.emptyVariants.contains(&irName) {
                    return irBlock.add(InstructionKind::FunctionCall(irName, Vec::new()));
                }
                error(format!("Unsupported expr"));
            }
            SimpleExpr::FieldAccess(_, _) => todo!(),
            SimpleExpr::Call(callable, args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env, irBlock);
                    irArgs.push(argId)
                }
                match &callable.expr {
                    SimpleExpr::Name(name) => {
                        let irName = self.moduleResolver.resolverName(name);
                        return irBlock.add(InstructionKind::FunctionCall(irName, irArgs));
                    }
                    SimpleExpr::Value(name) => {
                        if let Some(name) = env.resolve(&name.name) {
                            let refId = irBlock.add(InstructionKind::ValueRef(name));
                            return irBlock
                                .add(InstructionKind::DynamicFunctionCall(refId, irArgs));
                        } else {
                            let irName = self.moduleResolver.resolverName(name);
                            return irBlock.add(InstructionKind::FunctionCall(irName, irArgs));
                        }
                    }
                    _ => {
                        let callableId = self.resolveExpr(&callable, env, irBlock);
                        return irBlock
                            .add(InstructionKind::DynamicFunctionCall(callableId, irArgs));
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
                return irBlock.add(InstructionKind::If(condId, trueBranchId, falseBranchId));
            }
            SimpleExpr::For(_, _, _) => todo!(),
            SimpleExpr::BinaryOp(_, _, _) => todo!(),
            SimpleExpr::Match(_, _) => todo!(),
            SimpleExpr::Block(block) => {
                let blockId = self.resolveBlock(block, env);
                return irBlock.add(InstructionKind::BlockRef(blockId));
            }
            SimpleExpr::Tuple(args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env, irBlock);
                    irArgs.push(argId)
                }
                return irBlock.add(InstructionKind::Tuple(irArgs));
            }
            SimpleExpr::StringLiteral(_) => todo!(),
            SimpleExpr::IntegerLiteral(_) => todo!(),
            SimpleExpr::CharLiteral(_) => todo!(),
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
        match pat {
            Pattern::Named(name, args) => todo!(),
            Pattern::Bind(name, _) => {
                let valueId = self.valueId;
                self.valueId += 1;
                let new = format!("{}_{}", name.name, valueId);
                let bindId = irBlock.add(InstructionKind::Bind(new.clone(), value));
                env.addValue(name.toString(), new, bindId);
                bindId
            }
            Pattern::Tuple(args) => {
                for (index, arg) in args.iter().enumerate() {
                    let indexId = irBlock.add(InstructionKind::TupleIndex(value, index as u32));
                    self.resolvePattern(arg, env, irBlock, indexId);
                }
                InstructionId::empty()
            }
            Pattern::StringLiteral(_, _) => todo!(),
            Pattern::IntegerLiteral(_, _) => todo!(),
            Pattern::Wildcard => {
                let valueId = self.valueId;
                self.valueId += 1;
                let new = format!("wildcard_{}", valueId);
                irBlock.add(InstructionKind::Bind(new.clone(), value))
            }
        }
    }

    pub fn resolve<'e>(&mut self, body: &Block, env: &'e Environment<'e>) {
        self.resolveBlock(body, env);
    }

    pub fn body(self) -> Body {
        self.body
    }
}
