use core::panic;
use std::collections::{BTreeMap, BTreeSet};
use std::vec;

use crate::siko::hir::Block::BlockId;
use crate::siko::hir::BlockBuilder::BlockBuilder;
use crate::siko::hir::Body::Body;
use crate::siko::hir::BodyBuilder::BodyBuilder;
use crate::siko::hir::Data::{Enum, Struct};
use crate::siko::hir::Implicit::Implicit;
use crate::siko::hir::Instruction::{
    ClosureCreateInfo, EffectHandler as HirEffectHandler, FieldId, FieldInfo, ImplicitHandler as HirImplicitHandler,
    ImplicitIndex, InstructionKind, SyntaxBlockId, SyntaxBlockIdSegment, UnresolvedArgument,
    WithContext as HirWithContext, WithInfo,
};
use crate::siko::hir::Variable::{Variable, VariableName};
use crate::siko::location::Location::Location;
use crate::siko::location::Report::ReportContext;
use crate::siko::qualifiedname::builtins::{getVecNewName, getVecPushName};
use crate::siko::qualifiedname::{build, QualifiedName};
use crate::siko::resolver::matchcompiler::Compiler::MatchCompiler;
use crate::siko::syntax::Expr::{BinaryOp, Expr, FunctionArg, SimpleExpr, UnaryOp};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use crate::siko::syntax::Statement::Block;
use crate::siko::syntax::Statement::StatementKind;
use crate::siko::util::Runner::Runner;

use super::Environment::Environment;
use super::Error::ResolverError;
use super::ModuleResolver::ModuleResolver;
use super::TypeResolver::TypeResolver;

fn createOpName(traitName: &str, method: &str) -> QualifiedName {
    build("Std.Ops", traitName).add(method.to_string())
}

fn createCmpOpName(traitName: &str, method: &str) -> QualifiedName {
    build("Std.Cmp", traitName).add(method.to_string())
}

#[derive(Debug, Clone)]
struct LoopInfo {
    body: BlockId,
    exit: BlockId,
    var: Variable,
    result: Variable,
}

pub struct ExprResolver<'a> {
    pub ctx: &'a ReportContext,
    pub bodyBuilder: BodyBuilder,
    syntaxBlockId: u32,
    pub moduleResolver: &'a ModuleResolver<'a>,
    typeResolver: &'a TypeResolver<'a>,
    emptyVariants: &'a BTreeSet<QualifiedName>,
    structs: &'a BTreeMap<QualifiedName, Struct>,
    pub variants: &'a BTreeMap<QualifiedName, QualifiedName>,
    pub enums: &'a BTreeMap<QualifiedName, Enum>,
    implicits: &'a BTreeMap<QualifiedName, Implicit>,
    loopInfos: Vec<LoopInfo>,
    syntaxBlockIds: BTreeMap<BlockId, SyntaxBlockId>,
    resultVar: Option<Variable>,
    lambdaIndex: u32,
    name: &'a QualifiedName,
    jumpBlockId: u32,
    pub runner: Runner,
}

impl<'a> ExprResolver<'a> {
    pub fn new(
        name: &'a QualifiedName,
        bodyBuilder: BodyBuilder,
        ctx: &'a ReportContext,
        moduleResolver: &'a ModuleResolver,
        typeResolver: &'a TypeResolver<'a>,
        emptyVariants: &'a BTreeSet<QualifiedName>,
        structs: &'a BTreeMap<QualifiedName, Struct>,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
        implicits: &'a BTreeMap<QualifiedName, Implicit>,
        runner: Runner,
    ) -> ExprResolver<'a> {
        ExprResolver {
            name,
            ctx: ctx,
            bodyBuilder,
            syntaxBlockId: 0,
            moduleResolver: moduleResolver,
            typeResolver: typeResolver,
            emptyVariants: emptyVariants,
            structs: structs,
            variants: variants,
            enums: enums,
            implicits: implicits,
            loopInfos: Vec::new(),
            syntaxBlockIds: BTreeMap::new(),
            resultVar: None,
            lambdaIndex: 0,
            jumpBlockId: 0,
            runner,
        }
    }

    pub fn createSyntaxBlockIdSegment(&mut self) -> SyntaxBlockIdSegment {
        let blockId = self.syntaxBlockId;
        self.syntaxBlockId += 1;
        SyntaxBlockIdSegment { value: blockId }
    }

    pub fn getNextJumpBlockId(&mut self) -> String {
        let id = self.jumpBlockId;
        self.jumpBlockId += 1;
        format!("block_{}", id)
    }

    fn processFieldAssign<'e, 'f>(
        &mut self,
        receiver: &Expr,
        name: &Identifier,
        env: &'e Environment<'e>,
        rhsId: Variable,
        location: Location,
    ) {
        let mut receiver = receiver;
        let mut fields: Vec<FieldInfo> = Vec::new();
        fields.push(FieldInfo {
            name: FieldId::Named(name.toString()),
            location: name.location(),
            ty: None,
        });
        loop {
            match &receiver.expr {
                SimpleExpr::Value(name) => {
                    let value = env.resolve(&name.toString());
                    match value {
                        Some(value) => {
                            let value = value.withLocation(receiver.location.clone());
                            fields.reverse();
                            self.bodyBuilder
                                .current()
                                .addFieldAssign(value, rhsId, fields, location.clone());
                            return;
                        }
                        None => {
                            ResolverError::UnknownValue(name.name(), name.location()).report(self.ctx);
                        }
                    }
                }
                SimpleExpr::SelfValue => {
                    let selfStr = format!("self");
                    let value = match env.resolve(&selfStr) {
                        Some(var) => var.withLocation(receiver.location.clone()),
                        None => {
                            ResolverError::UnknownValue(selfStr.clone(), receiver.location.clone()).report(self.ctx);
                        }
                    };
                    fields.reverse();
                    self.bodyBuilder
                        .current()
                        .addFieldAssign(value.clone(), rhsId, fields, location.clone());
                    return;
                }
                SimpleExpr::FieldAccess(r, name) => {
                    receiver = r;
                    fields.push(FieldInfo {
                        name: FieldId::Named(name.toString()),
                        location: name.location(),
                        ty: None,
                    });
                }
                _ => {
                    ResolverError::InvalidAssignment(location.clone()).report(self.ctx);
                }
            }
        }
    }

    pub fn createBlock(&mut self, env: &Environment) -> BlockBuilder {
        let blockBuilder = self.bodyBuilder.createBlock();
        self.syntaxBlockIds
            .insert(blockBuilder.getBlockId(), env.getSyntaxBlockId());
        blockBuilder
    }

    pub fn addJump(&mut self, toBlock: BlockId, current: SyntaxBlockId, location: Location) -> Variable {
        let mut builder = self.bodyBuilder.current().clone();
        self.addJumpToBuilder(toBlock, location, current, &mut builder)
    }

    pub fn addJumpToBuilder(
        &mut self,
        toBlock: BlockId,
        location: Location,
        current: SyntaxBlockId,
        builder: &mut BlockBuilder,
    ) -> Variable {
        let parentId = self
            .syntaxBlockIds
            .get(&toBlock)
            .expect("toBlock ID not found in syntaxBlockIds");
        // println!("Adding jump from {} to {} / {}", current, parentId, toBlock);
        let diff = current.differenceToParent(&parentId);
        for d in diff {
            builder.implicit().addBlockEnd(d, location.clone());
        }
        builder.implicit().addJump(toBlock, location)
    }

    pub fn resolveBlock<'e>(
        &mut self,
        block: &Block,
        env: &'e Environment<'e>,
        resultValue: Variable,
    ) -> SyntaxBlockId {
        let syntaxBlockIdItem = self.createSyntaxBlockIdSegment();
        let mut env = Environment::child(env, syntaxBlockIdItem);
        // println!(
        //     "Resolving syntax block {} with var {} current {}",
        //     env.getSyntaxBlockId(),
        //     resultValue,
        //     self.bodyBuilder.getTargetBlockId()
        // );
        if let None = self.syntaxBlockIds.get(&self.bodyBuilder.getTargetBlockId()) {
            // println!(
            //     "Adding syntax block ID for block {}: {}",
            //     self.bodyBuilder.getTargetBlockId(),
            //     env.getSyntaxBlockId()
            // );
            self.syntaxBlockIds
                .insert(self.bodyBuilder.getTargetBlockId(), env.getSyntaxBlockId());
        }
        self.bodyBuilder
            .current()
            .implicit()
            .addBlockStart(env.getSyntaxBlockId(), block.location.clone());
        let mut lastHasSemicolon = false;
        let mut blockValue = self.bodyBuilder.createTempValue(block.location.clone());
        for (index, statement) in block.statements.iter().enumerate() {
            if index == block.statements.len() - 1 && statement.hasSemicolon {
                lastHasSemicolon = true;
            }
            match &statement.kind {
                StatementKind::Let(pat, rhs, ty) => {
                    let rhs = self.resolveExpr(rhs, &mut env);
                    if let Some(ty) = ty {
                        let ty = self.typeResolver.resolveType(ty);
                        rhs.setType(ty);
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
                                    self.bodyBuilder
                                        .current()
                                        .addAssign(value.clone(), rhsId, lhs.location.clone());
                                }
                                None => {
                                    if let Some(irName) = self.moduleResolver.tryResolverName(name) {
                                        if let Some(implicit) = self.implicits.get(&irName) {
                                            if !implicit.mutable {
                                                ResolverError::ImmutableImplicit(name.name(), name.location())
                                                    .report(self.ctx);
                                            }
                                            let kind = InstructionKind::WriteImplicit(
                                                ImplicitIndex::Unresolved(irName),
                                                rhsId,
                                            );
                                            self.bodyBuilder.current().addInstruction(kind, name.location());
                                        }
                                    } else {
                                        ResolverError::UnknownValue(name.name(), name.location()).report(self.ctx);
                                    }
                                }
                            }
                        }
                        SimpleExpr::FieldAccess(receiver, name) => {
                            self.processFieldAssign(receiver, name, &mut env, rhsId, lhs.location.clone());
                        }
                        SimpleExpr::UnaryOp(UnaryOp::Deref, inner) => {
                            let innerId = self.resolveExpr(inner, &mut env);
                            self.bodyBuilder
                                .current()
                                .addInstruction(InstructionKind::StorePtr(innerId, rhsId), lhs.location.clone());
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
            blockValue = self.bodyBuilder.current().implicit().addUnit(block.location.clone());
        }
        if !block.doesNotReturn() {
            self.bodyBuilder
                .current()
                .implicit()
                .addAssign(resultValue.clone(), blockValue, block.location.clone());
            self.bodyBuilder
                .current()
                .implicit()
                .addBlockEnd(env.getSyntaxBlockId(), block.location.clone());
        }
        env.getSyntaxBlockId()
    }

    pub fn resolveExpr(&mut self, expr: &Expr, env: &mut Environment) -> Variable {
        //println!("Resolving expression: {:?}", expr);
        match &expr.expr {
            SimpleExpr::Value(name) => match env.resolve(&name.name()) {
                Some(var) => var.withLocation(expr.location.clone()),
                None => {
                    if let Some(irName) = self.moduleResolver.tryResolverName(name) {
                        if self.implicits.contains_key(&irName) {
                            let implicitVar = self.bodyBuilder.createTempValue(expr.location.clone());
                            self.bodyBuilder
                                .current()
                                .implicit()
                                .addDeclare(implicitVar.clone(), expr.location.clone());
                            let kind: InstructionKind =
                                InstructionKind::ReadImplicit(implicitVar.clone(), ImplicitIndex::Unresolved(irName));
                            self.bodyBuilder.current().addInstruction(kind, name.location());
                            return implicitVar;
                        } else {
                            let var = self.bodyBuilder.createTempValue(expr.location.clone());
                            let kind = InstructionKind::FunctionPtr(var.clone(), irName);
                            self.bodyBuilder.current().addInstruction(kind, name.location());
                            return var;
                        }
                    }
                    ResolverError::UnknownValue(name.name(), name.location()).report(self.ctx);
                }
            },
            SimpleExpr::SelfValue => {
                let selfStr = format!("self");
                match env.resolve(&selfStr) {
                    Some(var) => var.withLocation(expr.location.clone()),
                    None => {
                        ResolverError::UnknownValue(selfStr.clone(), expr.location.clone()).report(self.ctx);
                    }
                }
            }
            SimpleExpr::Name(name) => {
                let irName = self.moduleResolver.resolveName(name);
                if self.emptyVariants.contains(&irName) {
                    return self
                        .bodyBuilder
                        .current()
                        .addFunctionCall(irName, (), expr.location.clone());
                }
                if self.moduleResolver.isGlobal(&irName) {
                    return self
                        .bodyBuilder
                        .current()
                        .addFunctionCall(irName, (), expr.location.clone());
                }
                ResolverError::UnknownValue(name.name(), name.location()).report(self.ctx);
            }
            SimpleExpr::FieldAccess(receiver, name) => {
                if let SimpleExpr::FieldAccess(_, _) = &receiver.expr {
                    let (receiverVar, fields) = self.processFieldRef(expr, env);
                    self.bodyBuilder
                        .current()
                        .addFieldAccess(receiverVar, fields, false, expr.location.clone())
                } else {
                    let receiver = self.resolveExpr(receiver, env);
                    let fieldInfos = vec![FieldInfo {
                        name: FieldId::Named(name.toString()),
                        ty: None,
                        location: name.location(),
                    }];
                    self.bodyBuilder
                        .current()
                        .addFieldAccess(receiver, fieldInfos, false, expr.location.clone())
                }
            }
            SimpleExpr::Call(callable, args) => self.resolveCall(expr, env, callable, args, false),
            SimpleExpr::MethodCall(receiver, name, args) => {
                let receiver = self.resolveExpr(&receiver, env);
                let irArgs = self.processFnArgs(env, args);
                self.bodyBuilder
                    .current()
                    .addMethodCall(name.toString(), receiver, irArgs, expr.location.clone())
            }
            SimpleExpr::TupleIndex(receiver, index) => {
                let receiver = self.resolveExpr(&receiver, env);
                self.bodyBuilder.current().addFieldAccess(
                    receiver,
                    vec![FieldInfo {
                        name: FieldId::Indexed(index.parse().unwrap()),
                        ty: None,
                        location: expr.location.clone(),
                    }],
                    false,
                    expr.location.clone(),
                )
            }
            SimpleExpr::Loop(pattern, init, body) => {
                let initId = self.resolveExpr(&init, env);
                let loopVar = self.bodyBuilder.createTempValue(expr.location.clone());
                let finalValue = self.bodyBuilder.createTempValue(expr.location.clone());
                let resultValue = self.bodyBuilder.createTempValue(expr.location.clone());
                self.bodyBuilder
                    .current()
                    .implicit()
                    .addBind(loopVar.clone(), initId, true, expr.location.clone());
                self.bodyBuilder
                    .current()
                    .implicit()
                    .addDeclare(resultValue.clone(), expr.location.clone());
                let mut loopExitBuilder = self.createBlock(env);
                let mut loopEnv = Environment::child(env, self.createSyntaxBlockIdSegment());
                let mut loopBodyBuilder = self.createBlock(&loopEnv);
                self.bodyBuilder
                    .current()
                    .implicit()
                    .addBlockStart(loopEnv.getSyntaxBlockId(), expr.location.clone());
                self.bodyBuilder
                    .current()
                    .addJump(loopBodyBuilder.getBlockId(), expr.location.clone());
                loopBodyBuilder.current();
                self.resolvePattern(pattern, &mut loopEnv, loopVar.clone());
                self.loopInfos.push(LoopInfo {
                    body: loopBodyBuilder.getBlockId(),
                    exit: loopExitBuilder.getBlockId(),
                    var: loopVar.clone(),
                    result: resultValue.clone(),
                });
                let bodyDoesNotReturn = match &body.expr {
                    SimpleExpr::Block(block) => {
                        self.resolveBlock(block, &loopEnv, loopVar.clone());
                        block.doesNotReturn()
                    }
                    _ => panic!("for body is not a block!"),
                };
                if !bodyDoesNotReturn {
                    self.bodyBuilder
                        .current()
                        .implicit()
                        .addJump(loopBodyBuilder.getBlockId(), expr.location.clone());
                }
                self.loopInfos.pop();
                loopExitBuilder.current();
                loopExitBuilder
                    .implicit()
                    .addBind(finalValue.clone(), resultValue, false, expr.location.clone());
                finalValue
            }
            SimpleExpr::BinaryOp(op, lhs, rhs) => {
                let lhsId = self.resolveExpr(lhs, env);
                let rhsId = self.resolveExpr(rhs, env);
                let name = match op {
                    BinaryOp::And => panic!("And operator reached resolver"),
                    BinaryOp::Or => panic!("Or operator reached resolver"),
                    BinaryOp::Add => createOpName("Add", "add"),
                    BinaryOp::Sub => createOpName("Sub", "sub"),
                    BinaryOp::Mul => createOpName("Mul", "mul"),
                    BinaryOp::Div => createOpName("Div", "div"),
                    BinaryOp::Equal => createCmpOpName("PartialEq", "eq"),
                    BinaryOp::NotEqual => createCmpOpName("PartialEq", "ne"),
                    BinaryOp::LessThan => createCmpOpName("PartialOrd", "lessThan"),
                    BinaryOp::GreaterThan => createCmpOpName("PartialOrd", "greaterThan"),
                    BinaryOp::LessThanOrEqual => createCmpOpName("PartialOrd", "lessOrEqual"),
                    BinaryOp::GreaterThanOrEqual => createCmpOpName("PartialOrd", "greaterOrEqual"),
                    BinaryOp::ShiftLeft => createOpName("ShiftLeft", "shiftLeft"),
                    BinaryOp::ShiftRight => createOpName("ShiftRight", "shiftRight"),
                    BinaryOp::BitAnd => createOpName("BitAnd", "bitAnd"),
                    BinaryOp::BitOr => createOpName("BitOr", "bitOr"),
                    BinaryOp::BitXor => createOpName("BitXor", "bitXor"),
                };
                self.bodyBuilder
                    .current()
                    .addFunctionCall(name, vec![lhsId, rhsId], expr.location.clone())
            }
            SimpleExpr::UnaryOp(op, rhs) => {
                let rhsId = self.resolveExpr(rhs, env);
                let name = match op {
                    UnaryOp::Not => createOpName("Not", "opNot"),
                    UnaryOp::Neg => createOpName("Neg", "negative"),
                    UnaryOp::Deref => {
                        let resVar = self.bodyBuilder.createTempValue(expr.location.clone());
                        self.bodyBuilder
                            .current()
                            .addInstruction(InstructionKind::LoadPtr(resVar.clone(), rhsId), expr.location.clone());
                        return resVar;
                    }
                };
                self.bodyBuilder
                    .current()
                    .addFunctionCall(name, vec![rhsId], expr.location.clone())
            }
            SimpleExpr::Match(body, branches) => {
                let mut processedBranches = Vec::new();
                for branch in branches {
                    if let SimplePattern::OrPattern(pats) = &branch.pattern.pattern {
                        for p in pats {
                            let mut newBranch = branch.clone();
                            newBranch.pattern = p.clone();
                            processedBranches.push(newBranch);
                        }
                    } else {
                        processedBranches.push(branch.clone());
                    }
                }
                //crate::siko::syntax::Format::format_any(expr);
                let bodyId = self.resolveExpr(body, env);
                let runner = self.runner.child("match_compiler");
                let mut matchResolver = MatchCompiler::new(
                    self,
                    bodyId,
                    expr.location.clone(),
                    body.location.clone(),
                    processedBranches.clone(),
                    env,
                    runner.clone(),
                );
                let v = runner.run(|| matchResolver.compile());
                v
            }
            SimpleExpr::Block(block) => {
                let blockValue = self.bodyBuilder.createTempValue(expr.location.clone());
                if !block.doesNotReturn() {
                    self.bodyBuilder
                        .current()
                        .implicit()
                        .addDeclare(blockValue.clone(), expr.location.clone());
                }
                self.resolveBlock(block, env, blockValue.clone());
                blockValue
            }
            SimpleExpr::Tuple(args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    irArgs.push(argId)
                }
                let implicitVar = self.bodyBuilder.createTempValue(expr.location.clone());
                let tupleVar = self.bodyBuilder.current().addTuple(irArgs, expr.location.clone());
                self.bodyBuilder
                    .current()
                    .implicit()
                    .addDeclare(implicitVar.clone(), expr.location.clone());
                self.bodyBuilder
                    .current()
                    .addAssign(implicitVar.clone(), tupleVar, expr.location.clone());
                implicitVar
            }
            SimpleExpr::StringLiteral(v) => self
                .bodyBuilder
                .current()
                .addStringLiteral(v.clone(), expr.location.clone()),
            SimpleExpr::IntegerLiteral(v) => self
                .bodyBuilder
                .current()
                .addIntegerLiteral(v.clone(), expr.location.clone()),
            SimpleExpr::CharLiteral(v) => self
                .bodyBuilder
                .current()
                .addCharLiteral(v.clone(), expr.location.clone()),
            SimpleExpr::Return(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => self.bodyBuilder.current().addUnit(expr.location.clone()),
                };
                let resultVar = self.resultVar.clone().expect("no result variable set for return");
                self.bodyBuilder.current().addInstruction(
                    InstructionKind::Converter(resultVar.clone(), argId.clone()),
                    expr.location.clone(),
                );

                let mut currentSyntaxBlockId = env.getSyntaxBlockId();
                loop {
                    self.bodyBuilder
                        .current()
                        .implicit()
                        .addBlockEnd(currentSyntaxBlockId.clone(), expr.location.clone());
                    let parent = currentSyntaxBlockId.getParent();
                    if parent == currentSyntaxBlockId {
                        break;
                    }
                    currentSyntaxBlockId = parent;
                }

                self.bodyBuilder.current().addReturn(resultVar, expr.location.clone())
            }
            SimpleExpr::Break(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => self.bodyBuilder.current().addUnit(expr.location.clone()),
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.bodyBuilder
                    .current()
                    .addAssign(info.result, argId, expr.location.clone());
                self.addJump(info.exit, env.getSyntaxBlockId(), expr.location.clone())
            }
            SimpleExpr::Continue(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => self.bodyBuilder.current().addUnit(expr.location.clone()),
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::ContinueOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.bodyBuilder
                    .current()
                    .addAssign(info.var, argId, expr.location.clone());
                self.addJump(info.body, env.getSyntaxBlockId(), expr.location.clone())
            }
            SimpleExpr::Ref(arg, isRaw) => {
                if let SimpleExpr::FieldAccess(_, _) = &arg.expr {
                    let (receiverVar, fields) = self.processFieldRef(arg, env);
                    if *isRaw {
                        let addrVar = self.bodyBuilder.createTempValue(expr.location.clone());
                        self.bodyBuilder.current().addInstruction(
                            InstructionKind::AddressOfField(addrVar.clone(), receiverVar, fields),
                            expr.location.clone(),
                        );
                        addrVar
                    } else {
                        self.bodyBuilder
                            .current()
                            .addFieldAccess(receiverVar, fields, true, expr.location.clone())
                    }
                } else {
                    let argVar = self.resolveExpr(arg, env);
                    if *isRaw {
                        let addrVar = self.bodyBuilder.createTempValue(expr.location.clone());
                        self.bodyBuilder
                            .current()
                            .addInstruction(InstructionKind::PtrOf(addrVar.clone(), argVar), expr.location.clone());
                        return addrVar;
                    }
                    self.bodyBuilder.current().addRef(argVar, expr.location.clone())
                }
            }
            SimpleExpr::List(args) => {
                let mut listVar =
                    self.bodyBuilder
                        .current()
                        .addFunctionCall(getVecNewName(), (), expr.location.clone());
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    listVar = self.bodyBuilder.current().addFunctionCall(
                        getVecPushName(),
                        vec![listVar, argId],
                        expr.location.clone(),
                    );
                }
                listVar
            }
            SimpleExpr::With(with) => {
                let mut handlers = Vec::new();
                for contextHandler in &with.handlers {
                    let resolvedName = self.moduleResolver.resolveName(&contextHandler.name);
                    if self.implicits.get(&resolvedName).is_some() {
                        let handlerName = env.resolve(&contextHandler.handler.name());
                        match handlerName {
                            Some(name) => {
                                let name = name.withLocation(contextHandler.handler.location().clone());
                                handlers.push(HirWithContext::Implicit(HirImplicitHandler {
                                    implicit: resolvedName.clone(),
                                    var: name,
                                    location: contextHandler.handler.location(),
                                }));
                            }
                            None => {
                                ResolverError::UnknownValue(
                                    contextHandler.handler.name(),
                                    contextHandler.handler.location(),
                                )
                                .report(self.ctx);
                            }
                        }
                    } else {
                        let handlerName = self.moduleResolver.resolveName(&contextHandler.handler);
                        handlers.push(HirWithContext::EffectHandler(HirEffectHandler {
                            method: resolvedName,
                            handler: handlerName,
                            location: contextHandler.handler.location(),
                            optional: contextHandler.optional,
                        }));
                    }
                }
                let withResultVar = self.bodyBuilder.createTempValue(expr.location.clone());
                let mut withBodyBuilder = self.createBlock(&env);
                self.bodyBuilder
                    .current()
                    .implicit()
                    .addDeclare(withResultVar.clone(), expr.location.clone());

                let parentBlockId = self.bodyBuilder.current().getBlockId();
                //self.bodyBuilder.current().addInstruction(kind, expr.location.clone());
                withBodyBuilder.current();
                let syntaxBlockId = match &with.body.expr {
                    SimpleExpr::Block(block) => self.resolveBlock(block, &env, withResultVar.clone()),
                    _ => panic!("with body is not a block!"),
                };
                let jumpResultVar = self.bodyBuilder.createTempValue(expr.location.clone());
                let withInfo = WithInfo {
                    contexts: handlers,
                    blockId: withBodyBuilder.getBlockId(),
                    parentSyntaxBlockId: SyntaxBlockId::new(),
                    syntaxBlockId,
                    operations: vec![],
                    contextTypes: vec![],
                };
                let kind = InstructionKind::With(jumpResultVar, withInfo);
                self.bodyBuilder
                    .block(parentBlockId)
                    .addInstruction(kind, expr.location.clone());
                withResultVar
            }
            SimpleExpr::Lambda(params, body) => {
                let currentLambdaIndex = self.lambdaIndex;
                self.lambdaIndex += 1;
                let lambdaVar = self.bodyBuilder.createTempValue(expr.location.clone());
                let targetBlock = self.bodyBuilder.getTargetBlockId();
                let mut savedLoopInfos = Vec::new();
                std::mem::swap(&mut savedLoopInfos, &mut self.loopInfos);
                self.bodyBuilder
                    .current()
                    .addDeclare(lambdaVar.clone(), expr.location.clone());
                let lambdaName = QualifiedName::Lambda(Box::new(self.name.clone()), currentLambdaIndex);
                let mut lambdaEnv = Environment::lambdaEnv(env, self.createSyntaxBlockIdSegment());
                let mut lambdaBodyBuilder = self.createBlock(&lambdaEnv);
                lambdaBodyBuilder.current();
                for (index, p) in params.iter().enumerate() {
                    let pVar = Variable::new(
                        VariableName::LambdaArg(lambdaBodyBuilder.getBlockId(), index as u32),
                        p.location.clone(),
                    );
                    self.resolvePattern(p, &mut lambdaEnv, pVar);
                }
                match &body.expr {
                    SimpleExpr::Block(block) => self.resolveBlock(block, &lambdaEnv, lambdaVar.clone()),
                    _ => panic!("lambda body is not a block!"),
                };
                self.bodyBuilder.current().implicit().addClosureReturn(
                    lambdaBodyBuilder.getBlockId(),
                    lambdaVar.clone(),
                    expr.location.clone(),
                );
                self.loopInfos = savedLoopInfos;
                self.bodyBuilder.setTargetBlockId(targetBlock);
                let captures = lambdaEnv.captures();
                let closureArgs: Vec<_> = captures.keys().cloned().collect();
                let mut lambdaBodyBuilder = self.bodyBuilder.iterator(lambdaBodyBuilder.getBlockId());
                for (index, closureArg) in closureArgs.iter().enumerate() {
                    let pVar = Variable::new(
                        VariableName::ClosureArg(lambdaBodyBuilder.getBlockId(), index as u32),
                        closureArg.location(),
                    );
                    lambdaBodyBuilder.implicit().addBind(
                        captures.get(closureArg).unwrap().clone(),
                        pVar.clone(),
                        false,
                        expr.location.clone(),
                    );
                    lambdaBodyBuilder.step();
                }
                let dest = self.bodyBuilder.createTempValue(expr.location.clone());
                let info = ClosureCreateInfo::new(
                    closureArgs,
                    lambdaBodyBuilder.getBlockId(),
                    lambdaName,
                    params.len() as u32,
                );
                let kind = InstructionKind::CreateClosure(dest.clone(), info);
                self.bodyBuilder.current().addInstruction(kind, expr.location.clone());
                dest
            }
            SimpleExpr::Yield(arg) => {
                let argId = self.resolveExpr(arg, env);
                let yieldVar = self.bodyBuilder.createTempValue(expr.location.clone());
                self.bodyBuilder
                    .current()
                    .addInstruction(InstructionKind::Yield(yieldVar.clone(), argId), expr.location.clone());
                yieldVar
            }
            SimpleExpr::SpawnCoroutine(arg) => match &arg.expr {
                SimpleExpr::Call(callable, args) => {
                    return self.resolveCall(expr, env, callable, args, true);
                }
                _ => ResolverError::InvalidCoroutineBody(arg.location.clone()).report(self.ctx),
            },
        }
    }

    fn resolveCall(
        &mut self,
        expr: &Expr,
        env: &mut Environment<'_>,
        callable: &Box<Expr>,
        args: &Vec<FunctionArg>,
        coroutineSpawn: bool,
    ) -> Variable {
        let irArgs = self.processFnArgs(env, args);
        if coroutineSpawn {
            match &callable.expr {
                SimpleExpr::Value(name) => {
                    let irName = self.moduleResolver.resolveName(name);
                    return self
                        .bodyBuilder
                        .current()
                        .addCoroutineFunctionCall(irName, irArgs, expr.location.clone());
                }
                _ => ResolverError::InvalidCoroutineBody(callable.location.clone()).report(self.ctx),
            }
        }
        match &callable.expr {
            SimpleExpr::Name(name) => {
                let irName = self.moduleResolver.resolveName(name);
                if self.enums.get(&irName).is_some() {
                    ResolverError::NotAConstructor(name.name(), name.location()).report(self.ctx);
                }
                return self
                    .bodyBuilder
                    .current()
                    .addFunctionCall(irName, irArgs, expr.location.clone());
            }
            SimpleExpr::Value(name) => {
                if let Some(newName) = env.resolve(&name.name()) {
                    let name = newName.withLocation(name.location());
                    let args = self.processDynamicCallArgs(irArgs);
                    self.bodyBuilder
                        .current()
                        .addDynamicFunctionCall(name, args, expr.location.clone())
                } else {
                    let irName = self.moduleResolver.resolveName(name);
                    self.bodyBuilder
                        .current()
                        .addFunctionCall(irName, irArgs, expr.location.clone())
                }
            }
            _ => {
                let callableId = self.resolveExpr(&callable, env);
                let args = self.processDynamicCallArgs(irArgs);
                self.bodyBuilder
                    .current()
                    .addDynamicFunctionCall(callableId, args, expr.location.clone())
            }
        }
    }

    fn processFnArgs(&mut self, env: &mut Environment<'_>, args: &Vec<FunctionArg>) -> Vec<UnresolvedArgument> {
        let mut irArgs = Vec::new();
        for arg in args {
            let arg = match arg {
                FunctionArg::Positional(arg) => {
                    let argId = self.resolveExpr(arg, env);
                    UnresolvedArgument::Positional(argId)
                }
                FunctionArg::Named(name, arg) => {
                    let argId = self.resolveExpr(arg, env);
                    UnresolvedArgument::Named(name.toString(), name.location(), argId)
                }
            };
            irArgs.push(arg)
        }
        irArgs
    }

    fn processDynamicCallArgs(&mut self, irArgs: Vec<UnresolvedArgument>) -> Vec<Variable> {
        let mut args = Vec::new();
        for arg in irArgs {
            match arg {
                UnresolvedArgument::Named(name, location, _) => {
                    ResolverError::NamedArgumentInDynamicFunctionCall(name.clone(), location.clone()).report(self.ctx);
                }
                UnresolvedArgument::Positional(arg) => {
                    args.push(arg);
                }
            }
        }
        args
    }

    fn resolvePattern(&mut self, pat: &Pattern, env: &mut Environment, root: Variable) {
        match &pat.pattern {
            SimplePattern::Named(name, args) => {
                let name = &self.moduleResolver.resolveName(name);
                match self.structs.get(name) {
                    Some(structDef) => {
                        if structDef.fields.len() != args.len() {
                            ResolverError::InvalidArgCount(
                                structDef.name.toString(),
                                structDef.fields.len() as i64,
                                args.len() as i64,
                                pat.location.clone(),
                            )
                            .report(self.ctx);
                        }
                        for (index, arg) in args.iter().enumerate() {
                            let field = &structDef.fields[index];
                            let fieldId = self.bodyBuilder.current().addFieldAccess(
                                root.clone(),
                                vec![FieldInfo {
                                    name: FieldId::Named(field.name.clone()),
                                    ty: None,
                                    location: pat.location.clone(),
                                }],
                                false,
                                pat.location.clone(),
                            );
                            self.resolvePattern(arg, env, fieldId);
                        }
                    }
                    _ => ResolverError::NotStructConstructor(name.toString(), pat.location.clone()).report(self.ctx),
                }
                // for (index, arg) in args.iter().enumerate() {
                //     let tupleValue =
                //         self.bodyBuilder
                //             .current()
                //             .addFieldRef(root.clone(), index as i32, pat.location.clone());
                //     self.resolvePattern(arg, env, tupleValue);
                // }
            }
            SimplePattern::Bind(name, mutable) => {
                let new = self.bodyBuilder.createLocalValue(&name.name(), pat.location.clone());
                self.bodyBuilder
                    .current()
                    .addBind(new.clone(), root, *mutable, pat.location.clone());
                env.addValue(name.toString(), new);
            }
            SimplePattern::Tuple(args) => {
                for (index, arg) in args.iter().enumerate() {
                    let tupleValue = self.bodyBuilder.current().addFieldAccess(
                        root.clone(),
                        vec![FieldInfo {
                            name: FieldId::Indexed(index as u32),
                            ty: None,
                            location: pat.location.clone(),
                        }],
                        false,
                        pat.location.clone(),
                    );
                    self.resolvePattern(arg, env, tupleValue);
                }
            }
            SimplePattern::StringLiteral(_) => {
                ResolverError::UnsupportedOrPatternInNonMatch(pat.location.clone()).report(self.ctx);
            }
            SimplePattern::IntegerLiteral(_) => {
                ResolverError::UnsupportedOrPatternInNonMatch(pat.location.clone()).report(self.ctx);
            }
            SimplePattern::Wildcard => {}
            SimplePattern::Guarded(_, _) => {
                ResolverError::UnsupportedOrPatternInNonMatch(pat.location.clone()).report(self.ctx);
            }
            SimplePattern::OrPattern(_) => {
                ResolverError::UnsupportedOrPatternInNonMatch(pat.location.clone()).report(self.ctx);
            }
        }
    }

    pub fn resolve<'e>(&mut self, body: &Block, env: &'e Environment<'e>) {
        let mut blockBuilder = self.bodyBuilder.createBlock();
        blockBuilder.current();
        let functionResult = self.bodyBuilder.createTempValue(body.location.clone());
        self.resultVar = Some(functionResult.clone());
        self.bodyBuilder
            .current()
            .implicit()
            .addDeclare(functionResult.clone(), body.location.clone());
        let syntaxBlockIdItem = self.createSyntaxBlockIdSegment();
        //println!("Resolving block {} with var {} current {}", syntaxBlock, resultValue, self.targetBlockId);
        let mut localEnv = Environment::child(env, syntaxBlockIdItem);
        self.bodyBuilder
            .current()
            .implicit()
            .addBlockStart(localEnv.getSyntaxBlockId(), body.location.clone());
        for value in env.values() {
            blockBuilder
                .implicit()
                .addDeclare(value.1.clone(), body.location.clone());
            let name = self.bodyBuilder.createLocalValue(value.0, body.location.clone());
            blockBuilder.implicit().addBind(
                name.clone(),
                value.1.clone(),
                env.isMutable(value.0),
                value.1.location().clone(),
            );
            localEnv.addValue(value.0.clone(), name);
        }
        let result = self.bodyBuilder.createTempValue(body.location.clone());
        self.bodyBuilder
            .current()
            .implicit()
            .addDeclare(result.clone(), body.location.clone());
        self.resolveBlock(body, &localEnv, result.clone());
        self.bodyBuilder.current().implicit().addInstruction(
            InstructionKind::Converter(functionResult.clone(), result),
            body.location.clone(),
        );
        self.bodyBuilder
            .current()
            .implicit()
            .addBlockEnd(localEnv.getSyntaxBlockId(), body.location.clone());
        self.bodyBuilder
            .current()
            .implicit()
            .addReturn(functionResult, body.location.clone());
    }

    pub fn body(self) -> Body {
        self.bodyBuilder.build()
    }

    pub fn processFieldRef(&mut self, arg: &Expr, env: &mut Environment<'_>) -> (Variable, Vec<FieldInfo>) {
        if let SimpleExpr::FieldAccess(_, _) = &arg.expr {
            let mut fields = vec![];
            let mut current = &*arg;
            loop {
                if let SimpleExpr::FieldAccess(receiver, name) = &current.expr {
                    let field = FieldInfo {
                        name: FieldId::Named(name.toString()),
                        location: name.location(),
                        ty: None,
                    };
                    fields.push(field);
                    current = receiver;
                } else {
                    break;
                }
            }
            let receiverVar = self.resolveExpr(current, env);
            fields.reverse();
            (receiverVar, fields)
        } else {
            panic!("processFieldRef called on non field access");
        }
    }
}
