use std::collections::BTreeMap;

use crate::mir::*;

pub struct InferenceInfo {
    expr_type_variables: BTreeMap<i64, TypeVariableInfo>,
    var_type_variables: BTreeMap<String, TypeVariableInfo>,
    info_allocator: TypeVariableInfoAllocator,
    result_info: TypeVariableInfo,
}

impl InferenceInfo {
    pub fn new() -> InferenceInfo {
        let mut allocator = TypeVariableInfoAllocator::new();
        let result_info = allocator.allocate();
        InferenceInfo {
            expr_type_variables: BTreeMap::new(),
            var_type_variables: BTreeMap::new(),
            info_allocator: allocator,
            result_info: result_info,
        }
    }

    fn add_expr(&mut self, id: i64) {
        let info = self.info_allocator.allocate();
        self.expr_type_variables.insert(id, info);
    }

    fn add_var(&mut self, id: String) {
        let info = self.info_allocator.allocate();
        self.var_type_variables.insert(id, info);
    }

    fn expr_info(&self, id: i64) -> TypeVariableInfo {
        self.expr_type_variables.get(&id).unwrap().clone()
    }

    fn var_info(&self, id: &String) -> TypeVariableInfo {
        self.var_type_variables.get(id).unwrap().clone()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TypeVariableInfo {
    pub ownership_var: i64,
    pub arg_group_var: i64,
}

#[derive(Clone, Copy, Debug)]
pub enum Constraint {
    Equal(i64, i64),
}

struct TypeVariableInfoAllocator {
    next: i64,
}

impl TypeVariableInfoAllocator {
    pub fn new() -> TypeVariableInfoAllocator {
        TypeVariableInfoAllocator { next: 1 }
    }

    pub fn allocate(&mut self) -> TypeVariableInfo {
        let ov = self.next;
        self.next += 1;
        let gv = self.next;
        self.next += 1;
        TypeVariableInfo {
            ownership_var: ov,
            arg_group_var: gv,
        }
    }
}

pub struct ConstraintStore {
    constraints: Vec<Constraint>,
}

impl ConstraintStore {
    pub fn new() -> ConstraintStore {
        ConstraintStore {
            constraints: Vec::new(),
        }
    }

    pub fn add_equal_exprs(&mut self, id1: i64, id2: i64, inference_info: &InferenceInfo) {
        let info1 = inference_info.expr_info(id1);
        let info2 = inference_info.expr_info(id2);
        self.add_equal_info(info1, info2);
    }

    pub fn add_equal_expr_var(&mut self, id: i64, var: &String, inference_info: &InferenceInfo) {
        let info1 = inference_info.expr_info(id);
        let info2 = inference_info.var_info(var);
        self.add_equal_info(info1, info2);
    }

    pub fn add_equal_info(&mut self, info1: TypeVariableInfo, info2: TypeVariableInfo) {
        self.constraints
            .push(Constraint::Equal(info1.ownership_var, info2.ownership_var));
        self.constraints
            .push(Constraint::Equal(info1.arg_group_var, info2.arg_group_var));
    }
}

struct LoopCollector {
    loops: Vec<i64>,
    breaks: BTreeMap<i64, Vec<i64>>,
    continues: BTreeMap<i64, Vec<i64>>,
}

impl LoopCollector {
    fn new() -> LoopCollector {
        LoopCollector {
            loops: Vec::new(),
            breaks: BTreeMap::new(),
            continues: BTreeMap::new(),
        }
    }
}

impl Visitor for LoopCollector {
    fn visit(&mut self, expr: &Expr) {
        match expr.kind {
            ExprKind::Loop(_, _, _) => {
                self.loops.push(expr.id);
            }
            ExprKind::Break(_) => {
                let loop_id = self.loops.last().unwrap();
                let breaks = self.breaks.entry(*loop_id).or_insert_with(|| Vec::new());
                breaks.push(expr.id);
            }
            ExprKind::Continue(_) => {
                let loop_id = self.loops.last().unwrap();
                let continues = self.continues.entry(*loop_id).or_insert_with(|| Vec::new());
                continues.push(expr.id);
            }
            _ => {}
        }
    }
}

fn process_function(f: &String, mir_program: &Program) {
    let mut inference_info = InferenceInfo::new();

    // initialization
    let f = mir_program.functions.get(f).unwrap();
    for (index, _) in f.args.iter().enumerate() {
        inference_info.add_var(format!("arg{}", index));
    }
    //println!("Processing {}", f.name);
    let mut loop_collector = LoopCollector::new();
    match &f.kind {
        FunctionKind::Normal(exprs) => {
            walk(exprs, &0, &mut loop_collector);
            for e in exprs.iter() {
                inference_info.add_expr(e.id);
                match &e.kind {
                    ExprKind::VarDecl(name, _) => {
                        inference_info.add_var(name.clone());
                    }
                    ExprKind::Loop(name, _, _) => {
                        inference_info.add_var(name.clone());
                    }
                    ExprKind::CaseOf(_, cases) => {
                        for case in cases {
                            match &case.checker {
                                Checker::Variant(_, name, _) => {
                                    inference_info.add_var(name.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    // constraint generation
    match &f.kind {
        FunctionKind::Normal(exprs) => {
            let mut constraints = ConstraintStore::new();
            for e in exprs.iter() {
                match &e.kind {
                    ExprKind::Do(items) => {
                        assert!(!items.is_empty());
                        let last = items.last().unwrap();
                        constraints.add_equal_exprs(*last, e.id, &inference_info);
                    }
                    ExprKind::VarDecl(name, rhs) => {
                        constraints.add_equal_expr_var(*rhs, name, &inference_info);
                    }
                    ExprKind::VarRef(name) => {
                        constraints.add_equal_expr_var(e.id, name, &inference_info);
                    }
                    ExprKind::If(_, true_branch, false_branch) => {
                        constraints.add_equal_exprs(*true_branch, e.id, &inference_info);
                        constraints.add_equal_exprs(*true_branch, *false_branch, &inference_info);
                    }
                    ExprKind::List(items) => match items.first() {
                        Some(first) => {
                            for (index, item) in items.iter().enumerate() {
                                if index != 0 {
                                    constraints.add_equal_exprs(*first, *item, &inference_info);
                                }
                            }
                        }
                        None => {}
                    },
                    ExprKind::Return(arg) => {
                        let result_info = inference_info.result_info;
                        let arg_info = inference_info.expr_info(*arg);
                        constraints.add_equal_info(result_info, arg_info);
                    }
                    ExprKind::Continue(arg) => {
                        /*
                        let result_info = inference_info.result_info;
                        let arg_info = inference_info.expr_info(*arg);
                        constraints.add_equal_info(result_info, arg_info);
                        */
                    }
                    ExprKind::Break(arg) => {
                        /*
                        let result_info = inference_info.result_info;
                        let arg_info = inference_info.expr_info(*arg);
                        constraints.add_equal_info(result_info, arg_info);
                        */
                    }
                    ExprKind::CaseOf(_, cases) => {
                        let first = cases.first().unwrap().body;
                        for (index, case) in cases.iter().enumerate() {
                            if index != 0 {
                                constraints.add_equal_exprs(first, case.body, &inference_info);
                            }
                        }
                    }
                    ExprKind::Loop(var, initializer, body) => {
                        constraints.add_equal_expr_var(*initializer, var, &inference_info);
                        constraints.add_equal_expr_var(*body, var, &inference_info);
                        if let Some(continues) = loop_collector.continues.get(&e.id) {
                            for c in continues {
                                constraints.add_equal_exprs(*c, *body, &inference_info);
                            }
                        }
                        if let Some(breaks) = loop_collector.breaks.get(&e.id) {
                            for b in breaks {
                                constraints.add_equal_exprs(*b, e.id, &inference_info);
                            }
                        }
                    }
                    ExprKind::Converter(arg) => {
                        let info1 = inference_info.expr_info(e.id);
                        let info2 = inference_info.expr_info(*arg);
                        constraints
                            .constraints
                            .push(Constraint::Equal(info1.arg_group_var, info2.arg_group_var));
                    }
                    _ => {}
                }
            }
            //println!("{} constraints", constraints.constraints.len());
        }
        _ => {}
    }
}

fn process_function_group(group: &Vec<String>, mir_program: &Program) {
    //println!("Processing f group {:?}", group);
    for f in group {
        process_function(&f, mir_program)
    }
}

pub fn inference(function_groups: Vec<Vec<String>>, mir_program: &Program) {
    println!("Inference started");
    for group in &function_groups {
        process_function_group(group, mir_program);
    }
}
