use crate::siko::{
    hir::{
        Block::BlockId,
        Instruction::{EnumCase, FieldId, FieldInfo, InstructionKind, IntegerCase},
        Variable::Variable,
    },
    location::Location::Location,
    qualifiedname::{
        builtins::{getCloneFnName, getStringEqName},
        QualifiedName,
    },
    resolver::{
        matchcompiler::{
            Context::CompileContext,
            DataPath::DataPath,
            Tree::{Case, End, MatchKind, Node, Switch, SwitchKind, Tuple},
        },
        Environment::Environment,
        ExprResolver::ExprResolver,
    },
    syntax::Expr::Branch,
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
        matchValue: Variable,
        branches: Vec<Branch>,
        resolver: &'a mut ExprResolver<'b>,
        parentEnv: &'a Environment<'a>,
        bodyLocation: Location,
        matchLocation: Location,
        contBlockId: BlockId,
    ) -> Self {
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
        let ctx = CompileContext::new().add(node.getDataPath(), self.bodyId.clone());
        let mut startBlockBuilder = self.resolver.bodyBuilder.current();

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
            Node::End(end) => self.compileLeaf(ctx, end),
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
        let root = ctx.get(&tuple.dataPath.getParent());
        let mut ctx = ctx.clone();
        for index in 0..tuple.size {
            let value = builder.addFieldRef(
                root.clone(),
                vec![FieldInfo {
                    name: FieldId::Indexed(index as u32),
                    ty: None,
                    location: self.bodyLocation.clone(),
                }],
                self.bodyLocation.clone(),
            );
            ctx = ctx.add(DataPath::TupleIndex(Box::new(tuple.dataPath.clone()), index), value);
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
                        let value = builder.addFieldRef(
                            transformValue.clone(),
                            vec![FieldInfo {
                                name: FieldId::Indexed(index as u32),
                                ty: None,
                                location: self.bodyLocation.clone(),
                            }],
                            self.bodyLocation.clone(),
                        );
                        let path = DataPath::Variant(Box::new(switch.dataPath.clone()), name.clone(), enumName.clone());
                        let path = DataPath::ItemIndex(Box::new(path), index as i64);
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
                    index,
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
                    let eqValue = builder.addFunctionCall(
                        getStringEqName(),
                        vec![root.clone(), value],
                        self.bodyLocation.clone(),
                    );
                    let mut cases = Vec::new();
                    if blocks.is_empty() {
                        cases.push(EnumCase {
                            index: 0,
                            branch: defaultBranch,
                        });
                    } else {
                        cases.push(EnumCase {
                            index: 0,
                            branch: blocks[0],
                        });
                    }
                    cases.push(EnumCase {
                        index: 1,
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

    fn compileLeaf(&mut self, ctx: &CompileContext, end: &End) -> BlockId {
        let m = end.matches.last().expect("no match");
        let index = if let MatchKind::UserDefined(index) = &m.kind {
            index.clone()
        } else {
            unreachable!()
        };
        let branch = &self.branches[index as usize];
        let syntaxBlockId = self.resolver.createSyntaxBlockIdSegment();
        let mut env = Environment::child(self.parentEnv, syntaxBlockId);
        let mut builder = self.resolver.createBlock(&env);
        //println!("Compile branch {} to block {}", index, builder.getBlockId());
        builder.current();
        self.resolver
            .bodyBuilder
            .current()
            .addBlockStart(env.getSyntaxBlockId(), self.bodyLocation.clone());
        for (path, name) in &m.bindings.bindings {
            let bindValue = ctx.get(path.decisions.last().unwrap());
            let new = self
                .resolver
                .bodyBuilder
                .createLocalValue(&name, self.bodyLocation.clone());
            self.resolver
                .bodyBuilder
                .current()
                .addBind(new.clone(), bindValue, false, self.bodyLocation.clone());
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
        builder.getBlockId()
    }
}
