use core::panic;
use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::BlockBuilder::BlockBuilder;
use crate::siko::hir::BodyBuilder::BodyBuilder;
use crate::siko::hir::Data::{Enum, Struct};
use crate::siko::hir::Function::BlockId;
use crate::siko::hir::Instruction::{FieldId, FieldInfo, InstructionKind, SyntaxBlockId, SyntaxBlockIdSegment};
use crate::siko::hir::Variable::Variable;
use crate::siko::location::Location::Location;
use crate::siko::location::Report::ReportContext;
use crate::siko::qualifiedname::{getVecNewName, getVecPushName, QualifiedName};
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
    QualifiedName::Item(
        Box::new(QualifiedName::Item(stdOps.clone(), traitName.to_string())),
        method.to_string(),
    )
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
    loopInfos: Vec<LoopInfo>,
    syntaxBlockIds: BTreeMap<BlockId, SyntaxBlockId>,
    resultVar: Option<Variable>,
}

impl<'a> ExprResolver<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        moduleResolver: &'a ModuleResolver,
        typeResolver: &'a TypeResolver<'a>,
        emptyVariants: &'a BTreeSet<QualifiedName>,
        structs: &'a BTreeMap<QualifiedName, Struct>,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> ExprResolver<'a> {
        ExprResolver {
            ctx: ctx,
            bodyBuilder: BodyBuilder::new(),
            syntaxBlockId: 0,
            moduleResolver: moduleResolver,
            typeResolver: typeResolver,
            emptyVariants: emptyVariants,
            structs: structs,
            variants: variants,
            enums: enums,
            loopInfos: Vec::new(),
            syntaxBlockIds: BTreeMap::new(),
            resultVar: None,
        }
    }

    pub fn createSyntaxBlockIdSegment(&mut self) -> SyntaxBlockIdSegment {
        let blockId = self.syntaxBlockId;
        self.syntaxBlockId += 1;
        SyntaxBlockIdSegment { value: blockId }
    }

    fn processFieldAssign<'e>(
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
            location: name.location.clone(),
            ty: None,
        });
        loop {
            match &receiver.expr {
                SimpleExpr::Value(name) => {
                    let value = env.resolve(&name.toString());
                    match value {
                        Some(value) => {
                            let mut value = value;
                            value.location = location.clone();
                            fields.reverse();
                            self.bodyBuilder
                                .current()
                                .addFieldAssign(value, rhsId, fields, location.clone());
                            return;
                        }
                        None => {
                            ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                        }
                    }
                }
                SimpleExpr::SelfValue => {
                    let selfStr = format!("self");
                    let value = match env.resolve(&selfStr) {
                        Some(mut var) => {
                            var.location = receiver.location.clone();
                            var
                        }
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
        //println!("Adding jump from {} to {} / {}", current, parentId, toBlock);
        let diff = current.differenceToParent(&parentId);
        for d in diff {
            builder.implicit().addBlockEnd(d, location.clone());
        }
        builder.implicit().addJump(toBlock, location)
    }

    fn resolveBlock<'e>(&mut self, block: &Block, env: &'e Environment<'e>, resultValue: Variable) {
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
                        self.bodyBuilder.setTypeInBody(rhs.clone(), ty);
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
                                    ResolverError::UnknownValue(name.name.clone(), name.location.clone())
                                        .report(self.ctx);
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
    }

    pub fn resolveExpr(&mut self, expr: &Expr, env: &mut Environment) -> Variable {
        match &expr.expr {
            SimpleExpr::Value(name) => match env.resolve(&name.name) {
                Some(mut var) => {
                    var.location = expr.location.clone();
                    var
                }
                None => {
                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                }
            },
            SimpleExpr::SelfValue => {
                let selfStr = format!("self");
                match env.resolve(&selfStr) {
                    Some(mut var) => {
                        var.location = expr.location.clone();
                        var
                    }
                    None => {
                        ResolverError::UnknownValue(selfStr.clone(), expr.location.clone()).report(self.ctx);
                    }
                }
            }
            SimpleExpr::Name(name) => {
                let irName = self.moduleResolver.resolverName(name);
                if self.emptyVariants.contains(&irName) {
                    return self
                        .bodyBuilder
                        .current()
                        .addFunctionCall(irName, Vec::new(), expr.location.clone());
                }
                ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
            }
            SimpleExpr::FieldAccess(receiver, name) => {
                let receiver = self.resolveExpr(receiver, env);
                let fieldInfos = vec![FieldInfo {
                    name: FieldId::Named(name.toString()),
                    ty: None,
                    location: name.location.clone(),
                }];
                self.bodyBuilder
                    .current()
                    .addFieldRef(receiver, fieldInfos, expr.location.clone())
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
                        if self.enums.get(&irName).is_some() {
                            ResolverError::NotAConstructor(name.name.clone(), name.location.clone()).report(self.ctx);
                        }
                        return self
                            .bodyBuilder
                            .current()
                            .addFunctionCall(irName, irArgs, expr.location.clone());
                    }
                    SimpleExpr::Value(name) => {
                        if let Some(name) = env.resolve(&name.name) {
                            self.bodyBuilder
                                .current()
                                .addDynamicFunctionCall(name, irArgs, expr.location.clone())
                        } else {
                            let irName = self.moduleResolver.resolverName(name);
                            self.bodyBuilder
                                .current()
                                .addFunctionCall(irName, irArgs, expr.location.clone())
                        }
                    }
                    _ => {
                        let callableId = self.resolveExpr(&callable, env);
                        self.bodyBuilder
                            .current()
                            .addDynamicFunctionCall(callableId, irArgs, expr.location.clone())
                    }
                }
            }
            SimpleExpr::MethodCall(receiver, name, args) => {
                let receiver = self.resolveExpr(&receiver, env);
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    irArgs.push(argId)
                }
                self.bodyBuilder
                    .current()
                    .addMethodCall(name.toString(), receiver, irArgs, expr.location.clone())
            }
            SimpleExpr::TupleIndex(receiver, index) => {
                let receiver = self.resolveExpr(&receiver, env);
                self.bodyBuilder.current().addFieldRef(
                    receiver,
                    vec![FieldInfo {
                        name: FieldId::Indexed(index.parse().unwrap()),
                        ty: None,
                        location: expr.location.clone(),
                    }],
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
                let mut loopBodyBuilder = self.createBlock(env);
                let mut loopExitBuilder = self.createBlock(env);
                let mut loopEnv = Environment::child(env, self.createSyntaxBlockIdSegment());
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
                    BinaryOp::And => createOpName("And", "opAnd"),
                    BinaryOp::Or => createOpName("Or", "opOr"),
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
                self.bodyBuilder
                    .current()
                    .addFunctionCall(name, vec![lhsId, rhsId], expr.location.clone())
            }
            SimpleExpr::UnaryOp(op, rhs) => {
                let rhsId = self.resolveExpr(rhs, env);
                let name = match op {
                    UnaryOp::Not => createOpName("Not", "not"),
                    UnaryOp::Neg => createOpName("Neg", "negative"),
                };
                let id = Identifier {
                    name: format!("{}", name),
                    location: expr.location.clone(),
                };
                let name = self.moduleResolver.resolverName(&id);
                self.bodyBuilder
                    .current()
                    .addFunctionCall(name, vec![rhsId], expr.location.clone())
            }
            SimpleExpr::Match(body, branches) => {
                let bodyId = self.resolveExpr(body, env);
                let mut matchResolver = MatchCompiler::new(
                    self,
                    bodyId,
                    expr.location.clone(),
                    body.location.clone(),
                    branches.clone(),
                    env,
                );
                matchResolver.compile()
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
            SimpleExpr::Ref(arg) => {
                let arg = self.resolveExpr(arg, env);
                self.bodyBuilder.current().addRef(arg, expr.location.clone())
            }
            SimpleExpr::List(args) => {
                let mut listVar =
                    self.bodyBuilder
                        .current()
                        .addFunctionCall(getVecNewName(), Vec::new(), expr.location.clone());
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
        }
    }

    fn resolvePattern(&mut self, pat: &Pattern, env: &mut Environment, root: Variable) {
        match &pat.pattern {
            SimplePattern::Named(name, args) => {
                let name = &self.moduleResolver.resolverName(name);
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
                            let fieldId = self.bodyBuilder.current().addFieldRef(
                                root.clone(),
                                vec![FieldInfo {
                                    name: FieldId::Named(field.name.clone()),
                                    ty: None,
                                    location: pat.location.clone(),
                                }],
                                pat.location.clone(),
                            );
                            self.resolvePattern(arg, env, fieldId);
                        }
                    }
                    _ => ResolverError::NotAConstructor(name.toString(), pat.location.clone()).report(self.ctx),
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
                let new = self.bodyBuilder.createLocalValue(&name.name, pat.location.clone());
                self.bodyBuilder
                    .current()
                    .addBind(new.clone(), root, *mutable, pat.location.clone());
                env.addValue(name.toString(), new);
            }
            SimplePattern::Tuple(args) => {
                for (index, arg) in args.iter().enumerate() {
                    let tupleValue = self.bodyBuilder.current().addFieldRef(
                        root.clone(),
                        vec![FieldInfo {
                            name: FieldId::Indexed(index as u32),
                            ty: None,
                            location: pat.location.clone(),
                        }],
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
                value.1.location.clone(),
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
}
