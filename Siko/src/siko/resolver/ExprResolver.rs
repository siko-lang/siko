use core::panic;
use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::Data::Enum;
use crate::siko::hir::Function::{Block as IrBlock, BlockId, BlockInfo, FieldInfo, InstructionKind, Variable, VariableName};
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
    syntaxBlockId: u32,
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
            syntaxBlockId: 0,
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

    pub fn createSyntaxBlockId(&mut self) -> String {
        let blockId = format!("block{}", self.syntaxBlockId);
        self.syntaxBlockId += 1;
        blockId
    }

    pub fn setTargetBlockId(&mut self, id: BlockId) {
        //println!("Setting target block {} => {}", self.targetBlockId, id);
        self.targetBlockId = id;
    }

    pub fn getTargetBlockId(&mut self) -> BlockId {
        self.targetBlockId
    }

    pub fn indexVar(&mut self, mut var: Variable) -> Variable {
        let index = self.varIndices.entry(var.value.to_string()).or_insert(1);
        var.index = *index;
        *index += 1;
        var
    }

    fn processFieldAssign<'e>(&mut self, receiver: &Expr, name: &Identifier, env: &'e Environment<'e>, rhsId: Variable, location: Location) {
        let mut receiver = receiver;
        let mut fields: Vec<FieldInfo> = Vec::new();
        fields.push(FieldInfo {
            name: name.toString(),
            location: name.location.clone(),
            ty: None,
        });
        loop {
            match &receiver.expr {
                SimpleExpr::Value(name) => {
                    let value = env.resolve(&name.toString());
                    match value {
                        Some(value) => {
                            fields.reverse();
                            self.addInstruction(InstructionKind::FieldAssign(value.clone(), rhsId, fields), location.clone());
                            return;
                        }
                        None => {
                            ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                        }
                    }
                }
                SimpleExpr::SelfValue => {
                    let value = Variable {
                        value: VariableName::Arg(format!("self")),
                        location: receiver.location.clone(),
                        ty: None,
                        index: 0,
                    };
                    fields.reverse();
                    self.addInstruction(InstructionKind::FieldAssign(value.clone(), rhsId, fields), location.clone());
                    return;
                }
                SimpleExpr::FieldAccess(r, name) => {
                    receiver = r;
                    fields.push(FieldInfo {
                        name: name.toString(),
                        location: name.location.clone(),
                        ty: None,
                    });
                }
                _ => {
                    ResolverError::InvalidAssignment(location.clone()).report(self.ctx);
                }
            }
        }
    }

    fn resolveBlock<'e>(&mut self, block: &Block, env: &'e Environment<'e>, resultValue: Variable) {
        let syntaxBlock = self.createSyntaxBlockId();
        //println!("Resolving block {} with var {} current {}", syntaxBlock, resultValue, self.targetBlockId);
        let blockInfo = BlockInfo { id: syntaxBlock };
        self.addImplicitInstruction(InstructionKind::BlockStart(blockInfo.clone()), block.location.clone());
        let mut env = Environment::child(env);
        let mut lastHasSemicolon = false;
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
                                    self.addInstruction(InstructionKind::Assign(value.clone(), rhsId), lhs.location.clone());
                                }
                                None => {
                                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                                }
                            }
                        }
                        SimpleExpr::FieldAccess(receiver, name) => {
                            self.processFieldAssign(receiver, name, &mut env, rhsId, lhs.location.clone());
                        }
                        _ => {
                            ResolverError::InvalidAssignment(lhs.location.clone()).report(self.ctx);
                        }
                    }
                }
                StatementKind::Expr(expr) => {
                    let var = self.resolveExpr(expr, &mut env);
                    blockValue = var;
                }
            }
        }
        if block.statements.is_empty() || lastHasSemicolon {
            let unitValue = self.createValue("unit", block.location.clone());
            self.addImplicitInstruction(InstructionKind::Tuple(unitValue.clone(), Vec::new()), block.location.clone());
            blockValue = unitValue;
        }
        if !block.doesNotReturn() {
            let blockValue = self.indexVar(blockValue);
            self.addImplicitInstruction(InstructionKind::Assign(resultValue.clone(), blockValue), block.location.clone());
        }
        self.addImplicitInstruction(InstructionKind::BlockEnd(blockInfo.clone()), block.location.clone());
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
                Some(mut var) => {
                    var.location = expr.location.clone();
                    self.indexVar(var)
                }
                None => {
                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                }
            },
            SimpleExpr::SelfValue => Variable {
                value: VariableName::Arg(format!("self")),
                location: expr.location.clone(),
                ty: None,
                index: 0,
            },
            SimpleExpr::Name(name) => {
                let irName = self.moduleResolver.resolverName(name);
                if self.emptyVariants.contains(&irName) {
                    let value = self.createValue("call", expr.location.clone());
                    self.addInstruction(InstructionKind::FunctionCall(value.clone(), irName, Vec::new()), expr.location.clone());
                    return value;
                }
                ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
            }
            SimpleExpr::FieldAccess(receiver, name) => {
                let id = self.resolveExpr(receiver, env);
                let value = self.createValue("fieldRef", expr.location.clone());
                self.addInstruction(InstructionKind::FieldRef(value.clone(), id, name.toString()), expr.location.clone());
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
                        self.addInstruction(InstructionKind::FunctionCall(value.clone(), irName, irArgs), expr.location.clone());
                        value
                    }
                    SimpleExpr::Value(name) => {
                        if let Some(name) = env.resolve(&name.name) {
                            let valueRef = self.createValue("valueRef", expr.location.clone());
                            self.addInstruction(InstructionKind::ValueRef(valueRef.clone(), name), expr.location.clone());
                            let value = self.createValue("call", expr.location.clone());
                            self.addInstruction(
                                InstructionKind::DynamicFunctionCall(value.clone(), valueRef, irArgs),
                                expr.location.clone(),
                            );
                            value
                        } else {
                            let irName = self.moduleResolver.resolverName(name);
                            let value = self.createValue("call", expr.location.clone());
                            self.addInstruction(InstructionKind::FunctionCall(value.clone(), irName, irArgs), expr.location.clone());
                            value
                        }
                    }
                    _ => {
                        let callableId = self.resolveExpr(&callable, env);
                        let value = self.createValue("call", expr.location.clone());
                        self.addInstruction(
                            InstructionKind::DynamicFunctionCall(value.clone(), callableId, irArgs),
                            expr.location.clone(),
                        );
                        value
                    }
                }
            }
            SimpleExpr::MethodCall(receiver, name, args) => {
                let receiver = self.resolveExpr(&receiver, env);
                let receiver = self.indexVar(receiver);
                let value = self.createValue("call", expr.location.clone());
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    let argId = self.indexVar(argId);
                    irArgs.push(argId)
                }
                self.addInstruction(
                    InstructionKind::MethodCall(value.clone(), receiver, name.toString(), irArgs),
                    expr.location.clone(),
                );
                value
            }
            SimpleExpr::TupleIndex(receiver, index) => {
                let receiver = self.resolveExpr(&receiver, env);
                let receiver = self.indexVar(receiver);
                let value = self.createValue("tupleIndex", expr.location.clone());
                self.addInstruction(
                    InstructionKind::TupleIndex(value.clone(), receiver, index.parse().unwrap()),
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
                    InstructionKind::ValueRef(finalValue.clone(), name.clone()),
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
                match &body.expr {
                    SimpleExpr::Block(block) => self.resolveBlock(block, &loopEnv, name),
                    _ => panic!("for body is not a block!"),
                };
                let jumpValue = self.createValue("jump", expr.location.clone());
                self.addImplicitInstruction(InstructionKind::Jump(jumpValue.clone(), loopBodyId), expr.location.clone());
                self.loopInfos.pop();
                self.setTargetBlockId(loopExitId);
                finalValue
            }
            SimpleExpr::BinaryOp(op, lhs, rhs) => {
                let lhsId = self.resolveExpr(lhs, env);
                let lhsId = self.indexVar(lhsId);
                let rhsId = self.resolveExpr(rhs, env);
                let rhsId = self.indexVar(rhsId);
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
                    InstructionKind::FunctionCall(value.clone(), name, vec![lhsId, rhsId]),
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
                self.addInstruction(InstructionKind::FunctionCall(value.clone(), name, vec![rhsId]), expr.location.clone());
                value
            }
            SimpleExpr::Match(body, branches) => {
                let bodyId = self.resolveExpr(body, env);
                let mut matchResolver = MatchCompiler::new(self, bodyId, expr.location.clone(), body.location.clone(), branches.clone(), env);
                matchResolver.compile()
            }
            SimpleExpr::Block(block) => {
                let blockValue = self.createValue("blockValue", expr.location.clone());
                if !block.doesNotReturn() {
                    self.addImplicitInstruction(InstructionKind::DeclareVar(blockValue.clone()), expr.location.clone());
                }
                self.resolveBlock(block, env, blockValue.clone());
                self.indexVar(blockValue)
            }
            SimpleExpr::Tuple(args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    irArgs.push(argId)
                }
                let value = self.createValue("tuple", expr.location.clone());
                self.addInstruction(InstructionKind::Tuple(value.clone(), irArgs), expr.location.clone());
                value
            }
            SimpleExpr::StringLiteral(v) => {
                let value = self.createValue("literal", expr.location.clone());
                self.addInstruction(InstructionKind::StringLiteral(value.clone(), v.clone()), expr.location.clone());
                value
            }
            SimpleExpr::IntegerLiteral(v) => {
                let value = self.createValue("lit", expr.location.clone());
                self.addInstruction(InstructionKind::IntegerLiteral(value.clone(), v.clone()), expr.location.clone());
                value
            }
            SimpleExpr::CharLiteral(v) => {
                let value = self.createValue("lit", expr.location.clone());
                self.addInstruction(InstructionKind::CharLiteral(value.clone(), v.clone()), expr.location.clone());
                value
            }
            SimpleExpr::Return(arg) => {
                let value = self.createValue("ret", expr.location.clone());
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => {
                        let value = self.createValue("unit", expr.location.clone());
                        self.addInstruction(InstructionKind::Tuple(value.clone(), Vec::new()), expr.location.clone());
                        value
                    }
                };
                self.addInstruction(InstructionKind::Return(value.clone(), argId), expr.location.clone());
                value
            }
            SimpleExpr::Break(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => {
                        let value = self.createValue("unit", expr.location.clone());
                        self.addInstruction(InstructionKind::Tuple(value.clone(), Vec::new()), expr.location.clone());
                        value
                    }
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.addInstruction(InstructionKind::Assign(info.var, argId), expr.location.clone());
                let jumpValue = self.createValue("jump", expr.location.clone());
                self.addInstruction(InstructionKind::Jump(jumpValue.clone(), info.exit), expr.location.clone());
                jumpValue
            }
            SimpleExpr::Continue(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => {
                        let value = self.createValue("unit", expr.location.clone());
                        self.addInstruction(InstructionKind::Tuple(value.clone(), Vec::new()), expr.location.clone());
                        value
                    }
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.addInstruction(InstructionKind::Assign(info.var, argId), expr.location.clone());
                let jumpValue = self.createValue("jump", expr.location.clone());
                self.addInstruction(InstructionKind::Jump(jumpValue.clone(), info.body), expr.location.clone());
                jumpValue
            }
            SimpleExpr::Ref(arg) => {
                let arg = self.resolveExpr(arg, env);
                let value = self.createValue("ref", expr.location.clone());
                self.addInstruction(InstructionKind::Ref(value.clone(), arg), expr.location.clone());
                value
            }
        }
    }

    pub fn createValue(&mut self, name: &str, location: Location) -> Variable {
        let valueId = self.valueId;
        self.valueId += 1;
        Variable {
            value: VariableName::Local(name.to_string(), valueId),
            location: location,
            ty: None,
            index: 0,
        }
    }

    fn resolvePattern(&mut self, pat: &Pattern, env: &mut Environment, root: Variable) {
        match &pat.pattern {
            SimplePattern::Named(_name, _args) => todo!(),
            SimplePattern::Bind(name, mutable) => {
                let new = self.createValue(&name.name, pat.location.clone());
                self.addInstruction(InstructionKind::Bind(new.clone(), root, *mutable), pat.location.clone());
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
        let functionResult = self.createValue("functionResult", body.location.clone());
        self.addImplicitInstruction(InstructionKind::DeclareVar(functionResult.clone()), body.location.clone());
        self.resolveBlock(body, env, functionResult.clone());
        let retValue = self.createValue("ret", body.location.clone());
        self.addImplicitInstruction(InstructionKind::Return(retValue, functionResult), body.location.clone());
        self.body.blocks.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn body(self) -> Body {
        self.body
    }
}
