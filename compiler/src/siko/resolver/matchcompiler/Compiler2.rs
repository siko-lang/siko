use crate::siko::hir::Data::Enum;
use crate::siko::hir::Type::Type;
use crate::siko::hir::Variable::Variable;
use crate::siko::location::Location::Location;
use crate::siko::qualifiedname::builtins::getStringEqName;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::Environment::Environment;
use crate::siko::resolver::Error::ResolverError;
use crate::siko::resolver::ExprResolver::ExprResolver;
use crate::siko::syntax::Expr::{Branch, Expr, SimpleExpr};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use crate::siko::syntax::Statement::{Block, Statement, StatementKind};
use std::collections::{BTreeMap, BTreeSet};
use std::iter::repeat;
use std::{fmt, vec};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataPath {
    Root,
    Tuple(Box<DataPath>, i64),
    TupleIndex(Box<DataPath>, i64),
    ItemIndex(Box<DataPath>, i64),
    Variant(Box<DataPath>, QualifiedName, QualifiedName),
    IntegerLiteral(Box<DataPath>, String),
    StringLiteral(Box<DataPath>, String),
    Struct(Box<DataPath>, QualifiedName),
    Wildcard(Box<DataPath>),
}

impl DataPath {
    fn isChild(&self, parent: &DataPath) -> bool {
        let mut selfParent = self.getParent();
        loop {
            if &selfParent == parent {
                return true;
            }
            if selfParent == DataPath::Root {
                return false;
            }
            selfParent = selfParent.getParent();
        }
    }

    fn getParent(&self) -> DataPath {
        match self {
            DataPath::Root => DataPath::Root,
            DataPath::Tuple(p, _) => *p.clone(),
            DataPath::TupleIndex(p, _) => *p.clone(),
            DataPath::ItemIndex(p, _) => *p.clone(),
            DataPath::Variant(p, _, _) => *p.clone(),
            DataPath::IntegerLiteral(p, _) => *p.clone(),
            DataPath::StringLiteral(p, _) => *p.clone(),
            DataPath::Struct(p, _) => *p.clone(),
            DataPath::Wildcard(p) => *p.clone(),
        }
    }
}

impl fmt::Display for DataPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataPath::Root => write!(f, "Root"),
            DataPath::Tuple(path, len) => write!(f, "{}/tuple{}", path, len),
            DataPath::TupleIndex(path, index) => {
                write!(f, "{}.t{}", path, index)
            }
            DataPath::ItemIndex(path, index) => {
                write!(f, "{}.i{}", path, index)
            }
            DataPath::Variant(path, name, _) => write!(f, "{}.{}", path, name),
            DataPath::IntegerLiteral(path, literal) => {
                write!(f, "{}[int:{}]", path, literal)
            }
            DataPath::StringLiteral(path, literal) => {
                write!(f, "{}[str:\"{}\"]", path, literal)
            }
            DataPath::Struct(path, name) => write!(f, "{}.{}", path, name),
            DataPath::Wildcard(path) => write!(f, "{}._", path),
        }
    }
}

impl fmt::Debug for DataPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug)]
pub enum DataType {
    Struct(QualifiedName),
    Enum(QualifiedName),
    Tuple(i64),
    Integer,
    String,
    Wildcard,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Struct(name) => write!(f, "Struct({})", name),
            DataType::Enum(name) => write!(f, "Enum({})", name),
            DataType::Tuple(size) => write!(f, "Tuple({})", size),
            DataType::Integer => write!(f, "Integer"),
            DataType::String => write!(f, "String"),
            DataType::Wildcard => write!(f, "_"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecisionPath {
    decisions: Vec<DataPath>,
}

impl DecisionPath {
    pub fn new() -> DecisionPath {
        DecisionPath { decisions: Vec::new() }
    }

    pub fn add(&self, path: DataPath) -> DecisionPath {
        let mut d = self.clone();
        d.decisions.push(path);
        d
    }
}

impl fmt::Display for DecisionPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decisions = self
            .decisions
            .iter()
            .map(|path| format!("{}", path))
            .collect::<Vec<String>>()
            .join(" -> ");

        write!(f, "{}", decisions)
    }
}

pub struct MatchCompiler2<'a, 'b> {
    resolver: &'a mut ExprResolver<'b>,
    bodyId: Variable,
    matchLocation: Location,
    bodyLocation: Location,
    branches: Vec<Branch>,
    errors: Vec<ResolverError>,
    usedPatterns: BTreeSet<i64>,
    missingPatterns: BTreeSet<Pattern>,
    nextVar: i32,
    nodes: BTreeMap<DecisionPath, Node>,
    parentEnv: &'a Environment<'a>,
    jumpBlocks: BTreeMap<String, Expr>,
    branchBlocks: BTreeMap<i64, String>,
}

impl<'a, 'b> MatchCompiler2<'a, 'b> {
    pub fn new(
        resolver: &'a mut ExprResolver<'b>,
        bodyId: Variable,
        matchLocation: Location,
        bodyLocation: Location,
        branches: Vec<Branch>,
        parentEnv: &'a Environment<'a>,
    ) -> MatchCompiler2<'a, 'b> {
        MatchCompiler2 {
            matchLocation: matchLocation,
            bodyLocation: bodyLocation,
            bodyId: bodyId,
            branches: branches,
            resolver: resolver,
            errors: Vec::new(),
            nextVar: 1,
            usedPatterns: BTreeSet::new(),
            missingPatterns: BTreeSet::new(),
            nodes: BTreeMap::new(),
            parentEnv: parentEnv,
            jumpBlocks: BTreeMap::new(),
            branchBlocks: BTreeMap::new(),
        }
    }

    fn resolve(&self, pattern: &Pattern) -> Pattern {
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.resolver.moduleResolver.resolveName(&origId);
                let id = Identifier::new(name.toString(), Location::empty());
                let args = args.iter().map(|p| self.resolve(p)).collect();
                Pattern {
                    pattern: SimplePattern::Named(id, args),
                    location: Location::empty(),
                }
            }
            SimplePattern::Bind(_, _) => pattern.clone(),
            SimplePattern::Tuple(args) => {
                let args = args.iter().map(|p| self.resolve(p)).collect();
                Pattern {
                    pattern: SimplePattern::Tuple(args),
                    location: Location::empty(),
                }
            }
            SimplePattern::StringLiteral(_) => pattern.clone(),
            SimplePattern::IntegerLiteral(_) => pattern.clone(),
            SimplePattern::Wildcard => pattern.clone(),
        }
    }

    fn generateChoices(&self, pattern: &Pattern) -> Vec<Pattern> {
        let wildcardPattern = Pattern {
            pattern: SimplePattern::Wildcard,
            location: Location::empty(),
        };
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.resolver.moduleResolver.resolveName(&origId);
                let mut result = Vec::new();
                if let Some(enumName) = self.resolver.variants.get(&name) {
                    let e = self.resolver.enums.get(enumName).expect("enum not found");
                    for variant in &e.variants {
                        if variant.name == name {
                            continue;
                        }
                        let id = Identifier::new(variant.name.toString(), Location::empty());

                        let args = repeat(wildcardPattern.clone()).take(variant.items.len()).collect();
                        let pat = Pattern {
                            pattern: SimplePattern::Named(id, args),
                            location: Location::empty(),
                        };
                        result.push(pat);
                    }
                    for (index, arg) in args.iter().enumerate() {
                        let choices = self.generateChoices(arg);
                        for choice in choices {
                            let mut choiceArgs = Vec::new();
                            choiceArgs.extend(args.iter().cloned().take(index));
                            choiceArgs.push(choice);
                            choiceArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                            let id = Identifier::new(name.toString(), Location::empty());
                            let pat = Pattern {
                                pattern: SimplePattern::Named(id, choiceArgs),
                                location: Location::empty(),
                            };
                            result.push(pat);
                        }
                    }
                }
                result
            }
            SimplePattern::Bind(_, _) => Vec::new(),
            SimplePattern::Tuple(args) => {
                let mut result = Vec::new();
                for (index, arg) in args.iter().enumerate() {
                    let choices = self.generateChoices(arg);
                    for choice in choices {
                        let mut choiceArgs = Vec::new();
                        choiceArgs.extend(args.iter().cloned().take(index));
                        choiceArgs.push(choice);
                        choiceArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                        let pat = Pattern {
                            pattern: SimplePattern::Tuple(choiceArgs),
                            location: Location::empty(),
                        };
                        result.push(pat);
                    }
                }
                result
            }
            SimplePattern::StringLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::IntegerLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::Wildcard => Vec::new(),
        }
    }

    fn generateDecisions(
        &mut self,
        pattern: &Pattern,
        parentData: &DataPath,
        decision: &DecisionPath,
        mut bindings: Bindings,
    ) -> (DecisionPath, Bindings) {
        //println!("generateDecisions: {}, {}, {}", pattern, parentData, decision);
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.resolver.moduleResolver.resolveName(&origId);
                if let Some(enumName) = self.resolver.variants.get(&name) {
                    let path = DataPath::Variant(Box::new(parentData.clone()), name, enumName.clone());
                    let mut decision = decision.add(path.clone());
                    for (index, arg) in args.iter().enumerate() {
                        let path = DataPath::ItemIndex(Box::new(path.clone()), index as i64);
                        (decision, bindings) = self.generateDecisions(arg, &path, &decision, bindings);
                    }
                    (decision, bindings)
                } else {
                    (
                        decision.add(DataPath::Struct(Box::new(parentData.clone()), name)),
                        bindings,
                    )
                }
            }
            SimplePattern::Bind(name, _) => {
                bindings
                    .bindings
                    .insert(decision.add(parentData.clone()), name.toString());
                (decision.add(DataPath::Wildcard(Box::new(parentData.clone()))), bindings)
            }
            SimplePattern::Tuple(args) => {
                let mut decision = decision.clone();
                let path = DataPath::Tuple(Box::new(parentData.clone()), args.len() as i64);
                decision = decision.add(path.clone());
                for (index, arg) in args.iter().enumerate() {
                    let path = DataPath::TupleIndex(Box::new(path.clone()), index as i64);
                    (decision, bindings) = self.generateDecisions(arg, &path, &decision, bindings);
                }
                (decision, bindings)
            }
            SimplePattern::StringLiteral(v) => (
                decision.add(DataPath::StringLiteral(Box::new(parentData.clone()), v.clone())),
                bindings,
            ),
            SimplePattern::IntegerLiteral(v) => (
                decision.add(DataPath::IntegerLiteral(Box::new(parentData.clone()), v.clone())),
                bindings,
            ),
            SimplePattern::Wildcard => (decision.add(DataPath::Wildcard(Box::new(parentData.clone()))), bindings),
        }
    }

    pub fn compile(&mut self) -> Variable {
        let mut matches = Vec::new();
        for (index, branch) in self.branches.clone().iter().enumerate() {
            let branchPattern = self.resolve(&branch.pattern);
            let (decision, bindings) =
                self.generateDecisions(&branchPattern, &DataPath::Root, &DecisionPath::new(), Bindings::new());
            //println!("{} Pattern {}\n decision: {}", index, branch.pattern, decision);
            let choices = self.generateChoices(&branchPattern);
            matches.push(Match {
                kind: MatchKind::UserDefined(index as i64),
                pattern: branchPattern,
                decisionPath: decision,
                bindings: bindings,
            });
            for choice in choices {
                //println!("   Alt: {}", choice);
                let (decision, bindings) =
                    self.generateDecisions(&choice, &DataPath::Root, &DecisionPath::new(), Bindings::new());
                matches.push(Match {
                    kind: MatchKind::Alternative,
                    pattern: choice,
                    decisionPath: decision,
                    bindings: bindings,
                });
            }
        }

        let mut dataTypes = BTreeMap::new();
        for m in &matches {
            for path in &m.decisionPath.decisions {
                match path {
                    DataPath::Root => {}
                    DataPath::Tuple(parent, count) => {
                        dataTypes.insert(parent.clone(), DataType::Tuple(*count));
                    }
                    DataPath::TupleIndex(_, _) => {}
                    DataPath::ItemIndex(_, _) => {}
                    DataPath::Variant(parent, _, enumName) => {
                        dataTypes.insert(parent.clone(), DataType::Enum(enumName.clone()));
                    }
                    DataPath::IntegerLiteral(parent, _) => {
                        dataTypes.insert(parent.clone(), DataType::Integer);
                    }
                    DataPath::StringLiteral(parent, _) => {
                        dataTypes.insert(parent.clone(), DataType::String);
                    }
                    DataPath::Struct(parent, name) => {
                        dataTypes.insert(parent.clone(), DataType::Struct(name.clone()));
                    }
                    DataPath::Wildcard(parent) => {
                        if !dataTypes.contains_key(parent.as_ref()) {
                            dataTypes.insert(parent.clone(), DataType::Wildcard);
                        }
                    }
                }
            }
        }

        // for (path, ty) in &dataTypes {
        //     println!("{} {}", path, ty);
        // }
        // for m in &matches {
        //     println!("Decision {}", m.decisionPath);
        // }

        let mut pendingPaths = Vec::new();
        pendingPaths.push(DataPath::Root);

        let mut node = self.buildNode(pendingPaths, &DecisionPath::new(), &dataTypes, &matches);
        node.add(self, &matches);

        for (index, branch) in self.branches.clone().iter().enumerate() {
            if !self.usedPatterns.contains(&(index as i64)) {
                self.errors
                    .push(ResolverError::RedundantPattern(branch.pattern.location.clone()));
            }
        }

        let missingPatterns: Vec<_> = self.missingPatterns.iter().map(|p| p.to_string()).collect();
        if !missingPatterns.is_empty() {
            self.errors.push(ResolverError::MissingPattern(
                missingPatterns,
                self.bodyLocation.clone(),
            ));
        }

        for err in &self.errors {
            err.reportOnly(self.resolver.ctx);
        }

        if !self.errors.is_empty() {
            std::process::exit(1);
        }

        let root = Identifier::new("root".to_string(), self.bodyLocation.clone());
        let ctx = CompileContext::new().add(node.getDataPath(), root);
        let _matchExpr = self.compileNode(&node, &ctx);
        // println!("------------------------");
        // crate::siko::syntax::Format::format_any(&_matchExpr);
        // for (_, body) in &self.jumpBlocks {
        //     crate::siko::syntax::Format::format_any(body);
        // }
        // println!("-----------------------");
        let value = self
            .resolver
            .bodyBuilder
            .createTempValueWithType(self.bodyLocation.clone(), Type::getUnitType());
        value
    }

    fn compileNode(&mut self, node: &Node, ctx: &CompileContext) -> Expr {
        //println!("compileNode: node {:?}, ctx {}", node, ctx);
        match node {
            Node::Tuple(tuple) => {
                let root = ctx.get(&tuple.dataPath.getParent());
                let mut ctx = ctx.clone();
                let mut block = Block {
                    statements: Vec::new(),
                    location: self.bodyLocation.clone(),
                };

                for index in 0..tuple.size {
                    let tupleFieldId = Identifier::new(format!("t{}", self.nextVar), self.bodyLocation.clone());
                    self.nextVar += 1;
                    let rootAccess = Expr {
                        expr: SimpleExpr::Value(root.clone()),
                        location: self.bodyLocation.clone(),
                    };
                    let tupleFieldExpr = SimpleExpr::TupleIndex(Box::new(rootAccess), format!("{}", index));
                    let tupleFieldExpr = Expr {
                        expr: tupleFieldExpr,
                        location: self.bodyLocation.clone(),
                    };
                    let tupleFieldIdPattern = Pattern {
                        pattern: SimplePattern::Bind(tupleFieldId.clone(), false),
                        location: self.bodyLocation.clone(),
                    };
                    block.statements.push(Statement {
                        kind: StatementKind::Let(tupleFieldIdPattern, tupleFieldExpr, None),
                        hasSemicolon: true,
                    });
                    ctx = ctx.add(
                        DataPath::TupleIndex(Box::new(tuple.dataPath.clone()), index),
                        tupleFieldId,
                    );
                }
                let nextId = self.compileNode(&tuple.next, &ctx);
                block.statements.push(Statement {
                    kind: StatementKind::Expr(nextId),
                    hasSemicolon: false,
                });
                let blockExpr = SimpleExpr::Block(block);
                Expr {
                    expr: blockExpr,
                    location: self.bodyLocation.clone(),
                }
            }
            Node::Switch(switch) => {
                let root = ctx.get(&switch.dataPath);
                match &switch.kind {
                    SwitchKind::Enum(enumName) => self.compileEnumSwitch(ctx, switch, &root, enumName),
                    SwitchKind::Integer => self.compileIntegerSwitch(ctx, switch, &root),
                    SwitchKind::String => self.compileStringSwitch(ctx, switch, root),
                }
            }
            Node::End(end) => {
                let m = end.matches.last().expect("no match");
                let index = if let MatchKind::UserDefined(index) = &m.kind {
                    *index
                } else {
                    unreachable!()
                };
                if self.branchBlocks.contains_key(&index) {
                    let jumpId = self.branchBlocks.get(&index).unwrap().clone();
                    let jumpExpr = SimpleExpr::Jump(jumpId.clone());
                    return Expr {
                        expr: jumpExpr,
                        location: self.bodyLocation.clone(),
                    };
                }

                let branch = &self.branches[index as usize];
                let mut block = Block {
                    statements: Vec::new(),
                    location: self.bodyLocation.clone(),
                };
                for (path, name) in &m.bindings.bindings {
                    let bindValue = ctx.get(&path.decisions.last().unwrap());
                    let bindId = Identifier::new(name.clone(), self.bodyLocation.clone());
                    let bindPattern = Pattern {
                        pattern: SimplePattern::Bind(bindId.clone(), false),
                        location: self.bodyLocation.clone(),
                    };
                    let bindValueExpr = Expr {
                        expr: SimpleExpr::Value(bindValue.clone()),
                        location: self.bodyLocation.clone(),
                    };
                    block.statements.push(Statement {
                        kind: StatementKind::Let(bindPattern, bindValueExpr, None),
                        hasSemicolon: true,
                    });
                }
                block.statements.push(Statement {
                    kind: StatementKind::Expr(branch.body.clone()),
                    hasSemicolon: false,
                });
                let blockExpr = SimpleExpr::Block(block);
                let blockExpr = Expr {
                    expr: blockExpr,
                    location: self.bodyLocation.clone(),
                };
                let jumpBlockId = self.resolver.getNextJumpBlockId();
                self.jumpBlocks.insert(
                    jumpBlockId.clone(),
                    Expr {
                        expr: SimpleExpr::JumpBlock(jumpBlockId.clone(), Box::new(blockExpr)),
                        location: self.bodyLocation.clone(),
                    },
                );
                self.branchBlocks.insert(index, jumpBlockId.clone());
                let jumpExpr = SimpleExpr::Jump(jumpBlockId.clone());
                Expr {
                    expr: jumpExpr,
                    location: self.bodyLocation.clone(),
                }
            }
            Node::Wildcard(w) => self.compileNode(&w.next, ctx),
        }
    }

    fn compileEnumSwitch(
        &mut self,
        ctx: &CompileContext,
        switch: &Switch,
        root: &Identifier,
        enumName: &QualifiedName,
    ) -> Expr {
        let enumDef = self.resolver.enums.get(enumName).expect("enum not found");
        let matchExpr = Expr {
            expr: SimpleExpr::EnumMatch(
                Box::new(Expr {
                    expr: SimpleExpr::Value(root.clone()),
                    location: self.bodyLocation.clone(),
                }),
                switch
                    .cases
                    .iter()
                    .map(|(case, node)| self.createVariantBranch(&switch.dataPath, ctx, enumDef, case, node))
                    .collect(),
            ),
            location: self.bodyLocation.clone(),
        };
        matchExpr
    }

    fn createVariantBranch(
        &mut self,
        path: &DataPath,
        ctx: &CompileContext,
        enumDef: &Enum,
        case: &Case,
        node: &Node,
    ) -> Branch {
        let (pattern, ctx) = match case {
            Case::Variant(name) => {
                let (pattern, ctx) = self.createVariantPattern(path, name, enumDef, ctx);
                (pattern, ctx)
            }
            Case::Default => (
                Pattern {
                    pattern: SimplePattern::Wildcard,
                    location: self.bodyLocation.clone(),
                },
                ctx.clone(),
            ),
            _ => unreachable!(),
        };
        Branch {
            pattern,
            body: { self.compileNode(node, &ctx) },
        }
    }

    fn createVariantPattern(
        &mut self,
        rootPath: &DataPath,
        name: &QualifiedName,
        enumDef: &Enum,
        ctx: &CompileContext,
    ) -> (Pattern, CompileContext) {
        let (variant, _) = enumDef.getVariant(name);
        let mut args = Vec::new();
        let mut argIds = Vec::new();
        for _ in &variant.items {
            let argId = Identifier::new(format!("v{}", self.nextVar), self.bodyLocation.clone());
            argIds.push(argId.clone());
            self.nextVar += 1;
            args.push(Pattern {
                pattern: SimplePattern::Bind(argId, false),
                location: self.bodyLocation.clone(),
            });
        }
        let pat = Pattern {
            pattern: SimplePattern::Named(Identifier::new(name.toString(), self.bodyLocation.clone()), args),
            location: self.bodyLocation.clone(),
        };
        let mut ctx = ctx.clone();
        if variant.items.len() > 0 {
            for (index, _) in variant.items.iter().enumerate() {
                let path = DataPath::Variant(Box::new(rootPath.clone()), name.clone(), enumDef.name.clone());
                let path = DataPath::ItemIndex(Box::new(path), index as i64);
                let value = argIds[index].clone();
                ctx = ctx.add(path, value.clone());
            }
        }
        (pat, ctx)
    }

    fn compileStringSwitch(&mut self, ctx: &CompileContext, switch: &Switch, root: Identifier) -> Expr {
        let mut block = Block {
            statements: Vec::new(),
            location: self.bodyLocation.clone(),
        };
        for (case, node) in switch.cases.iter() {
            match case {
                Case::String(v) => {
                    let valueId = Identifier::new(format!("v{}", self.nextVar), self.bodyLocation.clone());
                    self.nextVar += 1;
                    let valueExpr = Expr {
                        expr: SimpleExpr::StringLiteral(v.clone()),
                        location: self.bodyLocation.clone(),
                    };
                    block.statements.push(Statement {
                        kind: StatementKind::Let(
                            Pattern {
                                pattern: SimplePattern::Bind(valueId.clone(), false),
                                location: self.bodyLocation.clone(),
                            },
                            valueExpr,
                            None,
                        ),
                        hasSemicolon: true,
                    });
                    let eqValueId = Identifier::new(format!("eq{}", self.nextVar), self.bodyLocation.clone());
                    self.nextVar += 1;
                    let rootAccess = Expr {
                        expr: SimpleExpr::Value(root.clone()),
                        location: self.bodyLocation.clone(),
                    };
                    let eqExpr = Expr {
                        expr: SimpleExpr::Call(
                            Box::new(Expr {
                                expr: SimpleExpr::Value(Identifier::new(
                                    getStringEqName().toString(),
                                    self.bodyLocation.clone(),
                                )),
                                location: self.bodyLocation.clone(),
                            }),
                            vec![
                                rootAccess,
                                Expr {
                                    expr: SimpleExpr::Value(valueId.clone()),
                                    location: self.bodyLocation.clone(),
                                },
                            ],
                        ),
                        location: self.bodyLocation.clone(),
                    };
                    block.statements.push(Statement {
                        kind: StatementKind::Let(
                            Pattern {
                                pattern: SimplePattern::Bind(eqValueId.clone(), false),
                                location: self.bodyLocation.clone(),
                            },
                            eqExpr,
                            None,
                        ),
                        hasSemicolon: true,
                    });
                    let matchExpr = Expr {
                        expr: SimpleExpr::Match(
                            Box::new(Expr {
                                expr: SimpleExpr::Value(eqValueId.clone()),
                                location: self.bodyLocation.clone(),
                            }),
                            vec![
                                Branch {
                                    pattern: Pattern {
                                        pattern: SimplePattern::Named(
                                            Identifier::new("True".to_string(), self.bodyLocation.clone()),
                                            vec![],
                                        ),
                                        location: self.bodyLocation.clone(),
                                    },
                                    body: self.compileNode(node, ctx),
                                },
                                Branch {
                                    pattern: Pattern {
                                        pattern: SimplePattern::Named(
                                            Identifier::new("False".to_string(), self.bodyLocation.clone()),
                                            vec![],
                                        ),
                                        location: self.bodyLocation.clone(),
                                    },
                                    body: self.compileNode(node, ctx),
                                },
                            ],
                        ),
                        location: self.bodyLocation.clone(),
                    };
                    block.statements.push(Statement {
                        kind: StatementKind::Expr(matchExpr),
                        hasSemicolon: false,
                    });
                }
                Case::Default => {
                    let defaultExpr = self.compileNode(node, ctx);
                    block.statements.push(Statement {
                        kind: StatementKind::Expr(defaultExpr),
                        hasSemicolon: false,
                    });
                }
                _ => unreachable!("string case {:?}", case),
            };
        }
        let blockExpr = SimpleExpr::Block(block);
        Expr {
            expr: blockExpr,
            location: self.bodyLocation.clone(),
        }
    }

    fn compileIntegerSwitch(&mut self, ctx: &CompileContext, switch: &Switch, root: &Identifier) -> Expr {
        let matchExpr = Expr {
            expr: SimpleExpr::IntegerMatch(
                Box::new(Expr {
                    expr: SimpleExpr::Value(root.clone()),
                    location: self.bodyLocation.clone(),
                }),
                switch
                    .cases
                    .iter()
                    .map(|(case, node)| Branch {
                        pattern: match case {
                            Case::Integer(v) => Pattern {
                                pattern: SimplePattern::IntegerLiteral(v.clone()),
                                location: self.bodyLocation.clone(),
                            },
                            Case::Default => Pattern {
                                pattern: SimplePattern::Wildcard,
                                location: self.bodyLocation.clone(),
                            },
                            _ => unreachable!(),
                        },
                        body: { self.compileNode(node, &ctx) },
                    })
                    .collect(),
            ),
            location: self.bodyLocation.clone(),
        };
        matchExpr
    }

    fn buildNode(
        &mut self,
        mut pendingPaths: Vec<DataPath>,
        currentDecision: &DecisionPath,
        dataTypes: &BTreeMap<Box<DataPath>, DataType>,
        allMatches: &Vec<Match>,
    ) -> Node {
        //println!("buildNode: {:?} | {}", pendingPaths, currentDecision);
        if pendingPaths.is_empty() {
            let end = End {
                decisionPath: currentDecision.clone(),
                matches: Vec::new(),
            };
            return Node::End(end);
        }
        let currentPath = pendingPaths.remove(0);
        if let Some(ty) = dataTypes.get(&currentPath) {
            //println!("Building node for {}, {} / [{}] / {:?}", currentPath, ty, currentDecision, pendingPaths);
            match ty {
                DataType::Struct(_) => todo!(),
                DataType::Enum(enumName) => {
                    let e = self.resolver.enums.get(enumName).expect("enumName not found");
                    let mut cases = BTreeMap::new();
                    for variant in &e.variants {
                        let casePath =
                            DataPath::Variant(Box::new(currentPath.clone()), variant.name.clone(), enumName.clone());
                        let currentDecision = currentDecision.add(casePath.clone());
                        let mut pendings = pendingPaths.clone();
                        for index in 0..variant.items.len() {
                            pendings.insert(
                                0,
                                DataPath::ItemIndex(
                                    Box::new(casePath.clone()),
                                    (variant.items.len() - index - 1) as i64,
                                ),
                            );
                        }
                        let node = self.buildNode(pendings, &currentDecision, dataTypes, allMatches);
                        cases.insert(Case::Variant(variant.name.clone()), node);
                    }
                    let switch = Switch {
                        dataPath: currentPath.clone(),
                        kind: SwitchKind::Enum(enumName.clone()),
                        cases: cases,
                    };
                    Node::Switch(switch)
                }
                DataType::Tuple(size) => {
                    let path = DataPath::Tuple(Box::new(currentPath.clone()), *size);
                    let currentDecision = currentDecision.add(path.clone());
                    let mut pendings = Vec::new();
                    for index in 0..*size {
                        let argPath = DataPath::TupleIndex(Box::new(path.clone()), index);
                        pendings.insert(0, argPath);
                    }
                    pendings.reverse();
                    pendings.extend(pendingPaths.clone());
                    let node = self.buildNode(pendings, &currentDecision, dataTypes, allMatches);
                    let tuple = Tuple {
                        size: *size,
                        dataPath: path.clone(),
                        next: Box::new(node),
                    };
                    Node::Tuple(tuple)
                }
                DataType::Integer => {
                    let mut cases = BTreeMap::new();
                    let mut values = BTreeSet::new();
                    for m in allMatches {
                        if m.decisionPath.decisions.starts_with(&currentDecision.decisions[..]) {
                            if m.decisionPath.decisions.len() > currentDecision.decisions.len() {
                                match &m.decisionPath.decisions[currentDecision.decisions.len()] {
                                    DataPath::IntegerLiteral(_, value) => {
                                        values.insert(value.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    for value in values {
                        let path = DataPath::IntegerLiteral(Box::new(currentPath.clone()), value.clone());
                        let mut pendingPaths = pendingPaths.clone();
                        pendingPaths.insert(0, path.clone());
                        let currentDecision = &currentDecision.add(path);
                        let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                        cases.insert(Case::Integer(value.clone()), node);
                    }
                    let path = DataPath::Wildcard(Box::new(currentPath.clone()));
                    let mut pendingPaths = pendingPaths.clone();
                    pendingPaths.insert(0, path.clone());
                    let currentDecision = &currentDecision.add(path);
                    let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                    cases.insert(Case::Default, node);
                    let switch = Switch {
                        dataPath: currentPath.clone(),
                        kind: SwitchKind::Integer,
                        cases: cases,
                    };
                    Node::Switch(switch)
                }
                DataType::String => {
                    let mut cases = BTreeMap::new();
                    let mut values = BTreeSet::new();
                    for m in allMatches {
                        if m.decisionPath.decisions.starts_with(&currentDecision.decisions[..]) {
                            if m.decisionPath.decisions.len() >= currentDecision.decisions.len() + 1 {
                                match &m.decisionPath.decisions[currentDecision.decisions.len()] {
                                    DataPath::StringLiteral(_, value) => {
                                        values.insert(value.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    for value in values {
                        let path = DataPath::StringLiteral(Box::new(currentPath.clone()), value.clone());
                        let mut pendingPaths = pendingPaths.clone();
                        pendingPaths.insert(0, path.clone());
                        let currentDecision = &currentDecision.add(path);
                        let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                        cases.insert(Case::String(value.clone()), node);
                    }
                    let path = DataPath::Wildcard(Box::new(currentPath.clone()));
                    let mut pendingPaths = pendingPaths.clone();
                    pendingPaths.insert(0, path.clone());
                    let currentDecision = &currentDecision.add(path);
                    let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                    cases.insert(Case::Default, node);
                    let switch = Switch {
                        dataPath: currentPath.clone(),
                        kind: SwitchKind::String,
                        cases: cases,
                    };
                    Node::Switch(switch)
                }
                DataType::Wildcard => {
                    let path = DataPath::Wildcard(Box::new(currentPath.clone()));
                    pendingPaths.insert(0, path.clone());
                    let currentDecision = &currentDecision.add(path);
                    let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                    Node::Wildcard(Wildcard { next: Box::new(node) })
                }
            }
        } else {
            self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches)
        }
    }
}

#[derive(Clone, Debug)]
struct Tuple {
    size: i64,
    dataPath: DataPath,
    next: Box<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Case {
    Variant(QualifiedName),
    Integer(String),
    String(String),
    Default,
}

#[derive(Clone, Debug)]
enum SwitchKind {
    Enum(QualifiedName),
    Integer,
    String,
}

#[derive(Clone, Debug)]
struct Switch {
    dataPath: DataPath,
    kind: SwitchKind,
    cases: BTreeMap<Case, Node>,
}

#[derive(Clone, Debug)]
struct End {
    decisionPath: DecisionPath,
    matches: Vec<Match>,
}

#[derive(Clone, Debug)]
struct Wildcard {
    next: Box<Node>,
}

#[derive(Clone, Debug)]
enum Node {
    Tuple(Tuple),
    Switch(Switch),
    End(End),
    Wildcard(Wildcard),
}

impl Node {
    fn getDataPath(&self) -> DataPath {
        match self {
            Node::Tuple(tuple) => tuple.dataPath.getParent(),
            Node::Switch(switch) => switch.dataPath.clone(),
            Node::End(_) => unreachable!(),
            Node::Wildcard(_) => unreachable!(),
        }
    }

    fn add(&mut self, compiler: &mut MatchCompiler2, matches: &Vec<Match>) {
        match self {
            Node::Tuple(tuple) => tuple.next.add(compiler, matches),
            Node::Switch(switch) => {
                for (_, node) in &mut switch.cases {
                    node.add(compiler, matches);
                }
            }
            Node::Wildcard(w) => {
                w.next.add(compiler, matches);
            }
            Node::End(end) => {
                let mut localMatch: Option<Match> = None;
                for m in matches {
                    let matchResult = matchDecisions(end.decisionPath.clone(), m.decisionPath.clone());
                    if matchResult {
                        //println!("MATCH end {} //// {}", end.decisionPath, m.decisionPath);
                        match &localMatch {
                            Some(local) => match (&local.kind, &m.kind) {
                                (MatchKind::Alternative, MatchKind::UserDefined(_)) => {
                                    localMatch = Some(m.clone());
                                }
                                (MatchKind::UserDefined(i1), MatchKind::UserDefined(i2)) => {
                                    if *i2 < *i1 {
                                        localMatch = Some(m.clone());
                                    }
                                }
                                _ => {}
                            },
                            None => {
                                localMatch = Some(m.clone());
                            }
                        }
                    }
                }
                if let Some(m) = &localMatch {
                    match &m.kind {
                        MatchKind::Alternative => {
                            compiler.missingPatterns.insert(m.pattern.clone());
                        }
                        MatchKind::UserDefined(index) => {
                            compiler.usedPatterns.insert(*index);
                        }
                    }
                    //println!("M {}", m.decisionPath);
                    //println!("FINAL MATCH {} for {}, bindings: {}", end.decisionPath, m.kind, m.bindings);
                    end.matches.push(localMatch.unwrap());
                } else {
                    panic!("Empty end node in decision tree");
                }
            }
        }
    }
}

fn matchDecisions(mut nodeDecisionPath: DecisionPath, mut matchDecisionPath: DecisionPath) -> bool {
    loop {
        if matchDecisionPath.decisions.is_empty() {
            return nodeDecisionPath.decisions.is_empty();
        }
        let path = matchDecisionPath.decisions.remove(0);
        nodeDecisionPath = removePaths(&path, nodeDecisionPath);
    }
}

fn removePaths(path: &DataPath, mut nodeDecisionPath: DecisionPath) -> DecisionPath {
    loop {
        if nodeDecisionPath.decisions.is_empty() {
            break;
        }
        let nodePath = &nodeDecisionPath.decisions[0];
        let remove = match (path, nodePath) {
            (DataPath::Wildcard(parent), _) => nodePath.isChild(parent),
            (p1, p2) => p1 == p2,
        };
        if remove {
            nodeDecisionPath.decisions.remove(0);
        } else {
            break;
        }
    }
    nodeDecisionPath
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum MatchKind {
    UserDefined(i64),
    Alternative,
}

impl fmt::Display for MatchKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MatchKind::UserDefined(value) => {
                write!(f, "UserDefined({})", value)
            }
            MatchKind::Alternative => write!(f, "Alternative"),
        }
    }
}

#[derive(Clone, Debug)]
struct Match {
    kind: MatchKind,
    pattern: Pattern,
    decisionPath: DecisionPath,
    bindings: Bindings,
}

#[derive(Clone, Debug)]
struct Bindings {
    bindings: BTreeMap<DecisionPath, String>,
}

impl Bindings {
    pub fn new() -> Bindings {
        Bindings {
            bindings: BTreeMap::new(),
        }
    }
}

impl fmt::Display for Bindings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bindings {{ ")?;
        for (key, value) in &self.bindings {
            write!(f, "{}: {}, ", key, value)?;
        }
        write!(f, "}}")
    }
}

#[derive(Clone)]
struct CompileContext {
    values: BTreeMap<DataPath, Identifier>,
}

impl CompileContext {
    fn new() -> CompileContext {
        CompileContext {
            values: BTreeMap::new(),
        }
    }

    fn add(&self, path: DataPath, value: Identifier) -> CompileContext {
        let mut c = self.clone();
        c.values.insert(path, value);
        c
    }

    fn get(&self, path: &DataPath) -> Identifier {
        match self.values.get(path) {
            Some(id) => id.clone(),
            None => {
                panic!("not found value for path in compile context {}", path)
            }
        }
    }
}

impl fmt::Display for CompileContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CompileContext {{ ")?;
        for (key, value) in &self.values {
            write!(f, "{}: {}, ", key, value)?;
        }
        write!(f, "}}")
    }
}
