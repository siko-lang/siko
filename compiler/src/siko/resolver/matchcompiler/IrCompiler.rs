use std::collections::BTreeMap;

use crate::siko::{
    hir::{
        Block::BlockId,
        BlockBuilder::BlockBuilder,
        Instruction::{EnumCase, FieldId, FieldInfo, InstructionKind, IntegerCase},
        Variable::Variable,
    },
    location::Location::Location,
    qualifiedname::{builtins::getCloneFnName, QualifiedName},
    resolver::{
        matchcompiler::{
            Context::CompileContext,
            DataPath::{DataPath, DataPathRef, DataPathSegment},
            Tree::{Case, Leaf, MatchKind, Node, Switch, SwitchKind, Tuple},
        },
        Environment::Environment,
        ExprResolver::ExprResolver,
    },
    syntax::{Expr::Branch, Pattern::SimplePattern},
};

pub struct IrCompiler<'a, 'b> {
    bodyId: Variable,
    matchValue: Variable,
    branches: Vec<Branch>,
    pub resolver: &'a mut ExprResolver<'b>,
    parentEnv: &'a Environment<'a>,
    bodyLocation: Location,
    matchLocation: Location,
    contBlockId: BlockId,
}

impl<'a, 'b> IrCompiler<'a, 'b> {
    pub fn new(
        bodyId: Variable,
        branches: Vec<Branch>,
        resolver: &'a mut ExprResolver<'b>,
        parentEnv: &'a Environment<'a>,
        bodyLocation: Location,
        matchLocation: Location,
    ) -> Self {
        let matchValue = resolver.bodyBuilder.createTempValue(matchLocation.clone());
        let mut returns = false;
        for b in &branches {
            if !b.body.doesNotReturn() {
                returns = true;
            }
        }
        let mut contBlockId = BlockId::first();
        if returns {
            resolver
                .bodyBuilder
                .current()
                .addDeclare(matchValue.clone(), matchLocation.clone());
            contBlockId = resolver.createBlock(parentEnv).getBlockId();
        }
        IrCompiler {
            bodyId,
            matchValue,
            branches,
            resolver,
            parentEnv,
            bodyLocation,
            matchLocation,
            contBlockId,
        }
    }

    pub fn compileIr(&mut self, node: Node) -> Variable {
        let mut ctx = CompileContext::new();
        let dataPath = node.getDataPath();
        //println!("adding ctx for data path {}", dataPath);

        let mut startBlockBuilder = self.resolver.bodyBuilder.current();
        let rootRef = startBlockBuilder.addRef(self.bodyId.useVar(), self.bodyLocation.clone());
        ctx = ctx.add(dataPath.clone(), rootRef);
        let firstBlockId = self.compileNode(&node, &ctx);
        self.resolver.addJumpToBuilder(
            firstBlockId,
            self.bodyLocation.clone(),
            self.parentEnv.getSyntaxBlockId(),
            &mut startBlockBuilder,
        );
        self.resolver.bodyBuilder.setTargetBlockId(firstBlockId);
        let mut returns = false;
        for b in &self.branches {
            if !b.body.doesNotReturn() {
                returns = true;
            }
        }
        let value = self.resolver.bodyBuilder.createTempValue(self.bodyLocation.clone());
        if returns {
            let v = self.matchValue.clone();
            let mut builder = self.resolver.bodyBuilder.block(self.contBlockId);
            builder.current();
            builder.implicit().addDeclare(value.clone(), self.bodyLocation.clone());
            builder
                .implicit()
                .addInstruction(InstructionKind::Assign(value.clone(), v), self.matchLocation.clone());
        }
        value
    }

    pub fn compileNode(&mut self, node: &Node, ctx: &CompileContext) -> BlockId {
        //println!("compileNode: node {:?}, ctx {}", node, ctx);
        match node {
            Node::Tuple(tuple) => self.compileTuple(ctx, tuple),
            Node::Switch(switch) => self.compileSwitch(ctx, switch),
            Node::Leaf(end) => self.compileLeaf(end),
            Node::Wildcard(w) => self.compileNode(&w.next, ctx),
        }
    }

    fn compileSwitch(&mut self, ctx: &CompileContext, switch: &Switch) -> BlockId {
        let root = ctx.get(&switch.dataPath);
        let mut builder = self.resolver.createBlock(self.parentEnv);
        builder.current();
        match &switch.kind {
            SwitchKind::Enum(enumName) => {
                self.compileEnumSwitch(ctx, switch, &root, &mut builder, enumName);
            }
            SwitchKind::Integer => {
                self.compileIntegerSwitch(ctx, switch, root, &mut builder);
            }
            SwitchKind::String => {
                self.compileStringSwitch(ctx, switch, root, &mut builder);
            }
        }
        builder.getBlockId()
    }

    fn compileTuple(&mut self, ctx: &CompileContext, tuple: &Tuple) -> BlockId {
        let mut builder = self.resolver.createBlock(self.parentEnv);
        builder.current();
        let root = ctx.get(&tuple.dataPath.asRef().getParent().owned());
        let mut ctx = ctx.clone();
        for index in 0..tuple.size {
            let value = builder.addFieldAccess(
                root.clone(),
                vec![FieldInfo {
                    name: FieldId::Indexed(index as u32),
                    ty: None,
                    location: self.bodyLocation.clone(),
                }],
                false,
                self.bodyLocation.clone(),
            );
            ctx = ctx.add(tuple.dataPath.push(DataPathSegment::TupleIndex(index)), value);
        }
        let nextId = self.compileNode(&tuple.next, &ctx);
        self.resolver.addJumpToBuilder(
            nextId,
            self.bodyLocation.clone(),
            self.parentEnv.getSyntaxBlockId(),
            &mut builder,
        );
        builder.getBlockId()
    }

    fn compileEnumSwitch(
        &mut self,
        ctx: &CompileContext,
        switch: &Switch,
        root: &Variable,
        builder: &mut crate::siko::hir::BlockBuilder::BlockBuilder,
        enumName: &QualifiedName,
    ) {
        let mut cases = Vec::new();
        let enumDef = self.resolver.enums.get(enumName).expect("enum not found");
        for (case, node) in &switch.cases {
            if let Case::Variant(name) = case {
                let (v, index) = enumDef.getVariant(name);
                let (ctx, transformBlockId) = if v.items.len() > 0 {
                    let mut builder = self.resolver.createBlock(self.parentEnv);
                    builder.current();
                    let transformValue = builder.addTransform(root.clone(), index, self.bodyLocation.clone());
                    let mut ctx = ctx.clone();
                    for (index, _) in v.items.iter().enumerate() {
                        let value = builder.addFieldAccess(
                            transformValue.clone(),
                            vec![FieldInfo {
                                name: FieldId::Indexed(index as u32),
                                ty: None,
                                location: self.bodyLocation.clone(),
                            }],
                            false,
                            self.bodyLocation.clone(),
                        );
                        let path = switch
                            .dataPath
                            .push(DataPathSegment::Variant(name.clone(), enumName.clone()));
                        let path = path.push(DataPathSegment::ItemIndex(index as i64));
                        ctx = ctx.add(path, value.clone());
                    }
                    (ctx, Some(builder.getBlockId()))
                } else {
                    (ctx.clone(), None)
                };
                let mut caseBlockId = self.compileNode(&node, &ctx);
                if let Some(transformBlockId) = transformBlockId {
                    let mut transformBuilder = self.resolver.bodyBuilder.block(transformBlockId);
                    self.resolver.addJumpToBuilder(
                        caseBlockId,
                        self.bodyLocation.clone(),
                        self.parentEnv.getSyntaxBlockId(),
                        &mut transformBuilder,
                    );
                    caseBlockId = transformBlockId;
                }
                let c = EnumCase {
                    index: Some(index),
                    branch: caseBlockId,
                };
                cases.push(c);
            }
            if let Case::Default = case {
                let caseBlockId = self.compileNode(&node, &ctx);
                let c = EnumCase {
                    index: None,
                    branch: caseBlockId,
                };
                cases.push(c);
            }
        }
        builder.addInstruction(
            InstructionKind::EnumSwitch(root.clone(), cases),
            self.bodyLocation.clone(),
        );
    }

    fn compileIntegerSwitch(
        &mut self,
        ctx: &CompileContext,
        switch: &Switch,
        root: Variable,
        builder: &mut crate::siko::hir::BlockBuilder::BlockBuilder,
    ) {
        let mut cases = Vec::new();
        for (case, node) in &switch.cases {
            let value = match case {
                Case::Integer(v) => Some(v.clone()),
                Case::Default => None,
                _ => unreachable!(),
            };
            let blockId = self.compileNode(&node, ctx);
            let c = IntegerCase {
                value: value,
                branch: blockId,
            };
            cases.push(c);
        }
        let refValue = builder.addRef(root, self.bodyLocation.clone());
        let cloneValue = builder.addFunctionCall(getCloneFnName(), vec![refValue], self.bodyLocation.clone());
        builder.addInstruction(
            InstructionKind::IntegerSwitch(cloneValue, cases),
            self.bodyLocation.clone(),
        );
    }

    fn compileStringSwitch(
        &mut self,
        ctx: &CompileContext,
        switch: &Switch,
        root: Variable,
        builder: &mut crate::siko::hir::BlockBuilder::BlockBuilder,
    ) {
        let mut blocks = Vec::new();
        blocks.push(builder.getBlockId());
        for _ in 0..switch.cases.len() {
            if blocks.len() < switch.cases.len() - 1 {
                blocks.push(self.resolver.createBlock(self.parentEnv).getBlockId());
            }
        }
        let mut defaultBranch = BlockId::first();
        for (value, case) in &switch.cases {
            if let Case::Default = value {
                defaultBranch = self.compileNode(case, ctx);
            }
        }
        if switch.cases.len() == 1 {
            self.resolver.addJumpToBuilder(
                defaultBranch,
                self.bodyLocation.clone(),
                self.parentEnv.getSyntaxBlockId(),
                builder,
            );
        }
        for (case, node) in &switch.cases {
            match case {
                Case::String(v) => {
                    let current = blocks.remove(0);
                    let mut builder = self.resolver.bodyBuilder.block(current);
                    builder.current();
                    let value = builder
                        .implicit()
                        .addStringLiteral(v.clone(), self.bodyLocation.clone());
                    let eqValue =
                        builder.addMethodCall("eq".to_string(), root.clone(), vec![value], self.bodyLocation.clone());
                    let mut cases = Vec::new();
                    if blocks.is_empty() {
                        cases.push(EnumCase {
                            index: Some(0),
                            branch: defaultBranch,
                        });
                    } else {
                        cases.push(EnumCase {
                            index: Some(0),
                            branch: blocks[0],
                        });
                    }
                    cases.push(EnumCase {
                        index: Some(1),
                        branch: self.compileNode(&node, ctx),
                    });
                    builder
                        .implicit()
                        .addInstruction(InstructionKind::EnumSwitch(eqValue, cases), self.bodyLocation.clone());
                }
                Case::Default => {}
                c => unreachable!("string case {:?}", c),
            };
        }
    }

    fn compileLeaf(&mut self, leaf: &Leaf) -> BlockId {
        let m = leaf.finalMatch.as_ref().expect("no match");
        let index = if let MatchKind::UserDefined(index, _) = &m.kind {
            index.clone()
        } else {
            unreachable!()
        };
        let branch = &self.branches[index as usize];
        let syntaxBlockId = self.resolver.createSyntaxBlockIdSegment();
        let mut env = Environment::child(self.parentEnv, syntaxBlockId.clone());
        let mut envBlockbuilder = self.resolver.createBlock(&env);
        //println!("Compile branch {} to block {}", index, builder.getBlockId());
        envBlockbuilder.current();
        self.resolver
            .bodyBuilder
            .current()
            .addBlockStart(env.getSyntaxBlockId(), self.bodyLocation.clone());

        let mut leafBodyBuilder = self.resolver.createBlock(&env);
        if !leaf.guardedMatches.is_empty() {
            let mut guardBlocks = Vec::new();
            for guardMatch in leaf.guardedMatches.iter() {
                let mut guardEnv = Environment::child(self.parentEnv, syntaxBlockId.clone());
                let mut accessorMap = BTreeMap::new();
                for (path, name) in &guardMatch.bindings.bindings {
                    let refVar = self
                        .resolver
                        .bodyBuilder
                        .current()
                        .addRef(self.bodyId.clone(), self.bodyLocation.clone());
                    let accessor = generateAccessor(
                        self.resolver,
                        envBlockbuilder.clone(),
                        path.last().asRef(),
                        &refVar,
                        &mut accessorMap,
                    );
                    let new = self
                        .resolver
                        .bodyBuilder
                        .createLocalValue(&name, self.bodyLocation.clone());
                    self.resolver.bodyBuilder.current().addBind(
                        new.clone(),
                        accessor,
                        false,
                        self.bodyLocation.clone(),
                    );
                    guardEnv.addValue(name.clone(), new);
                }
                let guardTestBlock = self.resolver.createBlock(&guardEnv);
                let guardBodyBlock = self.resolver.createBlock(&guardEnv);
                guardBlocks.push((guardTestBlock, guardBodyBlock, guardEnv));
            }
            for (guardIndex, guard) in leaf.guardedMatches.iter().enumerate() {
                let guardPatternIndex = if let MatchKind::UserDefined(guardPatternIndex, _) = &guard.kind {
                    guardPatternIndex.clone()
                } else {
                    unreachable!()
                };
                let guardedBranch = &self.branches[guardPatternIndex as usize];
                let guardExpr = if let SimplePattern::Guarded(_, guardExpr) = &guardedBranch.pattern.pattern {
                    guardExpr
                } else {
                    unreachable!()
                };
                let mut guardTestBlock = guardBlocks[guardIndex].0.clone();
                guardTestBlock.current();
                let mut guardEnv = guardBlocks[guardIndex].2.clone();
                let guardValue = self.resolver.resolveExpr(&guardExpr, &mut guardEnv);
                let mut guardBodyBlockBuilder = guardBlocks[guardIndex].1.clone();
                guardBodyBlockBuilder.current();
                let guardBranchValue = self.resolver.resolveExpr(&guardedBranch.body, &mut guardEnv);
                if !guardedBranch.body.doesNotReturn() {
                    let mut builder = self.resolver.bodyBuilder.current().implicit();
                    builder.addAssign(self.matchValue.clone(), guardBranchValue, self.matchLocation.clone());
                    self.resolver.addJumpToBuilder(
                        self.contBlockId,
                        self.bodyLocation.clone(),
                        guardEnv.getSyntaxBlockId(),
                        &mut builder,
                    );
                }
                let mut guardCases = Vec::new();
                if guardIndex + 1 == leaf.guardedMatches.len() {
                    // last guard -> jump to leaf body
                    guardCases.push(EnumCase {
                        index: Some(0),
                        branch: leafBodyBuilder.getBlockId(),
                    });
                } else {
                    // jump to next guard test
                    guardCases.push(EnumCase {
                        index: Some(0),
                        branch: guardBlocks[guardIndex + 1].0.getBlockId(),
                    });
                }
                guardCases.push(EnumCase {
                    index: Some(1),
                    branch: guardBodyBlockBuilder.getBlockId(),
                });
                let enumSwitchKind = InstructionKind::EnumSwitch(guardValue, guardCases);
                guardTestBlock.addInstruction(enumSwitchKind, self.bodyLocation.clone());
            }
            // jump to first guard test
            envBlockbuilder.addJump(guardBlocks[0].0.getBlockId(), self.bodyLocation.clone());
        } else {
            envBlockbuilder.addJump(leafBodyBuilder.getBlockId(), self.bodyLocation.clone());
        }
        leafBodyBuilder.current();
        let mut accessorMap = BTreeMap::new();
        for (path, name) in &m.bindings.bindings {
            //println!("Creating binding for {} at {}", name, path.last());
            let accessor = generateAccessor(
                self.resolver,
                leafBodyBuilder.clone(),
                path.last().asRef(),
                &self.bodyId,
                &mut accessorMap,
            );
            let new = self
                .resolver
                .bodyBuilder
                .createLocalValue(&name, self.bodyLocation.clone());
            self.resolver
                .bodyBuilder
                .current()
                .addBind(new.clone(), accessor, false, self.bodyLocation.clone());
            env.addValue(name.clone(), new);
        }
        let exprValue = self.resolver.resolveExpr(&branch.body, &mut env);
        if !branch.body.doesNotReturn() {
            let mut builder = self.resolver.bodyBuilder.current().implicit();
            builder.addAssign(self.matchValue.clone(), exprValue, self.matchLocation.clone());
            self.resolver.addJumpToBuilder(
                self.contBlockId,
                self.bodyLocation.clone(),
                env.getSyntaxBlockId(),
                &mut builder,
            );
        }
        envBlockbuilder.getBlockId()
    }
}

fn generateAccessor(
    resolver: &mut ExprResolver,
    mut builder: BlockBuilder,
    path: DataPathRef,
    root: &Variable,
    accessorMap: &mut BTreeMap<DataPath, Variable>,
) -> Variable {
    if path.isRoot() {
        return root.clone();
    }
    if let Some(v) = accessorMap.get(&path.owned()) {
        return v.clone();
    }
    let parentPathVar = generateAccessor(resolver, builder.clone(), path.getParent(), root, accessorMap);
    let var = match path.last() {
        DataPathSegment::TupleIndex(index) => {
            let fields = vec![FieldInfo {
                name: FieldId::Indexed(*index as u32),
                ty: None,
                location: root.location(),
            }];
            builder.addFieldAccess(parentPathVar, fields, false, root.location())
        }
        DataPathSegment::ItemIndex(index) => {
            let fields = vec![FieldInfo {
                name: FieldId::Indexed(*index as u32),
                ty: None,
                location: root.location(),
            }];
            builder.addFieldAccess(parentPathVar, fields, false, root.location())
        }
        DataPathSegment::Variant(variantName, enumName) => {
            let enumDef = resolver.enums.get(enumName).expect("enum not found");
            let (_, index) = enumDef.getVariant(variantName);
            builder.addTransform(parentPathVar, index, root.location())
        }
        DataPathSegment::Tuple(_) => parentPathVar,
        p => unreachable!("unexpected data path {:?}", p),
    };
    accessorMap.insert(path.owned(), var.clone());
    var
}
