use std::collections::BTreeMap;

use crate::groups::*;
use crate::mir::*;

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

    pub fn add_equal_exprs(
        &mut self,
        id1: i64,
        id2: i64,
        expr_type_variables: &BTreeMap<i64, TypeVariableInfo>,
    ) {
        let info1 = expr_type_variables.get(&id1).unwrap();
        let info2 = expr_type_variables.get(&id2).unwrap();
        self.add_equal_info(info1, info2)
    }

    pub fn add_equal_expr_var(
        &mut self,
        id: i64,
        var: &String,
        expr_type_variables: &BTreeMap<i64, TypeVariableInfo>,
        var_type_variables: &BTreeMap<String, TypeVariableInfo>,
    ) {
        let info1 = expr_type_variables.get(&id).unwrap();
        let info2 = var_type_variables.get(var).unwrap();
        self.add_equal_info(info1, info2)
    }

    pub fn add_equal_info(&mut self, info1: &TypeVariableInfo, info2: &TypeVariableInfo) {
        self.constraints
            .push(Constraint::Equal(info1.ownership_var, info2.ownership_var));
        self.constraints
            .push(Constraint::Equal(info1.arg_group_var, info2.arg_group_var));
    }
}

fn process_function(f: &String, mir_program: &Program) {
    let mut expr_type_variables = BTreeMap::new();
    let mut var_type_variables = BTreeMap::new();
    let mut info_allocator = TypeVariableInfoAllocator::new();

    // initialization
    let f = mir_program.functions.get(f).unwrap();
    for (index, _) in f.args.iter().enumerate() {
        let info = info_allocator.allocate();
        var_type_variables.insert(format!("arg{}", index), info);
    }
    //println!("Processing {}", f.name);
    match &f.kind {
        FunctionKind::Normal(exprs) => {
            for e in exprs.iter() {
                let info = info_allocator.allocate();
                expr_type_variables.insert(e.id, info);
                match &e.kind {
                    ExprKind::VarDecl(name, _) => {
                        let info = info_allocator.allocate();
                        var_type_variables.insert(name.clone(), info);
                    }
                    ExprKind::Loop(name, _, _) => {
                        let info = info_allocator.allocate();
                        var_type_variables.insert(name.clone(), info);
                    }
                    ExprKind::CaseOf(_, cases) => {
                        for case in cases {
                            match &case.checker {
                                Checker::Variant(_, name, _) => {
                                    let info = info_allocator.allocate();
                                    var_type_variables.insert(name.clone(), info);
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
                        constraints.add_equal_exprs(*last, e.id, &expr_type_variables);
                    }
                    ExprKind::VarDecl(name, rhs) => {
                        constraints.add_equal_expr_var(
                            *rhs,
                            name,
                            &expr_type_variables,
                            &var_type_variables,
                        );
                    }
                    ExprKind::VarRef(name) => {
                        constraints.add_equal_expr_var(
                            e.id,
                            name,
                            &expr_type_variables,
                            &var_type_variables,
                        );
                    }
                    ExprKind::If(_, true_branch, false_branch) => {
                        constraints.add_equal_exprs(*true_branch, e.id, &expr_type_variables);
                        constraints.add_equal_exprs(
                            *true_branch,
                            *false_branch,
                            &expr_type_variables,
                        );
                    }
                    ExprKind::List(items) => match items.first() {
                        Some(first) => {
                            for (index, item) in items.iter().enumerate() {
                                if index != 0 {
                                    constraints.add_equal_exprs(
                                        *first,
                                        *item,
                                        &expr_type_variables,
                                    );
                                }
                            }
                        }
                        None => {}
                    },
                    ExprKind::CaseOf(_, cases) => {
                        let first = cases.first().unwrap().body;
                        for (index, case) in cases.iter().enumerate() {
                            if index != 0 {
                                constraints.add_equal_exprs(first, case.body, &expr_type_variables);
                            }
                        }
                    }
                    ExprKind::Loop(var, initializer, body) => {
                        constraints.add_equal_expr_var(
                            *initializer,
                            var,
                            &expr_type_variables,
                            &var_type_variables,
                        );
                        constraints.add_equal_expr_var(
                            *body,
                            var,
                            &expr_type_variables,
                            &var_type_variables,
                        );
                        constraints.add_equal_exprs(*body, e.id, &expr_type_variables);
                    }
                    ExprKind::Converter(arg) => {
                        let info1 = expr_type_variables.get(&e.id).unwrap();
                        let info2 = expr_type_variables.get(arg).unwrap();
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
